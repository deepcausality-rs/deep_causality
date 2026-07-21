/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DEC passive scalar advection–diffusion (`add-dec-scalar-transport-wall-heat-flux`).
//!
//! The references are analytic: a Fourier mode decays as `exp(−κk²t)`, a uniform advection translates
//! without diffusing, and a constant field is stationary because both `dT` and `Δ_dR T` vanish.

use deep_causality_cfd::DecScalarRate;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const TAU: f64 = core::f64::consts::TAU;

/// A periodic `n × n` torus of physical extent `2π`, so mode `k` has wavenumber `k`.
fn torus(n: usize) -> (Manifold<LatticeComplex<2, f64>, f64>, f64) {
    let h = TAU / n as f64;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [true, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    (
        Manifold::from_cubical_with_metric(lattice, data, metric, 0),
        h,
    )
}

fn zero_velocity(m: &Manifold<LatticeComplex<2, f64>, f64>) -> CausalTensor<f64> {
    let n1 = m.complex().num_cells(1);
    CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap()
}

/// Sample a function of vertex position onto the 0-cochain.
fn vertex_field<F: Fn(f64, f64) -> f64>(
    m: &Manifold<LatticeComplex<2, f64>, f64>,
    h: f64,
    f: F,
) -> CausalTensor<f64> {
    let vals: Vec<f64> = m
        .complex()
        .iter_cells(0)
        .map(|v| {
            let p = v.position();
            f(p[0] as f64 * h, p[1] as f64 * h)
        })
        .collect();
    let n = vals.len();
    CausalTensor::new(vals, vec![n]).unwrap()
}

#[test]
fn a_constant_field_is_stationary() {
    // The cheapest test that catches a grade or sign mix-up: for constant T both dT and Δ_dR T
    // vanish identically, so the rate must be exactly zero for *any* velocity and any κ. A term
    // wired at the wrong grade, or a Laplacian applied to the wrong operand, breaks this.
    let (m, _h) = torus(16);
    let rate = DecScalarRate::new(&m, 0.37).unwrap();
    let n0 = m.complex().num_cells(0);
    let t = CausalTensor::new(vec![2.5; n0], vec![n0]).unwrap();

    let n1 = m.complex().num_cells(1);
    // A non-zero, deliberately non-uniform velocity: the advective term must still vanish.
    let u: Vec<f64> = (0..n1).map(|i| 0.1 * ((i % 7) as f64 - 3.0)).collect();
    let u = CausalTensor::new(u, vec![n1]).unwrap();

    let r = rate.eval(&t, &u).unwrap();
    for (i, v) in r.as_slice().iter().enumerate() {
        assert!(v.abs() < 1e-12, "vertex {i}: constant field moved by {v}");
    }
}

#[test]
fn pure_diffusion_decays_a_fourier_mode_at_the_analytic_rate() {
    // T = cos(x) under ∂T/∂t = κ∇²T decays as exp(−κk²t) with k = 1. This pins the diffusive sign:
    // the opposite sign grows the mode instead, and no tolerance hides that.
    let n = 64usize;
    let (m, h) = torus(n);
    let kappa = 0.05;
    let rate = DecScalarRate::new(&m, kappa).unwrap();
    let u = zero_velocity(&m);

    let t0 = vertex_field(&m, h, |x, _y| x.cos());
    let dt = 0.2 * h * h / (4.0 * kappa);
    let steps = 40usize;

    let mut t = t0.clone();
    for _ in 0..steps {
        t = rate.step(&t, &u, dt).unwrap();
    }

    let elapsed = dt * steps as f64;
    let expected = (-kappa * elapsed).exp();
    // Amplitude via the peak of the (still cosine-shaped) field.
    let amp0 = t0.as_slice().iter().fold(0.0f64, |a, v| a.max(v.abs()));
    let amp = t.as_slice().iter().fold(0.0f64, |a, v| a.max(v.abs()));
    let measured = amp / amp0;

    assert!(
        (measured - expected).abs() < 2.0e-3,
        "mode decay: measured {measured:.6}, analytic exp(-kappa*k^2*t) = {expected:.6}"
    );
    assert!(
        measured < 1.0,
        "the mode must decay, not grow — a diffusive sign error shows up here as {measured:.6}"
    );
}

#[test]
fn diffusion_conserves_the_mean() {
    // ∇² is a divergence, so pure diffusion redistributes without creating: the mean is invariant.
    let (m, h) = torus(32);
    let rate = DecScalarRate::new(&m, 0.02).unwrap();
    let u = zero_velocity(&m);
    let t0 = vertex_field(&m, h, |x, y| 1.0 + 0.5 * x.cos() * y.sin());
    let mean0: f64 = t0.as_slice().iter().sum::<f64>() / t0.len() as f64;

    let mut t = t0;
    for _ in 0..30 {
        t = rate.step(&t, &u, 0.2 * h * h / (4.0 * 0.02)).unwrap();
    }
    let mean: f64 = t.as_slice().iter().sum::<f64>() / t.len() as f64;
    assert!(
        (mean - mean0).abs() < 1e-12,
        "diffusion moved the mean: {mean0} -> {mean}"
    );
}

#[test]
fn a_zero_diffusivity_and_zero_velocity_leave_the_field_untouched() {
    let (m, h) = torus(16);
    let rate = DecScalarRate::new(&m, 0.0).unwrap();
    let u = zero_velocity(&m);
    let t0 = vertex_field(&m, h, |x, y| x.sin() + y.cos());
    let t = rate.step(&t0, &u, 0.01).unwrap();
    for (a, b) in t0.as_slice().iter().zip(t.as_slice()) {
        assert_eq!(a, b, "an inert configuration must be bit-identical");
    }
}

#[test]
fn construction_refuses_a_negative_or_non_finite_diffusivity() {
    let (m, _h) = torus(8);
    assert!(
        DecScalarRate::new(&m, -1.0).is_err(),
        "a negative diffusivity is anti-diffusion and must be refused"
    );
    assert!(DecScalarRate::new(&m, f64::NAN).is_err());
    assert!(DecScalarRate::new(&m, f64::INFINITY).is_err());
    assert!(
        DecScalarRate::new(&m, 0.0).is_ok(),
        "zero diffusivity is valid (pure advection)"
    );
}

#[test]
fn the_rate_refuses_a_mis_sized_field() {
    let (m, _h) = torus(8);
    let rate = DecScalarRate::new(&m, 0.1).unwrap();
    let u = zero_velocity(&m);
    let wrong = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap();
    assert!(rate.eval(&wrong, &u).is_err());

    let n0 = m.complex().num_cells(0);
    let t = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    let wrong_u = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap();
    assert!(rate.eval(&t, &wrong_u).is_err());
}

// --- The Dirichlet wall condition (isothermal immersed body) --------------------------------------

use deep_causality_topology::{CutCell, CutCellRegistry};

/// A wall-bounded lattice with a solid block occupying the given top cells.
fn body_manifold(
    n: usize,
    solid: &[[usize; 2]],
) -> (
    Manifold<LatticeComplex<2, f64>, f64>,
    CutCellRegistry<2, f64>,
) {
    let lattice = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let cells: Vec<_> = lattice.iter_cells(2).collect();
    let mut reg = CutCellRegistry::<2, f64>::new();
    for base in solid {
        if let Some(id) = cells.iter().position(|c| c.position() == base) {
            reg.insert(id, CutCell::<2, f64>::solid(1.0));
        }
    }
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(1.0);
    (
        Manifold::from_cubical_with_metric(lattice, data, metric, 0),
        reg,
    )
}

#[test]
fn the_wall_holds_its_temperature() {
    let (m, reg) = body_manifold(8, &[[3, 3], [3, 4], [4, 3], [4, 4]]);
    let t_wall = 100.0;
    let rate = DecScalarRate::new(&m, 0.1)
        .unwrap()
        .with_isothermal_body(&reg, t_wall)
        .unwrap();
    assert!(
        !rate.pinned_vertices().is_empty(),
        "a solid block must pin vertices, else the condition is vacuous"
    );

    let u = zero_velocity(&m);
    let n0 = m.complex().num_cells(0);
    // Start the whole field cold; only the wall should stay hot.
    let mut t = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    for _ in 0..25 {
        t = rate.step(&t, &u, 0.05).unwrap();
    }
    for &v in rate.pinned_vertices() {
        assert!(
            (t.as_slice()[v] - t_wall).abs() < 1e-12,
            "pinned vertex {v} drifted to {} from T_w = {t_wall}",
            t.as_slice()[v]
        );
    }
}

#[test]
fn a_hot_body_heats_the_nearby_fluid_and_the_far_field_lags() {
    // The property that makes a wall gradient exist to measure at all.
    let (m, reg) = body_manifold(9, &[[4, 4]]);
    let t_wall = 50.0;
    let rate = DecScalarRate::new(&m, 0.1)
        .unwrap()
        .with_isothermal_body(&reg, t_wall)
        .unwrap();
    let u = zero_velocity(&m);
    let n0 = m.complex().num_cells(0);

    let mut t = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    for _ in 0..40 {
        t = rate.step(&t, &u, 0.05).unwrap();
    }

    // Vertex lookup by position.
    let idx = |px: usize, py: usize| -> usize {
        m.complex()
            .iter_cells(0)
            .position(|v| *v.position() == [px, py])
            .expect("vertex exists")
    };
    let near = t.as_slice()[idx(6, 4)]; // two cells from the body
    let far = t.as_slice()[idx(0, 0)]; // the opposite corner

    assert!(near > 0.0, "the near field must have warmed, got {near}");
    assert!(
        near > far,
        "the near field must lead the far field: near {near}, far {far}"
    );
    assert!(
        near < t_wall,
        "nothing may exceed the wall temperature under pure diffusion from it, got {near}"
    );
}

#[test]
fn without_a_body_nothing_is_pinned() {
    let (m, _reg) = body_manifold(8, &[]);
    let rate = DecScalarRate::new(&m, 0.1).unwrap();
    assert!(rate.pinned_vertices().is_empty());
    assert_eq!(rate.wall_temperature(), 0.0);
}

#[test]
fn a_non_finite_wall_temperature_is_refused() {
    let (m, reg) = body_manifold(8, &[[3, 3]]);
    let rate = DecScalarRate::new(&m, 0.1).unwrap();
    assert!(rate.with_isothermal_body(&reg, f64::NAN).is_err());
}
