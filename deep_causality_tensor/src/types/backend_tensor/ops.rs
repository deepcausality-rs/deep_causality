/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic and shape operations for BackendTensor.
//!
//! Note: Most shape operations (reshape, slice, permute, ravel, shifted_view)
//! are provided by the Tensor trait implementation in tensor_impl.rs.
//! This file provides operations not in the Tensor trait.

use super::BackendTensor;
use crate::traits::{TensorBackend, TensorData};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

impl<T: Clone, B: TensorBackend> BackendTensor<T, B> {
    /// Apply binary operation with broadcasting.
    pub fn broadcast_op<F>(&self, rhs: &Self, f: F) -> Result<Self, crate::CausalTensorError>
    where
        F: Fn(T, T) -> Result<T, crate::CausalTensorError>,
    {
        B::broadcast_op(&self.inner, &rhs.inner, f).map(Self::from_inner)
    }
}

impl<T: TensorData, B: TensorBackend> BackendTensor<T, B>
where
    B::Tensor<T>: Clone,
{
    /// Sums elements along specified axes.
    pub fn sum(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::sum(&self.inner, axes))
    }

    /// Finds maximum along specified axes.
    pub fn max(&self, axes: &[usize]) -> Self {
        Self::from_inner(B::max(&self.inner, axes))
    }

    /// Calculates the mean along specified axes.
    pub fn mean(&self, axes: &[usize]) -> Self
    where
        T: From<u32>,
    {
        Self::from_inner(B::mean(&self.inner, axes))
    }

    /// Returns indicies that would sort the tensor (1D only).
    pub fn arg_sort(&self) -> Result<Vec<usize>, crate::CausalTensorError> {
        B::arg_sort(&self.inner)
    }

    /// Executes an Einstein summation AST.
    pub fn ein_sum(
        ast: &crate::types::cpu_tensor::EinSumAST<Self>,
    ) -> Result<Self, crate::CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>,
    {
        use crate::types::cpu_tensor::EinSumAST;

        // Recursive helper to map the AST.
        fn map_ast_recursive<T, B, F>(
            ast: &EinSumAST<BackendTensor<T, B>>,
            f: &F,
        ) -> EinSumAST<B::Tensor<T>>
        where
            T: TensorData,
            B: TensorBackend,
            B::Tensor<T>: Clone, // Required for EinSumOp::TensorSource
            F: Fn(BackendTensor<T, B>) -> B::Tensor<T>,
        {
            let current_op = ast.value();

            // Map the generic EinSumOp to the target tensor type.
            // We use map_tensor which takes a closure converting the tensor.
            // Note: EinSumOp::map_tensor consumes self, but we have reference.
            // We clone the op first (EinSumOp is Clone).
            let new_op = current_op.clone().map_tensor(f);

            let children = ast.children();
            if children.is_empty() {
                EinSumAST::new(new_op)
            } else {
                let new_children: Vec<_> =
                    children.iter().map(|c| map_ast_recursive(c, f)).collect();
                EinSumAST::with_children(new_op, new_children)
            }
        }

        // Convert wrapper function
        let converter = |t: Self| t.into_inner();

        let inner_ast = map_ast_recursive(ast, &converter);

        B::ein_sum(&inner_ast).map(Self::from_inner)
    }
}

// --- Arithmetic Trait Implementations ---

