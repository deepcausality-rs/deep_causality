/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Generic real-field coverage for the condensed-matter kernels.
//!
//! The per-kernel suites (`moire_tests`, `phase_tests`, `qgt_tests`,
//! `wrappers_tests`) pin `f64` and assert exact numerics. This suite proves the
//! same kernels are genuinely generic over `R: RealField` by instantiating each
//! one at **both `f32` and `f64`**, and verifies the new constant accessors cast
//! the `f64` physical constants into the target precision. Tolerances are kept
//! loose so a single generic body is valid at both precisions.

use deep_causality_algebra::RealField;
use deep_causality_haft::Functor;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, Metric};
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    Concentration, Displacement, Energy, GRAPHENE_LATTICE_CONST, Length, Mobility, Momentum,
    OrderParameter, QuantumEigenvector, QuantumMetric, QuantumVelocity, REDUCED_PLANCK_CONSTANT,
    Ratio, Speed, Stiffness, TwistAngle, bistritzer_macdonald_kernel, cahn_hilliard_flux_kernel,
    effective_band_drude_weight_kernel, foppl_von_karman_strain_simple_kernel,
    ginzburg_landau_free_energy_kernel, graphene_lattice_const, quantum_geometric_tensor_kernel,
    real_from_f64, reduced_planck_constant,
};
use deep_causality_tensor::CausalTensor;

// ---------------------------------------------------------------------------
// Constant accessors
// ---------------------------------------------------------------------------

fn check_constants<R: RealField + FromPrimitive + core::fmt::Debug>() {
    // The typed accessor agrees with the raw `f64` constant cast through the
    // shared primitive.
    assert_eq!(
        graphene_lattice_const::<R>(),
        real_from_f64::<R>(GRAPHENE_LATTICE_CONST)
    );
    assert_eq!(
        reduced_planck_constant::<R>(),
        real_from_f64::<R>(REDUCED_PLANCK_CONSTANT)
    );
    // Both are strictly positive physical scales.
    assert!(graphene_lattice_const::<R>() > R::zero());
    assert!(reduced_planck_constant::<R>() > R::zero());
}

#[test]
fn constants_accessors_f32() {
    check_constants::<f32>();
}

#[test]
fn constants_accessors_f64() {
    check_constants::<f64>();
}

#[test]
fn real_from_f64_round_trips_both_precisions() {
    assert_eq!(real_from_f64::<f32>(2.5), 2.5_f32);
    assert_eq!(real_from_f64::<f64>(2.5), 2.5_f64);
}

// ---------------------------------------------------------------------------
// Bistritzer–MacDonald (the two-constant kernel)
// ---------------------------------------------------------------------------

fn run_bistritzer<R: RealField + FromPrimitive>() {
    let theta = TwistAngle::new(real_from_f64::<R>(0.02)).unwrap();
    let w = Energy::new(real_from_f64::<R>(0.11)).unwrap();
    let vf = Speed::new(real_from_f64::<R>(1.0e6)).unwrap();
    let k =
        Momentum::new(CausalMultiVector::new(vec![R::zero(); 8], Metric::Euclidean(3)).unwrap());

    let res = bistritzer_macdonald_kernel::<R>(theta, w, vf, k, 1);
    assert!(res.is_ok());
    let h = res.unwrap();
    assert_eq!(h.shape(), &[8, 8]);
    for c in h.as_slice() {
        assert!(c.re.is_finite() && c.im.is_finite());
    }
}

#[test]
fn bistritzer_f32() {
    run_bistritzer::<f32>();
}

#[test]
fn bistritzer_f64() {
    run_bistritzer::<f64>();
}

// ---------------------------------------------------------------------------
// Föppl–von Kármán (simple / local form)
// ---------------------------------------------------------------------------

fn run_foppl_simple<R: RealField + FromPrimitive + Default>() {
    let eps = CausalTensor::new(
        vec![
            real_from_f64::<R>(0.01),
            R::zero(),
            R::zero(),
            real_from_f64::<R>(0.02),
        ],
        vec![2, 2],
    )
    .unwrap();
    let disp = Displacement::new(eps);
    let e = Stiffness::new(real_from_f64::<R>(1.0e9)).unwrap();
    let nu = Ratio::new(real_from_f64::<R>(0.3)).unwrap();

    let res = foppl_von_karman_strain_simple_kernel::<R>(&disp, e, nu);
    assert!(res.is_ok());
    for s in res.unwrap().as_slice() {
        assert!(s.is_finite());
    }
}

