
# Pre-Specification for Einstein Summation (einsum) Implementation

## 1. Overview

This document outlines the pre-specification for implementing Einstein Summation (`einsum`) within the `deep_causality_tensor` crate. The proposed implementation will depart from traditional string-based parsing and instead leverage an Abstract Syntax Tree (AST) built upon the `ConstTree` from the `deep_causality_ast` crate. This approach offers several advantages, including compile-time validation of contraction paths and enhanced performance by avoiding runtime string interpretation.

## 2. EinSum AST Definition

The core of the `einsum` implementation will be an `EinSumAST`, which is a `ConstTree` of `EinSumOp`s.

### 2.1. `EinSumOp` Enum (Explicit API)

The `EinSumOp` enum is designed to be explicit and user-friendly, mapping directly to common tensor operations.

```rust
// The core enum for the EinSum AST. T is the element type (e.g., Complex<f64>)
pub enum EinSumOp<T> {
    // LEAF: The source Tensor data
    TensorSource { tensor: CausalTensor<T> },

    // GENERIC OPS (for flexibility)
    Contraction { lhs_axes: Vec<usize>, rhs_axes: Vec<usize> },
    Reduction { axes: Vec<usize> },

    // EXPLICIT EINSUM OPS (for usability)
    MatMul,          // Standard matrix multiplication (rank-2 tensors)
    DotProd,         // Dot product (rank-1 tensors)
    Trace { axes1: usize, axes2: usize }, // Trace over two specified axes
    TensorProduct,   // Outer product
    ElementWiseProduct, // Hadamard product
    Transpose { new_order: Vec<usize> },
    DiagonalExtraction { axes1: usize, axes2: usize },
    BatchMatMul,     // Batch matrix multiplication (rank-3 tensors)
}
```

### 2.2. `EinSumAST` Type Alias

```rust
use deep_causality_ast::ConstTree;

// The EinSum AST
pub type EinSumAST<T> = ConstTree<EinSumOp<T>>;
```

### 2.3. AST Construction Helpers

To simplify the creation of the `EinSumAST`, a set of static helper functions are provided on `EinSumOp`.

```rust
impl<T> EinSumOp<T> {
    pub fn tensor_source(tensor: CausalTensor<T>) -> EinSumAST<T> {
        EinSumAST::new(EinSumOp::TensorSource { tensor })
    }

    pub fn contraction(lhs: CausalTensor<T>, rhs: CausalTensor<T>, lhs_axes: Vec<usize>, rhs_axes: Vec<usize>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::Contraction { lhs_axes, rhs_axes }, vec![lhs_leaf, rhs_leaf])
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

    pub fn diagonal_extraction(operand: CausalTensor<T>, axes1: usize, axes2: usize) -> EinSumAST<T> {
        let operand_leaf = EinSumOp::tensor_source(operand);
        EinSumAST::with_children(EinSumOp::DiagonalExtraction { axes1, axes2 }, vec![operand_leaf])
    }

    pub fn batch_mat_mul(lhs: CausalTensor<T>, rhs: CausalTensor<T>) -> EinSumAST<T> {
        let lhs_leaf = EinSumOp::tensor_source(lhs);
        let rhs_leaf = EinSumOp::tensor_source(rhs);
        EinSumAST::with_children(EinSumOp::BatchMatMul, vec![lhs_leaf, rhs_leaf])
    }
}
```

## 3. Proposed API

The `einsum` functionality will be exposed through a new method on `CausalTensor`.

```rust
use deep_causality_ast::ConstTree;

impl<T> CausalTensor<T> {
    /// Executes a tensor contraction based on the provided EinSum AST.
    pub fn einsum(ast: &EinSumAST<T>) -> Result<CausalTensor<T>, CausalTensorError> {
        Self::execute_ein_sum(ast)
    }
}
```

## 4. Error Handling

To provide more granular error reporting for AST-related issues, a dedicated `EinSumValidationError` enum is introduced.

```rust
/// Specific errors that can occur during EinSum AST validation or execution.
pub enum EinSumValidationError {
    /// Indicates an incorrect number of child nodes for an AST operation.
    InvalidNumberOfChildren { expected: usize, found: usize },
    /// Indicates an issue with the specified axes for an operation (e.g., out of bounds, duplicate).
    InvalidAxesSpecification { message: String },
    /// Indicates an operation that is not yet implemented or is used in an unsupported context.
    UnsupportedOperation { operation: String },
    /// Indicates a mismatch in tensor shapes that prevents an operation from proceeding.
    ShapeMismatch { message: String },
    /// Indicates that a tensor has an unexpected rank for a given operation.
    RankMismatch { expected: usize, found: usize },
}

/// General errors that can occur within the CausalTensor operations.
pub enum CausalTensorError {
    ShapeMismatch,
    DimensionMismatch,
    DivisionByZero,
    AxisOutOfBounds,
    EmptyTensor,
    InvalidOperation,
    UnorderableValue,
    InvalidParameter(String),
    /// Encapsulates errors specific to EinSum AST validation and execution.
    EinSumError(EinSumValidationError),
}
```

