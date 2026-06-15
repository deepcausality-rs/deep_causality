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
//! **viscous (friction)** term `F_μ = ∮ μ(∇u+∇uᵀ)·n dA`, reconstructed from the velocity field at
//! the cut cells, plus the drag/lift coefficient helper. Both are read-only on a field snapshot;
//! neither is on the per-step hot path.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{CutCellRegistry, LatticeCell, LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
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

/// The **viscous (friction)** surface force on an immersed cut body:
/// `F_μ = ∮_S μ(∇u + ∇uᵀ)·n dA`, summed over every cut cell's fragments.
///
/// The velocity field is the edge 1-cochain `edge_form` (pass `state.as_one_form()`). It is
/// `sharp`-reconstructed to vertex vectors, and the velocity gradient `∇u` at each cut cell is
/// formed by central differences of that vertex field at the cell's base vertex (one-sided at a
/// domain boundary). The symmetric viscous stress `τ = μ(∇u + ∇uᵀ)` is then contracted with each
/// fragment's outward normal and integrated over its area. With `ρ = 1` the dynamic viscosity `mu`
/// equals the kinematic `ν` the solver carries.
///
/// This is a finite-difference wall-shear estimate, second-order on a smooth field and exact on a
/// linear one; on a cut cell it reads the actual no-slip shear (solid-side vertices sit at zero).
/// Its accuracy at the body is resolution-bound — it is meaningfully checked against the reference
/// drag, not a fast analytic gate.
///
/// # Errors
/// * `PhysicsError::TopologyError` wrapping a `sharp` failure (wrong edge count or missing metric).
/// * `PhysicsError::TopologyError` when the manifold's geometry is not axis-aligned (per-edge
///   graded geometry has no single per-axis spacing for the difference stencil).
pub fn viscous_surface_force<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    registry: &CutCellRegistry<D, R>,
    edge_form: &CausalTensor<R>,
    mu: R,
) -> Result<[R; D], PhysicsError> {
    use alloc::format;

    let dx = manifold
        .metric()
        .and_then(|g| g.axis_lengths())
        .ok_or_else(|| {
            PhysicsError::TopologyError(
                "viscous_surface_force requires an axis-aligned geometry (per-axis spacing)".into(),
            )
        })?;

    // Vertex velocity vectors (layout `vertex * D + axis`), keyed by lattice position.
    let vertex_vectors = manifold
        .sharp(edge_form)
        .map_err(|e| PhysicsError::TopologyError(format!("sharp failed: {e}")))?;
    let complex = manifold.complex();
    let velocity: BTreeMap<[usize; D], [R; D]> = complex
        .iter_cells(0)
        .zip(vertex_vectors.as_slice().chunks_exact(D))
        .map(|(vertex, v)| {
            let mut vec = [R::zero(); D];
            vec.copy_from_slice(v);
            (*vertex.position(), vec)
        })
        .collect();

    // The top cells (D-cells) in registry-key order, so a `CellId` resolves to its position.
    let cells: Vec<LatticeCell<D>> = complex.iter_cells(D).collect();

    let mut force = [R::zero(); D];
    for (&cell_id, cut) in registry.iter() {
        let base = *cells[cell_id].position();
        let grad = velocity_gradient(&velocity, &base, &dx);
        // Symmetric viscous stress τ = μ(∇u + ∇uᵀ): τ[i][j] = μ(∂u_i/∂x_j + ∂u_j/∂x_i).
        for fragment in cut.fragments() {
            let normal = fragment.outward_normal();
            let area = fragment.area();
            for (i, f) in force.iter_mut().enumerate() {
                let mut traction_i = R::zero();
                for (j, &n_j) in normal.iter().enumerate() {
                    traction_i += mu * (grad[i][j] + grad[j][i]) * n_j;
                }
                *f += traction_i * area;
            }
        }
    }
    Ok(force)
}

/// Velocity gradient `grad[i][j] = ∂u_i/∂x_j` at lattice vertex `base`: a central difference of
/// the vertex velocity field, degrading to a one-sided difference where a neighbour falls outside
/// the domain (the `or(center)` fallback with a halved step count). The base vertex of a cut cell
/// always has at least one in-axis neighbour, so the difference is well defined.
fn velocity_gradient<const D: usize, R: DecNsScalar>(
    velocity: &BTreeMap<[usize; D], [R; D]>,
    base: &[usize; D],
    dx: &[R; D],
) -> [[R; D]; D] {
    let zero = [R::zero(); D];
    let center = velocity.get(base);
    let mut grad = [[R::zero(); D]; D];
    for j in 0..D {
        let mut hi_pos = *base;
        hi_pos[j] += 1;
        let lo_pos = base[j].checked_sub(1).map(|v| {
            let mut p = *base;
            p[j] = v;
            p
        });
        let hi = velocity.get(&hi_pos);
        let lo = lo_pos.as_ref().and_then(|p| velocity.get(p));
        // One in-axis step on each side that exists; missing sides fall back to the centre value
        // (one-sided) and the step count drops to 1 so the denominator stays consistent.
        let steps = hi.is_some() as usize + lo.is_some() as usize;
        let plus = hi.or(center).unwrap_or(&zero);
        let minus = lo.or(center).unwrap_or(&zero);
        let denom = R::from_usize(steps).unwrap_or_else(R::one) * dx[j];
        for i in 0..D {
            grad[i][j] = (plus[i] - minus[i]) / denom;
        }
    }
    grad
}

/// A nondimensional force coefficient `C = F / (½ ρ U² A)` at `ρ = 1` (drag with the streamwise
/// component, lift with the transverse, given the reference speed `u_ref` and frontal area/length
/// `reference_area`).
pub fn force_coefficient<R: DecNsScalar>(force_component: R, u_ref: R, reference_area: R) -> R {
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    force_component / (half * u_ref * u_ref * reference_area)
}
