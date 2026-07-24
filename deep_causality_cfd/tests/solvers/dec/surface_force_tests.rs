/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group F — the surface-force diagnostic (`add-slip-boundaries-and-surface-forces`).
//!
//! Pinned analytically (2D disk + 3D cylinder): the fragment normals close (`∮ n dA = 0`); a
//! uniform pressure yields zero net force; a linear pressure gradient yields `−∇p · V_solid`
//! (the buoyancy-like force); and the coefficient helper normalizes correctly. The viscous
//! (friction) traction is pinned against a linear shear over a flat plate (analytic wall shear)
//! and exercised at the full cylinder drag in the D2/D3 example.

use deep_causality_cfd::{
    force_coefficient, fragment_area_vector, pressure_surface_force, viscous_surface_force,
    wall_heat_flux,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCell, CutCellRegistry, CutFaceFragment, LatticeComplex,
    Manifold, Primitive, SourceGeometry,
};

/// Discrete solid volume in the registry (solid cells full; cut cells their dry part).
fn solid_volume_2d(registry: &CutCellRegistry<2, f64>, full: f64) -> f64 {
    registry
        .iter()
        .map(|(_, c)| {
            if c.class().is_solid() {
                full
            } else {
                c.full_volume() - c.fluid_volume()
            }
        })
        .sum()
}

#[test]
fn pressure_force_on_a_disk_is_consistent_2d() {
    let n = 24;
    let h = 1.0 / (n - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let metric = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball([0.5, 0.5], 0.2);
    let registry = CutCellRegistry::from_primitive(&lattice, &metric, &disk).unwrap();

    // The fragment normals of a closed body sum to (approximately) zero.
    let area_vec = fragment_area_vector(&registry);
    assert!(
        area_vec[0].abs() < 1e-9 && area_vec[1].abs() < 1e-9,
        "fragment normals do not close: {area_vec:?}"
    );

    // A uniform pressure exerts zero net force (closed surface).
    let uniform = pressure_surface_force(&registry, |_| 3.0);
    assert!(
        uniform[0].abs() < 1e-9 && uniform[1].abs() < 1e-9,
        "uniform pressure gave a net force: {uniform:?}"
    );

    // A linear pressure gradient p = g·x exerts F = −g·V_solid (buoyancy-like).
    let cell_centers: Vec<[f64; 2]> = lattice
        .iter_cells(2)
        .map(|c| {
            let p = c.position();
            [(p[0] as f64 + 0.5) * h, (p[1] as f64 + 0.5) * h]
        })
        .collect();
    let g = [2.0_f64, 0.0];
    let force = pressure_surface_force(&registry, |id| {
        g[0] * cell_centers[id][0] + g[1] * cell_centers[id][1]
    });
    let v_solid = solid_volume_2d(&registry, h * h);
    let expected = -g[0] * v_solid;
    // Cell-center pressure evaluation ⇒ an O(h) approximation to the exact surface integral.
    assert!(
        (force[0] - expected).abs() < 0.1 * expected.abs().max(1e-6),
        "linear-gradient drag {} far from analytic {expected}",
        force[0]
    );
    assert!(
        force[1].abs() < 0.05 * expected.abs(),
        "spurious lift {}",
        force[1]
    );
}

#[test]
fn pressure_force_on_a_cylinder_is_consistent_3d() {
    let n = 14;
    let h = 1.0 / (n - 1) as f64;
    // periodic along the cylinder axis (z) ⇒ no caps ⇒ the curved surface is closed.
    let lattice = LatticeComplex::<3, f64>::new([n, n, n], [false, false, true]);
    let metric = CubicalReggeGeometry::<3, f64>::uniform(h);
    let cyl = Primitive::<3, f64>::cylinder(2, [0.5, 0.5, 0.5], 0.2);
    let registry = CutCellRegistry::from_primitive(&lattice, &metric, &cyl).unwrap();

    let area_vec = fragment_area_vector(&registry);
    assert!(
        area_vec.iter().all(|a| a.abs() < 1e-9),
        "3D cylinder fragment normals do not close: {area_vec:?}"
    );

    let uniform = pressure_surface_force(&registry, |_| 2.0);
    assert!(
        uniform.iter().all(|f| f.abs() < 1e-9),
        "3D uniform pressure gave a net force: {uniform:?}"
    );
}

/// A no-slip linear shear `u = (a·(y − y_wall), 0)` over a flat plate (half-space solid `y ≤ y_wall`,
/// outward normal `(0, 1)`, the plate at rest so `u = 0` at the wall) has the exact wall traction
/// `τ·n = (μ·a, 0)` per unit area, so the net viscous force is `F = (μ·a·A, 0)` with `A` the total
/// cut-face area. The one-sided wall-normal gradient anchored at the fragment centroid (on the wall,
/// where `u = 0`) recovers `∂u_x/∂n = a` exactly on this linear field — independent of the sample
/// distance `Δh` — so the force matches the analytic traction to rounding. (`A` is read from the
/// fragments rather than assumed; fragment-area exactness is gated in the cut-cell tests.)
#[test]
fn viscous_force_on_a_linear_shear_matches_analytic_2d() {
    let n = 20;
    let h = 0.05; // domain 1.0 × 1.0
    let a = 2.0; // shear rate dU/dy
    let mu = 0.1; // dynamic viscosity (ρ = 1 ⇒ μ = ν)
    let y_wall = 0.525; // plate plane (matches the half-space offset)

    let lattice = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let base = CubicalReggeGeometry::<2, f64>::uniform(h);
    // Solid below, fluid above; the cut row sits mid-cell so the cells are genuinely partial.
    let plate = Primitive::<2, f64>::halfspace([0.0, 1.0], y_wall);
    let registry = CutCellRegistry::from_primitive(&lattice, &base, &plate).unwrap();

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, base, 0);

    // Seed the no-slip linear shear at the vertices (zero at the wall plane y_wall).
    let n0 = manifold.complex().num_cells(0);
    let mut vv = vec![0.0; 2 * n0];
    for (i, v) in manifold.complex().iter_cells(0).enumerate() {
        let y = v.position()[1] as f64 * h;
        vv[2 * i] = a * (y - y_wall); // u_x, zero at the wall
        vv[2 * i + 1] = 0.0; // u_y
    }
    let vertex_vectors = CausalTensor::new(vv, vec![2 * n0]).unwrap();
    let edge_form = manifold.de_rham(&vertex_vectors).unwrap();

    // Total cut-face area (every fragment normal is +y for a horizontal plate).
    let total_area: f64 = registry
        .iter()
        .flat_map(|(_, c)| c.fragments())
        .map(|f| f.area())
        .sum();

    let force = viscous_surface_force(&manifold, &registry, &edge_form, mu).unwrap();
    let expected_fx = mu * a * total_area;
    assert!(
        (force[0] - expected_fx).abs() < 1e-9,
        "viscous drag {} far from analytic {expected_fx}",
        force[0]
    );
    assert!(force[1].abs() < 1e-9, "spurious viscous lift {}", force[1]);
}

