// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use criterion::{black_box, criterion_group, BenchmarkId, Criterion, SamplingMode, Throughput};
use dcl_data_structures::ring_buffer::prelude::*;
use std::sync::Arc;
use std::time::Duration;

const BUFFER_SIZE: usize = 65536;

const BATCH_SIZES: [u64; 3] = [1, 10, 100];

struct Checker;

impl EventHandler<i64> for Checker {
    fn handle_event(&self, event: &i64, seq: Sequence, _: bool) {
        assert_eq!(*event, seq as i64);
    }
}

fn ring_buffer_channel<S: Sequencer, F: FnOnce(&RingBuffer<i64, BUFFER_SIZE>) -> S>(
    n: u64,
    b: u64,
    f: F,
) {
    let data: Arc<RingBuffer<i64, BUFFER_SIZE>> = Arc::new(RingBuffer::new());
    let mut sequencer = f(data.as_ref());

    let gating_sequence = vec![sequencer.get_cursor()];
    let barrier = sequencer.create_barrier(&gating_sequence);
    let processor = BatchEventProcessor::create(Checker {});

    sequencer.add_gating_sequence(&processor.get_cursor());

    let executor = ThreadedExecutor::with_runnables(vec![processor.prepare(barrier, data.clone())]);

    let handle = executor.spawn();

    let mut counter = 0;
    for _ in 1..=n / b {
        let mut remainder = b;
        while remainder > 0 {
            let (start, end) = sequencer.next(remainder as usize);
            let count = end - start + 1;
            remainder -= count;
            for sequence in start..=end {
                counter += 1;
                unsafe { *data.get_mut(sequence) = sequence as i64 };
            }
            sequencer.publish(start, end);
        }
    }

    sequencer.drain();
    handle.join();
    assert_eq!(counter, n);
}

fn criterion_benchmark(c: &mut Criterion) {
    const N: u64 = 1_000_000;

    let mut group = c.benchmark_group("single_producer_spinning");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in BATCH_SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    ring_buffer_channel(black_box(N), *batch_size, |d| {
                        SingleProducerSequencer::new(d.buffer_size(), SpinLoopWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("single_producer_blocking");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in BATCH_SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    ring_buffer_channel(black_box(N), *batch_size, |d| {
                        SingleProducerSequencer::new(d.buffer_size(), BlockingWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("multi_producer_spinning");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in BATCH_SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    ring_buffer_channel(black_box(N), *batch_size, |d| {
                        MultiProducerSequencer::new(d.buffer_size(), SpinLoopWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();

    let mut group = c.benchmark_group("multi_producer_blocking");
    group.throughput(Throughput::Elements(N));
    group.warm_up_time(Duration::from_secs(10));
    group.sampling_mode(SamplingMode::Flat);
    for batch_size in BATCH_SIZES {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, batch_size| {
                b.iter(|| {
                    ring_buffer_channel(black_box(N), *batch_size, |d| {
                        MultiProducerSequencer::new(d.buffer_size(), BlockingWaitStrategy::new())
                    })
                });
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = ring_buffer;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark,
}
