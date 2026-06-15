/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DEC-native integral diagnostics of an edge-cochain velocity field.
//!
//! All functions take the edge cochain as a raw tensor so both carriers
//! flow in without duplication: pass `state.as_one_form()` for a
//! [`deep_causality_physics::SolenoidalField`] or `v.as_tensor()` for a
//! [`deep_causality_physics::VelocityOneForm`]. The `dec_` prefix keeps the names disjoint
//! from the pointwise kernel vocabulary (`kinetic_energy`,
//! `enstrophy_density_kernel`, …) at the crate root.

use alloc::collections::BTreeMap;
use alloc::format;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;
use deep_causality_physics::PhysicsError;

/// Validates a grade-`k` field against the manifold (length and metric
/// presence) before the `_of` operators evaluate it directly — no scratch
/// manifold and no data-slab copy.
fn validate_field<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    field: &[R],
    k: usize,
) -> Result<(), PhysicsError> {
    let nk = manifold.complex().num_cells(k);
    if field.len() != nk {
        return Err(PhysicsError::DimensionMismatch(format!(
            "dec diagnostics: expected {nk} grade-{k} coefficients, got {}",
            field.len()
        )));
    }
    if manifold.metric().is_none() {
        return Err(PhysicsError::TopologyError(
            "dec diagnostics require a metric-bearing manifold".into(),
        ));
    }
    Ok(())
}

/// Kinetic energy `E = ½ Σ_e u_e (⋆u)_e` — the discrete `½ ∫ u♭ ∧ ⋆u♭`
/// through the diagonal Hodge star.
///
/// # Errors
/// `PhysicsError::DimensionMismatch` on a wrong edge count;
/// `PhysicsError::TopologyError` when the manifold has no metric.
pub fn dec_kinetic_energy<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
) -> Result<R, PhysicsError> {
    validate_field(manifold, edge_form.as_slice(), 1)?;
    let star_u = manifold.hodge_star_of(edge_form.as_slice(), 1);
    let half = R::from_f64(0.5)
        // Coverage exemption: 0.5 lifts into every real field.
        .expect("0.5 lifts into R");
    let sum = edge_form
        .as_slice()
        .iter()
        .zip(star_u.as_slice().iter())
        .fold(R::zero(), |acc, (u, su)| acc + *u * *su);
    Ok(sum * half)
}

/// Enstrophy `Z = ½ Σ_f ω_f (⋆ω)_f` with `ω = d u♭`.
///
/// # Errors
/// As [`dec_kinetic_energy`].
pub fn dec_enstrophy<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
) -> Result<R, PhysicsError> {
    validate_field(manifold, edge_form.as_slice(), 1)?;
    let du = manifold.exterior_derivative_of(edge_form.as_slice(), 1);
    let star_w = manifold.hodge_star_of(du.as_slice(), 2);
    let half = R::from_f64(0.5)
        // Coverage exemption: 0.5 lifts into every real field.
        .expect("0.5 lifts into R");
    let sum = du
        .as_slice()
        .iter()
        .zip(star_w.as_slice().iter())
        .fold(R::zero(), |acc, (w, sw)| acc + *w * *sw);
    Ok(sum * half)
}

/// Helicity `H = Σ_c (u♭ ∧ du♭)_c` — the top-form cochain of the wedge,
/// whose coefficients are already cell integrals. Three-dimensional flows
/// only: in any other dimension the quantity is meaningless and the call
/// is rejected.
///
/// # Errors
/// `PhysicsError::DimensionMismatch` when `D != 3` or on a wrong edge
/// count; `PhysicsError::TopologyError` when the manifold has no metric.
pub fn dec_helicity<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
) -> Result<R, PhysicsError> {
    if D != 3 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "helicity is a 3D invariant (u♭ ∧ du♭ is a 3-form); got D = {D}"
        )));
    }
    validate_field(manifold, edge_form.as_slice(), 1)?;
    let du = manifold.exterior_derivative_of(edge_form.as_slice(), 1);
    let h = manifold
        .wedge(edge_form, 1, &du, 2)
        .map_err(|e| PhysicsError::TopologyError(format!("helicity wedge failed: {e}")))?;
    Ok(h.as_slice().iter().fold(R::zero(), |acc, x| acc + *x))
}

