/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fourier-law wall heat flux over cut-cell fragments
//! (`add-dec-scalar-transport-wall-heat-flux`).
//!
//! The primary reference is **exact**, not tolerated: under a linear temperature profile the
//! one-sided wall-normal reconstruction `(T_sample − T_w)/Δh` reproduces `∂T/∂n` with no truncation
//! error, so `q = −k·G·A` holds to machine precision. That single case pins the sign convention, the
//! area weighting, the wall-normal step and the multilinear sample at once — if any one of them is
//! wrong, the number moves by a factor, not by a rounding.

use deep_causality_cfd::wall_heat_flux;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, CutFaceFragment, LatticeComplex,
    Manifold, SourceGeometry,
};

const N: usize = 8;
/// Deliberately **not** 1.0. With unit spacing the wall-normal step `Δh` equals 1 and dividing by it
/// is a no-op, so a test built on `H = 1` cannot see whether `Δh` is applied at all — an injection
/// that deletes the division passes unnoticed. A non-unit spacing makes the factor observable.
const H: f64 = 0.25;

/// A manifold carrying one cut cell at `base` with a single planar fragment of `area`, whose
/// outward normal is `+x` and whose centroid sits on that cell's `+x` face at mid height.
fn plate_manifold(
    area: f64,
) -> (
    Manifold<LatticeComplex<2, f64>, f64>,
    CutCellRegistry<2, f64>,
) {
    let lattice = LatticeComplex::<2, f64>::new([N, N], [false, false]);
    let cells: Vec<_> = lattice.iter_cells(2).collect();
    let base = [3usize, 3usize];
    let cell_id = cells
        .iter()
        .position(|c| *c.position() == base)
        .expect("the base cell exists");

    let fragment = CutFaceFragment::<2, f64>::new(
        area,
        [1.0, 0.0], // outward normal: +x
        // Physical lattice coordinates (`index · H`), not index units. Cell [3,3] spans
        // x ∈ [3H, 4H], so its +x face is at x = 4H with the fragment centroid at mid height 3.5H.
        [4.0 * H, 3.5 * H],
        SourceGeometry::Plane,
    );
    let mut registry = CutCellRegistry::<2, f64>::new();
    registry.insert(
        cell_id,
        CutCell::<2, f64>::cut(1.0, 0.5, [[1.0, 1.0], [1.0, 1.0]], vec![fragment]),
    );

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric = CubicalReggeGeometry::<2, f64>::uniform(H);
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);
    (manifold, registry)
}

/// The 0-cochain `T(x, y) = t_wall + gradient·(x − x_wall)` — linear in the wall-normal direction.
fn linear_field(
    m: &Manifold<LatticeComplex<2, f64>, f64>,
    t_wall: f64,
    gradient: f64,
    x_wall: f64,
) -> CausalTensor<f64> {
    let vals: Vec<f64> = m
        .complex()
        .iter_cells(0)
        .map(|v| {
            let p = v.position();
            t_wall + gradient * (p[0] as f64 * H - x_wall)
        })
        .collect();
    let n = vals.len();
    CausalTensor::new(vals, vec![n]).unwrap()
}

#[test]
fn a_linear_profile_reproduces_fouriers_law_exactly() {
    // ∂T/∂n = G by construction, so q = −k·G·A with no discretisation error to absorb.
    let (m, reg) = plate_manifold(1.0);
    let (t_wall, gradient, k) = (10.0, 2.0, 3.0);
    let t = linear_field(&m, t_wall, gradient, 4.0 * H);

    let q = wall_heat_flux(&m, &reg, &t, t_wall, k).unwrap();
    let expected = -k * gradient * 1.0;
    assert!(
        (q - expected).abs() < 1e-12,
        "Fourier flux: got {q}, analytic −k·G·A = {expected}"
    );
}

