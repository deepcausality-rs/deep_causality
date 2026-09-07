/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Norms.
//!
//! # Two shapes, one body each
//!
//! The matrix norms are generic over any [`MatrixView`](crate::MatrixView), so they apply to the
//! sparse and the packed representations too. The vector norms are generic over a **slice**, and
//! [`DenseVector`](crate::DenseVector)'s methods of the same names delegate to them.
//!
//! The slice form exists because the callers being consolidated do not hold a `DenseVector`.
//! `deep_causality_multivector`'s coefficients are a `Vec<T>` and `CausalMultiField`'s are a
//! tensor's buffer; routing either through a `DenseVector` would allocate and copy the whole
//! coefficient vector on every norm. A slice is what both already have, and the method delegating
//! to the function is what keeps this one body rather than two.
//!
//! # Defined once
//!
//! The workspace answers the Euclidean-norm question four times today: `CausalTensor::norm_l2` and
//! `norm_sq`, `frobenius_norm` in `deep_causality_quantum`, `MultiVectorL2Norm::norm_l2` and
//! `CausalMultiField::squared_magnitude` in `deep_causality_multivector`. Each is correct where it
//! sits; together they are four places to fix a bug.
//!
//! # The bound is [`Normed`], not [`NormedScalar`](deep_causality_algebra::NormedScalar)
//!
//! `Normed` is what these functions actually use: a modulus, a squared modulus, and an associated
//! `Real` that is ordered and carries `sqrt`. `NormedScalar` adds `Field`, `FromPrimitive` and
//! `Copy` on top, none of which any body here calls for — and asking for them costs callers real
//! generality, because `Complex<T>: FromPrimitive` holds only when `T: FromPrimitive` does, while
//! `Complex<T>: Normed` holds for every `T: RealField`. Taking the weaker bound is what lets
//! `deep_causality_quantum` reach the scaled norm without widening its own public signatures.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use deep_causality_algebra::{Normed, RealField};
use deep_causality_num::Zero;

/// The 1-norm of a vector, `Σ |aᵢ|`.
pub fn vector_norm_l1<T: Normed>(v: &[T]) -> <T as Normed>::Real {
    let mut acc = <T as Normed>::Real::zero();
    for x in v {
        acc += x.modulus();
    }
    acc
}

/// The squared 2-norm of a vector, `Σ |aᵢ|²`.
///
/// Separate from [`vector_norm_l2`] because the square root is the expensive part and comparisons
/// rarely need it. `deep_causality_multivector`'s `CausalMultiField::squared_magnitude` is exactly
/// this quantity and now is exactly this call.
pub fn vector_norm_sq<T: Normed>(v: &[T]) -> <T as Normed>::Real {
    let mut acc = <T as Normed>::Real::zero();
    for x in v {
        acc += x.modulus_squared();
    }
    acc
}

/// The 2-norm of a vector, `sqrt(Σ |aᵢ|²)`, computed in the scaled form.
///
/// This is the one definition of the Euclidean norm in the workspace.
///
/// # Why not `vector_norm_sq(v).sqrt()`
///
/// That form overflows where the answer does not. A vector holding a component near `T::MAX`
/// squares to an infinity and returns `inf` for a norm that is representable; one whose components
/// sit near `T::MIN_POSITIVE` squares to zero and returns zero for a norm that is not. The squared
/// norm is a genuinely larger quantity than the norm, and routing the norm through it gives away
/// half the exponent range.
///
/// The scaled form factors the largest modulus out — `max · sqrt(Σ (|aᵢ|/max)²)`, where every ratio
/// lies in `[0, 1]` and cannot overflow. It is the reasoning
/// [`Normed::modulus`](deep_causality_algebra::Normed::modulus) already applies to a complex's two
/// components, at the length of a vector.
///
/// # Cost
///
/// One pass, accumulating a running scale and sum of squares rather than taking a maximum and then
/// a second pass, so the slice is read once. It asks each entry for `modulus` where the naive form
/// asked for `modulus_squared`, which for a real is `abs` against a multiply and for a complex is a
/// square root against a multiply-add, and it divides once per entry. A caller that wants the
/// squared norm, or is only comparing magnitudes, should call [`vector_norm_sq`] and pay neither.
///
/// # Non-finite entries
///
/// Unchanged from the naive form: a `NaN` modulus anywhere gives `NaN`, and an infinite one with no
/// `NaN` present gives `+∞`.
pub fn vector_norm_l2<T: Normed>(v: &[T]) -> <T as Normed>::Real {
    let mut acc = ScaledNorm::new();
    for x in v {
        acc.push(x.modulus());
    }
    acc.finish()
}

