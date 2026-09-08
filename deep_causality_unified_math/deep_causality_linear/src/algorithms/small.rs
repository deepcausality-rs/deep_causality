/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fixed-size dense linear algebra: the inner product, and the `3×3` and `4×4` closed forms.
//!
//! # Why these live here rather than at their call sites
//!
//! Every operation below was found open-coded in a consumer crate, in most cases more than once
//! (`unified-math-next` task 6.7, inventoried in `notes/c4-site-inventory.md`). They are linear
//! algebra, so under the dual mandate their home is this crate and the consumer dispatches.
//!
//! # Why closed forms rather than the general path
//!
//! [`solve`](crate::algorithms::solve) and [`elimination`](crate::algorithms::elimination) already
//! compute every quantity here, and for a general `n` they are the right answer. At `n = 3` and
//! `n = 4` they are not: each allocates a working copy, indexes through a trait, and runs a pivot
//! search over three or four candidates. The closed forms are branch-free polynomials in the
//! entries. They are also **exact over a commutative ring** — the cofactor determinant needs no
//! division, so it is defined over ℤ where Gaussian elimination leaves ℤ on its first pivot.
//!
//! # Bounds
//!
//! Banded as the crate bands everything: the weakest trait that makes the operation correct. The
//! determinant, the contractions and the matrix-vector product are polynomials and take
//! [`CommutativeRing`]; the inverses divide by the determinant and take [`Field`]; the symmetric
//! eigenvalues need a square root, a cube-root-free trigonometric solve and an ordering, so they
//! take [`RealField`] and [`FromPrimitive`].
//!
//! `PartialEq` joins the bound on the two inverses, and only there: the refusal is an *exact* test
//! against zero, so equality is what it needs and an ordering would be more than it needs.
//!
//! # One reciprocal, not nine divisions
//!
//! Both inverses form `1/det` once and multiply, which is the standard idiom for an adjugate
//! inverse and what the consumer copies did. Dividing each of the nine (or sixteen) entries would
//! be correctly rounded per entry rather than carrying up to one extra unit in the last place —
//! and would also change the numbers a shipped general-relativity solver returns, by an amount no
//! caller can act on, in exchange for eight more divisions. The idiom is kept, and recorded here so
//! the choice is visible.
//!
//! # The singularity policy
//!
//! [`inverse_3x3`] and [`inverse_4x4`] refuse only an **exactly** zero determinant — the
//! mathematical refusal, the one case where no inverse exists. A *near*-singular threshold is a
//! caller's judgement about its own physics, not a property of the matrix, so it stays at the call
//! site. `deep_causality_physics`'s general-relativity metrics keep their own `1e-12` and `1e-14`
//! guards in front of these calls for exactly that reason.

use crate::errors::linear_error::LinearError;
use deep_causality_algebra::{CommutativeRing, Field, RealField};
use deep_causality_num::FromPrimitive;

/// The inner product `Σ aᵢ·bᵢ` of two equal-length slices.
///
/// # Errors
///
/// [`LinearError::LengthMismatch`] when the slices differ in length. Folding over the shorter of
/// the two is the same silent-truncation class this stage removed from the CSR matrix-vector
/// product: it turns a caller's indexing mistake into a plausible wrong number rather than a
/// refusal.
///
/// The dot product of two empty slices is zero, being the empty sum.
pub fn dot<T>(a: &[T], b: &[T]) -> Result<T, LinearError>
where
    T: CommutativeRing + Copy,
{
    if a.len() != b.len() {
        return Err(LinearError::LengthMismatch(a.len(), b.len()));
    }
    Ok(a.iter()
        .zip(b.iter())
        .fold(T::zero(), |acc, (&x, &y)| acc + x * y))
}

