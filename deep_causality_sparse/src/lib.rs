/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod errors;
mod extensions;
mod types;

// Sparse type
pub use crate::errors::SparseMatrixError;
pub use crate::extensions::ext_hkt::CsrMatrixWitness;
pub use crate::types::sparse_matrix::CsrMatrix;
