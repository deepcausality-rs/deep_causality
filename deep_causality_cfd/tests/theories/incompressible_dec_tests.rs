/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `DecIncompressible` — the DEC-native incompressible Navier–Stokes regime as a
//! `FluidTheory`. It wraps a validated `DecNsRate` with the projection options; `FluidTheory::rate`
//! reads `ν` from the ambient (between Rk4 steps) and returns the projected, divergence-free rate.

use deep_causality_cfd::{Ambient, DecIncompressible, DecNsRate, FluidTheory, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

const NU: f64 = 0.01;

fn unit_manifold(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn zero_velocity(manifold: &Manifold<LatticeComplex<2, f64>, f64>) -> VelocityOneForm<f64> {
    let n1 = manifold.complex().num_cells(1);
    VelocityOneForm::new(
        CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap(),
        manifold,
    )
    .unwrap()
}

#[test]
fn test_new_and_rate_getter() {
    let manifold = unit_manifold(8);
    let rate = DecNsRate::new(&manifold, NU, None).unwrap();
    let theory = DecIncompressible::new(rate, HodgeDecomposeOptions::default());
    assert_eq!(theory.rate().nu(), NU);
}

#[test]
fn test_fluid_theory_rate_reads_nu_from_ambient() {
    let manifold = unit_manifold(8);
    let rate = DecNsRate::new(&manifold, NU, None).unwrap();
    let theory = DecIncompressible::new(rate, HodgeDecomposeOptions::default());

    let u = zero_velocity(&manifold);
    let ambient = Ambient::new(0.05_f64, 0.0, None);

    let result = FluidTheory::rate(&theory, &u, &ambient).expect("projected rate evaluates");

    // The ambient viscosity is read into the underlying rate (interior mutability between steps).
    assert_eq!(theory.rate().nu(), 0.05);
    // A divergence-free projection of the zero state is again the zero rate.
    for &c in result.as_tensor().as_slice() {
        assert!(c.abs() < 1e-12, "rate of rest should be zero, got {c}");
    }
}

#[test]
fn test_debug_impl() {
    let manifold = unit_manifold(4);
    let rate = DecNsRate::new(&manifold, NU, None).unwrap();
    let theory = DecIncompressible::new(rate, HodgeDecomposeOptions::default());
    assert!(format!("{theory:?}").contains("DecIncompressible"));
}