/// The running state of the scaled Euclidean norm: the largest modulus seen, and the sum of squared
/// ratios against it.
///
/// # The recurrence
///
/// Holding `scale = max |aᵢ|` over the entries so far and `ssq = Σ (|aᵢ|/scale)²`, a new entry `a`
/// either fits under the current scale, contributing `(a/scale)²`, or replaces it, which rescales
/// the accumulated sum by `(scale_old/a)²`. Both ratios lie in `[0, 1]`, so neither the square nor
/// the sum can overflow while the answer is representable. The first non-zero entry takes the
/// replacing branch against `scale = 0`, whose ratio is zero, leaving `ssq = 1` — the norm of one
/// entry being that entry.
///
/// This is one pass. Taking the maximum first and forming the ratios second would be two, and would
/// ask each entry for its modulus twice, which for a complex entry is a square root each time.
///
/// # Non-finite entries
///
/// Kept separate from the accumulation rather than allowed into it. An infinity would drive `scale`
/// to infinity and make every later ratio `finite/∞ = 0`, which is harmless — but a *second*
/// infinity gives `∞/∞ = NaN`, turning a norm that is infinite into one that is undefined. A `NaN`
/// entry never compares greater than `scale`, so without the check it would fall into the
/// contributing branch and be added as a `NaN` ratio, which happens to give the right answer for the
/// wrong reason. Both are decided here, and the answer matches what summing the squares gave: `NaN`
/// wins over infinity, and infinity over the finite entries.
struct ScaledNorm<R> {
    scale: R,
    ssq: R,
    saw_nan: bool,
    infinite: Option<R>,
}

impl<R: RealField> ScaledNorm<R> {
    fn new() -> Self {
        Self {
            scale: R::zero(),
            ssq: R::one(),
            saw_nan: false,
            infinite: None,
        }
    }

    /// Folds in one modulus, which is non-negative by construction.
    fn push(&mut self, a: R) {
        if a.is_nan() {
            self.saw_nan = true;
        } else if a.is_infinite() {
            self.infinite = Some(a);
        } else if a > self.scale {
            let r = self.scale / a;
            self.ssq = R::one() + self.ssq * r * r;
            self.scale = a;
        } else if a > R::zero() {
            // Guarded against a zero scale, which only happens when `a` is zero too: the ratio
            // would be `0/0`, and a vector of zeros would come back `NaN` rather than zero.
            let r = a / self.scale;
            self.ssq += r * r;
        }
    }

    fn finish(self) -> R {
        if self.saw_nan {
            return R::nan();
        }
        if let Some(infinity) = self.infinite {
            return infinity;
        }
        // Zero for the empty vector and the all-zero one: `scale` never left zero, and the factor
        // in front is what makes that come out as zero rather than as `sqrt(1)`.
        self.scale * self.ssq.sqrt()
    }
}

/// The ∞-norm of a vector, `max |aᵢ|`.
///
/// Zero for the empty vector, which is the supremum over an empty set in the convention this crate
/// uses, and never `NaN`.
pub fn vector_norm_inf<T: Normed>(v: &[T]) -> <T as Normed>::Real {
    let mut best = <T as Normed>::Real::zero();
    for x in v {
        let m = x.modulus();
        if m > best {
            best = m;
        }
    }
    best
}

/// The 1-norm: the largest column sum of moduli.
pub fn matrix_norm_l1<M>(m: &M) -> Result<<M::Scalar as Normed>::Real, LinearError>
where
    M: MatrixView,
    M::Scalar: Normed,
{
    let (r, c) = (m.rows(), m.cols());
    let mut best = <M::Scalar as Normed>::Real::zero();
    for j in 0..c {
        let mut col = <M::Scalar as Normed>::Real::zero();
        for i in 0..r {
            col += m.get(i, j)?.modulus();
        }
        if col > best {
            best = col;
        }
    }
    Ok(best)
}

/// The ∞-norm: the largest row sum of moduli.
pub fn matrix_norm_inf<M>(m: &M) -> Result<<M::Scalar as Normed>::Real, LinearError>
where
    M: MatrixView,
    M::Scalar: Normed,
{
    let (r, c) = (m.rows(), m.cols());
    let mut best = <M::Scalar as Normed>::Real::zero();
    for i in 0..r {
        let mut row = <M::Scalar as Normed>::Real::zero();
        for j in 0..c {
            row += m.get(i, j)?.modulus();
        }
        if row > best {
            best = row;
        }
    }
    Ok(best)
}

/// The Frobenius norm: `sqrt(Σ |aᵢⱼ|²)`.
///
/// Equal to the 2-norm of the entries read as one vector, which is a test rather than a remark —
/// and the reason this is scaled the way [`vector_norm_l2`] is. The two are the same quantity over
/// the same numbers, so a fix to one that left the other summing squares would put the crate's own
/// identity test in the position of asserting an equality that fails at the extremes it was fixed
/// for.
pub fn matrix_norm_frobenius<M>(m: &M) -> Result<<M::Scalar as Normed>::Real, LinearError>
where
    M: MatrixView,
    M::Scalar: Normed,
{
    let (r, c) = (m.rows(), m.cols());
    let mut acc = ScaledNorm::new();
    for i in 0..r {
        for j in 0..c {
            acc.push(m.get(i, j)?.modulus());
        }
    }
    Ok(acc.finish())
}