/// Maximum pointwise speed: `sharp` recovers vertex vectors
/// (layout `vertex * D + axis`), the maximum Euclidean norm is returned.
///
/// # Errors
/// `PhysicsError::TopologyError` wrapping a `sharp` failure (wrong edge
/// count or missing metric).
pub fn dec_max_speed<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
) -> Result<R, PhysicsError> {
    let vertex_vectors = manifold
        .sharp(edge_form)
        .map_err(|e| PhysicsError::TopologyError(format!("sharp failed: {e}")))?;
    let v = vertex_vectors.as_slice();
    let mut max_sq = R::zero();
    for chunk in v.chunks_exact(D) {
        let norm_sq = chunk.iter().fold(R::zero(), |acc, x| acc + *x * *x);
        if norm_sq > max_sq {
            max_sq = norm_sq;
        }
    }
    Ok(max_sq.sqrt())
}

/// The velocity vector at the physical point `p` (in spacing units), by `sharp`-
/// reconstructing the vertex vector field and multilinearly interpolating. Used by the
/// Flow wake-probe (Strouhal signal) and centerline (Ghia profile) observations; a
/// read-only point query, not on the step hot path. Corners outside the domain
/// contribute zero (the wall / no-slip value at a boundary line).
///
/// # Errors
/// `PhysicsError::TopologyError` wrapping a `sharp` failure, or when the manifold's
/// geometry is not axis-aligned (no single per-axis spacing).
pub fn dec_sample_velocity<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
    p: &[R; D],
) -> Result<[R; D], PhysicsError> {
    let dx = manifold
        .metric()
        .and_then(|g| g.axis_lengths())
        .ok_or_else(|| {
            PhysicsError::TopologyError(
                "dec_sample_velocity requires an axis-aligned geometry (per-axis spacing)".into(),
            )
        })?;
    let vertex_vectors = manifold
        .sharp(edge_form)
        .map_err(|e| PhysicsError::TopologyError(format!("sharp failed: {e}")))?;
    let velocity: BTreeMap<[usize; D], [R; D]> = manifold
        .complex()
        .iter_cells(0)
        .zip(vertex_vectors.as_slice().chunks_exact(D))
        .map(|(vertex, v)| {
            let mut vec = [R::zero(); D];
            vec.copy_from_slice(v);
            (*vertex.position(), vec)
        })
        .collect();

    let mut lo = [0usize; D];
    let mut frac = [R::zero(); D];
    for j in 0..D {
        let g = p[j] / dx[j];
        let mut k = 0usize;
        while R::from_usize(k + 1).unwrap_or_else(R::one) <= g {
            k += 1;
        }
        lo[j] = k;
        frac[j] = g - R::from_usize(k).unwrap_or_else(R::zero);
    }
    let mut out = [R::zero(); D];
    for corner in 0..(1usize << D) {
        let mut pos = [0usize; D];
        let mut weight = R::one();
        for j in 0..D {
            let bit = (corner >> j) & 1;
            pos[j] = lo[j] + bit;
            weight *= if bit == 1 {
                frac[j]
            } else {
                R::one() - frac[j]
            };
        }
        if let Some(v) = velocity.get(&pos) {
            for (o, &vi) in out.iter_mut().zip(v.iter()) {
                *o += weight * vi;
            }
        }
    }
    Ok(out)
}

/// Post-projection divergence residual `‖δu♭‖_∞` — the projection-
/// exactness witness.
///
/// # Errors
/// As [`dec_kinetic_energy`].
pub fn dec_divergence_residual<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
) -> Result<R, PhysicsError> {
    validate_field(manifold, edge_form.as_slice(), 1)?;
    let div = manifold.codifferential_of(edge_form.as_slice(), 1);
    Ok(div.as_slice().iter().fold(
        R::zero(),
        |acc, x| if x.abs() > acc { x.abs() } else { acc },
    ))
}