/// A quiescent field exerts no viscous force (zero gradient over every fragment).
#[test]
fn viscous_force_is_zero_for_a_quiescent_field_2d() {
    let n = 16;
    let h = 1.0 / (n - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let base = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball([0.5, 0.5], 0.2);
    let registry = CutCellRegistry::from_primitive(&lattice, &base, &disk).unwrap();

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, base, 0);

    let n1 = manifold.complex().num_cells(1);
    let edge_form = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let force = viscous_surface_force(&manifold, &registry, &edge_form, 0.1).unwrap();
    assert!(
        force.iter().all(|f| f.abs() < 1e-12),
        "quiescent field gave a viscous force: {force:?}"
    );
}

/// A per-edge (graded) geometry has no single per-axis spacing for the difference stencil, so the
/// viscous diagnostic rejects it rather than silently using a wrong `dx`.
#[test]
fn viscous_force_rejects_per_edge_geometry_2d() {
    let n = 8;
    let h = 1.0 / (n - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let uniform = CubicalReggeGeometry::<2, f64>::uniform(h);
    let disk = Primitive::<2, f64>::ball([0.5, 0.5], 0.2);
    let registry = CutCellRegistry::from_primitive(&lattice, &uniform, &disk).unwrap();

    // A per-edge geometry with all lengths equal is still represented as `PerEdge`, so
    // `axis_lengths()` returns `None`.
    let n1 = lattice.num_cells(1);
    let per_edge = CubicalReggeGeometry::<2, f64>::from_edge_lengths(vec![h; n1]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, per_edge, 0);

    let edge_form = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    assert!(viscous_surface_force(&manifold, &registry, &edge_form, 0.1).is_err());
}

#[test]
fn force_coefficient_normalizes_correctly() {
    // C = F / (½ρU²A), ρ = 1.
    let c = force_coefficient(2.7_f64, 3.0, 0.2);
    let expected = 2.7 / (0.5 * 9.0 * 0.2);
    assert!((c - expected).abs() < 1e-12);
}

// --- Fourier-law wall heat flux over cut-cell fragments -------------------------------------------
// (`add-dec-scalar-transport-wall-heat-flux`). The thermal counterpart of the viscous traction
// above: same fragments, same one-sided wall-normal reconstruction, temperature in place of
// velocity. The reference is **exact**, not tolerated — under a linear profile the reconstruction
// reproduces ∂T/∂n with no truncation error, so q = −k·G·A holds to machine precision, and holds at
// every spacing. That pins the sign convention, the area weighting, the wall-normal step and the
// multilinear sample at once: if any one is wrong the number moves by a factor, not a rounding.

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
