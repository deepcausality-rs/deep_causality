/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The dense decomposition kernels, over flat row-major slices.
//!
//! # Why the kernels are separate from the entry points
//!
//! [`decomposition`](crate::algorithms::decomposition) is generic over
//! [`MatrixView`](crate::MatrixView) and has to read entries through it. These take `&[T]` and the
//! shape, so the numerics are one layer down from the seam and can be read, tested and compared
//! against a reference without a container in the way.
//!
//! # Bounded on `ConjugateScalar`, not `RealField`
//!
//! `ConjugateScalar` spans the three scalar families numerical linear algebra needs — real fields,
//! dual numbers for forward-mode AD, and complex — by naming exactly what a Hermitian kernel uses:
//! conjugation, a real squared modulus, the real part, and injection of a real. `RealField` admits
//! only the first, and `Complex` is not a `RealField`: it is unordered, so no ordered-field bound
//! can ever cover it.
//!
//! Every magnitude, threshold and rotation angle here lives in `T::Real`, and only the rotations
//! themselves are injected back into `T`. For a real scalar `conjugate` is the identity and
//! `modulus_squared` is `x²`, so each kernel reduces exactly to its ordinary real form.
//!
//! # Thresholds are relative
//!
//! Both kernels scale their stopping threshold by the input's Frobenius norm. An absolute `ε` test
//! does not terminate for a large-magnitude matrix — it burns the whole sweep budget every time and
//! returns whatever it had reached — and it fires immediately for a small-magnitude one. The
//! Frobenius norm is invariant under the rotations, so it is computed once from the input.

use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::{ConjugateScalar, Real};
use deep_causality_num::{One, Zero};

/// The real type carrying magnitudes and thresholds for a scalar.
type Re<T> = <T as ConjugateScalar>::Real;

/// The most sweeps a Jacobi iteration will take before returning what it has.
///
/// Cyclic Jacobi converges quadratically once the off-diagonal is small, so a well-conditioned
/// matrix finishes in under ten. The cap is what stops a pathological input running forever.
const MAX_SWEEPS: usize = 100;

