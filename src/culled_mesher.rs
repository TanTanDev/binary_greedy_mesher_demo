use bevy::{math::ivec3, prelude::*};

use crate::{
    chunk_mesh::ChunkMesh,
    chunks_refs::ChunksRefs,
    lod::Lod,
    quad::{Direction, Quad},
    utils::{generate_indices, index_to_ivec3, make_vertex_u32},
};

fn push_face(mesh: &mut ChunkMesh, dir: Direction, vpos: IVec3, color: Color, block_type: u32) {
    let quad = Quad::from_direction(dir, vpos, color);
    for corner in quad.corners.into_iter() {
        mesh.vertices.push(make_vertex_u32(
            IVec3::from_array(corner),
            0,
            dir.get_normal() as u32,
            block_type,
        ));
    }
}

pub fn build_chunk_mesh_no_ao(chunks_refs: ChunksRefs, _lod: Lod) -> Option<ChunkMesh> {
    let mut mesh = ChunkMesh::default();
    for i in 0..32 * 32 * 32 {
        let local = index_to_ivec3(i);
        let (current, back, left, down) = chunks_refs.get_adjacent_blocks(local);
        match current.block_type.is_solid() {
            true => {
                if !left.block_type.is_solid() {
                    push_face(
                        &mut mesh,
                        Direction::Left,
                        local,
                        Color::GREEN,
                        current.block_type as u32,
                    );
                }
                if !back.block_type.is_solid() {
                    push_face(
                        &mut mesh,
                        Direction::Back,
                        local,
                        Color::GREEN,
                        current.block_type as u32,
                    );
                }
                if !down.block_type.is_solid() {
                    push_face(
                        &mut mesh,
                        Direction::Down,
                        local,
                        Color::GREEN,
                        current.block_type as u32,
                    );
                }
            }
            false => {
                if left.block_type.is_solid() {
                    push_face(
                        &mut mesh,
                        Direction::Right,
                        local,
                        Color::GREEN,
                        left.block_type as u32,
                    );
                }
                if back.block_type.is_solid() {
                    push_face(
                        &mut mesh,
                        Direction::Forward,
                        local,
                        Color::GREEN,
                        back.block_type as u32,
                    );
                }
                if down.block_type.is_solid() {
                    push_face(
                        &mut mesh,
                        Direction::Up,
                        local,
                        Color::GREEN,
                        down.block_type as u32,
                    );
                }
            }
        }
    }
    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}

///! helper for fetching voxels that will contribute to the current voxels ao value
pub fn ambient_corner_voxels(
    chunks_refs: &ChunksRefs,
    direction: Direction,
    local_pos: IVec3,
) -> [bool; 8] {
    #[rustfmt::skip]
    let mut positions = match direction {
        Direction::Left => [ivec3(-1,0,-1),ivec3(-1,-1,-1),ivec3(-1,-1,0),ivec3(-1,-1,1),ivec3(-1,0,1),ivec3(-1,1,1),ivec3(-1, 1, 0),ivec3(-1,1,-1),],
        Direction::Down => [ivec3(-1, -1, 0),ivec3(-1, -1, -1),ivec3(0, -1, -1), ivec3(1,-1,-1),ivec3(1,-1,0),ivec3(1, -1, 1),ivec3(0,-1,1),ivec3(-1,-1,1),],
        Direction::Back => [ivec3(0,-1,-1),ivec3(-1,-1,-1),ivec3(-1,0,-1),ivec3(-1,1,-1), ivec3(0,1,-1), ivec3(1,1,-1),ivec3(1,0,-1), ivec3(1,-1,-1)],

        Direction::Right => [ivec3(0,0,-1), ivec3(0,1,-1), ivec3(0,1,0), ivec3(0,1,1),ivec3(0,0,1),ivec3(0,-1,1),ivec3(0,-1,0),ivec3(0,-1,-1)],
        Direction::Up => [ivec3(-1,0,0),ivec3(-1,0,1),ivec3(0,0,1),ivec3(1,0,1),ivec3(1,0,0),ivec3(1,0,-1),ivec3(0,0,-1),ivec3(-1,0,-1),],
        Direction::Forward => [ivec3(0,-1,0),ivec3(1,-1,0),ivec3(1,0,0),ivec3(1,1,0),ivec3(0,1,0),ivec3(-1,1,0),ivec3(-1,0,0),ivec3(-1,-1,0),],
    };

    positions.iter_mut().for_each(|p| *p = local_pos + *p);

    let mut result = [false; 8];
    for i in 0..8 {
        result[i] = chunks_refs.get_block(positions[i]).block_type.is_solid();
    }
    result
}
pub fn ambient_corner_voxels_cloned(
    chunks_refs: &ChunksRefs,
    direction: Direction,
    local_pos: IVec3,
) -> Option<[bool; 8]> {
    #[rustfmt::skip]
    let mut positions = match direction {
        Direction::Left => [ivec3(-1,0,-1),ivec3(-1,-1,-1),ivec3(-1,-1,0),ivec3(-1,-1,1),ivec3(-1,0,1),ivec3(-1,1,1),ivec3(-1, 1, 0),ivec3(-1,1,-1),],
        Direction::Down => [ivec3(-1, -1, 0),ivec3(-1, -1, -1),ivec3(0, -1, -1), ivec3(1,-1,-1),ivec3(1,-1,0),ivec3(1, -1, 1),ivec3(0,-1,1),ivec3(-1,-1,1),],
        Direction::Back => [ivec3(0,-1,-1),ivec3(-1,-1,-1),ivec3(-1,0,-1),ivec3(-1,1,-1), ivec3(0,1,-1), ivec3(1,1,-1),ivec3(1,0,-1), ivec3(1,-1,-1)],

        Direction::Right => [ivec3(0,0,-1), ivec3(0,1,-1), ivec3(0,1,0), ivec3(0,1,1),ivec3(0,0,1),ivec3(0,-1,1),ivec3(0,-1,0),ivec3(0,-1,-1)],
        Direction::Up => [ivec3(-1,0,0),ivec3(-1,0,1),ivec3(0,0,1),ivec3(1,0,1),ivec3(1,0,0),ivec3(1,0,-1),ivec3(0,0,-1),ivec3(-1,0,-1),],
        Direction::Forward => [ivec3(0,-1,0),ivec3(1,-1,0),ivec3(1,0,0),ivec3(1,1,0),ivec3(0,1,0),ivec3(-1,1,0),ivec3(-1,0,0),ivec3(-1,-1,0),],
    };

    positions.iter_mut().for_each(|p| *p = local_pos + *p);

    let mut result = [false; 8];
    for i in 0..8 {
        result[i] = chunks_refs.get_block(positions[i]).block_type.is_solid();
    }
    Some(result)
}

