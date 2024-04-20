use std::sync::Arc;

use bevy::{
    asset::LoadState,
    diagnostic::{Diagnostic, DiagnosticPath, Diagnostics, RegisterDiagnostic},
    prelude::*,
    render::{
        mesh::Indices, primitives::Aabb, render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
    tasks::{block_on, AsyncComputeTaskPool, Task},
    utils::{HashMap, HashSet},
};
use bevy_screen_diagnostics::{Aggregate, ScreenDiagnostics};

use crate::{
    chunk::ChunkData,
    chunk_mesh::ChunkMesh,
    chunks_refs::ChunksRefs,
    constants::CHUNK_SIZE_I32,
    lod::Lod,
    rendering::{GlobalChunkMaterial, ATTRIBUTE_VOXEL},
    scanner::Scanner,
    utils::{get_edging_chunk, vec3_to_index},
    voxel::{BlockData, BlockType},
};
use futures_lite::future;

pub struct VoxelEnginePlugin;

pub const MAX_DATA_TASKS: usize = 64;
pub const MAX_MESH_TASKS: usize = 32;

impl Plugin for VoxelEnginePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VoxelEngine::default());
        // app.add_systems(Update, (start_data_tasks, start_mesh_tasks));
        app.add_systems(PostUpdate, (start_data_tasks, start_mesh_tasks));
        // app.add_systems(PostUpdate, (join_data, join_mesh));
        app.add_systems(Update, start_modifications);
        app.add_systems(
            // PostUpdate,
            Update,
            ((join_data, join_mesh), (unload_data, unload_mesh)).chain(),
        );
        app.add_systems(Update, debug_inputs);

        app.add_systems(Startup, setup_diagnostics);
        app.register_diagnostic(Diagnostic::new(DIAG_LOAD_MESH_QUEUE));
        app.register_diagnostic(Diagnostic::new(DIAG_UNLOAD_MESH_QUEUE));
        app.register_diagnostic(Diagnostic::new(DIAG_LOAD_DATA_QUEUE));
        app.register_diagnostic(Diagnostic::new(DIAG_UNLOAD_DATA_QUEUE));
        app.register_diagnostic(Diagnostic::new(DIAG_VERTEX_COUNT));
        app.register_diagnostic(Diagnostic::new(DIAG_MESH_TASKS));
        app.register_diagnostic(Diagnostic::new(DIAG_DATA_TASKS));
        app.add_systems(Update, diagnostics_count);
    }
}

pub fn debug_inputs(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut voxel_engine: ResMut<VoxelEngine>,
    scanners: Query<(&GlobalTransform, &Scanner)>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // swap meshing algorithm
        use MeshingMethod as MM;
        voxel_engine.meshing_method = match voxel_engine.meshing_method {
            MM::VertexCulled => MM::BinaryGreedyMeshing,
            MM::BinaryGreedyMeshing => MM::VertexCulled,
        };
        let (scanner_transform, scanner) = scanners.single();
        // unload all meshes
        voxel_engine.unload_all_meshes(scanner, scanner_transform);
    }
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        // toggle rendering method
    }
}

#[derive(Debug, Reflect, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MeshingMethod {
    VertexCulled,
    BinaryGreedyMeshing,
}

///! holds all voxel world data
#[derive(Resource)]
pub struct VoxelEngine {
    pub world_data: HashMap<IVec3, Arc<ChunkData>>,
    pub vertex_diagnostic: HashMap<IVec3, i32>,
    pub load_data_queue: Vec<IVec3>,
    pub load_mesh_queue: Vec<IVec3>,
    pub unload_data_queue: Vec<IVec3>,
    pub unload_mesh_queue: Vec<IVec3>,
    pub data_tasks: HashMap<IVec3, Option<Task<ChunkData>>>,
    // pub mesh_tasks: HashMap<IVec3, Option<Task<Option<ChunkMesh>>>>,
    pub mesh_tasks: Vec<(IVec3, Option<Task<Option<ChunkMesh>>>)>,
    pub chunk_entities: HashMap<IVec3, Entity>,
    pub lod: Lod,
    pub meshing_method: MeshingMethod,
    pub chunk_modifications: HashMap<IVec3, Vec<ChunkModification>>,
}

pub struct ChunkModification(pub IVec3, pub BlockType);

