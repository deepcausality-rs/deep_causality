/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensor;
use deep_causality_ast::ConstTree;

// The core enum for the EinSum AST. T is the element type (e.g., Complex<f64>)
pub enum EinSumOp<T> {
    // LEAF: The source Tensor data
    TensorSource {
        tensor: CausalTensor<T>,
    },

    // GENERIC OPS (for flexibility)
    Contraction {
        lhs_axes: Vec<usize>,
        rhs_axes: Vec<usize>,
    },
    Reduction {
        axes: Vec<usize>,
    },

    // EXPLICIT EINSUM OPS (for usability)
    MatMul,  // Standard matrix multiplication (rank-2 tensors)
    DotProd, // Dot product (rank-1 tensors)
    Trace {
        axes1: usize,
        axes2: usize,
    }, // Trace over two specified axes
    TensorProduct, // Outer product
    ElementWiseProduct, // Hadamard product
    Transpose {
        new_order: Vec<usize>,
    },
    DiagonalExtraction {
        axes1: usize,
        axes2: usize,
    },
    BatchMatMul, // Batch matrix multiplication (rank-3 tensors)
}

// The EinSum AST
pub type EinSumAST<T> = ConstTree<EinSumOp<T>>;

impl<T> EinSumOp<T> {
    pub fn tensor_source(tensor: CausalTensor<T>) -> EinSumAST<T> {
        EinSumAST::new(EinSumOp::TensorSource { tensor })
    }

    pub fn contraction(
        lhs: CausalTensor<T>,
        rhs: CausalTensor<T>,
        lhs_axes: Vec<usize>,
        rhs_axes: Vec<usize>,
    ) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(
            EinSumOp::Contraction { lhs_axes, rhs_axes },
            vec![lhs_leaf, rhs_leaf],
        )
    }

    pub fn reduction(operand: CausalTensor<T>, axes: Vec<usize>) -> EinSumAST<T> {
        let operand_leaf = EinSumOp::tensor_source(operand);
        EinSumAST::with_children(EinSumOp::Reduction { axes }, vec![operand_leaf])
    }

    pub fn mat_mul(lhs: CausalTensor<T>, rhs: CausalTensor<T>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::MatMul, vec![lhs_leaf, rhs_leaf])
    }

    pub fn dot_prod(lhs: CausalTensor<T>, rhs: CausalTensor<T>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::DotProd, vec![lhs_leaf, rhs_leaf])
    }

    pub fn trace(operand: CausalTensor<T>, axes1: usize, axes2: usize) -> EinSumAST<T> {
        let operand_leaf = EinSumOp::tensor_source(operand);
        EinSumAST::with_children(EinSumOp::Trace { axes1, axes2 }, vec![operand_leaf])
    }

    pub fn tensor_product(lhs: CausalTensor<T>, rhs: CausalTensor<T>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::TensorProduct, vec![lhs_leaf, rhs_leaf])
    }

    pub fn element_wise_product(lhs: CausalTensor<T>, rhs: CausalTensor<T>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::ElementWiseProduct, vec![lhs_leaf, rhs_leaf])
    }

    pub fn transpose(operand: CausalTensor<T>, new_order: Vec<usize>) -> EinSumAST<T> {
        let operand_leaf = EinSumOp::tensor_source(operand);
        EinSumAST::with_children(EinSumOp::Transpose { new_order }, vec![operand_leaf])
    }

    pub fn diagonal_extraction(
        operand: CausalTensor<T>,
        axes1: usize,
        axes2: usize,
    ) -> EinSumAST<T> {
        let operand_leaf = EinSumOp::tensor_source(operand);
        EinSumAST::with_children(
            EinSumOp::DiagonalExtraction { axes1, axes2 },
            vec![operand_leaf],
        )
    }

    pub fn batch_mat_mul(lhs: CausalTensor<T>, rhs: CausalTensor<T>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::BatchMatMul, vec![lhs_leaf, rhs_leaf])
    }
}