pub fn build_chunk_mesh_ao(chunks_refs: &ChunksRefs, _lod: Lod) -> Option<ChunkMesh> {
    let mut mesh = ChunkMesh::default();
    for i in 0..32 * 32 * 32 {
        let local = index_to_ivec3(i);
        let (current, back, left, down) = chunks_refs.get_adjacent_blocks(local);
        match current.block_type.is_solid() {
            true => {
                if !left.block_type.is_solid() {
                    push_face_ao(
                        chunks_refs,
                        &mut mesh,
                        Direction::Left,
                        local,
                        Color::GREEN,
                        current.block_type as u32,
                    );
                }
                if !back.block_type.is_solid() {
                    push_face_ao(
                        chunks_refs,
                        &mut mesh,
                        Direction::Back,
                        local,
                        Color::GREEN,
                        current.block_type as u32,
                    );
                }
                if !down.block_type.is_solid() {
                    push_face_ao(
                        chunks_refs,
                        &mut mesh,
                        Direction::Down,
                        local,
                        Color::GREEN,
                        current.block_type as u32,
                    );
                }
            }
            false => {
                if left.block_type.is_solid() {
                    push_face_ao(
                        chunks_refs,
                        &mut mesh,
                        Direction::Right,
                        local,
                        Color::GREEN,
                        left.block_type as u32,
                    );
                }
                if back.block_type.is_solid() {
                    push_face_ao(
                        chunks_refs,
                        &mut mesh,
                        Direction::Forward,
                        local,
                        Color::GREEN,
                        back.block_type as u32,
                    );
                }
                if down.block_type.is_solid() {
                    push_face_ao(
                        chunks_refs,
                        &mut mesh,
                        Direction::Up,
                        local,
                        Color::GREEN,
                        down.block_type as u32,
                    );
                }
            }
        }
    }
    if mesh.vertices.is_empty() {
        None
    } else {
        mesh.indices = generate_indices(mesh.vertices.len());
        Some(mesh)
    }
}

fn push_face_ao(
    chunks_refs: &ChunksRefs,
    mesh: &mut ChunkMesh,
    dir: Direction,
    vpos: IVec3,
    color: Color,
    block_type: u32,
) {
    let ambient_corners = ambient_corner_voxels(&chunks_refs, dir, vpos);
    let quad = Quad::from_direction(dir, vpos, color);
    for (i, corner) in quad.corners.into_iter().enumerate() {
        let index = i * 2;

        let side_1 = ambient_corners[index] as u32;
        let side_2 = ambient_corners[(index + 2) % 8] as u32;
        let side_corner = ambient_corners[(index + 1) % 8] as u32;
        let mut ao_count = side_1 + side_2 + side_corner;
        // fully ambient occluded if both
        if side_1 == 1 && side_2 == 1 {
            ao_count = 3;
        }

        mesh.vertices.push(make_vertex_u32(
            IVec3::from_array(corner),
            ao_count,
            dir.get_normal() as u32,
            block_type,
        ));
    }
}