#[test]
fn foppl_simple_f32() {
    run_foppl_simple::<f32>();
}

#[test]
fn foppl_simple_f64() {
    run_foppl_simple::<f64>();
}

// ---------------------------------------------------------------------------
// Ginzburg–Landau free energy
// ---------------------------------------------------------------------------

fn run_ginzburg<R: RealField + FromPrimitive + core::iter::Sum>() {
    let psi = OrderParameter::new(Complex::new(real_from_f64::<R>(1.0), R::zero()));
    let alpha = real_from_f64::<R>(1.0);
    let beta = real_from_f64::<R>(2.0);
    let grad = CausalMultiVector::new(vec![R::zero(); 4], Metric::Euclidean(2)).unwrap();
    let grad_c = CausalMultiVectorWitness::fmap(grad, |x| Complex::new(x, R::zero()));

    let res = ginzburg_landau_free_energy_kernel::<R>(psi, alpha, beta, &grad_c, None);
    assert!(res.is_ok());
    // F = α|ψ|² + (β/2)|ψ|⁴ = 1·1 + 1·1 = 2.
    let f = res.unwrap().value();
    let tol = real_from_f64::<R>(1.0e-4);
    assert!((f - real_from_f64::<R>(2.0)).abs() < tol);
}

#[test]
fn ginzburg_f32() {
    run_ginzburg::<f32>();
}

#[test]
fn ginzburg_f64() {
    run_ginzburg::<f64>();
}

// ---------------------------------------------------------------------------
// Cahn–Hilliard flux
// ---------------------------------------------------------------------------

fn run_cahn<R: RealField + FromPrimitive>() {
    let conc =
        Concentration::new(CausalTensor::new(vec![real_from_f64::<R>(0.5)], vec![1]).unwrap())
            .unwrap();
    let m = Mobility::new(real_from_f64::<R>(2.0)).unwrap();
    let grad = deep_causality_physics::ChemicalPotentialGradient::new(
        CausalTensor::new(vec![real_from_f64::<R>(10.0)], vec![1]).unwrap(),
    );

    let res = cahn_hilliard_flux_kernel::<R>(&conc, m, &grad);
    assert!(res.is_ok());
    // M(c) = 2·0.5·0.5 = 0.5, J = -0.5·10 = -5.
    let j = res.unwrap().data()[0];
    let tol = real_from_f64::<R>(1.0e-4);
    assert!((j - real_from_f64::<R>(-5.0)).abs() < tol);
}

#[test]
fn cahn_f32() {
    run_cahn::<f32>();
}

#[test]
fn cahn_f64() {
    run_cahn::<f64>();
}

// ---------------------------------------------------------------------------
// Quantum geometric tensor + Drude weight
// ---------------------------------------------------------------------------

fn run_qgt<R: RealField + FromPrimitive>() {
    let energies = CausalTensor::new(vec![R::zero(), R::one()], vec![2]).unwrap();
    let u = QuantumEigenvector::new(
        CausalTensor::new(vec![Complex::new(R::one(), R::zero()); 4], vec![2, 2]).unwrap(),
    );
    let v = QuantumVelocity::new(
        CausalTensor::new(
            vec![Complex::new(real_from_f64::<R>(0.1), real_from_f64::<R>(0.2)); 4],
            vec![2, 2],
        )
        .unwrap(),
    );
    let reg = real_from_f64::<R>(1.0e-9);

    let res = quantum_geometric_tensor_kernel::<R>(&energies, &u, &v, &v, 0, reg);
    assert!(res.is_ok());
    let q = res.unwrap();
    assert!(q.re.is_finite() && q.im.is_finite());
}

#[test]
fn qgt_f32() {
    run_qgt::<f32>();
}

#[test]
fn qgt_f64() {
    run_qgt::<f64>();
}

fn run_drude<R: RealField + FromPrimitive>() {
    let en = Energy::new(real_from_f64::<R>(1.0)).unwrap();
    let e0 = Energy::new(real_from_f64::<R>(0.5)).unwrap();
    let curv = real_from_f64::<R>(0.3);
    let g = QuantumMetric::new(real_from_f64::<R>(0.2)).unwrap();
    let a = Length::new(real_from_f64::<R>(1.0)).unwrap();

    let res = effective_band_drude_weight_kernel::<R>(en, e0, curv, g, a);
    assert!(res.is_ok());
    assert!(res.unwrap().value().is_finite());
}

#[test]
fn drude_f32() {
    run_drude::<f32>();
}

#[test]
fn drude_f64() {
    run_drude::<f64>();
}
