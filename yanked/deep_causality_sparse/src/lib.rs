/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! **This crate is retired and exports nothing. Use [`deep_causality_linear`].**
//!
//! | this crate | `deep_causality_linear` |
//! |---|---|
//! | `CsrMatrix` | `CsrMatrix` |
//! | `CsrMatrixWitness` | `CsrMatrixWitness` |
//! | `cg_solve`, `cg_solve_preconditioned`, `cg_solve_preconditioned_from` | same names |
//! | `CgFailure` | `CgFailure` (an enum) |
//! | `SparseMatrixError` | `LinearError` |
//! | `CsrFromTensorError`, `to_dense` | in `deep_causality_tensor` |
//!
//! [`deep_causality_linear`]: https://docs.rs/deep_causality_linear

#![no_std]
