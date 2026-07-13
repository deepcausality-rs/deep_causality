/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{
    CausalMultiVector, HilbertState, KET_COLUMN, Metric, MultiVector,
};
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

type C = Complex<f64>;

fn unit_ket(d: usize, hot: usize) -> CausalTensor<C> {
    let mut v = vec![Complex::new(0.0, 0.0); d];
    v[hot] = Complex::new(1.0, 0.0);
    CausalTensor::new(v, vec![d, 1]).unwrap()
}

fn pseudo_random_ket(d: usize) -> CausalTensor<C> {
    let v: Vec<C> = (0..d)
        .map(|i| {
            let x = ((i * 2654435761) % 1000) as f64 / 500.0 - 1.0;
            let y = ((i * 40503 + 12345) % 1000) as f64 / 500.0 - 1.0;
            Complex::new(x, y)
        })
        .collect();
    CausalTensor::new(v, vec![d, 1]).unwrap()
}

fn max_col_diff(a: &CausalTensor<C>, b: &CausalTensor<C>) -> f64 {
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| ((x.re - y.re).powi(2) + (x.im - y.im).powi(2)).sqrt())
        .fold(0.0, f64::max)
}

#[test]
fn test_ket_column_convention() {
    assert_eq!(KET_COLUMN, 0);
}

#[test]
fn test_to_ket_from_ket_round_trip_cl02() {
    let metric = Metric::NonEuclidean(2); // Cl(0,2), D = 2
    let v = pseudo_random_ket(2);
    let psi = HilbertState::<f64>::from_ket(&v, metric).unwrap();
    let back = psi.to_ket().unwrap();
    assert!(max_col_diff(&v, &back) < 1e-12, "to_ket(from_ket(v)) != v");
}

#[test]
// Ignored under Miri: Cl(0,10) has 2^10 = 1024 blades and the D=32 matrix-rep
// round-trip is ~9 s natively -> hours under Miri. The from_ket/to_ket UB path
// is covered under Miri by the small Cl(0,2) case; full check runs in normal CI.
#[cfg_attr(miri, ignore)]
fn test_to_ket_from_ket_round_trip_cl010() {
    let metric = Metric::NonEuclidean(10); // Cl(0,10), D = 32
    let v = pseudo_random_ket(32);
    let psi = HilbertState::<f64>::from_ket(&v, metric).unwrap();
    let back = psi.to_ket().unwrap();
    assert!(
        max_col_diff(&v, &back) < 1e-10,
        "Cl(0,10) round trip drifted: {}",
        max_col_diff(&v, &back)
    );
}

#[test]
fn test_from_ket_to_ket_idempotent_on_mli() {
    // A state built by from_ket lies in the minimal left ideal; the bridge is
    // the identity on it.
    let metric = Metric::NonEuclidean(4); // D = 4
    let v = pseudo_random_ket(4);
    let psi = HilbertState::<f64>::from_ket(&v, metric).unwrap();
    let psi2 = HilbertState::<f64>::from_ket(&psi.to_ket().unwrap(), metric).unwrap();
    let d = psi
        .mv()
        .data()
        .iter()
        .zip(psi2.mv().data())
        .map(|(a, b)| ((a.re - b.re).powi(2) + (a.im - b.im).powi(2)).sqrt())
        .fold(0.0, f64::max);
    assert!(
        d < 1e-12,
        "from_ket∘to_ket not idempotent on the MLI: {}",
        d
    );
}

#[test]
fn test_operator_action_commutes_with_bridge_euclidean() {
    // For a EUCLIDEAN metric, to_matrix is an algebra homomorphism, so left
    // multiplication intertwines with the bridge:
    // to_ket(g·ψ) == to_matrix(g)·to_ket(ψ).
    let metric = Metric::Euclidean(2);
    let v = pseudo_random_ket(2);
    let psi = HilbertState::<f64>::from_ket(&v, metric).unwrap();

    // An arbitrary multivector g (not in the ideal).
    let g_data: Vec<C> = (0..4)
        .map(|i| Complex::new(0.3 * (i as f64) - 0.5, 0.1 * (i as f64 * i as f64) - 0.2))
        .collect();
    let g = CausalMultiVector::new(g_data, metric).unwrap();

    let lhs = HilbertState::<f64>::from_multivector(g.geometric_product(psi.mv()))
        .to_ket()
        .unwrap();
    let rhs = g.to_matrix().matmul(&psi.to_ket().unwrap()).unwrap();
    assert!(
        max_col_diff(&lhs, &rhs) < 1e-12,
        "bridge does not intertwine the operator action: {}",
        max_col_diff(&lhs, &rhs)
    );
}

