/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! **This crate is retired. Its contents live in [`deep_causality_linear`].**
//!
//! Everything below is a re-export. The implementations moved, and the names are kept so that a
//! caller written against this crate keeps compiling while it repoints.
//!
//! # What to use instead
//!
//! | this crate | `deep_causality_linear` |
//! |---|---|
//! | `CsrMatrix` | `CsrMatrix` |
//! | `CsrMatrixWitness` | `CsrMatrixWitness` |
//! | `cg_solve`, `cg_solve_preconditioned`, `cg_solve_preconditioned_from` | same names |
//! | `CgFailure` | `CgFailure` — **now an enum**, see below |
//! | `SparseMatrixError` | `LinearError` — one error type across representations |
//! | `CsrFromTensorError`, `to_dense` | moved to `deep_causality_tensor` |
//!
//! `deep_causality_linear` also carries the dense, bit-packed 𝔽₂ and vector representations, the
//! eliminations, the decompositions and the exact integer path — none of which had a home here.
//!
//! # Two names are re-exported with a different shape
//!
//! Both keep their old names. Their shapes changed, and carrying the old shapes forward would
//! have been worse than the break.
//!
//! **`CgFailure` changed from a struct to an enum.** It was one struct carrying `iterations` and
//! `residual` for every failure mode, including two that are not non-convergence and have no
//! residual to report. It is now three named cases. Code that destructured the struct gets a
//! compile error rather than a plausible wrong message, which is the good outcome.
//!
//! **`CsrMatrixWitness` no longer claims `Monad` or `Adjunction`.** The `bind` here violates monad
//! right identity: it flattens to `1 × count` and renumbers the columns, so a sparse row comes back
//! with its non-zeros in different places. Measured cases are in
//! `openspec/notes/unified_math/HKT-LAW-FINDINGS.md`. `Functor`, `Foldable`, `Pure`, `Applicative` and
//! `CoMonad` are all present and lawful.
//!
//! # Where the old implementation went
//!
//! `deep_causality_sparse/reverted/` holds it, detached from the build. It is the record of what
//! the replacement was checked against — `deep_causality_linear`'s `ported_*` test files are that
//! suite, run against the new implementation.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Solvers
pub use deep_causality_linear::{
    CgFailure, cg_solve, cg_solve_preconditioned, cg_solve_preconditioned_from,
};

// The sparse representation and its witness
pub use deep_causality_linear::{CsrMatrix, CsrMatrixWitness};

/// The error type, under its former name.
///
/// `deep_causality_linear` carries one error across every representation rather than one per
/// container, so the sparse-specific type is an alias for it. The four failures this crate named
/// are all present; two of them carry more than they used to — a positional `(row, col)` where the
/// old one carried a flat index, and a split between a product's inner dimensions and a vector's
/// length, which the old `DimensionMismatch` conflated.
pub use deep_causality_linear::LinearError as SparseMatrixError;
