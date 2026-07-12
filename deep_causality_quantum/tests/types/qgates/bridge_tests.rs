/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{QuantumOps, dirac_bracket_kernel};
use deep_causality_tensor::{CausalTensor, Tensor};

type C = Complex<f64>;

fn pseudo_random_ket(d: usize, seed: usize) -> CausalTensor<C> {
    let v: Vec<C> = (0..d)
        .map(|i| {
            let x = (((i + seed) * 2654435761) % 1000) as f64 / 500.0 - 1.0;
            let y = (((i + seed) * 40503 + 12345) % 1000) as f64 / 500.0 - 1.0;
            Complex::new(x, y)
        })
        .collect();
    CausalTensor::new(v, vec![d, 1]).unwrap()
}

/// The raw column inner product `kᴴ(φ)·k(ψ)` on the tensor side.
fn column_inner(phi: &CausalTensor<C>, psi: &CausalTensor<C>) -> C {
    let p = phi.dagger().unwrap().matmul(psi).unwrap();
    p.as_slice()[0]
}

#[test]
fn test_dirac_bracket_matches_column_product_negative_signature() {
    // Cl(0,2) and Cl(0,10): the metric-correct Dirac product equals the raw
    // column inner product of the to_ket bridge.
    for (metric, d) in [(Metric::NonEuclidean(2), 2), (Metric::NonEuclidean(10), 32)] {
        let kv = pseudo_random_ket(d, 1);
        let kw = pseudo_random_ket(d, 9);
        let phi = HilbertState::<f64>::from_ket(&kv, metric).unwrap();
        let psi = HilbertState::<f64>::from_ket(&kw, metric).unwrap();

        let col = column_inner(&phi.to_ket().unwrap(), &psi.to_ket().unwrap());
        let dirac = dirac_bracket_kernel(&phi, &psi).unwrap();
        assert!(
            (col.re - dirac.re).abs() < 1e-10 && (col.im - dirac.im).abs() < 1e-10,
            "{:?}: column ({}, {}) != dirac ({}, {})",
            metric,
            col.re,
            col.im,
            dirac.re,
            dirac.im
        );
    }
}

#[test]
fn test_dirac_bracket_matches_column_product_positive_signature() {
    // Euclidean(2): dag-based bracket is the Dirac product; dirac_bracket_kernel
    // and QuantumOps::bracket agree with the column product.
    let metric = Metric::Euclidean(2);
    let kv = pseudo_random_ket(2, 3);
    let kw = pseudo_random_ket(2, 11);
    let phi = HilbertState::<f64>::from_ket(&kv, metric).unwrap();
    let psi = HilbertState::<f64>::from_ket(&kw, metric).unwrap();

    let col = column_inner(&phi.to_ket().unwrap(), &psi.to_ket().unwrap());
    let dirac = dirac_bracket_kernel(&phi, &psi).unwrap();
    let dag_bracket = phi.mv().bracket(psi.mv());

    assert!((col.re - dirac.re).abs() < 1e-12 && (col.im - dirac.im).abs() < 1e-12);
    assert!((dag_bracket.re - dirac.re).abs() < 1e-12 && (dag_bracket.im - dirac.im).abs() < 1e-12);
}

#[test]
fn test_reversion_bracket_degenerates_on_negative_signature_mli() {
    // Documented boundary (established numerically, add-quantum-crate 2.1):
    // on Cl(0,n), QuantumOps::bracket (reversion + conjugation) vanishes
    // identically on the minimal left ideal — the Clifford-conjugation form
    // is the metric-correct Dirac product there.
    let metric = Metric::NonEuclidean(2);
    let kv = pseudo_random_ket(2, 5);
    let phi = HilbertState::<f64>::from_ket(&kv, metric).unwrap();
    let psi = HilbertState::<f64>::from_ket(&pseudo_random_ket(2, 6), metric).unwrap();

    let dag_bracket = phi.mv().bracket(psi.mv());
    assert!(
        dag_bracket.re.abs() < 1e-12 && dag_bracket.im.abs() < 1e-12,
        "reversion bracket unexpectedly non-degenerate: ({}, {})",
        dag_bracket.re,
        dag_bracket.im
    );

    // While the metric-correct product is non-trivial for these states.
    let dirac = dirac_bracket_kernel(&phi, &psi).unwrap();
    assert!(dirac.re.abs() + dirac.im.abs() > 1e-6);
}

#[test]
fn test_dirac_bracket_unit_ket_normalization() {
    // A unit column gives ⟨ψ|ψ⟩ = 1 under the bridge normalization.
    let metric = Metric::NonEuclidean(4); // D = 4
    let mut v = vec![Complex::new(0.0, 0.0); 4];
    v[2] = Complex::new(0.6, 0.0);
    v[3] = Complex::new(0.0, 0.8);
    let k = CausalTensor::new(v, vec![4, 1]).unwrap();
    let psi = HilbertState::<f64>::from_ket(&k, metric).unwrap();

    let norm = dirac_bracket_kernel(&psi, &psi).unwrap();
    assert!(
        (norm.re - 1.0).abs() < 1e-12 && norm.im.abs() < 1e-12,
        "⟨ψ|ψ⟩ = ({}, {})",
        norm.re,
        norm.im
    );
}

#[test]
fn test_dirac_bracket_rejects_metric_mismatch_and_mixed_signature() {
    let a =
        HilbertState::<f64>::from_ket(&pseudo_random_ket(2, 1), Metric::NonEuclidean(2)).unwrap();
    let b = HilbertState::<f64>::from_ket(&pseudo_random_ket(2, 1), Metric::Euclidean(2)).unwrap();
    assert!(dirac_bracket_kernel(&a, &b).is_err());

    // Mixed signature (Minkowski) is not supported by the ket bridge product.
    let data = vec![Complex::new(1.0, 0.0); 16];
    let m = HilbertState::<f64>::new(data.clone(), Metric::Minkowski(4)).unwrap();
    let m2 = HilbertState::<f64>::new(data, Metric::Minkowski(4)).unwrap();
    assert!(dirac_bracket_kernel(&m, &m2).is_err());
}
