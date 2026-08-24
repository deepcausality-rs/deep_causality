/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Conjugate gradient, matrix-free.
//!
//! Moves here from `deep_causality_sparse` with its signatures, convergence behaviour and iteration
//! counts unchanged. `openspec/specs/neumann-poisson/spec.md` names the preconditioned variant
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
    max_iterations: usize,
    tolerance: R,
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
            inv_diagonal: None,
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
    /// The Jacobi preconditioner — the reciprocal of the operator's diagonal — or `None` for the
    /// plain iteration, where the preconditioning step is the identity.
    inv_diagonal: Option<&'a [R]>,
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
        inv_diagonal,
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
    if let Some(d) = inv_diagonal
        && d.len() != n
    {
        return Err(CgFailure::LengthMismatch {
            expected: n,
            found: d.len(),
        });
    }

    let precondition = |v: &[R]| -> Vec<R> {
        match inv_diagonal {
            Some(d) => v.iter().zip(d).map(|(x, m)| *x * *m).collect(),
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
    b: &[R],
    inv_diagonal: &[R],
    max_iterations: usize,
    tolerance: R,
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
            inv_diagonal: Some(inv_diagonal),
            initial: &zero_start,
            max_iterations,
            tolerance,
        },
    )
}

/// As [`cg_solve_preconditioned`], starting from a supplied initial guess.
pub fn cg_solve_preconditioned_from<R, Apply>(
    apply: Apply,
    b: &[R],
    inv_diagonal: &[R],
    initial: &[R],
    max_iterations: usize,
    tolerance: R,
) -> Result<Vec<R>, CgFailure<R>>
where
    R: RealField + FromPrimitive,
    Apply: Fn(&[R]) -> Vec<R>,
{
    cg_core(
        apply,
        b,
        CgSettings {
            inv_diagonal: Some(inv_diagonal),
            initial,
            max_iterations,
            tolerance,
        },
    )
}
