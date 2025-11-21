/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use criterion::{Criterion, criterion_group, criterion_main};
use deep_causality_multivector::{
    CausalMultiVector, HilbertState, Metric, QuantumGates, QuantumOps,
};
use deep_causality_multivector::{MultiVector, PGA3DMultiVector};
use deep_causality_num::Complex64;
use std::hint::black_box;

const DIM: usize = 10; // For Cl(0,10)
const SIZE: usize = 1 << DIM; // 1024

// Helper to create a HilbertState for Cl(0,10) for benchmarks
fn create_cl0_10_hilbert_state(scalar_val: Complex64) -> HilbertState {
    let mut data = vec![Complex64::new(0.0, 0.0); SIZE];
    data[0] = scalar_val;
    // Fill some other parts to make it a non-trivial state
    if SIZE > 1 {
        data[1] = Complex64::new(0.5, 0.5); // e1 component
    }
    if SIZE > 3 {
        data[3] = Complex64::new(0.2, -0.3); // e12 component
    }

    HilbertState::new_spin10(data).unwrap()
}

fn bench_geometric_product_euclidean_2d(c: &mut Criterion) {
    let m = Metric::Euclidean(2);
    let a = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let b = CausalMultiVector::new(vec![4.0, 3.0, 2.0, 1.0], m).unwrap();

    c.bench_function("geometric_product_euclidean_2d", |bencher| {
        bencher.iter(|| black_box(a.clone()) * black_box(b.clone()))
    });
}

fn bench_geometric_product_pga_3d(c: &mut Criterion) {
    // PGA 3D is 4D algebra (16 elements)
    let p = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);
    let t = PGA3DMultiVector::translator(2.0, 0.0, 0.0);

    c.bench_function("geometric_product_pga_3d", |bencher| {
        bencher.iter(|| black_box(t.clone()) * black_box(p.clone()))
    });
}

fn bench_addition_euclidean_3d(c: &mut Criterion) {
    let m = Metric::Euclidean(3);
    let data = vec![1.0; 8];
    let a = CausalMultiVector::new(data.clone(), m).unwrap();
    let b = CausalMultiVector::new(data, m).unwrap();

    c.bench_function("addition_euclidean_3d", |bencher| {
        bencher.iter(|| black_box(a.clone()) + black_box(b.clone()))
    });
}

fn bench_reversion_pga_3d(c: &mut Criterion) {
    let t = PGA3DMultiVector::translator(2.0, 0.0, 0.0);

    c.bench_function("reversion_pga_3d", |bencher| {
        bencher.iter(|| black_box(t.clone()).reversion())
    });
}

// --- Quantum Ops Benchmarks ---

fn bench_quantum_ops_dag(c: &mut Criterion) {
    let state = create_cl0_10_hilbert_state(Complex64::new(1.0, 0.0));
    c.bench_function("quantum_ops_dag", |bencher| {
        bencher.iter(|| black_box(state.clone()).dag())
    });
}

fn bench_quantum_ops_bracket(c: &mut Criterion) {
    let state1 = create_cl0_10_hilbert_state(Complex64::new(1.0, 0.0));
    let state2 = create_cl0_10_hilbert_state(Complex64::new(0.5, 0.5));
    c.bench_function("quantum_ops_bracket", |bencher| {
        bencher.iter(|| black_box(state1.clone()).bracket(black_box(&state2)))
    });
}

fn bench_quantum_ops_expectation_value(c: &mut Criterion) {
    let state = create_cl0_10_hilbert_state(Complex64::new(1.0, 0.0));
    let operator = HilbertState::gate_z(); // Use a predefined gate as an operator
    c.bench_function("quantum_ops_expectation_value", |bencher| {
        bencher.iter(|| black_box(state.clone()).expectation_value(black_box(&operator)))
    });
}

fn bench_quantum_ops_normalize(c: &mut Criterion) {
    let state = create_cl0_10_hilbert_state(Complex64::new(2.0, 0.0)); // Unnormalized state
    c.bench_function("quantum_ops_normalize", |bencher| {
        bencher.iter(|| black_box(state.clone()).normalize())
    });
}

criterion_group!(
    benches,
    bench_geometric_product_euclidean_2d,
    bench_geometric_product_pga_3d,
    bench_addition_euclidean_3d,
    bench_reversion_pga_3d,
    bench_quantum_ops_dag,
    bench_quantum_ops_bracket,
    bench_quantum_ops_expectation_value,
    bench_quantum_ops_normalize
);
criterion_main!(benches);