#[test]
fn test_matrix_rep_is_linear_only_for_negative_signature() {
    // Documented boundary (established numerically, add-quantum-crate 2.1):
    // for a NEGATIVE-signature metric Cl(0,n), the gamma basis squares to +1
    // while the multivector generators square to −1, so to_matrix is a LINEAR
    // (trace-orthogonal) isomorphism but NOT an algebra homomorphism. A
    // multivector gate on a Cl(0,n) ket must be applied as a geometric
    // product (native) or built directly as a matrix — never converted via
    // to_matrix and matmul'd.
    let metric = Metric::NonEuclidean(2);
    let mut e0_data: Vec<C> = vec![Complex::new(0.0, 0.0); 4];
    e0_data[1] = Complex::new(1.0, 0.0); // the generator e0
    let e0 = CausalMultiVector::new(e0_data, metric).unwrap();

    // Multivector product: e0² = −1 (Cl(0,2)).
    let mv_sq = e0.geometric_product(&e0);
    assert!((mv_sq.data()[0].re + 1.0).abs() < 1e-12);

    // Matrix product: γ0² = +1 (the Euclidean gamma basis).
    let mat_sq = e0.to_matrix().matmul(&e0.to_matrix()).unwrap();
    assert!((mat_sq.as_slice()[0].re - 1.0).abs() < 1e-12);
}

#[test]
fn test_density_matrix_from_unit_ket_is_valid_state() {
    // ρ = k·kᴴ for a normalized ket: Hermitian, unit trace, idempotent (purity).
    let metric = Metric::NonEuclidean(4); // D = 4
    let k = unit_ket(4, 1);
    let psi = HilbertState::<f64>::from_ket(&k, metric).unwrap();
    let ket = psi.to_ket().unwrap();

    let rho = ket.matmul(&ket.dagger().unwrap()).unwrap();
    let r = rho.as_slice();
    let d = 4usize;

    // Hermitian
    for i in 0..d {
        for j in 0..d {
            let a = r[i * d + j];
            let b = r[j * d + i];
            assert!((a.re - b.re).abs() < 1e-12 && (a.im + b.im).abs() < 1e-12);
        }
    }
    // Unit trace
    let tr: f64 = (0..d).map(|i| r[i * d + i].re).sum();
    assert!((tr - 1.0).abs() < 1e-12, "trace = {}", tr);
    // Idempotent (rank-1 purity): ρ² = ρ
    let rho2 = rho.matmul(&rho).unwrap();
    for (a, b) in rho2.as_slice().iter().zip(r) {
        assert!((a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12);
    }
}

#[test]
fn test_odd_metric_rejected() {
    // Cl(3) is odd-dimensional: to_matrix is not a bijection, the bridge errors.
    let data = vec![Complex::new(1.0, 0.0); 8];
    let psi =
        HilbertState::<f64>::new(data, Metric::Euclidean(3)).expect("state construction works");
    assert!(psi.to_ket().is_err());

    let v = unit_ket(2, 0);
    assert!(HilbertState::<f64>::from_ket(&v, Metric::Euclidean(3)).is_err());
}

#[test]
fn test_wrong_ket_shape_rejected() {
    let metric = Metric::NonEuclidean(4); // D = 4
    let too_short = unit_ket(2, 0); // D = 2 column against a D = 4 metric
    assert!(HilbertState::<f64>::from_ket(&too_short, metric).is_err());

    let square = CausalTensor::new(vec![Complex::new(1.0, 0.0); 16], vec![4, 4]).unwrap();
    assert!(HilbertState::<f64>::from_ket(&square, metric).is_err());
}

#[test]
fn test_flat_vector_shape_accepted() {
    // from_ket accepts both [D] and [D, 1].
    let metric = Metric::NonEuclidean(2);
    let flat = CausalTensor::new(
        vec![Complex::new(0.6, 0.0), Complex::new(0.0, 0.8)],
        vec![2],
    )
    .unwrap();
    let psi = HilbertState::<f64>::from_ket(&flat, metric).unwrap();
    let back = psi.to_ket().unwrap();
    assert_eq!(back.shape(), &[2, 1]);
    assert!((back.as_slice()[0].re - 0.6).abs() < 1e-12);
    assert!((back.as_slice()[1].im - 0.8).abs() < 1e-12);
}
