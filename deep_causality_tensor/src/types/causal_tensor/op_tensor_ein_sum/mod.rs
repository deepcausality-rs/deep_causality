/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
pub(crate) mod ein_sum_impl;
pub(crate) mod ein_sum_op;

use crate::errors::causal_tensor_error::CausalTensorError;
use crate::types::causal_tensor::CausalTensor;
use ein_sum_op::{EinSumAST, EinSumOp};
use std::ops::{Add, Mul};

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>,
{
    /// Public API for Einstein summation.
    pub fn einsum(ast: &EinSumAST<T>) -> Result<CausalTensor<T>, CausalTensorError> {
        Self::execute_ein_sum(ast)
    }

    /// Executes the Einstein summation by recursively traversing the AST.
    fn execute_ein_sum(ast: &EinSumAST<T>) -> Result<CausalTensor<T>, CausalTensorError> {
        let node = ast.value();
        let children = ast.children();

        match node {
            EinSumOp::TensorSource { tensor } => Ok(tensor.clone()),

            // Generic Ops
            EinSumOp::Contraction { lhs_axes, rhs_axes } => {
                let (lhs, rhs) = Self::get_binary_operands(children)?;
                Self::contract(&lhs, &rhs, lhs_axes, rhs_axes)
            }
            EinSumOp::Reduction { axes } => {
                let operand = Self::get_unary_operand(children)?;
                operand.sum_axes(axes)
            }

            // Explicit Ops
            EinSumOp::MatMul => {
                let (lhs, rhs) = Self::get_binary_operands(children)?;
                Self::contract(&lhs, &rhs, &[1], &[0]) // Contract last axis of LHS with first of RHS
            }
            EinSumOp::DotProd => {
                let (lhs, rhs) = Self::get_binary_operands(children)?;
                Self::contract(&lhs, &rhs, &[0], &[0]) // Contract the single axis of both vectors
            }
            EinSumOp::Trace { axes1, axes2 } => {
                let operand = Self::get_unary_operand(children)?;
                Self::trace(&operand, *axes1, *axes2)
            }
            EinSumOp::TensorProduct => {
                let (lhs, rhs) = Self::get_binary_operands(children)?;
                lhs.tensor_product(&rhs)
            }
            EinSumOp::ElementWiseProduct => {
                let (lhs, rhs) = Self::get_binary_operands(children)?;
                Self::element_wise_mul(&lhs, &rhs)
            }
            EinSumOp::Transpose { new_order } => {
                let operand = Self::get_unary_operand(children)?;
                operand.permute_axes(new_order)
            }
            EinSumOp::DiagonalExtraction { axes1, axes2 } => {
                let operand = Self::get_unary_operand(children)?;
                Self::diagonal(&operand, *axes1, *axes2)
            }
            EinSumOp::BatchMatMul => {
                let (lhs, rhs) = Self::get_binary_operands(children)?;
                // Assuming batch dimension is the first (0)
                Self::contract(&lhs, &rhs, &[2], &[1])
            }
        }
    }
}
