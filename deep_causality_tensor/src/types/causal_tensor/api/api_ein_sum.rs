/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, CausalTensorError, EinSumAST};
use std::ops::{Add, Mul};

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>,
{
    /// Public API for Einstein summation.
    ///
    /// This method serves as the entry point for performing Einstein summation operations
    /// on `CausalTensor`s. It takes an `EinSumAST` (Abstract Syntax Tree) as input,
    /// which defines the sequence of tensor operations to be executed.
    ///
    /// # Arguments
    ///
    /// * `ast` - A reference to the `EinSumAST` that describes the Einstein summation operation.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(CausalTensor<T>)` containing the result of the Einstein summation.
    /// - `Err(CausalTensorError)` if any error occurs during the execution of the AST.
    ///
    /// # Errors
    ///
    /// Returns errors propagated from `execute_ein_sum`.
    pub fn ein_sum(ast: &EinSumAST<T>) -> Result<CausalTensor<T>, CausalTensorError> {
        Self::execute_ein_sum(ast)
    }
}
