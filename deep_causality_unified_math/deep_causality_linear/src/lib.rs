/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Linear algebra for the DeepCausality crates.
//!
//! This crate owns the workspace's matrix representations and the algorithms over them. The split
//! against `deep_causality_tensor` is by **arity, not density**: a two-index object is a matrix
//! and lives here; `ein_sum`, `broadcast`, `kronecker`, the axis reductions and the tensor-train
//! stack take an N-index object and stay there.
//!
//! # Three representations, side by side
//!
//! Choosing a representation is the central decision in linear algebra, and this crate holds all
//! three so that the choice is local rather than architectural:
//!
//! | representation | for |
//! |---|---|
//! | compressed sparse row | boundary and coboundary operators, discrete Laplacians |
//! | dense row-major | covariance, metric tensors, density matrices |
//! | bit-packed 𝔽₂ | mod-2 elimination, homology read as a code |
//!
//! plus a dense vector, which the construction census found to be the larger need: 60 rank-1
//! constructions against 46 rank-2 across the seven consumer crates.
//!
//! # Banded on the tower, not on `f64`
//!
//! Every operation is bounded on the weakest trait from `deep_causality_algebra` that makes it
//! correct, so the crate is generic over the integers as well as the floats. The determinant is a
//! polynomial in the entries and needs no division, so it is defined over any commutative ring —
//! bounding it on `Field` is what would have excluded ℤ. Gaussian elimination divides by its pivot
//! and so leaves ℤ on the first step, and is bounded accordingly.
//!
//! The crate defines no scalar trait, marker or newtype of its own.
//!
//! # Position in the tier graph
//!
//! This crate sits **below** `deep_causality_tensor` and must not depend on it under any feature.
//! `CausalTensor`'s decompositions delegate into here, and a crate can only call into what it
//! depends on. Given that edge, the orphan rule leaves `deep_causality_tensor` as the only place the
//! access-trait impl for `CausalTensor` can be written.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate core;

mod algorithms;
mod errors;
mod extensions;
mod traits;
mod types;
// Test fixtures, public because Bazel test targets cannot reach the `tests` tree, hidden
// because they are not API.
#[doc(hidden)]
pub mod utils_tests;

// Errors
pub use crate::errors::linear_error::{LinearError, LinearErrorEnum};

// Access traits
pub use crate::traits::matrix_build::MatrixBuild;
pub use crate::traits::matrix_view::MatrixView;
pub use crate::traits::row_ops::RowOps;

// Containers
pub use crate::types::csr_matrix::CsrMatrix;
pub use crate::types::dense_matrix::DenseMatrix;
pub use crate::types::dense_vector::DenseVector;
pub use crate::types::packed_gf2::PackedGf2;
pub use crate::types::packed_gf2_vector::PackedGf2Vector;

// Elimination and everything read off it
pub use crate::algorithms::elimination::{
    Reduced, determinant, image_basis, kernel_basis, rank, rank_stable, rref, rref_stable,
};

// Solving
pub use crate::algorithms::cholesky::{cholesky, solve_least_squares};
pub use crate::algorithms::solve::{Lu, inverse, solve, solve_lower, solve_upper};

// Exact paths, kept distinct from each other and from the numerical one
pub use crate::algorithms::gf2::{image_basis_gf2, kernel_basis_gf2, rank_gf2};
pub use crate::algorithms::integer::{determinant_exact, rank_exact};

// Decompositions, whose bodies `CausalTensor` delegates to
pub use crate::algorithms::decomposition::{
    EigenPair, QrFactors, SvdFactors, SvdReal, Truncation, eigen_hermitian, qr, singular_values,
    svd, svd_sorted, svd_truncated,
};

// Norms, defined once. The `DenseVector` methods of the same names delegate to the vector forms.
pub use crate::algorithms::norms::{
    matrix_norm_frobenius, matrix_norm_inf, matrix_norm_l1, vector_norm_inf, vector_norm_l1,
    vector_norm_l2, vector_norm_sq,
};

// Conjugate gradient, matrix-free
pub use crate::algorithms::cg::{
    CgFailure, cg_solve, cg_solve_preconditioned, cg_solve_preconditioned_from,
};

// Conversions among the representations, explicit at every call site
pub use crate::extensions::conversions::{
    csr_i8_to_dense_i64, csr_to_dense, csr_to_packed_gf2_mod2, csr_to_packed_gf2_strict,
    dense_gf2_to_packed, dense_to_csr, packed_to_dense_gf2,
};

// Higher-kinded witnesses
pub use crate::extensions::hkt::csr_matrix_witness::CsrMatrixWitness;
pub use crate::extensions::hkt::dense_matrix_witness::DenseMatrixWitness;
pub use crate::extensions::hkt::dense_vector_witness::DenseVectorWitness;
pub use crate::extensions::hkt::zip_dense_vector_witness::ZipDenseVectorWitness;
