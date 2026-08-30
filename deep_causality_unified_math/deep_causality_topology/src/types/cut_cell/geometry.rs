/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact closed-form clipping primitives shared by the analytic intersection routines.
//!
//! Everything here works in **cell-local coordinates** (the box is `∏ [0, l_i]`) and returns
//! **measures** — volumes and areas, never pointwise samples — the cochain convention the
//! `graded-metrics` capability established and that the cut-geometry exactness tests inherit.
//!
//! Two families:
//!
//! - **Box ∩ half-space** (any dimension): the exact volume of the part of an axis-aligned
//!   box on the `≥` side of a plane, plus the exact `(D−1)`-area of the planar cross-section,
//!   via the standard inclusion–exclusion-over-corners formula
//!   `vol{Σ aᵢxᵢ ≤ c} = (1 / (m! · ∏ aᵢ)) · Σ_{S} (−1)^{|S|} relu(c − Σ_{i∈S} aᵢ lᵢ)^m`.
//! - **Rectangle ∩ disk** (2D): the exact area via the four-corner decomposition of the
//!   circular-quadrant area `∫∫_{u≤x, v≤y} 1_{u²+v²≤r²}`, whose antiderivative is elementary.

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// `max(a, b)` for a `RealField` (which carries `PartialOrd` but no inherent `max`).
pub(super) fn rmax<R: RealField>(a: R, b: R) -> R {
    if a > b { a } else { b }
}

/// `min(a, b)` for a `RealField`.
pub(super) fn rmin<R: RealField>(a: R, b: R) -> R {
    if a < b { a } else { b }
}

/// `relu(t)^p`: `t^p` for `t > 0` (with `t^0 = 1`), else `0`. The truncated power that makes
/// the box ∩ half-space corner sum exact.
fn relu_pow<R: RealField>(t: R, p: usize) -> R {
    if t <= R::zero() {
        return R::zero();
    }
    let mut acc = R::one();
    for _ in 0..p {
        acc *= t;
    }
    acc
}

/// `n!` as an `R`.
fn factorial<R: RealField + FromPrimitive>(n: usize) -> R {
    let mut acc = R::one();
    for k in 2..=n {
        acc *= R::from_usize(k).expect("factorial factor fits in R");
    }
    acc
}

/// Reduce a box ∩ half-space problem to all-positive coefficients by reflecting the axes
/// whose normal component is negative (`y_i = l_i − x_i`, volume-preserving). Returns
/// `(positive_coeffs, positive_lengths, zero_axis_length_product, shifted_offset)` in local
/// coordinates, where the half-space is `{ Σ pos_a·y ≤ shifted_c }` over the positive axes,
/// scaled by the product of the lengths of axes whose coefficient is exactly zero (those are
/// tangent to the plane and factor out).
fn reduce_halfspace<R: RealField>(l: &[R], n: &[R], c: R) -> (Vec<R>, Vec<R>, R, R) {
    let mut shifted_c = c;
    let mut pos_a: Vec<R> = Vec::new();
    let mut pos_l: Vec<R> = Vec::new();
    let mut zero_prod = R::one();
    for (i, &ni) in n.iter().enumerate() {
        if ni < R::zero() {
            // y_i = l_i − x_i : the term −n_i·y_i has positive coeff |n_i|, and the constant
            // n_i·l_i moves to the right-hand side (shifted_c grows since n_i < 0).
            shifted_c -= ni * l[i];
            pos_a.push(-ni);
            pos_l.push(l[i]);
        } else if ni > R::zero() {
            pos_a.push(ni);
            pos_l.push(l[i]);
        } else {
            zero_prod *= l[i];
        }
    }
    (pos_a, pos_l, zero_prod, shifted_c)
}

