use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use new_voxel_testing::chunk::ChunkData;

fn bench_chunk(world_pos: IVec3) {
    let _chunk = ChunkData::generate(world_pos);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("build chunk data", |b| {
        b.iter_with_setup(
            || {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let b = 100;
                let y = 20;
                black_box(IVec3::new(
                    rng.gen_range(-b..b),
                    rng.gen_range(-y..y),
                    rng.gen_range(-b..b),
                ))
            },
            |i| bench_chunk(i),
        )
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
