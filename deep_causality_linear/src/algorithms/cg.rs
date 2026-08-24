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
//! repoints every caller, so the signature is a contract rather than a preference. `openspec/specs/neumann-poisson/spec.md` names the preconditioned variant
//! normatively, and that requirement moves with the code.
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

    let mut x = initial.to_vec();
    let ax = apply(&x);
    let mut r: Vec<R> = b.iter().zip(&ax).map(|(bb, a)| *bb - *a).collect();
    let mut z = precondition(&r);
    let mut p = z.clone();
    let mut rz = dot(&r, &z);

    for iteration in 0..max_iterations {
        let residual = dot(&r, &r).sqrt();
        if residual <= tolerance {
            return Ok(x);
        }
        let ap = apply(&p);
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
    if residual <= tolerance {
        return Ok(x);
    }
    Err(CgFailure::NotConverged {
        iterations: max_iterations,
        residual,
    })
}

/// Solves `Ax = b` by Jacobi-preconditioned conjugate gradient.
///
/// `inv_diagonal` supplies the reciprocal of the operator's diagonal, which is the preconditioner.
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
