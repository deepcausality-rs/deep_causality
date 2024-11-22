use criterion::{black_box, criterion_group, Criterion};
use dcl_data_structures::ring_buffer::utils::bit_map::BitMap;
use std::num::NonZeroUsize;

fn bitmap_benchmark(c: &mut Criterion) {
    let capacity = NonZeroUsize::new(1024).unwrap();
    let bitmap = BitMap::new(capacity);

    // Benchmark set operation
    c.bench_function("bitmap_set", |b| {
        b.iter(|| {
            for i in 0..100 {
                bitmap.set(black_box(i));
            }
        })
    });

    // Benchmark unset operation
    c.bench_function("bitmap_unset", |b| {
        b.iter(|| {
            for i in 0..100 {
                bitmap.unset(black_box(i));
            }
        })
    });

    // Benchmark is_set operation
    c.bench_function("bitmap_is_set", |b| {
        b.iter(|| {
            for i in 0..100 {
                black_box(bitmap.is_set(black_box(i)));
            }
        })
    });

    // Benchmark mixed operations
    c.bench_function("bitmap_mixed_ops", |b| {
        b.iter(|| {
            for i in 0..100 {
                bitmap.set(black_box(i));
                black_box(bitmap.is_set(black_box(i)));
                bitmap.unset(black_box(i));
            }
        })
    });

    // Benchmark sequential vs random access
    c.bench_function("bitmap_sequential_access", |b| {
        b.iter(|| {
            for i in 0..100 {
                bitmap.set(black_box(i));
            }
        })
    });

    c.bench_function("bitmap_random_access", |b| {
        b.iter(|| {
            for i in [73, 2, 45, 12, 89, 34, 67, 91, 23, 56].iter().cycle().take(100) {
                bitmap.set(black_box(*i));
            }
        })
    });

    // Benchmark different capacities
    let small_capacity = NonZeroUsize::new(64).unwrap();
    let small_bitmap = BitMap::new(small_capacity);
    c.bench_function("bitmap_small_capacity", |b| {
        b.iter(|| {
            for i in 0..50 {
                small_bitmap.set(black_box(i));
                black_box(small_bitmap.is_set(black_box(i)));
            }
        })
    });

    let large_capacity = NonZeroUsize::new(4096).unwrap();
    let large_bitmap = BitMap::new(large_capacity);
    c.bench_function("bitmap_large_capacity", |b| {
        b.iter(|| {
            for i in 0..50 {
                large_bitmap.set(black_box(i));
                black_box(large_bitmap.is_set(black_box(i)));
            }
        })
    });
}

criterion_group! {
    name = bitmap;
    config = Criterion::default().sample_size(100);
    targets =
        bitmap_benchmark,
}
