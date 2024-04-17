use std::{collections::VecDeque, sync::Arc, time::Instant};

use bevy::{prelude::*, utils::HashMap};
use rand::Rng;

use crate::{
    chunk::ChunkData,
    chunk_mesh::ChunkMesh,
    chunks_refs::ChunksRefs,
    face_direction::FaceDir,
    lod::Lod,
    utils::{generate_indices, make_vertex_u32},
    voxel::MESHABLE_BLOCK_TYPES,
};

pub fn build_chunk_mesh(chunks_refs: ChunksRefs, lod: Lod) -> Option<ChunkMesh> {
    let mut mesh = ChunkMesh::default();
    let mut quads = vec![];
    quads.extend(vertices_from_face(FaceDir::Up, &chunks_refs, &lod));
    quads.extend(vertices_from_face(FaceDir::Left, &chunks_refs, &lod));
    // quads.extend(vertices_from_face(FaceDir::Right, &chunks_refs, &lod));
    // quads.extend(vertices_from_face(FaceDir::Down, &chunks_refs, &lod));
    quads.extend(vertices_from_face(FaceDir::Forward, &chunks_refs, &lod));
    // quads.extend(vertices_from_face(FaceDir::Back, &chunks_refs, &lod));
    mesh.vertices.extend(quads);
    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}

pub fn build_chunk_mesh_no_ao(chunks_refs: ChunksRefs, lod: Lod) -> Option<ChunkMesh> {
    let mut mesh = ChunkMesh::default();
    let mut quads = vec![];
    quads.extend(vertices_from_face_no_ao(FaceDir::Up, &chunks_refs, &lod));
    quads.extend(vertices_from_face_no_ao(FaceDir::Left, &chunks_refs, &lod));
    quads.extend(vertices_from_face_no_ao(FaceDir::Right, &chunks_refs, &lod));
    quads.extend(vertices_from_face_no_ao(FaceDir::Down, &chunks_refs, &lod));
    quads.extend(vertices_from_face_no_ao(
        FaceDir::Forward,
        &chunks_refs,
        &lod,
    ));
    quads.extend(vertices_from_face_no_ao(FaceDir::Back, &chunks_refs, &lod));
    mesh.vertices.extend(quads);
    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}

///! generate vertices for the facing direction, all planes of a chunk
pub fn vertices_from_face(face_dir: FaceDir, chunks_refs: &ChunksRefs, lod: &Lod) -> Vec<u32> {
    // generate -x plane
    let mut vertices = vec![];
    let size = lod.size();
    for axis in 0..size {
        // not optimal... save ambient occlusion data
        let mut ao_data = [[0; 34]; 34];
        for y in -1..33 {
            for x in -1..33 {
                let pos = face_dir.world_to_sample(axis, x, y, lod);
                let pos = pos * lod.jump_index();
                let is_solid = chunks_refs
                    .get_block(pos + face_dir.air_sample_dir() * lod.jump_index())
                    .block_type
                    .is_solid();
                ao_data[(x + 1) as usize][(y + 1) as usize] = is_solid as u32;
            }
        }

        // key: ao + color's
        let mut x_data = HashMap::<u32, [u32; 32]>::new();
        for i in 0..size * size {
            let x = i % size;
            let y = (i / size) as i32;
            let pos = face_dir.world_to_sample(axis, x, y, lod);
            let pos = pos * lod.jump_index();
            let (current, neg_z_block) =
                chunks_refs.get_2(pos, face_dir.air_sample_dir() * lod.jump_index());
            let x = x as usize;
            let y = y as usize;
            let ao_index = ao_data[x + 0][y + 0]
                | (ao_data[x + 0][y + 1] << 1)
                | (ao_data[x + 0][y + 2] << 2)
                | (ao_data[x + 1][y + 0] << 3)
                | (ao_data[x + 1][y + 1] << 4)
                | (ao_data[x + 1][y + 2] << 5)
                | (ao_data[x + 2][y + 0] << 6)
                | (ao_data[x + 2][y + 1] << 7)
                | (ao_data[x + 2][y + 2] << 8);
            let is_solid = current.block_type.is_solid() && !neg_z_block.block_type.is_solid();
            // can merge with ao?
            let p_index = ao_index | ((current.block_type as u32) << 9);
            let data = match x_data.get_mut(&p_index) {
                Some(d) => d,
                None => {
                    x_data.insert(p_index, [0u32; 32]);
                    x_data.get_mut(&p_index).unwrap()
                }
            };

            // set bit to 1 or 0 depending if solid
            data[x as usize] |= (1 << y) * is_solid as u32;
        } // axis type loop
        for (p_index, data) in x_data.into_iter() {
            let quads_from_axis = greedy_mesh_binary_plane(data, lod.size() as u32);
            let ao = p_index & 0b111111111;
            let block_type = p_index >> 9;

            quads_from_axis.into_iter().for_each(|q| {
                q.append_vertices(&mut vertices, face_dir, axis as u32, lod, ao, block_type)
            });
        }
    }
    vertices
}

