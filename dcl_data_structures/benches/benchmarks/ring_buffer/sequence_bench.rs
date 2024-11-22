use criterion::{black_box, criterion_group, Criterion};
use dcl_data_structures::ring_buffer::sequence::sequence::AtomicSequence;

fn sequence_benchmark(c: &mut Criterion) {
    let sequence = AtomicSequence::default();

    // Benchmark get operation
    c.bench_function("sequence_get", |b| {
        b.iter(|| {
            black_box(sequence.get());
        })
    });

    // Benchmark set operation
    c.bench_function("sequence_set", |b| {
        b.iter(|| {
            sequence.set(black_box(42));
        })
    });

    // Benchmark compare_exchange operation (success case)
    c.bench_function("sequence_compare_exchange_success", |b| {
        sequence.set(0);
        b.iter(|| {
            black_box(sequence.compare_exchange(0, 1));
            sequence.set(0); // Reset for next iteration
        })
    });

    // Benchmark compare_exchange operation (failure case)
    c.bench_function("sequence_compare_exchange_failure", |b| {
        sequence.set(1);
        b.iter(|| {
            black_box(sequence.compare_exchange(0, 2));
        })
    });

    // Benchmark sequence creation from value
    c.bench_function("sequence_from_value", |b| {
        b.iter(|| {
            black_box(AtomicSequence::from(black_box(42)));
        })
    });
}

criterion_group! {
    name = sequence;
    config = Criterion::default().sample_size(100);
    targets = sequence_benchmark
}
