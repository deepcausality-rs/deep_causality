/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod errors;
mod types;
mod extensions;

// Sparse type
pub use crate::errors::SparseMatrixError;
pub use crate::extensions::ext_hkt::CsrMatrixWitness;
pub use crate::types::sparse_matrix::CsrMatrix;