#[test]
fn the_flux_scales_linearly_with_area_conductivity_and_gradient() {
    // Each factor enters once. A squared or dropped factor shows up here as a ratio far from 1.
    let (t_wall, x_wall) = (10.0, 4.0 * H);
    let base = {
        let (m, reg) = plate_manifold(1.0);
        let t = linear_field(&m, t_wall, 2.0, x_wall);
        wall_heat_flux(&m, &reg, &t, t_wall, 3.0).unwrap()
    };

    let double_area = {
        let (m, reg) = plate_manifold(2.0);
        let t = linear_field(&m, t_wall, 2.0, x_wall);
        wall_heat_flux(&m, &reg, &t, t_wall, 3.0).unwrap()
    };
    let double_k = {
        let (m, reg) = plate_manifold(1.0);
        let t = linear_field(&m, t_wall, 2.0, x_wall);
        wall_heat_flux(&m, &reg, &t, t_wall, 6.0).unwrap()
    };
    let double_grad = {
        let (m, reg) = plate_manifold(1.0);
        let t = linear_field(&m, t_wall, 4.0, x_wall);
        wall_heat_flux(&m, &reg, &t, t_wall, 3.0).unwrap()
    };

    for (name, v) in [
        ("area", double_area),
        ("conductivity", double_k),
        ("gradient", double_grad),
    ] {
        assert!(
            (v / base - 2.0).abs() < 1e-12,
            "doubling {name} must double the flux: ratio {}",
            v / base
        );
    }
}

#[test]
fn an_isothermal_field_carries_no_flux() {
    let (m, reg) = plate_manifold(1.0);
    let t_wall = 7.5;
    let t = linear_field(&m, t_wall, 0.0, 4.0 * H);
    let q = wall_heat_flux(&m, &reg, &t, t_wall, 3.0).unwrap();
    assert!(
        q.abs() < 1e-14,
        "an isothermal field must carry no flux, got {q}"
    );
}

#[test]
fn the_flux_reverses_with_the_temperature_difference() {
    // The sign convention: with n the body's outward normal, positive q is heat leaving the wall.
    // A fluid colder than the wall (negative gradient outward) must give q > 0.
    let (m, reg) = plate_manifold(1.0);
    let (t_wall, k) = (10.0, 3.0);

    let hot_fluid =
        wall_heat_flux(&m, &reg, &linear_field(&m, t_wall, 2.0, 4.0 * H), t_wall, k).unwrap();
    let cold_fluid = wall_heat_flux(
        &m,
        &reg,
        &linear_field(&m, t_wall, -2.0, 4.0 * H),
        t_wall,
        k,
    )
    .unwrap();

    assert!(
        cold_fluid > 0.0,
        "a fluid colder than the wall must draw heat out of it (q > 0), got {cold_fluid}"
    );
    assert!(
        hot_fluid < 0.0,
        "a fluid hotter than the wall must drive heat into it (q < 0), got {hot_fluid}"
    );
    assert!(
        (hot_fluid + cold_fluid).abs() < 1e-12,
        "reversing ΔT must reverse the sign and preserve the magnitude: {hot_fluid} vs {cold_fluid}"
    );
}

#[test]
fn the_flux_refuses_malformed_inputs() {
    let (m, reg) = plate_manifold(1.0);
    let t = linear_field(&m, 10.0, 2.0, 4.0 * H);

    assert!(
        wall_heat_flux(&m, &reg, &t, 10.0, f64::NAN).is_err(),
        "a non-finite conductivity must be refused"
    );
    assert!(wall_heat_flux(&m, &reg, &t, f64::INFINITY, 3.0).is_err());

    let wrong = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap();
    assert!(
        wall_heat_flux(&m, &reg, &wrong, 10.0, 3.0).is_err(),
        "a scalar that is not one value per vertex must be refused"
    );
}

#[test]
fn an_empty_registry_carries_no_flux() {
    let (m, _reg) = plate_manifold(1.0);
    let empty = CutCellRegistry::<2, f64>::new();
    let t = linear_field(&m, 10.0, 2.0, 4.0 * H);
    let q = wall_heat_flux(&m, &empty, &t, 10.0, 3.0).unwrap();
    assert_eq!(q, 0.0, "no body means no wetted surface and no flux");
}