impl<T: TensorData, B: TensorBackend> Add for BackendTensor<T, B> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::add(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Add for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn add(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::add(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Sub for BackendTensor<T, B> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::sub(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Sub for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn sub(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::sub(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Mul for BackendTensor<T, B> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::mul(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Mul for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn mul(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::mul(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Div for BackendTensor<T, B> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::from_inner(B::div(&self.inner, &rhs.inner))
    }
}

impl<T: TensorData, B: TensorBackend> Div for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;

    fn div(self, rhs: Self) -> Self::Output {
        BackendTensor::from_inner(B::div(&self.inner, &rhs.inner))
    }
}

// --- Assignment Arithmetic Implementations ---

impl<T: TensorData, B: TensorBackend> AddAssign for BackendTensor<T, B> {
    fn add_assign(&mut self, rhs: Self) {
        self.inner = B::add(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> AddAssign<&Self> for BackendTensor<T, B> {
    fn add_assign(&mut self, rhs: &Self) {
        self.inner = B::add(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> SubAssign for BackendTensor<T, B> {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner = B::sub(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> SubAssign<&Self> for BackendTensor<T, B> {
    fn sub_assign(&mut self, rhs: &Self) {
        self.inner = B::sub(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> MulAssign for BackendTensor<T, B> {
    fn mul_assign(&mut self, rhs: Self) {
        self.inner = B::mul(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> MulAssign<&Self> for BackendTensor<T, B> {
    fn mul_assign(&mut self, rhs: &Self) {
        self.inner = B::mul(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> DivAssign for BackendTensor<T, B> {
    fn div_assign(&mut self, rhs: Self) {
        self.inner = B::div(&self.inner, &rhs.inner);
    }
}

impl<T: TensorData, B: TensorBackend> DivAssign<&Self> for BackendTensor<T, B> {
    fn div_assign(&mut self, rhs: &Self) {
        self.inner = B::div(&self.inner, &rhs.inner);
    }
}

// --- Scalar Assignment Arithmetic ---
macro_rules! impl_scalar_assign_arithmetic {
    ($($t:ty),*) => {
        $(
            impl<B: TensorBackend> AddAssign<$t> for BackendTensor<$t, B> {
                fn add_assign(&mut self, rhs: $t) {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
                    self.inner = B::create_from_vec(data, &B::shape(&self.inner));
                }
            }

            impl<B: TensorBackend> SubAssign<$t> for BackendTensor<$t, B> {
                fn sub_assign(&mut self, rhs: $t) {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
                    self.inner = B::create_from_vec(data, &B::shape(&self.inner));
                }
            }

            impl<B: TensorBackend> MulAssign<$t> for BackendTensor<$t, B> {
                fn mul_assign(&mut self, rhs: $t) {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x * rhs).collect();
                    self.inner = B::create_from_vec(data, &B::shape(&self.inner));
                }
            }

            impl<B: TensorBackend> DivAssign<$t> for BackendTensor<$t, B> {
                fn div_assign(&mut self, rhs: $t) {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
                    self.inner = B::create_from_vec(data, &B::shape(&self.inner));
                }
            }
        )*
    };
}

impl_scalar_assign_arithmetic!(f64, f32, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// --- Mixed Arithmetic Implementations (Val + Ref, Ref + Val) ---

// Add
impl<T: TensorData, B: TensorBackend> Add<&BackendTensor<T, B>> for BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn add(self, rhs: &BackendTensor<T, B>) -> Self::Output {
        Self::from_inner(B::add(&self.inner, &rhs.inner))
    }
}
impl<T: TensorData, B: TensorBackend> Add<BackendTensor<T, B>> for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn add(self, rhs: BackendTensor<T, B>) -> Self::Output {
        BackendTensor::from_inner(B::add(&self.inner, &rhs.inner))
    }
}

// Sub
impl<T: TensorData, B: TensorBackend> Sub<&BackendTensor<T, B>> for BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn sub(self, rhs: &BackendTensor<T, B>) -> Self::Output {
        Self::from_inner(B::sub(&self.inner, &rhs.inner))
    }
}
impl<T: TensorData, B: TensorBackend> Sub<BackendTensor<T, B>> for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn sub(self, rhs: BackendTensor<T, B>) -> Self::Output {
        BackendTensor::from_inner(B::sub(&self.inner, &rhs.inner))
    }
}

// Mul
impl<T: TensorData, B: TensorBackend> Mul<&BackendTensor<T, B>> for BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn mul(self, rhs: &BackendTensor<T, B>) -> Self::Output {
        Self::from_inner(B::mul(&self.inner, &rhs.inner))
    }
}
impl<T: TensorData, B: TensorBackend> Mul<BackendTensor<T, B>> for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn mul(self, rhs: BackendTensor<T, B>) -> Self::Output {
        BackendTensor::from_inner(B::mul(&self.inner, &rhs.inner))
    }
}

// Div
impl<T: TensorData, B: TensorBackend> Div<&BackendTensor<T, B>> for BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn div(self, rhs: &BackendTensor<T, B>) -> Self::Output {
        Self::from_inner(B::div(&self.inner, &rhs.inner))
    }
}
impl<T: TensorData, B: TensorBackend> Div<BackendTensor<T, B>> for &BackendTensor<T, B> {
    type Output = BackendTensor<T, B>;
    fn div(self, rhs: BackendTensor<T, B>) -> Self::Output {
        BackendTensor::from_inner(B::div(&self.inner, &rhs.inner))
    }
}

macro_rules! impl_scalar_arithmetic {
    ($($t:ty),*) => {
        $(
            // --- Addition ---
            impl<B: TensorBackend> Add<$t> for BackendTensor<$t, B> {
                type Output = Self;
                fn add(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
                    Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
            impl<B: TensorBackend> Add<BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn add(self, rhs: BackendTensor<$t, B>) -> Self::Output {
                    rhs + self
                }
            }
            impl<B: TensorBackend> Add<$t> for &BackendTensor<$t, B> {
                type Output = BackendTensor<$t, B>;
                fn add(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x + rhs).collect();
                    BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
            // For Scalar + &Tensor
            impl<B: TensorBackend> Add<&BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn add(self, rhs: &BackendTensor<$t, B>) -> Self::Output {
                    rhs + self
                }
            }

            // --- Subtraction ---
            impl<B: TensorBackend> Sub<$t> for BackendTensor<$t, B> {
                type Output = Self;
                fn sub(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
                    Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
            impl<B: TensorBackend> Sub<BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn sub(self, rhs: BackendTensor<$t, B>) -> Self::Output {
                   let data: Vec<$t> = B::to_vec(&rhs.inner).into_iter().map(|x| self - x).collect();
                   BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&rhs.inner)))
                }
            }

            impl<B: TensorBackend> Sub<$t> for &BackendTensor<$t, B> {
                type Output = BackendTensor<$t, B>;
                fn sub(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x - rhs).collect();
                    BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
             // For Scalar - &Tensor
             impl<B: TensorBackend> Sub<&BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn sub(self, rhs: &BackendTensor<$t, B>) -> Self::Output {
                   let data: Vec<$t> = B::to_vec(&rhs.inner).into_iter().map(|x| self - x).collect();
                   BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&rhs.inner)))
                }
            }




            // --- Multiplication ---
            impl<B: TensorBackend> Mul<$t> for BackendTensor<$t, B> {
                type Output = Self;
                fn mul(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x * rhs).collect();
                    Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
            impl<B: TensorBackend> Mul<BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn mul(self, rhs: BackendTensor<$t, B>) -> Self::Output {
                    rhs * self
                }
            }
             impl<B: TensorBackend> Mul<$t> for &BackendTensor<$t, B> {
                type Output = BackendTensor<$t, B>;
                fn mul(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x * rhs).collect();
                    BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
            // For Scalar * &Tensor
            impl<B: TensorBackend> Mul<&BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn mul(self, rhs: &BackendTensor<$t, B>) -> Self::Output {
                    rhs * self
                }
            }

            // --- Division ---

            impl<B: TensorBackend> Div<$t> for BackendTensor<$t, B> {
                type Output = Self;
                fn div(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
                    Self::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
             impl<B: TensorBackend> Div<BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn div(self, rhs: BackendTensor<$t, B>) -> Self::Output {
                   let data: Vec<$t> = B::to_vec(&rhs.inner).into_iter().map(|x| self / x).collect();
                   BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&rhs.inner)))
                }
            }

            impl<B: TensorBackend> Div<$t> for &BackendTensor<$t, B> {
                type Output = BackendTensor<$t, B>;
                fn div(self, rhs: $t) -> Self::Output {
                    let data: Vec<$t> = B::to_vec(&self.inner).into_iter().map(|x| x / rhs).collect();
                    BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&self.inner)))
                }
            }
             // For Scalar / &Tensor
             impl<B: TensorBackend> Div<&BackendTensor<$t, B>> for $t {
                type Output = BackendTensor<$t, B>;
                fn div(self, rhs: &BackendTensor<$t, B>) -> Self::Output {
                   let data: Vec<$t> = B::to_vec(&rhs.inner).into_iter().map(|x| self / x).collect();
                   BackendTensor::from_inner(B::create_from_vec(data, &B::shape(&rhs.inner)))
                }
            }

        )*
    };
}

impl_scalar_arithmetic!(f64, f32, i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