const DIAG_LOAD_DATA_QUEUE: DiagnosticPath = DiagnosticPath::const_new("load_data_queue");
const DIAG_UNLOAD_DATA_QUEUE: DiagnosticPath = DiagnosticPath::const_new("unload_data_queue");
const DIAG_LOAD_MESH_QUEUE: DiagnosticPath = DiagnosticPath::const_new("load_mesh_queue");
const DIAG_UNLOAD_MESH_QUEUE: DiagnosticPath = DiagnosticPath::const_new("unload_mesh_queue");
const DIAG_VERTEX_COUNT: DiagnosticPath = DiagnosticPath::const_new("vertex_count");
const DIAG_MESH_TASKS: DiagnosticPath = DiagnosticPath::const_new("mesh_tasks");
const DIAG_DATA_TASKS: DiagnosticPath = DiagnosticPath::const_new("data_tasks");

fn setup_diagnostics(mut onscreen: ResMut<ScreenDiagnostics>) {
    onscreen
        .add("load_data_queue".to_string(), DIAG_LOAD_DATA_QUEUE)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>4.0}"));
    onscreen
        .add("unload_data_queue".to_string(), DIAG_UNLOAD_DATA_QUEUE)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>3.0}"));
    onscreen
        .add("load_mesh_queue".to_string(), DIAG_LOAD_MESH_QUEUE)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>4.0}"));
    onscreen
        .add("unload_mesh_queue".to_string(), DIAG_UNLOAD_MESH_QUEUE)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>3.0}"));
    onscreen
        .add("vertex_count".to_string(), DIAG_VERTEX_COUNT)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>7.0}"));
    onscreen
        .add("mesh_tasks".to_string(), DIAG_MESH_TASKS)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>4.0}"));
    onscreen
        .add("data_tasks".to_string(), DIAG_DATA_TASKS)
        .aggregate(Aggregate::Value)
        .format(|v| format!("{v:0>2.0}"));
}

fn diagnostics_count(mut diagnostics: Diagnostics, voxel_engine: Res<VoxelEngine>) {
    diagnostics.add_measurement(&DIAG_LOAD_DATA_QUEUE, || {
        voxel_engine.load_data_queue.len() as f64
    });
    diagnostics.add_measurement(&DIAG_UNLOAD_DATA_QUEUE, || {
        voxel_engine.unload_data_queue.len() as f64
    });
    diagnostics.add_measurement(&DIAG_LOAD_MESH_QUEUE, || {
        voxel_engine.load_mesh_queue.len() as f64
    });
    diagnostics.add_measurement(&DIAG_UNLOAD_MESH_QUEUE, || {
        voxel_engine.unload_mesh_queue.len() as f64
    });
    diagnostics.add_measurement(&DIAG_MESH_TASKS, || voxel_engine.mesh_tasks.len() as f64);
    diagnostics.add_measurement(&DIAG_DATA_TASKS, || voxel_engine.data_tasks.len() as f64);
    diagnostics.add_measurement(&DIAG_VERTEX_COUNT, || {
        voxel_engine
            .vertex_diagnostic
            .iter()
            .map(|(_, v)| v)
            .sum::<i32>() as f64
    });
}

