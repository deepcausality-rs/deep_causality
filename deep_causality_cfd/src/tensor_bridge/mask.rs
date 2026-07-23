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

use super::codec::{dequantize_2d, quantize_2d};
use crate::CfdScalar;
use alloc::vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Truncation};

/// A mask excursion beyond this fraction of the `[0, 1]` range is treated as a wrong mask, not
/// truncation noise, and fails construction. It separates rounding artifact from modelling error:
/// truncation excursion grows with grid resolution at a fixed bond cap, so the same geometry gives
/// `min χ ≈ −1.4e-3` at bond 24 on the shipped `L = 8` cylinder ladder but `min χ ≈ −0.15` at bond 4
/// there — the coarse cap on a fine grid *is* a wrong mask, and `0.05` correctly refuses it while
/// admitting the bond-24/48 ladder rungs. Relative, so it means the same thing for any smoothed
/// indicator; a caller wanting a valid mask at an aggressive cap must raise the cap, not the threshold.
const MASK_GROSS_EXCURSION: f64 = 0.05;

/// Enforce `χ ∈ [0, 1]` on a quantized mask as far as a lossy tensor-train allows.
///
/// The analytic smoothed indicator is in `[0, 1]` by construction, but tensor-train rounding drives
/// the quantized field outside it — measured `min χ = −1.78e-3` across 188 cells at bond cap 4. A
/// negative `χ` inverts the sign of the penalization forcing `−(1/η)·χ·(u−u_b)`, so the invariant is
/// not cosmetic.
///
/// **What is and is not achievable.** Clamping the dequantized field to `[0, 1]` and re-quantizing
/// removes the *gross* excursion but not the last of it: the second rounding reintroduces a smaller
/// one (measured `−1.78e-3 → −1.21e-3` at bond 4 on a 32² grid), because a fixed-rank tensor train
/// cannot represent an arbitrary clamped field exactly. So this clamps to remove the bulk and
/// **rejects** an excursion large enough to be a wrong mask rather than rounding noise; the residual
/// is inherent to lossy compression and is orders below the body value `χ = 1`.
///
/// The residual is **not zero even at the operating cap.** On the shipped `L = 8` cylinder ladder the
/// η sweep runs at bond cap 48, where the post-clamp mask measures `min χ ≈ −7e-7` — non-negative only
/// *to truncation tolerance*, not exactly. So a bounded truncation-noise negative *can* reach the
/// penalization forcing `−(1/η)·χ·(u−u_b)` and flip its local sign in a handful of skirt cells; at
/// `−7e-7` against `χ = 1` this is numerical noise, not a modelling error, and the `MASK_GROSS_EXCURSION`
/// gate is what guarantees it stays that small. The honest invariant is *"bounded within truncation
/// tolerance"*, not *"non-negative"* — enforcing exact non-negativity would need a per-use clamp the
/// tensor-train forcing path does not have.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] if the mask leaves `[0, 1]` by more than
/// [`MASK_GROSS_EXCURSION`] — a mask that wrong is a modelling error, not truncation.
fn clamp_mask_to_unit_interval<R>(
    mask: CausalTensorTrain<R>,
    lx: usize,
    ly: usize,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let (nx, ny) = (1usize << lx, 1usize << ly);
    let dense = dequantize_2d(&mask, lx, ly)?;
    let one = R::one();
    let gross = R::from_f64(MASK_GROSS_EXCURSION).unwrap_or_else(R::one);

    let mut worst_below = R::zero();
    let mut worst_above = R::zero();
    let mut clamped = vec::Vec::with_capacity(nx * ny);
    for &v in dense.as_slice() {
        if v < R::zero() && (R::zero() - v) > worst_below {
            worst_below = R::zero() - v;
        }
        if v > one && (v - one) > worst_above {
            worst_above = v - one;
        }
        clamped.push(if v < R::zero() {
            R::zero()
        } else if v > one {
            one
        } else {
            v
        });
    }

    if worst_below > gross || worst_above > gross {
        return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
            "body mask leaves [0, 1] by more than {MASK_GROSS_EXCURSION} of the range (min excess \
             below 0 = {worst_below:?}, max excess above 1 = {worst_above:?}) — this is a wrong mask, \
             not tensor-train rounding noise"
        )));
    }

    // Re-quantize the clamped field. The residual excursion this reintroduces is bounded by the
    // truncation tolerance and far below the gross threshold checked above.
    let field = CausalTensor::new(clamped, vec![nx, ny])?;
    quantize_2d(&field, trunc)
}

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
    // Enforce χ ∈ [0, 1] on the quantized mask (item 14): the analytic field is in range, but
    // tensor-train rounding drives it out, and a negative χ inverts the penalization forcing sign.
    // Every mask constructor routes through here, so the invariant is established once.
    let mask = quantize_2d(&field, trunc)?;
    clamp_mask_to_unit_interval(mask, lx, ly, trunc)
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