## 5. Implementation Draft

The implementation uses a dispatcher (`execute_ein_sum`) to map the explicit API to a set of private, generic helper methods.

```rust
use deep_causality_ast::ConstTree;
use std::ops::{Add, Mul};

impl<T>
    CausalTensor<T>
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

    // --- Private Helper Methods ---

    /// Helper to get two operands from the AST children.
    fn get_binary_operands(
        children: &[EinSumAST<T>],
    ) -> Result<(CausalTensor<T>, CausalTensor<T>), CausalTensorError> {
        if children.len() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidNumberOfChildren { expected: 2, found: children.len() },
            ));
        }
        let lhs = Self::execute_ein_sum(&children[0])?;
        let rhs = Self::execute_ein_sum(&children[1])?;
        Ok((lhs, rhs))
    }

    /// Helper to get a single operand from the AST children.
    fn get_unary_operand(children: &[EinSumAST<T>]) -> Result<CausalTensor<T>, CausalTensorError> {
        if children.len() != 1 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidNumberOfChildren { expected: 1, found: children.len() },
            ));
        }
        Self::execute_ein_sum(&children[0])
    }

    /// Private method for generic tensor contraction.
    /// This optimized version uses permutation and reshaping to reduce contraction to matrix multiplication.
    fn contract(
        lhs: &CausalTensor<T>,
        rhs: &CausalTensor<T>,
        lhs_contract_axes: &[usize],
        rhs_contract_axes: &[usize],
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        // 1. Validate input axes and shapes
        if lhs_contract_axes.len() != rhs_contract_axes.len() {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: "Number of LHS and RHS contraction axes must match".to_string(),
                },
            ));
        }

        for (&lhs_axis, &rhs_axis) in lhs_contract_axes.iter().zip(rhs_contract_axes.iter()) {
            if lhs.shape[lhs_axis] != rhs.shape[rhs_axis] {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::ShapeMismatch {
                        message: format!(
                            "Contracted axes have mismatched dimensions: lhs_axis {} (dim {}), rhs_axis {} (dim {})",
                            lhs_axis, lhs.shape[lhs_axis], rhs_axis, rhs.shape[rhs_axis]
                        ),
                    },
                ));
            }
        }

        // 2. Identify remaining (uncontracted) axes and build permutation maps
        let mut lhs_remaining_axes: Vec<usize> = (0..lhs.num_dim())
            .filter(|&i| !lhs_contract_axes.contains(&i))
            .collect();
        let mut rhs_remaining_axes: Vec<usize> = (0..rhs.num_dim())
            .filter(|&i| !rhs_contract_axes.contains(&i))
            .collect();

        // Create permutation for LHS: (remaining_lhs, contracted_lhs)
        let mut lhs_perm_order = lhs_remaining_axes.clone();
        lhs_perm_order.extend_from_slice(lhs_contract_axes);

        // Create permutation for RHS: (contracted_rhs, remaining_rhs)
        let mut rhs_perm_order = rhs_contract_axes.to_vec();
        rhs_perm_order.extend_from_slice(&rhs_remaining_axes);

        // 3. Permute and reshape tensors
        let permuted_lhs = lhs.permute_axes(&lhs_perm_order)?;
        let permuted_rhs = rhs.permute_axes(&rhs_perm_order)?;

        let contracted_dim_size: usize = lhs_contract_axes.iter().map(|&ax| lhs.shape[ax]).product();

        let lhs_rows: usize = lhs_remaining_axes.iter().map(|&ax| lhs.shape[ax]).product();
        let rhs_cols: usize = rhs_remaining_axes.iter().map(|&ax| rhs.shape[ax]).product();

        let reshaped_lhs = permuted_lhs.reshape(&[lhs_rows, contracted_dim_size])?;
        let reshaped_rhs = permuted_rhs.reshape(&[contracted_dim_size, rhs_cols])?;

        // 4. Perform matrix multiplication
        let result_matrix = Self::mat_mul_2d(&reshaped_lhs, &reshaped_rhs)?;

        // 5. Reshape result back to final tensor shape
        let mut final_shape = Vec::new();
        final_shape.extend(lhs_remaining_axes.iter().map(|&ax| lhs.shape[ax]));
        final_shape.extend(rhs_remaining_axes.iter().map(|&ax| rhs.shape[ax]));

        result_matrix.reshape(&final_shape)
    }

    /// Private helper for 2D matrix multiplication.
    fn mat_mul_2d(
        lhs: &CausalTensor<T>,
        rhs: &CausalTensor<T>,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if lhs.num_dim() != 2 || rhs.num_dim() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch { expected: 2, found: lhs.num_dim() },
            ));
        }
        if lhs.shape[1] != rhs.shape[0] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Matrix dimensions mismatch for multiplication: {} vs {}",
                        lhs.shape[1], rhs.shape[0]
                    ),
                },
            ));
        }

        let m = lhs.shape[0];
        let k = lhs.shape[1]; // Also rhs.shape[0]
        let n = rhs.shape[1];

        let mut result_data = vec![T::default(); m * n];

        for i in 0..m {
            for j in 0..n {
                let mut sum = T::default();
                for l in 0..k {
                    let lhs_val = lhs.get(&[i, l]).unwrap().clone();
                    let rhs_val = rhs.get(&[l, j]).unwrap().clone();
                    sum = sum + lhs_val * rhs_val;
                }
                result_data[i * n + j] = sum;
            }
        }

        CausalTensor::new(result_data, vec![m, n])
    }

    /// Private method for element-wise multiplication.
    fn element_wise_mul(
        lhs: &CausalTensor<T>,
        rhs: &CausalTensor<T>,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        lhs.broadcast_op(rhs, |a, b| Ok(a * b))
    }

    /// Private method for tracing (summing over diagonal axes).
    fn trace(
        tensor: &CausalTensor<T>,
        axis1: usize,
        axis2: usize,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if axis1 >= tensor.num_dim() || axis2 >= tensor.num_dim() || axis1 == axis2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: format!("Invalid trace axes: {}, {}", axis1, axis2),
                },
            ));
        }
        if tensor.shape[axis1] != tensor.shape[axis2] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Trace axes have mismatched dimensions: axis {} (dim {}), axis {} (dim {})",
                        axis1, tensor.shape[axis1], axis2, tensor.shape[axis2]
                    ),
                },
            ));
        }

        let mut new_shape: Vec<usize> = tensor.shape.iter().enumerate()
            .filter(|&(i, _)| i != axis1 && i != axis2)
            .map(|(_, &dim)| dim)
            .collect();

        if new_shape.is_empty() {
            let mut total_sum = T::default();
            for i in 0..tensor.shape[axis1] {
                let mut index = vec![0; tensor.num_dim()];
                index[axis1] = i;
                index[axis2] = i;
                total_sum = total_sum + tensor.get(&index).unwrap().clone();
            }
            return CausalTensor::new(vec![total_sum], vec![]);
        }

        let mut result_tensor = CausalTensor::full(&new_shape, T::default());
        let mut current_index = vec![0; tensor.num_dim()];

        for i in 0..tensor.len() {
            if current_index[axis1] == current_index[axis2] {
                let mut result_index: Vec<usize> = current_index.iter().enumerate()
                    .filter(|&(i, _)| i != axis1 && i != axis2)
                    .map(|(_, &val)| val)
                    .collect();

                if let Some(res_val) = result_tensor.get_mut(&result_index) {
                    *res_val = res_val.clone() + tensor.data[i].clone();
                }
            }

            for j in (0..tensor.num_dim()).rev() {
                current_index[j] += 1;
                if current_index[j] < tensor.shape[j] {
                    break;
                }
                current_index[j] = 0;
            }
        }

        Ok(result_tensor)
    }

    /// Private method for extracting a diagonal.
    fn diagonal(
        tensor: &CausalTensor<T>,
        axis1: usize,
        axis2: usize,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if axis1 >= tensor.num_dim() || axis2 >= tensor.num_dim() || axis1 == axis2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: format!("Invalid diagonal axes: {}, {}", axis1, axis2),
                },
            ));
        }
        if tensor.shape[axis1] != tensor.shape[axis2] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Diagonal axes have mismatched dimensions: axis {} (dim {}), axis {} (dim {})",
                        axis1, tensor.shape[axis1], axis2, tensor.shape[axis2]
                    ),
                },
            ));
        }

        let diag_len = tensor.shape[axis1];
        let mut new_data = Vec::with_capacity(diag_len);

        for i in 0..diag_len {
            let mut index = vec![0; tensor.num_dim()];
            index[axis1] = i;
            index[axis2] = i;
            new_data.push(tensor.get(&index).unwrap().clone());
        }

        CausalTensor::new(new_data, vec![diag_len])
    }
}
```

## 6. Usage Example

This example demonstrates how to perform a matrix multiplication using the new simplified API.

```rust
use deep_causality_tensor::{CausalTensor, EinSumOp, EinSumAST};

// Create two matrices
let lhs = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
let rhs = CausalTensor::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();

// Create the MatMul AST using the helper function
let ast = EinSumOp::mat_mul(lhs, rhs);

// Execute the einsum operation
let result = CausalTensor::einsum(&ast).unwrap();

// Verify the result
// [[1*5 + 2*7, 1*6 + 2*8], [3*5 + 4*7, 3*6 + 4*8]] =
// [[19, 22], [43, 50]]
assert_eq!(result.as_slice(), &[19, 22, 43, 50]);
assert_eq!(result.shape(), &[2, 2]);
```
