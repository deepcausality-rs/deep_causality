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
    let _ = (apply, b, max_iterations, tolerance);
    todo!("cg_solve")
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
    let _ = (apply, b, inv_diagonal, max_iterations, tolerance);
    todo!("cg_solve_preconditioned")
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
    let _ = (apply, b, inv_diagonal, initial, max_iterations, tolerance);
    todo!("cg_solve_preconditioned_from")
}