/// The inner product of two arrays of the same **compile-time** length.
///
/// Identical arithmetic to [`dot`], and returns `T` rather than `Result<T, _>` because the type
/// already proves what [`dot`]'s length check tests. A caller holding `&[R; 3]` or `&[R; 17]` should
/// reach for this: routing it through the slice form would add a comparison the compiler cannot
/// always fold and, worse, an error arm no input can reach, which a call site then has to answer
/// with an `unwrap` or an `expect` in the middle of a solver loop.
pub fn dot_n<T, const N: usize>(a: &[T; N], b: &[T; N]) -> T
where
    T: CommutativeRing + Copy,
{
    a.iter()
        .zip(b.iter())
        .fold(T::zero(), |acc, (&x, &y)| acc + x * y)
}

/// The determinant of a `3×3`, by cofactor expansion along the first row.
pub fn determinant_3x3<T>(m: &[[T; 3]; 3]) -> T
where
    T: CommutativeRing + Copy,
{
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// The determinant of a `4×4`, by the six `2×2` minors of the first two rows.
///
/// The minors are shared with [`inverse_4x4`]'s adjugate, which is why the `4×4` determinant is
/// written this way rather than as a four-term expansion along a row.
pub fn determinant_4x4<T>(m: &[[T; 4]; 4]) -> T
where
    T: CommutativeRing + Copy,
{
    let (s, c) = minors_4x4(m);
    s[0] * c[5] - s[1] * c[4] + s[2] * c[3] + s[3] * c[2] - s[4] * c[1] + s[5] * c[0]
}

/// The trace of the square, `tr(A²) = Σᵢ Σⱼ aᵢⱼ·aⱼᵢ`, for a `3×3`.
///
/// Note the transposed second index: this is **not** `Σ aᵢⱼ²`, which is the squared Frobenius norm
/// and equals `tr(AᵀA)`. The two agree exactly when `A` is symmetric, which is the coincidence
/// that lets a wrong one pass a symmetric fixture.
pub fn trace_of_square_3x3<T>(a: &[[T; 3]; 3]) -> T
where
    T: CommutativeRing + Copy,
{
    a[0][0] * a[0][0]
        + a[0][1] * a[1][0]
        + a[0][2] * a[2][0]
        + a[1][0] * a[0][1]
        + a[1][1] * a[1][1]
        + a[1][2] * a[2][1]
        + a[2][0] * a[0][2]
        + a[2][1] * a[1][2]
        + a[2][2] * a[2][2]
}

/// The Frobenius inner product `A : B = Σᵢ Σⱼ aᵢⱼ·bᵢⱼ`, for two `3×3` matrices.
///
/// Equal to `tr(AᵀB)`. Distinct from [`trace_of_square_3x3`] in exactly the transposition of the
/// second factor's indices, and the two coincide for symmetric arguments.
pub fn double_dot_3x3<T>(a: &[[T; 3]; 3], b: &[[T; 3]; 3]) -> T
where
    T: CommutativeRing + Copy,
{
    a[0][0] * b[0][0]
        + a[0][1] * b[0][1]
        + a[0][2] * b[0][2]
        + a[1][0] * b[1][0]
        + a[1][1] * b[1][1]
        + a[1][2] * b[1][2]
        + a[2][0] * b[2][0]
        + a[2][1] * b[2][1]
        + a[2][2] * b[2][2]
}

/// The matrix-vector product `M·v` for a `3×3` and a 3-vector.
pub fn mat3_vec<T>(m: &[[T; 3]; 3], v: &[T; 3]) -> [T; 3]
where
    T: CommutativeRing + Copy,
{
    core::array::from_fn(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

/// The inverse of a `3×3`, as the adjugate over the determinant.
///
/// # Errors
///
/// [`LinearError::Singular`] when the determinant is exactly zero. See the module docs on why the
/// near-singular threshold is the caller's.
pub fn inverse_3x3<T>(m: &[[T; 3]; 3]) -> Result<[[T; 3]; 3], LinearError>
where
    T: Field + Copy + PartialEq,
{
    let det = determinant_3x3(m);
    if det == T::zero() {
        return Err(LinearError::Singular(0));
    }
    let inv = T::one() / det;
    Ok([
        [
            inv * (m[1][1] * m[2][2] - m[1][2] * m[2][1]),
            inv * (m[0][2] * m[2][1] - m[0][1] * m[2][2]),
            inv * (m[0][1] * m[1][2] - m[0][2] * m[1][1]),
        ],
        [
            inv * (m[1][2] * m[2][0] - m[1][0] * m[2][2]),
            inv * (m[0][0] * m[2][2] - m[0][2] * m[2][0]),
            inv * (m[0][2] * m[1][0] - m[0][0] * m[1][2]),
        ],
        [
            inv * (m[1][0] * m[2][1] - m[1][1] * m[2][0]),
            inv * (m[0][1] * m[2][0] - m[0][0] * m[2][1]),
            inv * (m[0][0] * m[1][1] - m[0][1] * m[1][0]),
        ],
    ])
}

/// The inverse of a `4×4`, as the adjugate over the determinant.
///
/// # Errors
///
/// [`LinearError::Singular`] when the determinant is exactly zero.
pub fn inverse_4x4<T>(m: &[[T; 4]; 4]) -> Result<[[T; 4]; 4], LinearError>
where
    T: Field + Copy + PartialEq,
{
    let (s, c) = minors_4x4(m);
    let det = s[0] * c[5] - s[1] * c[4] + s[2] * c[3] + s[3] * c[2] - s[4] * c[1] + s[5] * c[0];
    if det == T::zero() {
        return Err(LinearError::Singular(0));
    }
    let d = T::one() / det;
    let z = T::zero();
    Ok([
        [
            d * (m[1][1] * c[5] - m[1][2] * c[4] + m[1][3] * c[3]),
            d * (z - (m[0][1] * c[5] - m[0][2] * c[4] + m[0][3] * c[3])),
            d * (m[3][1] * s[5] - m[3][2] * s[4] + m[3][3] * s[3]),
            d * (z - (m[2][1] * s[5] - m[2][2] * s[4] + m[2][3] * s[3])),
        ],
        [
            d * (z - (m[1][0] * c[5] - m[1][2] * c[2] + m[1][3] * c[1])),
            d * (m[0][0] * c[5] - m[0][2] * c[2] + m[0][3] * c[1]),
            d * (z - (m[3][0] * s[5] - m[3][2] * s[2] + m[3][3] * s[1])),
            d * (m[2][0] * s[5] - m[2][2] * s[2] + m[2][3] * s[1]),
        ],
        [
            d * (m[1][0] * c[4] - m[1][1] * c[2] + m[1][3] * c[0]),
            d * (z - (m[0][0] * c[4] - m[0][1] * c[2] + m[0][3] * c[0])),
            d * (m[3][0] * s[4] - m[3][1] * s[2] + m[3][3] * s[0]),
            d * (z - (m[2][0] * s[4] - m[2][1] * s[2] + m[2][3] * s[0])),
        ],
        [
            d * (z - (m[1][0] * c[3] - m[1][1] * c[1] + m[1][2] * c[0])),
            d * (m[0][0] * c[3] - m[0][1] * c[1] + m[0][2] * c[0]),
            d * (z - (m[3][0] * s[3] - m[3][1] * s[1] + m[3][2] * s[0])),
            d * (m[2][0] * s[3] - m[2][1] * s[1] + m[2][2] * s[0]),
        ],
    ])
}

/// The three eigenvalues of a **real symmetric** `3×3`, sorted descending.
///
/// Only the upper triangle is read, so a caller holding a symmetric matrix in a full array need not
/// symmetrise it first, and a caller that passes a non-symmetric matrix gets the eigenvalues of its
/// symmetric part rather than a silently wrong answer for the matrix it passed.
///
/// # Method
///
/// The closed form: shift by the mean eigenvalue `q = tr(M)/3`, normalise, and read the three roots
/// of the resulting characteristic cubic off `acos`. The diagonal case is returned directly,
/// because the normalisation divides by a quantity that vanishes there.
///
/// The matrix is first divided by its largest entry magnitude and the three roots multiplied back
/// by it, because the cubic's coefficients are sums of squares of the entries: without that, a
/// finite matrix near either end of the scalar's range overflows them to infinity or underflows
/// them to zero and the spectrum comes back `NaN` or plainly wrong. The full spectral range is
/// therefore usable, not just the square-representable middle of it.
///
/// # Accuracy, and where it is worst
///
/// About `ε` relative for a well-separated spectrum, and about **`√ε`** where two eigenvalues
/// coincide. The angle comes out of `acos`, whose derivative is unbounded at `±1`, and a repeated
/// root puts the argument exactly there — so a perturbation of `ε` in the argument moves each
/// eigenvalue by roughly `√ε·‖M‖`. Measured: the rank-one `v vᵀ` for `v = (−3, −3, −1)`, whose
/// exact spectrum is `(19, 0, 0)`, comes back with its repeated zero at `5.4e-8`.
///
/// This is the algorithm rather than the implementation, and
/// [`eigen_hermitian`](crate::eigen_hermitian) does not share it: cyclic Jacobi is backward stable
/// and does not lose half the digits at a degeneracy. A caller that reads a *difference* of
/// eigenvalues near a degeneracy — `deep_causality_physics`'s `lambda2_kernel` sits exactly on that
/// boundary, since `λ₂ = 0` separates vortex from non-vortex topology — should know which of the
/// two it wants.
///
/// # Reference
///
/// O. K. Smith, "Eigenvalues of a symmetric 3 × 3 matrix", *Communications of the ACM* **4**(4),
/// 168 (1961).
///
/// # Errors
///
/// [`LinearError::Overflow`] when a small integer constant has no image in `T`, which
/// [`FromPrimitive`] permits an exotic scalar to refuse.
pub fn eigen_symmetric_3x3<T>(m: &[[T; 3]; 3]) -> Result<[T; 3], LinearError>
where
    T: RealField + FromPrimitive,
{
    let third: T = constant(1.0 / 3.0)?;
    let half: T = constant(0.5)?;
    let two: T = constant(2.0)?;
    let three: T = constant(3.0)?;
    let six: T = constant(6.0)?;
    let two_pi_over_3: T = constant(2.0 * core::f64::consts::PI / 3.0)?;

    // Only the upper triangle is read, so the result is the spectrum of the symmetric completion.
    //
    // Divide it by its largest magnitude before anything is squared. `p1` and `p2` are sums of
    // squares of the entries, so an entry near either end of the scalar's range takes them out of
    // it — `p1` overflows to infinity and every eigenvalue comes back `NaN`, or it underflows to
    // zero and the wrong spectrum comes back finite — from a matrix whose own eigenvalues are
    // perfectly representable. Measured at `f64` on `[[2,1,0],[1,2,0],[0,0,3]]`, whose spectrum is
    // `(3, 3, 1)`: scaled by `1e160` it returned `[inf, NaN, -inf]`, and scaled by `1e-200` it
    // returned `(3, 2, 2)·1e-200`. The spectrum is homogeneous of degree one, `λ(M/s) = λ(M)/s`,
    // so the cubic runs on `M/s` and the three roots are multiplied back by `s` at the end.
    let scale = max_magnitude_6(m);
    if scale == T::zero() {
        // Every entry read is zero, so the symmetric completion is the zero matrix.
        return Ok([T::zero(); 3]);
    }
    let a00 = m[0][0] / scale;
    let a11 = m[1][1] / scale;
    let a22 = m[2][2] / scale;
    let a01 = m[0][1] / scale;
    let a02 = m[0][2] / scale;
    let a12 = m[1][2] / scale;

    let p1 = a01 * a01 + a02 * a02 + a12 * a12;
    if p1 == T::zero() {
        // Diagonal: the normalisation below divides by a quantity that vanishes here. The original
        // entries are returned rather than the rescaled ones, so this branch stays exact.
        let mut e = [m[0][0], m[1][1], m[2][2]];
        sort_desc_3(&mut e);
        return Ok(e);
    }

    let q = (a00 + a11 + a22) * third; // the mean eigenvalue of the normalised matrix
    let d00 = a00 - q;
    let d11 = a11 - q;
    let d22 = a22 - q;
    let p2 = d00 * d00 + d11 * d11 + d22 * d22 + two * p1;
    let p = (p2 / six).sqrt();

    // B = (A − qI)/p, whose determinant is twice the cosine of three times the angle wanted.
    let inv_p = T::one() / p;
    let b00 = d00 * inv_p;
    let b11 = d11 * inv_p;
    let b22 = d22 * inv_p;
    let b01 = a01 * inv_p;
    let b02 = a02 * inv_p;
    let b12 = a12 * inv_p;
    let det_b = b00 * (b11 * b22 - b12 * b12) - b01 * (b01 * b22 - b12 * b02)
        + b02 * (b01 * b12 - b11 * b02);

    // Clamp before `acos`: at a repeated eigenvalue the exact value is ±1 and rounding overshoots,
    // where `acos` returns NaN and every eigenvalue with it.
    let mut r = det_b * half;
    if r < -T::one() {
        r = -T::one();
    }
    if r > T::one() {
        r = T::one();
    }
    let phi = r.acos() * third;

    let eig1 = q + two * p * phi.cos();
    let eig3 = q + two * p * (phi + two_pi_over_3).cos();
    // The third from the trace rather than a third cosine: `Σλ = tr(A)` holds exactly, so this is
    // the better-conditioned of the two ways to get it.
    let eig2 = three * q - eig1 - eig3;

    // Back out of the normalisation. `scale` is positive, so the ordering is unaffected by it.
    let mut e = [eig1 * scale, eig2 * scale, eig3 * scale];
    sort_desc_3(&mut e);
    Ok(e)
}

/// The largest magnitude among the six entries [`eigen_symmetric_3x3`] reads.
fn max_magnitude_6<T: RealField>(m: &[[T; 3]; 3]) -> T {
    let mut max = T::zero();
    for x in [m[0][0], m[1][1], m[2][2], m[0][1], m[0][2], m[1][2]] {
        let a = x.abs();
        if a > max {
            max = a;
        }
    }
    max
}

/// Sorts three values in descending order. Three comparisons, which is the sorting-network optimum
/// for `n = 3`.
fn sort_desc_3<T: RealField>(a: &mut [T; 3]) {
    if a[0] < a[1] {
        a.swap(0, 1);
    }
    if a[1] < a[2] {
        a.swap(1, 2);
    }
    if a[0] < a[1] {
        a.swap(0, 1);
    }
}

/// Lifts a small integer constant into `T`, or reports that the scalar has no image for it.
fn constant<T: RealField + FromPrimitive>(v: f64) -> Result<T, LinearError> {
    T::from_f64(v).ok_or_else(|| LinearError::Overflow("eigen_symmetric_3x3 constant"))
}

/// The six `2×2` minors of the first two rows, and the six of the last two.
///
/// Shared by [`determinant_4x4`] and [`inverse_4x4`]: the determinant is a signed combination of
/// the twelve, and every entry of the adjugate is a signed combination of a row and six of them.
/// Computing them once is why the `4×4` determinant is not written as a four-term row expansion.
fn minors_4x4<T>(m: &[[T; 4]; 4]) -> ([T; 6], [T; 6])
where
    T: CommutativeRing + Copy,
{
    let s = [
        m[0][0] * m[1][1] - m[0][1] * m[1][0],
        m[0][0] * m[1][2] - m[0][2] * m[1][0],
        m[0][0] * m[1][3] - m[0][3] * m[1][0],
        m[0][1] * m[1][2] - m[0][2] * m[1][1],
        m[0][1] * m[1][3] - m[0][3] * m[1][1],
        m[0][2] * m[1][3] - m[0][3] * m[1][2],
    ];
    let c = [
        m[2][0] * m[3][1] - m[2][1] * m[3][0],
        m[2][0] * m[3][2] - m[2][2] * m[3][0],
        m[2][0] * m[3][3] - m[2][3] * m[3][0],
        m[2][1] * m[3][2] - m[2][2] * m[3][1],
        m[2][1] * m[3][3] - m[2][3] * m[3][1],
        m[2][2] * m[3][3] - m[2][3] * m[3][2],
    ];
    (s, c)
}
