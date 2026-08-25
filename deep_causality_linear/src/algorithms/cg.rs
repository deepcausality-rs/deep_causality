/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Conjugate gradient, matrix-free.
//!
//! Moves here from `deep_causality_sparse` with its signatures, convergence behaviour and iteration
//! counts unchanged.
//!
//! # The parameter order is the old one, deliberately
//!
//! `(apply, diag_a, b, tolerance, max_iterations)` reads awkwardly — the operator's diagonal before
//! the right-hand side, the tolerance before the iteration cap — and it is kept exactly.
//!
//! An earlier version of this module reordered it to `(apply, b, inv_diagonal, max_iterations,
//! tolerance)` and took the **reciprocal** of the diagonal rather than the diagonal. Porting the
//! sparse crate's own tests found it. Both breaks are silent: `diag_a` and `b` are both `&[R]`, so
//! swapping them compiles and preconditions on the right-hand side; and passing a diagonal where a
//! reciprocal is expected computes a different preconditioner with nothing to catch it.
//!
//! `openspec/specs/neumann-poisson/spec.md` names the preconditioned variant normatively and phase 5
//! repoints every caller, so the signature is a contract rather than a preference.
//!
//! # The convergence threshold is relative, also deliberately
//!
//! `tolerance` is scaled by `‖b‖`. An earlier version compared against it directly, which makes the
//! criterion `‖b‖` times stricter than the caller asked for — silent, because the answer is right
//! when it converges at all, and the difference shows up only as a solve that stops converging
//! within its iteration budget. `deep_causality_topology` documents its default as a "tight
//! relative residual", so the scaling is the contract its callers were written against.
//!
//! `‖b‖ = 0` has no scale to be relative to and takes the tolerance as given.
//!
//! # What the operator returns is checked
//!
//! `apply` is the caller's closure and can return a vector of the wrong length. Zipping against `b`
//! would truncate the longer case and index past the end of the residual in the shorter one, so
//! both are rejected with `LengthMismatch` before they reach the iteration.
//!
//! # Where this does differ from the sparse crate
//!
//! Two places, both deliberate:
//!
//! - [`CgFailure`] is an enum of three named cases where the sparse crate had one struct carrying
//!   `iterations` and `residual` for every failure mode. Three failure modes that need different
//!   responses should not share one shape.
//! - The curvature guard rejects any non-positive `pᵀAp`, where the sparse crate rejected only an
//!   exact zero. A negative value means the operator is not positive definite, and dividing by it
//!   takes a step in the wrong direction.
//!
//! # Matrix-free
//!
//! These take a closure applying the operator rather than a matrix. That is what lets a caller with
//! a Laplacian it never assembles use them, and it is why the sparse-versus-dense question does not
//! arise here at all.
//!
//! # The sparse path stays iterative
//!
//! LU on a sparse matrix fills in: the factors are dense even when the matrix is not, so applying
//! the dense path to a large sparse system silently allocates the square. These cover the symmetric
//! positive-definite case, which is the one the workspace actually solves.

use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Why a conjugate-gradient solve stopped without converging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CgFailure<R> {
    /// The iteration limit was reached. Carries the residual it reached.
    NotConverged { iterations: usize, residual: R },
    /// A breakdown: the operator is not positive definite, so a search direction had non-positive
    /// curvature.
    NotPositiveDefinite { iteration: usize },
    /// The right-hand side's length does not match what the operator produces.
    LengthMismatch { expected: usize, found: usize },
}

impl<R: core::fmt::Display> core::fmt::Display for CgFailure<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CgFailure::NotConverged {
                iterations,
                residual,
            } => write!(
                f,
                "conjugate gradient did not converge in {iterations} iterations (final residual {residual})"
            ),
            CgFailure::NotPositiveDefinite { iteration } => write!(
                f,
                "conjugate gradient broke down at iteration {iteration}: the operator is not positive definite, so a search direction had non-positive curvature"
            ),
            CgFailure::LengthMismatch { expected, found } => write!(
                f,
                "length mismatch: the operator produced {found} entries against a right-hand side of {expected}"
            ),
        }
    }
}

impl<R: core::fmt::Debug + core::fmt::Display> core::error::Error for CgFailure<R> {}

/// Solves `Ax = b` by conjugate gradient, with `apply` supplying `A`.
pub fn cg_solve<R, Apply>(
    apply: Apply,
    b: &[R],
    tolerance: R,
    max_iterations: usize,
) -> Result<Vec<R>, CgFailure<R>>
where
    R: RealField + FromPrimitive,
    Apply: Fn(&[R]) -> Vec<R>,
{
    let zero_start = vec![R::zero(); b.len()];
    cg_core(
        apply,
        b,
        CgSettings {
            diag_a: None,
            initial: &zero_start,
            max_iterations,
            tolerance,
        },
    )
}

/// What separates the three public entry points from each other.
///
/// Grouped rather than passed as four positional parameters: `cg_core` would otherwise take seven,
/// and at that width a caller transposing `max_iterations` and a length would get a silently
/// different solve rather than a compile error. The names make the call sites read as configuration.
struct CgSettings<'a, R> {
    /// The **diagonal of `A`**, from which the Jacobi preconditioner `M⁻¹ = diag(1/diag_a)` is
    /// formed here — or `None` for the plain iteration, where the preconditioning step is the
    /// identity.
    ///
    /// The diagonal itself rather than its reciprocal. The two are the same type and the opposite
    /// quantity, so taking one where the other is meant is a defect nothing catches; this matches
    /// the signature the code being moved has, so a repointed caller keeps working.
    ///
    /// An entry at or below zero is treated as `1` — no preconditioning on that row — which keeps
    /// the preconditioner positive definite for clipped or partially-degenerate diagonals.
    diag_a: Option<&'a [R]>,
    /// Where the iteration starts. Zero unless a caller supplies a guess.
    initial: &'a [R],
    max_iterations: usize,
    tolerance: R,
}

