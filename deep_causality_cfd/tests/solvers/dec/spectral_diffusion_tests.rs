/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the opt-in spectral viscous term: rounding-level agreement
//! with the operator path on tori (including anisotropic spacings), the
//! periodic-only construction boundary, and march-level equivalence (the
//! gate any future default-on must pass — states agreeing at rounding
//! imply identical convergence tables).

use deep_causality_cfd::{DecNsRate, DecNsSolver, VelocityOneForm};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.05;

fn manifold_with_metric<const D: usize>(
    lattice: LatticeComplex<D, f64>,
    metric: CubicalReggeGeometry<D, f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn random_cochain(len: usize, seed: u64) -> Vec<f64> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            2.0 * unit - 1.0
        })
        .collect()
}

/// The spectral and operator viscous evaluations must agree at rounding
/// level (scaled by the operator's `h⁻²` magnitudes).
fn assert_spectral_matches_operator<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    seed: u64,
) {
    let n1 = manifold.complex().num_cells(1);
    let u_raw = random_cochain(n1, seed);
    let u = VelocityOneForm::new(CausalTensor::new(u_raw, vec![n1]).unwrap(), manifold).unwrap();

    let operator = DecNsRate::new(manifold, NU, None).unwrap();
    let spectral = DecNsRate::new(manifold, NU, None)
        .unwrap()
        .with_spectral_diffusion()
        .unwrap();

    let a = operator.eval_unprojected(&u);
    let b = spectral.eval_unprojected(&u);
    for (i, (x, y)) in a
        .as_tensor()
        .as_slice()
        .iter()
        .zip(b.as_tensor().as_slice().iter())
        .enumerate()
    {
        assert!(
            (x - y).abs() < 1e-10,
            "edge {i}: operator {x} vs spectral {y}"
        );
    }
}

#[test]
fn spectral_matches_operator_2d_unit_torus() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(8),
        CubicalReggeGeometry::unit(),
    );
    assert_spectral_matches_operator(&m, 71);
}

#[test]
fn spectral_matches_operator_2d_anisotropic() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::new([4, 6], [true, true]),
        CubicalReggeGeometry::per_axis([0.5, 0.25]),
    );
    assert_spectral_matches_operator(&m, 73);
}

#[test]
fn spectral_matches_operator_3d_uniform() {
    let m = manifold_with_metric(
        LatticeComplex::<3, f64>::cubic_torus(4),
        CubicalReggeGeometry::uniform(0.5),
    );
    assert_spectral_matches_operator(&m, 79);
}

#[test]
fn spectral_construction_rejects_mixed_periodicity() {
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::new([8, 8], [true, false]),
        CubicalReggeGeometry::unit(),
    );
    let err = DecNsRate::new(&m, NU, None)
        .unwrap()
        .with_spectral_diffusion()
        .unwrap_err();
    assert!(format!("{err}").contains("periodic"), "{err}");
}

/// March-level gate: a multi-step solver run with spectral diffusion must
/// track the operator path at rounding, which implies identical observed
/// convergence orders on the Taylor–Green ladder.
#[test]
fn spectral_march_tracks_operator_march() {
    let n = 8usize;
    let m = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(n),
        CubicalReggeGeometry::unit(),
    );
    let n1 = m.complex().num_cells(1);
    // Divergence-free-ish smooth seed via the solver's own projection.
    let n0 = m.complex().num_cells(0);
    let two_pi = core::f64::consts::TAU;
    let mut vertex = vec![0.0; 2 * n0];
    for j in 0..n {
        for i in 0..n {
            let (x, y) = (i as f64 / n as f64, j as f64 / n as f64);
            let v = j * n + i;
            vertex[2 * v] = (two_pi * x).sin() * (two_pi * y).cos();
            vertex[2 * v + 1] = -(two_pi * x).cos() * (two_pi * y).sin();
        }
    }
    let vertex_t = CausalTensor::new(vertex, vec![2 * n0]).unwrap();

    let operator = DecNsSolver::new(&m, NU, 0.01, None).unwrap();
    let spectral = DecNsSolver::new(&m, NU, 0.01, None)
        .unwrap()
        .with_spectral_diffusion()
        .unwrap();

    let mut s_a = operator.seed_from_vertex_vectors(&vertex_t).unwrap();
    let mut s_b = spectral.seed_from_vertex_vectors(&vertex_t).unwrap();
    for _ in 0..5 {
        s_a = operator.step(&s_a).unwrap().into_state();
        s_b = spectral.step(&s_b).unwrap().into_state();
    }
    let _ = n1;
    for (i, (x, y)) in s_a
        .as_one_form()
        .as_slice()
        .iter()
        .zip(s_b.as_one_form().as_slice().iter())
        .enumerate()
    {
        assert!(
            (x - y).abs() < 1e-9,
            "edge {i} after 5 steps: operator {x} vs spectral {y}"
        );
    }
}