/// Exact volume of the part of the local box `∏ [0, l_i]` on the **solid** side of the plane
/// `Σ nᵢ xᵢ ≤ c` (local coordinates: pass `c − n·lo`). All dimensions.
pub(super) fn box_halfspace_solid_volume<R: RealField + FromPrimitive>(
    l: &[R],
    n: &[R],
    c: R,
) -> R {
    let (pos_a, pos_l, zero_prod, cc) = reduce_halfspace(l, n, c);
    let m = pos_a.len();

    // Full box volume (product of all lengths).
    let mut full = R::one();
    for &li in l {
        full *= li;
    }

    if m == 0 {
        // Constraint is 0 ≤ cc: the whole box is solid iff cc ≥ 0.
        return if cc >= R::zero() { full } else { R::zero() };
    }

    // After the reduction the constraint is `Σ aⱼ yⱼ ≤ cc` with every `aⱼ > 0` and
    // `yⱼ ∈ [0, lⱼ]`, so the sum ranges over `[0, Σ aⱼ lⱼ]`. Outside that range the answer needs
    // no quadrature, and taking it here is what keeps the answer right rather than merely fast.
    //
    // The inclusion-exclusion below sums `2^m` terms of size `cc^m` and divides by `m! ∏aⱼ`. The
    // terms cancel down to a result of order the cell volume, so once `cc` is large against the
    // cell the cancellation is total: at `cc ≈ 1.6e5` on a unit cell the rounding error already
    // exceeds the result. When the error came out negative the clamp at the end reported zero
    // solid and the cell was classified `Fluid` while lying entirely inside the solid halfspace.
    // Measured: a halfspace with unit normal `[1/√3; 3]` at offset `150000` over the unit cube
    // gave `Fluid`.
    let mut span = R::zero();
    for (&aj, &lj) in pos_a.iter().zip(pos_l.iter()) {
        span += aj * lj;
    }
    if cc >= span {
        // Every point of the box satisfies the constraint.
        return full;
    }
    if cc <= R::zero() {
        // No point does: the sum is non-negative and `cc` is not.
        return R::zero();
    }

    let num_subsets = 1usize << m;
    let mut total = R::zero();
    for subset in 0..num_subsets {
        let mut t = cc;
        let mut bits = 0usize;
        for (j, (&aj, &lj)) in pos_a.iter().zip(pos_l.iter()).enumerate() {
            if (subset >> j) & 1 == 1 {
                t -= aj * lj;
                bits += 1;
            }
        }
        let term = relu_pow(t, m);
        if bits.is_multiple_of(2) {
            total += term;
        } else {
            total -= term;
        }
    }

    let mut denom = factorial::<R>(m);
    for &aj in &pos_a {
        denom *= aj;
    }
    let mut solid = zero_prod * total / denom;
    // Guard against tiny negative / overshoot from rounding.
    if solid < R::zero() {
        solid = R::zero();
    }
    if solid > full {
        solid = full;
    }
    solid
}

/// Exact `(D−1)`-area of the cross-section `{ Σ nᵢ xᵢ = c } ∩ box` in local coordinates.
/// Requires `n` to be a **unit** normal (the caller normalises). Returns `0` for a null
/// normal.
pub(super) fn box_halfspace_cross_area<R: RealField + FromPrimitive>(l: &[R], n: &[R], c: R) -> R {
    let (pos_a, pos_l, zero_prod, cc) = reduce_halfspace(l, n, c);
    let m = pos_a.len();
    if m == 0 {
        return R::zero();
    }

    let num_subsets = 1usize << m;
    let mut total = R::zero();
    for subset in 0..num_subsets {
        let mut t = cc;
        let mut bits = 0usize;
        for (j, (&aj, &lj)) in pos_a.iter().zip(pos_l.iter()).enumerate() {
            if (subset >> j) & 1 == 1 {
                t -= aj * lj;
                bits += 1;
            }
        }
        let term = relu_pow(t, m - 1);
        if bits.is_multiple_of(2) {
            total += term;
        } else {
            total -= term;
        }
    }

    let mut denom = factorial::<R>(m - 1);
    for &aj in &pos_a {
        denom *= aj;
    }
    let area = zero_prod * total / denom;
    if area < R::zero() { R::zero() } else { area }
}

/// `∫_a^b sqrt(r² − u²) du`, the elementary antiderivative
/// `½(u·sqrt(r²−u²) + r²·asin(u/r))`, with `u` clamped to `[−r, r]`.
fn sqrt_integral<R: RealField>(a: R, b: R, r: R) -> R {
    // A circle of zero radius encloses nothing, and `(u / r).asin()` below would be `asin(0/0)`.
    // That NaN propagated all the way out: a ball or cylinder of radius zero produced a cell
    // classified `Cut` with NaN volumes, because both comparisons against a NaN are false.
    if r <= R::zero() {
        return R::zero();
    }
    let prim = |u: R| -> R {
        let uc = if u < -r {
            -r
        } else if u > r {
            r
        } else {
            u
        };
        let s = rmax(r * r - uc * uc, R::zero()).sqrt();
        let half = R::one() / (R::one() + R::one());
        half * (uc * s + r * r * (uc / r).asin())
    };
    prim(b) - prim(a)
}

