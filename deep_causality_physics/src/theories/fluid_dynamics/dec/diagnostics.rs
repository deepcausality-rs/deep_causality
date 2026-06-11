/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DEC-native integral diagnostics of an edge-cochain velocity field.
//!
//! All functions take the edge cochain as a raw tensor so both carriers
//! flow in without duplication: pass `state.as_one_form()` for a
//! [`crate::SolenoidalField`] or `v.as_tensor()` for a
//! [`crate::VelocityOneForm`]. The `dec_` prefix keeps the names disjoint
//! from the pointwise kernel vocabulary (`kinetic_energy`,
//! `enstrophy_density_kernel`, …) at the crate root.

use alloc::format;
use alloc::vec;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

/// Builds a scratch manifold carrying `field` in its grade-`k` slice, so
/// the data-reading topology operators (`d`, `δ`, `⋆`) can evaluate it.
fn scratch_manifold<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    field: &[R],
    k: usize,
) -> Result<Manifold<LatticeComplex<D, R>, R>, PhysicsError> {
    let complex = manifold.complex();
    let nk = complex.num_cells(k);
    if field.len() != nk {
        return Err(PhysicsError::DimensionMismatch(format!(
            "dec diagnostics: expected {nk} grade-{k} coefficients, got {}",
            field.len()
        )));
    }
    let metric = manifold
        .metric()
        .ok_or_else(|| {
            PhysicsError::TopologyError("dec diagnostics require a metric-bearing manifold".into())
        })?
        .clone();

    let offset: usize = (0..k).map(|g| complex.num_cells(g)).sum();
    let total: usize = (0..=D).map(|g| complex.num_cells(g)).sum();
    let mut slab = vec![R::zero(); total];
    slab[offset..offset + nk].copy_from_slice(field);
    let data = CausalTensor::new(slab, vec![total])
        // Coverage exemption: a 1-D tensor of the computed slab length
        // cannot fail to allocate.
        .expect("1-D tensor allocation cannot fail");
    Ok(Manifold::from_cubical_with_metric(
        complex.clone(),
        data,
        metric,
        0,
    ))
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
    let m_u = scratch_manifold(manifold, edge_form.as_slice(), 1)?;
    let star_u = m_u.hodge_star(1);
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
    let m_u = scratch_manifold(manifold, edge_form.as_slice(), 1)?;
    let du = m_u.exterior_derivative(1);
    let m_w = scratch_manifold(manifold, du.as_slice(), 2)?;
    let star_w = m_w.hodge_star(2);
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
    let m_u = scratch_manifold(manifold, edge_form.as_slice(), 1)?;
    let du = m_u.exterior_derivative(1);
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

/// Post-projection divergence residual `‖δu♭‖_∞` — the projection-
/// exactness witness.
///
/// # Errors
/// As [`dec_kinetic_energy`].
pub fn dec_divergence_residual<const D: usize, R: DecNsScalar>(
    manifold: &Manifold<LatticeComplex<D, R>, R>,
    edge_form: &CausalTensor<R>,
) -> Result<R, PhysicsError> {
    let m_u = scratch_manifold(manifold, edge_form.as_slice(), 1)?;
    let div = m_u.codifferential(1);
    Ok(div.as_slice().iter().fold(
        R::zero(),
        |acc, x| if x.abs() > acc { x.abs() } else { acc },
    ))
}