/// Eigendecomposition of a **Hermitian** row-major `n×n` matrix by cyclic Jacobi rotations.
///
/// Returns `(eigenvalues, v)` where the columns of the row-major `n×n` `v` are the eigenvectors, so
/// `A = V diag(λ) Vᴴ`. The eigenvalues of a Hermitian matrix are real; they come back as `T` with a
/// zero imaginary part rather than as `T::Real`, because the caller's matrix is in `T` and a
/// separate real type at this boundary would force a conversion on every caller. They are unsorted.
///
/// For a real scalar the rotation phase `ρ` is `±1` and this is the ordinary real-symmetric Jacobi.
///
/// # Reference
///
/// G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ. Press,
/// 2013), §8.5 (Jacobi methods).
pub(crate) fn sym_eig<T: ConjugateScalar>(mat: &[T], n: usize) -> (Vec<T>, Vec<T>) {
    let mut a = mat.to_vec();
    let mut v = vec![T::zero(); n * n];
    for i in 0..n {
        v[i * n + i] = T::one();
    }
    let one = Re::<T>::one();
    let two = one + one;

    // The off-diagonal budget is ε²·‖A‖²_F. ‖A‖_F is invariant under the rotations, so it is read
    // once from the input.
    let eps2 = Re::<T>::epsilon() * Re::<T>::epsilon();
    let norm_sq = a
        .iter()
        .fold(Re::<T>::zero(), |acc, x| acc + x.modulus_squared());
    // A pathologically large but finite matrix can overflow ‖A‖²_F. Left as `∞`, the `off <=
    // threshold` test becomes `∞ <= ∞` and breaks before any rotation, handing back the
    // undiagonalised input as its own eigendecomposition. The absolute budget is the fallback.
    let threshold = if norm_sq.is_finite() {
        eps2 * norm_sq
    } else {
        eps2
    };

    for _ in 0..MAX_SWEEPS {
        // Σ_{p<q} |a[p,q]|², a real quantity even when the entries are not.
        let mut off = Re::<T>::zero();
        for p in 0..n {
            for q in (p + 1)..n {
                off += a[p * n + q].modulus_squared();
            }
        }
        if off <= threshold {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p * n + q];
                let gmod = apq.modulus(); // |γ|
                if gmod <= Re::<T>::zero() {
                    continue;
                }
                // A Hermitian matrix has a real diagonal.
                let app = a[p * n + p].real_part();
                let aqq = a[q * n + q].real_part();
                let zeta = (app - aqq) / (two * gmod);
                let t = if zeta == Re::<T>::zero() {
                    one
                } else {
                    let sgn = if zeta < Re::<T>::zero() { -one } else { one };
                    sgn / (zeta.abs() + (zeta * zeta + one).sqrt())
                };
                let c = one / (t * t + one).sqrt();
                let s = t * c;

                // U = diag(1, conj(ρ))·[[c, -s], [s, c]] with the phase ρ = γ/|γ|; apply Uᴴ A U.
                let rho = apq * T::from_real(one / gmod);
                let ct = T::from_real(c);
                let cs = T::from_real(s);
                let conj_rho = rho.conjugate();
                let srho = conj_rho * cs;
                let crho = conj_rho * ct;
                let rs = rho * cs;
                let rc = rho * ct;

                // A ← A·U, columns p and q.
                for i in 0..n {
                    let aip = a[i * n + p];
                    let aiq = a[i * n + q];
                    a[i * n + p] = ct * aip + srho * aiq;
                    a[i * n + q] = crho * aiq - cs * aip;
                }
                // A ← Uᴴ·A, rows p and q.
                for j in 0..n {
                    let apj = a[p * n + j];
                    let aqj = a[q * n + j];
                    a[p * n + j] = ct * apj + rs * aqj;
                    a[q * n + j] = rc * aqj - cs * apj;
                }
                // V ← V·U, accumulating the eigenvectors.
                for i in 0..n {
                    let vip = v[i * n + p];
                    let viq = v[i * n + q];
                    v[i * n + p] = ct * vip + srho * viq;
                    v[i * n + q] = crho * viq - cs * vip;
                }
            }
        }
    }
    let evals: Vec<T> = (0..n).map(|i| a[i * n + i]).collect();
    (evals, v)
}