/// Area of `disk(0, r) ∩ { u ≤ x, v ≤ y }` (center-relative coordinates). The four-corner
/// inclusion–exclusion of this primitive yields the exact rectangle ∩ disk area.
fn circular_quadrant_area<R: RealField>(x: R, y: R, r: R) -> R {
    // A degenerate disk has no area. Checked before the comparisons below, which are all
    // satisfied at `r = 0` and would carry the degeneracy into `sqrt_integral`.
    if r <= R::zero() {
        return R::zero();
    }
    if x <= -r || y <= -r {
        return R::zero();
    }
    let xc = if x > r { r } else { x };
    // Whole vertical strip up to xc when the horizontal cut is above the disk.
    if y >= r {
        return (R::one() + R::one()) * sqrt_integral(-r, xc, r);
    }

    // t = |u| where the circle height s = sqrt(r²−u²) equals |y|.
    let t = rmax(r * r - y * y, R::zero()).sqrt();
    let two = R::one() + R::one();

    if y >= R::zero() {
        // Height profile of {v ≤ y} ∩ disk over u: 2s for |u| > t (s < y), y + s for |u| ≤ t.
        let mut area = R::zero();
        // 2s on [−r, min(xc, −t)]
        let u2 = if xc < -t { xc } else { -t };
        if u2 > -r {
            area += two * sqrt_integral(-r, u2, r);
        }
        // y + s on [−t, min(xc, t)]
        let u2 = if xc < t { xc } else { t };
        if u2 > -t {
            area += y * (u2 - (-t)) + sqrt_integral(-t, u2, r);
        }
        // 2s on [t, xc]
        if xc > t {
            area += two * sqrt_integral(t, xc, r);
        }
        area
    } else {
        // y < 0: nonzero only where s ≥ −y, i.e. |u| ≤ t; height there is y + s.
        let u1 = if -t > -r { -t } else { -r };
        let u2 = if xc < t { xc } else { t };
        if u2 > u1 {
            y * (u2 - u1) + sqrt_integral(u1, u2, r)
        } else {
            R::zero()
        }
    }
}

/// Exact area of the axis-aligned rectangle `[lo0, hi0] × [lo1, hi1]` intersected with the
/// disk of radius `r` centred at `center`. The **solid** area (inside the disk).
pub(super) fn rect_disk_solid_area<R: RealField>(
    lo: [R; 2],
    hi: [R; 2],
    center: [R; 2],
    r: R,
) -> R {
    let x0 = lo[0] - center[0];
    let x1 = hi[0] - center[0];
    let y0 = lo[1] - center[1];
    let y1 = hi[1] - center[1];
    circular_quadrant_area(x1, y1, r)
        - circular_quadrant_area(x0, y1, r)
        - circular_quadrant_area(x1, y0, r)
        + circular_quadrant_area(x0, y0, r)
}

/// Arc length of the circle of radius `r` centred at `center` that lies inside the
/// axis-aligned rectangle `[lo0, hi0] × [lo1, hi1]`. Used for the 2D / cylinder cut-face
/// fragment measure. Computed as the angular measure of the in-rectangle arc times `r`.
pub(super) fn circle_in_rect_arc_len<R: RealField + FromPrimitive>(
    lo: [R; 2],
    hi: [R; 2],
    center: [R; 2],
    r: R,
) -> R {
    if r <= R::zero() {
        return R::zero();
    }
    // The in-rectangle indicator is piecewise constant in θ, and it can only change where the
    // circle crosses one of the four rectangle lines. Collecting those crossing angles and
    // testing one interior point per resulting interval is exact, with no resolution to tune.
    //
    // This was a fixed 2048-point uniform sweep. A cell subtending less than one step, 2π/2048
    // radians, could contain no sample at all and measured an arc length of exactly zero, so a
    // genuinely cut cell recorded no cut-face fragment and the aperture-resolved no-slip stage
    // dropped that wetted surface. Measured on a 1e-4 cell on the unit circle at polar angle
    // 512.5·2π/2048, which is midway between two samples: the sweep returned 0.
    let two_pi = R::pi() * (R::one() + R::one());
    let mut breaks: Vec<R> = Vec::with_capacity(10);
    breaks.push(R::zero());
    breaks.push(two_pi);

    // A vertical line x = X is met where cos θ = (X − cx)/r, giving ±acos of that ratio.
    for &x in &[lo[0], hi[0]] {
        let t = (x - center[0]) / r;
        if t >= -R::one() && t <= R::one() {
            let a = t.acos();
            breaks.push(a);
            breaks.push(two_pi - a);
        }
    }
    // A horizontal line y = Y is met where sin θ = (Y − cy)/r, giving asin and π − asin.
    for &y in &[lo[1], hi[1]] {
        let t = (y - center[1]) / r;
        if t >= -R::one() && t <= R::one() {
            let a = t.asin();
            let wrapped = if a < R::zero() { a + two_pi } else { a };
            breaks.push(wrapped);
            breaks.push(R::pi() - a);
        }
    }

    breaks.retain(|&b| b >= R::zero() && b <= two_pi);
    breaks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(core::cmp::Ordering::Equal));

    let half = R::one() / (R::one() + R::one());
    let mut inside_angle = R::zero();
    for w in breaks.windows(2) {
        let (a, b) = (w[0], w[1]);
        if b <= a {
            continue;
        }
        let mid = (a + b) * half;
        let px = center[0] + r * mid.cos();
        let py = center[1] + r * mid.sin();
        if px >= lo[0] && px <= hi[0] && py >= lo[1] && py <= hi[1] {
            inside_angle += b - a;
        }
    }
    r * inside_angle
}
