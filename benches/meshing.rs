// use bevy::prelude::*;
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use new_voxel_testing::{chunk::ChunkData, voxel::*};

// fn bench_chunk(world_pos: IVec3) {
//     let chunk = ChunkData::generate(world_pos);
// }

// fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function_("chunk", |b| {
//         b.iter_with_setup(
//             || {
//                 let chunk = ChunkData::generate(world_pos);
//                 chunk
//             },
//             ||
//             {
//                 bench_chunk(black_box(IVec3::ZERO)
//             }))
//     });
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);