pub fn vertices_from_face_no_ao(
    face_dir: FaceDir,
    chunks_refs: &ChunksRefs,
    lod: &Lod,
) -> Vec<u32> {
    // generate -x plane
    let mut vertices = vec![];
    let size = lod.size();
    for axis in 0..size {
        for block_type in MESHABLE_BLOCK_TYPES.iter() {
            let mut x_data = [0u32; 32];
            for i in 0..size * size {
                let row = i % size;
                let column = (i / size) as i32;
                let pos = face_dir.world_to_sample(axis, row, column, lod);
                let pos = pos * lod.jump_index();
                let (current, neg_z_block) =
                    chunks_refs.get_2(pos, face_dir.air_sample_dir() * lod.jump_index());
                // don't merge different block types
                if &current.block_type != block_type {
                    continue;
                }
                let is_solid = current.block_type.is_solid() && !neg_z_block.block_type.is_solid();
                // set bit to 1 or 0 depending if solid
                x_data[row as usize] = ((1 << column) * is_solid as u32) | x_data[row as usize];
            }
            let quads_from_axis = greedy_mesh_binary_plane(x_data, lod.size() as u32);
            quads_from_axis
                .into_iter()
                .for_each(|q| q.append_vertices(&mut vertices, face_dir, axis as u32, lod, 0, 0));
        } // block type loop
    }
    vertices
}

///! todo: compress further?
#[derive(Debug)]
pub struct GreedyQuad {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl GreedyQuad {
    ///! compress this quad data into the input vertices vec
    pub fn append_vertices(
        &self,
        vertices: &mut Vec<u32>,
        face_dir: FaceDir,
        axis: u32,
        lod: &Lod,
        ao: u32,
        block_type: u32,
    ) {
        let negate_axis = face_dir.negate_axis();
        let axis = axis as i32 + negate_axis;
        let jump = lod.jump_index();

        // pack ambient occlusion strength into vertex
        let v1ao = ((ao >> 0) & 1) + ((ao >> 1) & 1) + ((ao >> 3) & 1);
        let v2ao = ((ao >> 3) & 1) + ((ao >> 6) & 1) + ((ao >> 7) & 1);
        let v3ao = ((ao >> 5) & 1) + ((ao >> 8) & 1) + ((ao >> 7) & 1);
        let v4ao = ((ao >> 1) & 1) + ((ao >> 2) & 1) + ((ao >> 5) & 1);

        let v1 = make_vertex_u32(
            face_dir.world_to_sample(axis as i32, self.x as i32, self.y as i32, &lod) * jump,
            v1ao,
            face_dir.normal_index(),
            block_type,
        );
        let v2 = make_vertex_u32(
            face_dir.world_to_sample(
                axis as i32,
                self.x as i32 + self.w as i32,
                self.y as i32,
                &lod,
            ) * jump,
            v2ao,
            face_dir.normal_index(),
            block_type,
        );
        let v3 = make_vertex_u32(
            face_dir.world_to_sample(
                axis as i32,
                self.x as i32 + self.w as i32,
                self.y as i32 + self.h as i32,
                &lod,
            ) * jump,
            v3ao,
            face_dir.normal_index(),
            block_type,
        );
        let v4 = make_vertex_u32(
            face_dir.world_to_sample(
                axis as i32,
                self.x as i32,
                self.y as i32 + self.h as i32,
                &lod,
            ) * jump,
            v4ao,
            face_dir.normal_index(),
            block_type,
        );

        // the quad vertices to be added
        let mut new_vertices = VecDeque::from([v1, v2, v3, v4]);

        // triangle rendering order is different depending on the facing direction
        if face_dir.reverse_order() {
            // keep first index, but reverse the rest
            let o = new_vertices.split_off(1);
            o.into_iter().rev().for_each(|i| new_vertices.push_back(i));
        }

        // anisotropy flip
        if (v1ao > 0) ^ (v3ao > 0) {
            // right shift array, to swap triangle intersection angle
            let f = new_vertices.pop_front().unwrap();
            new_vertices.push_back(f);
        }

        vertices.extend(new_vertices);
    }
}

///! generate quads of a binary slice
///! lod not implemented yet
pub fn greedy_mesh_binary_plane(mut data: [u32; 32], lod_size: u32) -> Vec<GreedyQuad> {
    let mut greedy_quads = vec![];
    for row in 0..data.len() {
        let mut y = 0;
        while y < lod_size {
            // find first solid, "air/zero's" could be first so skip
            y += (data[row] >> y).trailing_zeros();
            if y >= lod_size {
                // reached top
                continue;
            }
            let h = (data[row] >> y).trailing_ones();

            // convert height 'num' to positive bits repeated 'num' times aka:
            // 1 = 0b1, 2 = 0b11, 4 = 0b1111
            let h_as_mask = u32::checked_shl(1, h).map_or(!0, |v| v - 1);
            // offset the mask to the correct y pos
            let mask = h_as_mask << y;

            // grow horizontally
            let mut w = 1;
            while row + w < lod_size as usize {
                // fetch bits spanning height, in the next row
                let next_row_h = (data[row + w] >> y) & h_as_mask;
                if next_row_h != h_as_mask {
                    break; // can no longer expand
                }

                // nuke the bits we expanded into
                data[row + w] = data[row + w] & !mask;

                w += 1;
            }
            greedy_quads.push(GreedyQuad {
                y,
                w: w as u32,
                h,
                x: row as u32,
            });
            y += h;
        }
    }
    greedy_quads
}
