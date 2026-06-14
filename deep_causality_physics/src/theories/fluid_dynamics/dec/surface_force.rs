/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Surface-force diagnostics on an immersed cut body (CFD Stage-4
//! `add-slip-boundaries-and-surface-forces`).
//!
//! The hydrodynamic force on an immersed body is `F = ∮_S (−p n + μ(∇u+∇uᵀ)·n) dA`. This module
//! ships the **pressure** term — `F_p = −∮ p n dA`, integrated over the body's `CutFaceFragment`s
//! (each carrying area and outward normal) with a caller-supplied per-cell pressure — and the
//! drag/lift coefficient helper. The **viscous (friction)** traction lands with the cylinder
//! validation (it is only meaningfully verified against the reference drag, not a fast analytic
//! gate). Read-only on a field snapshot; not on the per-step hot path.

use deep_causality_topology::CutCellRegistry;

use crate::theories::fluid_dynamics::dec::DecNsScalar;

/// The **pressure** surface force on an immersed cut body: `F_p = −∮ p n dA`, summed over every
/// cut cell's fragments with the cell pressure from `cell_pressure` (keyed by the cell's
/// `iter_cells(D)` index — the registry's `CellId`).
///
/// For a closed body the net force is `−p₀ ∮ n dA = 0` under a uniform pressure `p₀` (the
/// fragment normals sum to zero), and `−∇p · V_solid` under a linear pressure field.
pub fn pressure_surface_force<const D: usize, R: DecNsScalar>(
    registry: &CutCellRegistry<D, R>,
    cell_pressure: impl Fn(usize) -> R,
) -> [R; D] {
    let mut force = [R::zero(); D];
    for (&cell_id, cut) in registry.iter() {
        let p = cell_pressure(cell_id);
        for fragment in cut.fragments() {
            let normal = fragment.outward_normal();
            let area = fragment.area();
            for (d, f) in force.iter_mut().enumerate() {
                *f -= p * normal[d] * area;
            }
        }
    }
    force
}

/// The net outward area vector `∮ n dA` of a body's fragments — zero for a closed surface (a
/// consistency check on the fragment normals, independent of any field).
pub fn fragment_area_vector<const D: usize, R: DecNsScalar>(
    registry: &CutCellRegistry<D, R>,
) -> [R; D] {
    let mut area_vector = [R::zero(); D];
    for (_, cut) in registry.iter() {
        for fragment in cut.fragments() {
            let normal = fragment.outward_normal();
            let area = fragment.area();
            for (d, a) in area_vector.iter_mut().enumerate() {
                *a += normal[d] * area;
            }
        }
    }
    area_vector
}

/// A nondimensional force coefficient `C = F / (½ ρ U² A)` at `ρ = 1` (drag with the streamwise
/// component, lift with the transverse, given the reference speed `u_ref` and frontal area/length
/// `reference_area`).
pub fn force_coefficient<R: DecNsScalar>(force_component: R, u_ref: R, reference_area: R) -> R {
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    force_component / (half * u_ref * u_ref * reference_area)
}
