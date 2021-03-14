use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u128>::new(&[4, 4, 4, 4, 4]);
    c.bench_function("u128 6 x 4", |b| b.iter(|| black_box(scheduler.step())));

    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u64>::new(&[4, 4, 4, 4, 4]);
    c.bench_function("u64 6 x 4", |b| b.iter(|| black_box(scheduler.step())));

    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u32>::new(&[4, 4, 4, 4, 4]);
    c.bench_function("u32 6 x 4", |b| b.iter(|| black_box(scheduler.step())));

    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u16>::new(&[4, 4, 4, 4, 4]);
    c.bench_function("u16 6 x 4", |b| b.iter(|| black_box(scheduler.step())));

    let mut scheduler = df_social_schedule::df_schedule::DFScheduler::<u8>::new(&[4, 4, 4, 4, 4]);
    c.bench_function("u8 6 x 4", |b| b.iter(|| black_box(scheduler.step())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
