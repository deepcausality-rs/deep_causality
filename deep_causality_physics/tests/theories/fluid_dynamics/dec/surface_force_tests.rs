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

use deep_causality_physics::{
    force_coefficient, fragment_area_vector, pressure_surface_force, viscous_surface_force,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, LatticeComplex, Manifold, Primitive,
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
