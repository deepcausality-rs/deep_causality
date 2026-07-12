/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    DensityMatrix, Projection, born_projective_prob, born_projective_probability,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn c(re: f64, im: f64) -> C {
    Complex::new(re, im)
}

fn ket(v: Vec<C>) -> CausalTensor<C> {
    CausalTensor::new(v, vec![2, 1]).unwrap()
}

fn proj(v: Vec<C>) -> Projection<f64, 2> {
    Projection::from_ket(&ket(v)).unwrap()
}

fn rho_from_ket(v: Vec<C>) -> DensityMatrix<f64> {
    DensityMatrix::from_ket(&ket(v)).unwrap()
}

#[test]
fn test_born_rule_on_pure_states() {
    let rho0 = rho_from_ket(vec![c(1., 0.), c(0., 0.)]); // |0><0|
    let p0 = proj(vec![c(1., 0.), c(0., 0.)]);
    let p1 = proj(vec![c(0., 0.), c(1., 0.)]);
    let pplus = proj(vec![c(1., 0.), c(1., 0.)]);

    // <0|P0|0> = 1, <0|P1|0> = 0, <0|P+|0> = 1/2.
    assert!((born_projective_probability(&rho0, &p0).unwrap() - 1.0).abs() < 1e-12);
    assert!(born_projective_probability(&rho0, &p1).unwrap().abs() < 1e-12);
    assert!((born_projective_probability(&rho0, &pplus).unwrap() - 0.5).abs() < 1e-12);
}

#[test]
fn test_born_probabilities_sum_to_one_over_a_resolution() {
    // P and P^⊥ partition unity: Tr(Pρ) + Tr(P^⊥ρ) = 1.
    let rho = rho_from_ket(vec![c(0.6, 0.), c(0.0, 0.8)]);
    let p = proj(vec![c(1., 0.), c(1., 0.)]); // |+>
    let a = born_projective_probability(&rho, &p).unwrap();
    // The complement projection's probability.
    let m = CausalTensor::new(
        vec![c(0.5, 0.), c(-0.5, 0.), c(-0.5, 0.), c(0.5, 0.)],
        vec![2, 2],
    )
    .unwrap(); // |-><-|
    let p_perp = Projection::<f64, 2>::new(m).unwrap();
    let b = born_projective_probability(&rho, &p_perp).unwrap();
    assert!((a + b - 1.0).abs() < 1e-12, "a+b = {}", a + b);
}

#[test]
fn test_born_on_maximally_mixed_state() {
    // I/2: any rank-1 projection has probability 1/2.
    let m = CausalTensor::new(
        vec![c(0.5, 0.), c(0., 0.), c(0., 0.), c(0.5, 0.)],
        vec![2, 2],
    )
    .unwrap();
    let rho = DensityMatrix::new(m).unwrap();
    for p in [
        proj(vec![c(1., 0.), c(0., 0.)]),
        proj(vec![c(1., 0.), c(1., 0.)]),
        proj(vec![c(1., 0.), c(0., 1.)]),
    ] {
        assert!((born_projective_probability(&rho, &p).unwrap() - 0.5).abs() < 1e-12);
    }
}

#[test]
fn test_born_prob_verdict_boundary() {
    // The Prob MV-algebra verdict at the measurement boundary.
    let rho0 = rho_from_ket(vec![c(1., 0.), c(0., 0.)]);
    let pplus = proj(vec![c(1., 0.), c(1., 0.)]);
    let prob = born_projective_prob(&rho0, &pplus).unwrap();
    assert!((prob.0 - 0.5).abs() < 1e-12);
}

#[test]
fn test_born_rejects_dim_mismatch() {
    let rho3 = {
        let m = CausalTensor::new(
            vec![
                c(1., 0.),
                c(0., 0.),
                c(0., 0.),
                c(0., 0.),
                c(0., 0.),
                c(0., 0.),
                c(0., 0.),
                c(0., 0.),
                c(0., 0.),
            ],
            vec![3, 3],
        )
        .unwrap();
        DensityMatrix::new(m).unwrap()
    };
    let p0 = proj(vec![c(1., 0.), c(0., 0.)]);
    assert!(born_projective_probability(&rho3, &p0).is_err());
}