/// Thin Householder QR of a row-major `m×n` matrix.
///
/// Returns `(q, r, k)` with `k = min(m, n)`, `q` row-major `m×k` with orthonormal columns and `r`
/// row-major `k×n` upper-triangular, so `A = Q·R`.
///
/// **Thin, not full.** A wide matrix (`n > m`) gets `Q` of `m×m` and `R` of `m×n`. A `Q` with more
/// columns than rows cannot have orthonormal columns, so producing one would be returning a factor
/// that does not satisfy the property `Q` is named for.
///
/// For a complex scalar these are the genuine Householder reflectors `H = I − β v vᴴ`, with
/// conjugated inner products and a unitary `Q`; for a real scalar the conjugation is the identity
/// and it reduces to the ordinary real Householder QR.
///
/// # Reference
///
/// G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ. Press,
/// 2013), §5.2 (Householder QR factorization).
pub(crate) fn householder_qr<T: ConjugateScalar>(
    a_in: &[T],
    m: usize,
    n: usize,
) -> (Vec<T>, Vec<T>, usize) {
    let k = m.min(n);

    // The working copy becomes R as the reflectors are applied; Q accumulates full at m×m and is
    // narrowed to its first k columns at the end.
    let mut r = a_in.to_vec();
    let mut q = vec![T::zero(); m * m];
    for i in 0..m {
        q[i * m + i] = T::one();
    }

    // Noise floor on a sub-column's squared norm. A column whose norm is below ‖A‖·ε is
    // numerically zero, and reflecting it makes `vᴴv` denormal and `β = 2/(vᴴv)` overflow to `∞`,
    // which poisons `β·dot` with NaN. Such a column is already upper-triangular to the precision
    // available, so its reflector is skipped and Q keeps the identity column.
    let eps = Re::<T>::epsilon();
    let mut frob_sq = Re::<T>::zero();
    for x in &r {
        frob_sq += x.modulus_squared();
    }
    let floor = frob_sq * eps * eps;

    for j in 0..k {
        let mut norm_sq = Re::<T>::zero();
        for i in j..m {
            norm_sq += r[i * n + j].modulus_squared();
        }
        if norm_sq <= floor {
            continue;
        }
        let norm = norm_sq.sqrt();

        // α = −phase(r[j,j])·‖x‖. Choosing the phase against the pivot avoids cancellation, and
        // reduces to the real ±sign convention when the phase is ±1.
        let pivot = r[j * n + j];
        let pmod = pivot.modulus();
        let phase = if pmod > Re::<T>::zero() {
            pivot * T::from_real(Re::<T>::one() / pmod)
        } else {
            T::one()
        };
        let alpha = -(phase * T::from_real(norm));

        // v = x − α·e_j, supported on rows j..m.
        let mut v = vec![T::zero(); m];
        for i in j..m {
            v[i] = r[i * n + j];
        }
        v[j] -= alpha;

        let mut v_norm_sq = Re::<T>::zero();
        for vi in v.iter().skip(j) {
            v_norm_sq += vi.modulus_squared();
        }
        if v_norm_sq <= Re::<T>::zero() {
            continue;
        }
        let beta = (Re::<T>::one() + Re::<T>::one()) / v_norm_sq; // 2 / (vᴴv)

        // R's trailing columns: R_col -= β·(vᴴ R_col)·v.
        for col in j..n {
            let mut dot = T::zero();
            for i in j..m {
                dot += v[i].conjugate() * r[i * n + col];
            }
            let factor = T::from_real(beta) * dot;
            for i in j..m {
                r[i * n + col] -= factor * v[i];
            }
        }
        // Q ← Q·H = Q − β·(Q v)·vᴴ.
        for row in 0..m {
            let mut dot = T::zero();
            for i in j..m {
                dot += q[row * m + i] * v[i];
            }
            let factor = T::from_real(beta) * dot;
            for i in j..m {
                q[row * m + i] -= factor * v[i].conjugate();
            }
        }
    }

    // Thin Q: the first k columns of the accumulator.
    let mut q_thin = vec![T::zero(); m * k];
    for i in 0..m {
        for c in 0..k {
            q_thin[i * k + c] = q[i * m + c];
        }
    }
    // Thin R: the first k rows, with the strictly-lower triangle cleaned to exact zero. The
    // reflectors leave rounding noise there, and a caller reading R as triangular should not have
    // to decide whether 1e-17 counts.
    let mut r_thin = vec![T::zero(); k * n];
    for i in 0..k {
        for col in 0..n {
            r_thin[i * n + col] = if col < i { T::zero() } else { r[i * n + col] };
        }
    }

    (q_thin, r_thin, k)
}

/// Conjugate-transposes a row-major `rows × cols` buffer into `cols × rows`.
///
/// A plain transpose for a real scalar, a Hermitian transpose for a complex one.
pub(crate) fn conj_transpose<T: ConjugateScalar>(data: &[T], rows: usize, cols: usize) -> Vec<T> {
    let mut out = vec![T::zero(); rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            out[j * rows + i] = data[i * cols + j].conjugate();
        }
    }
    out
}

