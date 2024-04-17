// IMPORTANT: THIS MESHER IS NOT FULLY IMPLEMENTED
// it doesn't work ;)
// It started as an idea to reduce big amount of sampling we are doing
// we estimate if a chunk is mostly air or solid.
// if it's mostly air, we only care for blocks that are solid, vice versa
// the problem arrise when adjacent chunks use different estimated solids.

use std::collections::VecDeque;

use bevy::{math::ivec3, prelude::*};

use crate::{
    chunk_mesh::ChunkMesh,
    chunks_refs::ChunksRefs,
    lod::Lod,
    quad::{Direction, Quad},
    utils::{generate_indices, index_to_ivec3, is_on_edge, make_vertex_u32},
};

// construct vertices for a face in provided direciton
fn push_face(
    mesh: &mut ChunkMesh,
    dir: Direction,
    pos: IVec3,
    color: Color,
    block_type: u32,
    flip_winding_order: bool,
) {
    let quad = Quad::from_direction(dir, pos, color);

    let mut corners = VecDeque::from(quad.corners);
    let flip_winding_order = !flip_winding_order;
    if flip_winding_order {
        // keep first index, but reverse the rest
        let o = corners.split_off(1);
        o.into_iter().rev().for_each(|i| corners.push_back(i));
    }

    let normal = match flip_winding_order {
        true => dir.get_opposite().get_normal() as u32,
        false => dir.get_normal() as u32,
    };

    for corner in corners.into_iter() {
        mesh.vertices.push(make_vertex_u32(
            IVec3::from_array(corner),
            0,
            normal,
            block_type,
        ));
    }
}

pub fn build_chunk_mesh(chunks_refs: ChunksRefs, lod: Lod) -> Option<ChunkMesh> {
    let mut mesh = ChunkMesh::default();
    // estimate if chunk is mostly solid or air
    let most_solid = chunks_refs
        .get_block(IVec3::splat(16))
        .block_type
        .is_solid();
    // let most_solid = true;

    for i in 0..32 * 32 * 32 {
        let local = index_to_ivec3(i);
        let current = chunks_refs.get_block(local);
        if match most_solid {
            true => current.block_type.is_solid(),
            false => current.block_type.is_air(),
        } {
            continue;
        }

        let Some(neighbours) = chunks_refs.get_von_neumann(local) else {
            panic!();
        };

        for (dir, block) in neighbours.iter() {
            let con = match most_solid {
                true => block.block_type.is_solid(),
                false => block.block_type.is_air(),
            };
            let block = match most_solid {
                false => current.block_type,
                true => block.block_type,
            };
            if con {
                let flip_winding_order = !most_solid;
                push_face(
                    &mut mesh,
                    *dir,
                    local,
                    Color::GREEN,
                    block as u32,
                    flip_winding_order,
                );
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
