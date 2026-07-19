/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Immersed-body **mask** encoding for the QTT solver.
//!
//! A body enters the periodic tensor-train flow by **volume penalization** (Brinkman): a mask field
//! `χ_body ∈ [0, 1]` (1 inside the body, 0 outside) multiplies a forcing term that drives the velocity
//! to the body velocity inside the solid — no cut cells, so everything stays on the uniform power-of-two
//! lattice the codec assumes.
//!
//! **Rank is the central risk** (gap-one note §3.4: boundary conditions are the fiddliest, rank-sensitive
//! part). A sharp 0/1 indicator is a 2-D step function — high tensor-train rank. So the mask is a
//! **smoothed volume fraction** `χ = ½(1 − tanh(d/δ))` over the signed distance `d` to the body surface,
//! smeared over a few cells `δ`; the smoothing both lowers the bond dimension and regularizes the
//! penalization. The resulting bond is inspectable on the returned train (its cores), so `δ` can be tuned
//! against rank.

use super::codec::quantize_2d;
use crate::CfdScalar;
use alloc::vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

/// Samples a scalar field `f(x, y)` over the `2^Lx × 2^Ly` grid of spacings `dx`/`dy` (node `(i, j)` at
/// `(i·dx, j·dy)`, row-major `[Nx, Ny]`) and quantizes it to a rounded tensor train — the generic mask
/// constructor (any smoothed indicator).
///
/// # Errors
/// Propagates codec errors.
pub fn mask_from_fn<R, F>(
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    f: F,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
    F: Fn(R, R) -> R,
{
    let (nx, ny) = (1usize << lx, 1usize << ly);
    let mut data = vec![R::zero(); nx * ny];
    for i in 0..nx {
        let x = R::from_usize(i).expect("a lattice index lifts into every real field") * dx;
        for j in 0..ny {
            let y = R::from_usize(j).expect("a lattice index lifts into every real field") * dy;
            data[i * ny + j] = f(x, y);
        }
    }
    let field = CausalTensor::new(data, vec![nx, ny])?;
    quantize_2d(&field, trunc)
}

/// A **smoothed plume-region** volume-fraction mask: an axis-aligned ellipse with semi-axes
/// `half_length` (along `x`, the retro-jet axis) and `max_radius` (along `y`), centered at
/// `(cx, cy)`, smoothed with the same `χ = ½(1 − tanh(d/δ))` skirt as [`body_mask_2d`]. The
/// distance proxy is the normalized-ellipse level set rescaled to length units by the smaller
/// semi-axis, `d ≈ (‖((x−cx)/a, (y−cy)/b)‖ − 1)·min(a, b)` — not a true signed distance, but
/// monotone through the boundary, which is all the smoothed skirt needs.
///
/// The semi-axes are the CFD-side shaping of an analytic retro-plume boundary (Cordell's
/// obstruction geometry: maximum plume radius and penetration length); deriving them from the
/// propulsion kernels is the caller's job — this constructor is pure geometry.
///
/// # Errors
/// Propagates codec errors.
#[allow(clippy::too_many_arguments)]
pub fn plume_mask_2d<R>(
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    cx: R,
    cy: R,
    half_length: R,
    max_radius: R,
    smoothing: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    let scale = if half_length < max_radius {
        half_length
    } else {
        max_radius
    };
    mask_from_fn(
        lx,
        ly,
        dx,
        dy,
        |x, y| {
            let ex = (x - cx) / half_length;
            let ey = (y - cy) / max_radius;
            let dist = ((ex * ex + ey * ey).sqrt() - R::one()) * scale;
            half * (R::one() - (dist / smoothing).tanh())
        },
        trunc,
    )
}

/// A **smoothed cylinder** volume-fraction mask: `χ = ½(1 − tanh(d/δ))` over the signed distance
/// `d = ‖(x, y) − (cx, cy)‖ − radius` to the cylinder surface, smeared over `smoothing` (= `δ`). Inside
/// the body (`d < 0`) `χ → 1`; outside `χ → 0`; on the surface `χ = ½`. Larger `smoothing` → lower bond.
///
/// # Errors
/// Propagates codec errors.
#[allow(clippy::too_many_arguments)]
pub fn body_mask_2d<R>(
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    cx: R,
    cy: R,
    radius: R,
    smoothing: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    mask_from_fn(
        lx,
        ly,
        dx,
        dy,
        |x, y| {
            let (ex, ey) = (x - cx, y - cy);
            let dist = (ex * ex + ey * ey).sqrt() - radius;
            half * (R::one() - (dist / smoothing).tanh())
        },
        trunc,
    )
}