/// One-sided Jacobi SVD of a **tall-or-square** matrix (`rows ≥ cols`).
///
/// Returns `(u, sigma, v)` with `u` row-major `rows × cols` carrying orthonormal columns wherever
/// the singular value is non-zero, `sigma` the length-`cols` vector of **real** singular values
/// (unsorted), and `v` the `cols × cols` unitary matrix of right singular vectors.
///
/// Columns are orthogonalised under the Hermitian inner product `⟨x|y⟩ = Σ x̄ᵢ yᵢ`, and each 2×2
/// sub-problem is reduced to a real Jacobi rotation by splitting the complex off-diagonal into a
/// modulus and a phase. For a real scalar the phase is `±1` and this is the ordinary real one-sided
/// Jacobi.
///
/// Chosen over power iteration because it converges for repeated and clustered singular values —
/// the identity has them in abundance, and the captured baseline showed power iteration reaching
/// only about `1e-8` there.
///
/// **Tall-or-square is a precondition, not a check.** A wide matrix is handled by the caller, which
/// decomposes the conjugate transpose and swaps the roles of `U` and `V` on the way out.
pub(crate) fn jacobi_svd<T>(mut u: Vec<T>, rows: usize, cols: usize) -> (Vec<T>, Vec<Re<T>>, Vec<T>)
where
    T: ConjugateScalar,
{
    let two = Re::<T>::one() + Re::<T>::one();
    // A small multiple of the working epsilon, so the threshold scales with the precision rather
    // than being pinned to `f64`'s.
    let mut tol = Re::<T>::epsilon();
    for _ in 0..6 {
        tol = tol + tol; // epsilon · 64
    }
    let max_sweeps = 60usize;

    let mut v = vec![T::zero(); cols * cols];
    for i in 0..cols {
        v[i * cols + i] = T::one();
    }

    // Noise floor on a column's squared norm: below `‖A‖·ε` a column is numerically zero. A
    // rank-deficient matrix drives its surplus columns there, and rotating a near-zero column gives
    // pathological angles — `gmod → 0` makes `ζ → ∞` — that overflow or NaN the rotation. Skipping
    // such a column leaves its singular value at ~0, which the caller's rank gate then drops.
    let eps = Re::<T>::epsilon();
    let mut max_diag = Re::<T>::zero();
    for j in 0..cols {
        let mut nrm = Re::<T>::zero();
        for i in 0..rows {
            nrm += u[i * cols + j].modulus_squared();
        }
        if nrm > max_diag {
            max_diag = nrm;
        }
    }

    // The sweep is scale-free in exact arithmetic and is not in floating point, because every
    // magnitude here is reached by squaring. `gmod` is `|γ|` computed as `sqrt(γ·conj(γ))`, so it
    // needs `|γ|²` to be representable — and for a matrix of small entries it is not. At
    // `|γ| < 2⁻⁵³⁷` the square underflows to zero, `gmod` is zero, `rel` is zero, `rel <= tol`
    // holds for every column pair, no rotation is ever applied, and this returns the raw column
    // norms as though they were singular values. Silently: the factors still multiply back to the
    // input exactly, so a reconstruction check passes while `U` is not orthogonal.
    //
    // Measured on `[[x, x], [0, x]]`, whose singular values are `x(√5±1)/2`: correct at
    // `x = 2⁻²⁶⁰`, and at `2⁻²⁷⁰` it returned `(√2·x, x)` — the two column norms exactly.
    //
    // Fixed by working at unit scale. `scale` is a power of two, so multiplying by it and dividing
    // the singular values by it afterwards are both exact: a well-scaled matrix decomposes to the
    // same bits it did before, and a badly-scaled one is brought into range rather than losing its
    // off-diagonal to underflow.
    let four = two * two;
    // The reciprocals are exact — both divisors are powers of two — so scaling down multiplies by
    // them rather than dividing, which `Real` supports as an assign operation and which produces
    // the identical bits.
    let quarter = Re::<T>::one() / four;
    let half = Re::<T>::one() / two;
    let mut scale = Re::<T>::one();
    // Bounded rather than `while`: an entry above `√MAX` squares to infinity, and `∞ · ¼` is `∞`,
    // so an unbounded loop never leaves. The bound is far outside any binary exponent range — it
    // cannot stop a real rescale early, and it makes termination independent of the input.
    const MAX_RESCALE_STEPS: usize = 4096;
    if max_diag > Re::<T>::zero() && max_diag.is_finite() {
        // `max_diag` is a squared norm, so stepping it by four steps the scale by two.
        let mut steps = 0usize;
        while max_diag < Re::<T>::one() && steps < MAX_RESCALE_STEPS {
            max_diag *= four;
            scale = scale + scale;
            steps += 1;
        }
        while max_diag >= four && steps < MAX_RESCALE_STEPS {
            max_diag *= quarter;
            scale *= half;
            steps += 1;
        }
    }
    if scale != Re::<T>::one() {
        let f = T::from_real(scale);
        for entry in u.iter_mut() {
            *entry *= f;
        }
    }
    let floor = max_diag * eps * eps;

    for _sweep in 0..max_sweeps {
        let mut max_off = Re::<T>::zero();
        for p in 0..cols {
            for q in (p + 1)..cols {
                // The Hermitian Gram entries of columns p and q. The diagonal ones are real; the
                // off-diagonal one is not.
                let mut alpha = Re::<T>::zero();
                let mut beta = Re::<T>::zero();
                let mut gamma = T::zero();
                for i in 0..rows {
                    let uip = u[i * cols + p];
                    let uiq = u[i * cols + q];
                    alpha += uip.modulus_squared();
                    beta += uiq.modulus_squared();
                    gamma += uip.conjugate() * uiq;
                }
                if alpha <= floor || beta <= floor {
                    continue;
                }
                let gmod = gamma.modulus();
                let denom = (alpha * beta).sqrt();
                if denom > Re::<T>::zero() {
                    // The off-diagonal measured *relative* to the two column norms. An absolute
                    // test would never fire for a large-magnitude matrix and always fire for a
                    // small one.
                    let rel = gmod / denom;
                    if rel > max_off {
                        max_off = rel;
                    }
                    if rel <= tol {
                        continue;
                    }
                } else {
                    continue;
                }

                let zeta = (beta - alpha) / (two * gmod);
                let sign = if zeta < Re::<T>::zero() {
                    -Re::<T>::one()
                } else {
                    Re::<T>::one()
                };
                // `sqrt(1 + ζ²)` without overflowing. A near-zero column drives `gmod → 0` and so
                // `ζ → ∞`, where `ζ²` overflows and `sqrt(∞)` is NaN on a double-double scalar,
                // poisoning the rotation. Factoring `|ζ|` out leaves `1/ζ²` to underflow harmlessly.
                let az = zeta.abs();
                let root = if az > Re::<T>::one() {
                    let inv = Re::<T>::one() / az;
                    az * (Re::<T>::one() + inv * inv).sqrt()
                } else {
                    (Re::<T>::one() + zeta * zeta).sqrt()
                };
                let t = sign / (az + root);
                let c = Re::<T>::one() / (Re::<T>::one() + t * t).sqrt();
                let s = c * t;

                // The complex Givens rotation with phase ρ = γ/|γ|:
                //   x'_p = c·x_p − conj(ρ)·s·x_q,   x'_q = ρ·s·x_p + c·x_q
                // For a real scalar ρ = ±1 and this is the ordinary real Jacobi rotation.
                let rho = gamma * T::from_real(Re::<T>::one() / gmod);
                let ct = T::from_real(c);
                let es = rho * T::from_real(s);
                let conj_es = rho.conjugate() * T::from_real(s);

                for i in 0..rows {
                    let uip = u[i * cols + p];
                    let uiq = u[i * cols + q];
                    u[i * cols + p] = ct * uip - conj_es * uiq;
                    u[i * cols + q] = es * uip + ct * uiq;
                }
                for i in 0..cols {
                    let vip = v[i * cols + p];
                    let viq = v[i * cols + q];
                    v[i * cols + p] = ct * vip - conj_es * viq;
                    v[i * cols + q] = es * vip + ct * viq;
                }
            }
        }
        if max_off <= tol {
            break;
        }
    }

    // The singular values are the column norms; normalising the columns turns `u` into the left
    // factor in place.
    let mut sigma = vec![Re::<T>::zero(); cols];
    for j in 0..cols {
        let mut norm_sq = Re::<T>::zero();
        for i in 0..rows {
            norm_sq += u[i * cols + j].modulus_squared();
        }
        let norm = norm_sq.sqrt();
        // Back to the caller's scale. Exact: `scale` is a power of two.
        sigma[j] = norm / scale;
        if norm > Re::<T>::zero() {
            let inv_norm = T::from_real(Re::<T>::one() / norm);
            for i in 0..rows {
                u[i * cols + j] *= inv_norm;
            }
        }
    }

    (u, sigma, v)
}
