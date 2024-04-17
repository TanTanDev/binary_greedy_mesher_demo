use bevy::math::{ivec3, IVec3};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use new_voxel_testing::{
    chunks_refs::ChunksRefs,
    constants::{CHUNK_SIZE, CHUNK_SIZE_I32, CHUNK_SIZE_P},
    utils::vec3_to_index,
    voxel::{BlockData, BlockType},
};

fn iter_chunkrefs_padding(chunks_refs: ChunksRefs) {
    for x in 0..CHUNK_SIZE_P {
        for z in 0..CHUNK_SIZE_P {
            for y in 0..CHUNK_SIZE_P {
                let pos = ivec3(x as i32, y as i32, z as i32) - IVec3::ONE;
                let _b = chunks_refs.get_block(pos);
            }
        }
    }
}

fn iter_chunkrefs(chunks_refs: ChunksRefs) {
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let pos = ivec3(x as i32, y as i32, z as i32);
                let _b = chunks_refs.get_block(pos);
            }
        }
    }
}

fn iter_vec(data: Vec<BlockData>) {
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pos = ivec3(x as i32, y as i32, z as i32);
                let index = vec3_to_index(pos, 32);
                let _b = black_box(data[index]);
            }
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("iter chunk_refs ", |b| {
        b.iter_with_setup(
            || ChunksRefs::make_dummy_chunk_refs(0),
            |i| iter_chunkrefs(i),
        )
    });
    c.bench_function("iter chunk_refs padding ", |b| {
        b.iter_with_setup(
            || ChunksRefs::make_dummy_chunk_refs(0),
            |i| iter_chunkrefs_padding(i),
        )
    });
    c.bench_function("iter vec", |b| {
        b.iter_with_setup(
            || {
                let mut d = vec![];
                for _ in 0..CHUNK_SIZE_I32 * CHUNK_SIZE_I32 * CHUNK_SIZE_I32 {
                    d.push(BlockData {
                        block_type: BlockType::Air,
                    });
                }
                d
            },
            |i| iter_vec(i),
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