/// The iteration every entry point here runs.
///
/// `inv_diagonal` is the Jacobi preconditioner when present; absent, the preconditioned step is the
/// identity and this is plain conjugate gradient. One body rather than three, so the three cannot
/// drift apart.
fn cg_core<R, Apply>(
    apply: Apply,
    b: &[R],
    settings: CgSettings<'_, R>,
) -> Result<Vec<R>, CgFailure<R>>
where
    R: RealField + FromPrimitive,
    Apply: Fn(&[R]) -> Vec<R>,
{
    let CgSettings {
        diag_a,
        initial,
        max_iterations,
        tolerance,
    } = settings;

    let n = b.len();
    if initial.len() != n {
        return Err(CgFailure::LengthMismatch {
            expected: n,
            found: initial.len(),
        });
    }
    if let Some(d) = diag_a
        && d.len() != n
    {
        return Err(CgFailure::LengthMismatch {
            expected: n,
            found: d.len(),
        });
    }

    let precondition = |v: &[R]| -> Vec<R> {
        match diag_a {
            Some(d) => v
                .iter()
                .zip(d)
                .map(|(&x, &di)| if di > R::zero() { x / di } else { x })
                .collect(),
            None => v.to_vec(),
        }
    };
    let dot = |u: &[R], v: &[R]| -> R {
        let mut acc = R::zero();
        for (a, c) in u.iter().zip(v) {
            acc += *a * *c;
        }
        acc
    };

    // The tolerance is **relative** to `‖b‖`, which is the contract the callers were written
    // against: `deep_causality_topology` documents its default as a "tight relative residual".
    // Comparing against the raw tolerance instead makes the criterion `‖b‖` times stricter, so a
    // caller with a fixed iteration budget stops converging on systems it used to solve. `‖b‖ = 0`
    // has no scale to be relative to, so the tolerance is taken as given.
    let b_norm = dot(b, b).sqrt();
    let abs_tol = if b_norm == R::zero() {
        tolerance
    } else {
        tolerance * b_norm
    };

    let mut x = initial.to_vec();
    let ax = apply(&x);
    // `zip` would stop at the shorter side and leave a short residual for the loop below to index
    // past. The operator is the caller's closure, so a wrong length is a caller error and gets the
    // same typed failure the directly-passed arguments get.
    if ax.len() != n {
        return Err(CgFailure::LengthMismatch {
            expected: n,
            found: ax.len(),
        });
    }
    let mut r: Vec<R> = b.iter().zip(&ax).map(|(bb, a)| *bb - *a).collect();
    let mut z = precondition(&r);
    let mut p = z.clone();
    let mut rz = dot(&r, &z);

    for iteration in 0..max_iterations {
        let residual = dot(&r, &r).sqrt();
        if residual <= abs_tol {
            return Ok(x);
        }
        let ap = apply(&p);
        if ap.len() != n {
            return Err(CgFailure::LengthMismatch {
                expected: n,
                found: ap.len(),
            });
        }
        let denominator = dot(&p, &ap);
        if denominator <= R::zero() {
            return Err(CgFailure::NotPositiveDefinite { iteration });
        }
        let alpha = rz / denominator;
        for i in 0..n {
            x[i] += alpha * p[i];
            r[i] -= alpha * ap[i];
        }
        z = precondition(&r);
        let rz_next = dot(&r, &z);
        let beta = rz_next / rz;
        for i in 0..n {
            p[i] = z[i] + beta * p[i];
        }
        rz = rz_next;
    }

    let residual = dot(&r, &r).sqrt();
    if residual <= abs_tol {
        return Ok(x);
    }
    Err(CgFailure::NotConverged {
        iterations: max_iterations,
        residual,
    })
}

/// Solves `Ax = b` by Jacobi-preconditioned conjugate gradient.
///
/// `diag_a` is the operator's **diagonal**, not its reciprocal: the solver forms
/// `M⁻¹ = diag(1/diag_a)` itself. An entry at or below zero is left unpreconditioned rather than
/// inverted, which keeps the preconditioner positive definite for a clipped diagonal.
///
/// `tolerance` is relative to `‖b‖`, as in [`cg_solve`].
pub fn cg_solve_preconditioned<R, Apply>(
    apply: Apply,
    diag_a: &[R],
    b: &[R],
    tolerance: R,
    max_iterations: usize,
) -> Result<Vec<R>, CgFailure<R>>
where
    R: RealField + FromPrimitive,
    Apply: Fn(&[R]) -> Vec<R>,
{
    let zero_start = vec![R::zero(); b.len()];
    cg_core(
        apply,
        b,
        CgSettings {
            diag_a: Some(diag_a),
            initial: &zero_start,
            max_iterations,
            tolerance,
        },
    )
}

/// As [`cg_solve_preconditioned`], starting from a supplied initial guess.
pub fn cg_solve_preconditioned_from<R, Apply>(
    apply: Apply,
    diag_a: &[R],
    b: &[R],
    x0: &[R],
    tolerance: R,
    max_iterations: usize,
) -> Result<Vec<R>, CgFailure<R>>
where
    R: RealField + FromPrimitive,
    Apply: Fn(&[R]) -> Vec<R>,
{
    cg_core(
        apply,
        b,
        CgSettings {
            diag_a: Some(diag_a),
            initial: x0,
            max_iterations,
            tolerance,
        },
    )
}