impl VoxelEngine {
    pub fn unload_all_meshes(&mut self, scanner: &Scanner, scanner_transform: &GlobalTransform) {
        // stop all any current proccessing
        self.load_mesh_queue.clear();
        // self.unload_mesh_queue.clear();
        self.mesh_tasks.clear();
        let scan_pos =
            ((scanner_transform.translation() - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3();
        for offset in &scanner.mesh_sampling_offsets {
            let wpos = scan_pos + *offset;
            self.load_mesh_queue.push(wpos);
            // self.unload_mesh_queue.push(wpos);
        }
    }
}

impl Default for VoxelEngine {
    fn default() -> Self {
        VoxelEngine {
            world_data: HashMap::new(),
            load_data_queue: Vec::new(),
            load_mesh_queue: Vec::new(),
            unload_data_queue: Vec::new(),
            unload_mesh_queue: Vec::new(),
            data_tasks: HashMap::new(),
            mesh_tasks: Vec::new(),
            // mesh_tasks: HashMap::new(),
            chunk_entities: HashMap::new(),
            lod: Lod::L32,
            // meshing_method: MeshingMethod::VertexCulled,
            meshing_method: MeshingMethod::BinaryGreedyMeshing,
            vertex_diagnostic: HashMap::new(),
            chunk_modifications: HashMap::new(),
        }
    }
}

///! begin data building tasks for chunks in range
pub fn start_data_tasks(
    mut voxel_engine: ResMut<VoxelEngine>,
    scanners: Query<&GlobalTransform, With<Scanner>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let VoxelEngine {
        load_data_queue,
        data_tasks,
        ..
    } = voxel_engine.as_mut();

    let scanner_g = scanners.single();
    let scan_pos = ((scanner_g.translation() - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3();
    load_data_queue.sort_by(|a, b| {
        a.distance_squared(scan_pos)
            .cmp(&b.distance_squared(scan_pos))
    });

    let tasks_left = (MAX_DATA_TASKS as i32 - data_tasks.len() as i32)
        .min(load_data_queue.len() as i32)
        .max(0) as usize;
    for world_pos in load_data_queue.drain(0..tasks_left) {
        // for world_pos in load_data_queue.drain(0..MAX_DATA_TASKS.min(load_data_queue.len())) {
        // for world_pos in load_data_queue.drain(..) {
        let k = world_pos;
        let task = task_pool.spawn(async move {
            let cd = ChunkData::generate(k);
            cd
        });
        data_tasks.insert(world_pos, Some(task));
    }
}

///! destroy enqueued, chunk data
pub fn unload_data(mut voxel_engine: ResMut<VoxelEngine>) {
    let VoxelEngine {
        unload_data_queue,
        world_data,
        ..
    } = voxel_engine.as_mut();
    for chunk_pos in unload_data_queue.drain(..) {
        world_data.remove(&chunk_pos);
    }
}

///! destroy enqueued, chunk mesh entities
pub fn unload_mesh(mut commands: Commands, mut voxel_engine: ResMut<VoxelEngine>) {
    let VoxelEngine {
        unload_mesh_queue,
        chunk_entities,
        vertex_diagnostic,
        ..
    } = voxel_engine.as_mut();
    let mut retry = Vec::new();
    for chunk_pos in unload_mesh_queue.drain(..) {
        let Some(chunk_id) = chunk_entities.remove(&chunk_pos) else {
            continue;
        };
        vertex_diagnostic.remove(&chunk_pos);
        if let Some(mut entity_commands) = commands.get_entity(chunk_id) {
            entity_commands.despawn();
        }
        // world_data.remove(&chunk_pos);
    }
    unload_mesh_queue.append(&mut retry);
}

///! begin mesh building tasks for chunks in range
pub fn start_mesh_tasks(
    mut voxel_engine: ResMut<VoxelEngine>,
    scanners: Query<&GlobalTransform, With<Scanner>>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    let VoxelEngine {
        load_mesh_queue,
        mesh_tasks,
        world_data,
        lod,
        meshing_method,
        ..
    } = voxel_engine.as_mut();

    let scanner_g = scanners.single();
    let scan_pos = ((scanner_g.translation() - Vec3::splat(16.0)) * (1.0 / 32.0)).as_ivec3();
    load_mesh_queue.sort_by(|a, b| {
        a.distance_squared(scan_pos)
            .cmp(&b.distance_squared(scan_pos))
    });
    let tasks_left = (MAX_MESH_TASKS as i32 - mesh_tasks.len() as i32)
        .min(load_mesh_queue.len() as i32)
        .max(0) as usize;
    for world_pos in load_mesh_queue.drain(0..tasks_left) {
        // for world_pos in load_mesh_queue.drain(..) {
        let Some(chunks_refs) = ChunksRefs::try_new(world_data, world_pos) else {
            continue;
        };
        let llod = *lod;
        let task = match meshing_method {
            MeshingMethod::BinaryGreedyMeshing => task_pool.spawn(async move {
                crate::greedy_mesher_optimized::build_chunk_mesh(chunks_refs, llod)
            }),
            MeshingMethod::VertexCulled => task_pool.spawn(async move {
                crate::culled_mesher::build_chunk_mesh_ao(&chunks_refs, llod)
            }),
        };

        mesh_tasks.push((world_pos, Some(task)));
    }
}

// start
pub fn start_modifications(mut voxel_engine: ResMut<VoxelEngine>) {
    let VoxelEngine {
        world_data,
        chunk_modifications,
        load_mesh_queue,
        ..
    } = voxel_engine.as_mut();
    for (pos, mods) in chunk_modifications.drain() {
        // say i want to load mesh now :)
        let Some(chunk_data) = world_data.get_mut(&pos) else {
            continue;
        };
        let new_chunk_data = Arc::make_mut(chunk_data);
        let mut adj_chunk_set = HashSet::new();
        for ChunkModification(local_pos, block_type) in mods.into_iter() {
            let i = vec3_to_index(local_pos, 32);
            if new_chunk_data.voxels.len() == 1 {
                let mut voxels = vec![];
                for _ in 0..CHUNK_SIZE_I32 * CHUNK_SIZE_I32 * CHUNK_SIZE_I32 {
                    voxels.push(BlockData {
                        block_type: new_chunk_data.voxels[0].block_type,
                    });
                }
                new_chunk_data.voxels = voxels;
            }
            new_chunk_data.voxels[i].block_type = block_type;
            if let Some(edge_chunk) = get_edging_chunk(local_pos) {
                adj_chunk_set.insert(edge_chunk);
            }
        }
        for adj_chunk in adj_chunk_set.into_iter() {
            load_mesh_queue.push(pos + adj_chunk);
        }
        load_mesh_queue.push(pos);
    }
}

///! join the chunkdata threads
pub fn join_data(mut voxel_engine: ResMut<VoxelEngine>) {
    let VoxelEngine {
        world_data,
        data_tasks,
        ..
    } = voxel_engine.as_mut();
    for (world_pos, task_option) in data_tasks.iter_mut() {
        let Some(mut task) = task_option.take() else {
            // should never happend, because we drop None values later
            warn!("someone modified task?");
            continue;
        };
        let Some(chunk_data) = block_on(future::poll_once(&mut task)) else {
            *task_option = Some(task);
            continue;
        };

        world_data.insert(*world_pos, Arc::new(chunk_data));
    }
    data_tasks.retain(|_k, op| op.is_some());
}

#[derive(Component)]
pub struct WaitingToLoadMeshTag;

pub fn promote_dirty_meshes(
    mut commands: Commands,
    children: Query<(Entity, &Handle<Mesh>, &Parent), With<WaitingToLoadMeshTag>>,
    mut parents: Query<&mut Handle<Mesh>, Without<WaitingToLoadMeshTag>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, handle, parent) in children.iter() {
        if let Some(state) = asset_server.get_load_state(handle) {
            match state {
                LoadState::Loaded | LoadState::Failed => {
                    let Ok(mut parent_handle) = parents.get_mut(parent.get()) else {
                        continue;
                    };
                    info!("updgraded!");
                    *parent_handle = handle.clone();
                    commands.entity(entity).despawn();
                }
                LoadState::Loading => {
                    info!("loading cool");
                }
                _ => (),
            }
        }
    }
}

///! join the multithreaded chunk mesh tasks, and construct a finalized chunk entity
pub fn join_mesh(
    mut voxel_engine: ResMut<VoxelEngine>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    global_chunk_material: Res<GlobalChunkMaterial>,
) {
    let VoxelEngine {
        mesh_tasks,
        chunk_entities,
        vertex_diagnostic,
        ..
    } = voxel_engine.as_mut();
    for (world_pos, task_option) in mesh_tasks.iter_mut() {
        let Some(mut task) = task_option.take() else {
            // should never happend, because we drop None values later
            warn!("someone modified task?");
            continue;
        };
        let Some(chunk_mesh_option) = block_on(future::poll_once(&mut task)) else {
            // failed polling, keep task alive
            *task_option = Some(task);
            continue;
        };

        let Some(mesh) = chunk_mesh_option else {
            continue;
        };
        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        vertex_diagnostic.insert(*world_pos, mesh.vertices.len() as i32);
        bevy_mesh.insert_attribute(ATTRIBUTE_VOXEL, mesh.vertices.clone());
        // bevy_mesh.set_indices(Some(Indices::U32(mesh.indices.clone().into())));
        bevy_mesh.insert_indices(Indices::U32(mesh.indices.clone().into()));
        let mesh_handle = meshes.add(bevy_mesh);

        if let Some(entity) = chunk_entities.get(world_pos) {
            commands.entity(*entity).despawn();
        }

        // spawn chunk entity
        let chunk_entity = commands
            .spawn((
                Aabb::from_min_max(Vec3::ZERO, Vec3::splat(32.0)),
                MaterialMeshBundle {
                    transform: Transform::from_translation(world_pos.as_vec3() * Vec3::splat(32.0)),
                    mesh: mesh_handle,
                    material: global_chunk_material.0.clone(),
                    ..default()
                },
            ))
            .id();
        chunk_entities.insert(*world_pos, chunk_entity);
    }
    mesh_tasks.retain(|(_p, op)| op.is_some());
}
