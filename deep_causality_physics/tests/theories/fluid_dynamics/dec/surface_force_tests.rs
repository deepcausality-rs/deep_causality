/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Group F — the surface-force diagnostic (`add-slip-boundaries-and-surface-forces`).
//!
//! Pinned analytically (2D disk + 3D cylinder): the fragment normals close (`∮ n dA = 0`); a
//! uniform pressure yields zero net force; a linear pressure gradient yields `−∇p · V_solid`
//! (the buoyancy-like force); and the coefficient helper normalizes correctly. The viscous
//! (friction) traction is validated with the cylinder drag in the D2/D3 example.

use deep_causality_physics::{force_coefficient, fragment_area_vector, pressure_surface_force};
use deep_causality_topology::{CubicalReggeGeometry, CutCellRegistry, LatticeComplex, Primitive};

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

#[test]
fn force_coefficient_normalizes_correctly() {
    // C = F / (½ρU²A), ρ = 1.
    let c = force_coefficient(2.7_f64, 3.0, 0.2);
    let expected = 2.7 / (0.5 * 9.0 * 0.2);
    assert!((c - expected).abs() < 1e-12);
}
