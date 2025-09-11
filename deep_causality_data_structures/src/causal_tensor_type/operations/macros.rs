/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// --- Trait Implementations for Tensor-Scalar Arithmetic ---

// Helper macro to reduce boilerplate for the four main arithmetic operations.
// It implements `Op<Rhs=T>` and `OpAssign<Rhs=T>`.
#[macro_export]
macro_rules! impl_tensor_scalar_op {
    ($trait:ident, $method:ident, $trait_assign:ident, $method_assign:ident) => {
        // Implementation for `&CausalTensor<T> OP T`
        impl<'a, T> $trait<T> for &'a CausalTensor<T>
        where
            T: $trait<T, Output = T> + Copy,
        {
            type Output = CausalTensor<T>;

            fn $method(self, rhs: T) -> Self::Output {
                let mut new_data = self.data.clone();
                for item in &mut new_data {
                    // This correctly calls the trait implementation for type T.
                    // e.g., for f64, this becomes `*item = (*item).add(rhs);`
                    *item = (*item).$method(rhs);
                }
                CausalTensor {
                    data: new_data,
                    shape: self.shape.clone(),
                    strides: self.strides.clone(),
                }
            }
        }

        // Implementation for `CausalTensor<T> OP T` (consuming)
        impl<T> $trait<T> for CausalTensor<T>
        where
            T: $trait<T, Output = T> + Copy,
        {
            type Output = CausalTensor<T>;
            fn $method(self, rhs: T) -> Self::Output {
                (&self).$method(rhs)
            }
        }

        // Implementation for `CausalTensor<T> OP= T` (in-place)
        impl<T> $trait_assign<T> for CausalTensor<T>
        where
            T: $trait_assign + Copy,
        {
            fn $method_assign(&mut self, rhs: T) {
                for item in &mut self.data {
                    item.$method_assign(rhs);
                }
            }
        }
    };
}

// We also need to handle the case where the scalar is on the left-hand side,
// e.g., `5.0 * my_tensor`.

// Helper macro for `T OP CausalTensor<T>`
#[macro_export]
macro_rules! impl_scalar_tensor_op {
    ($trait:ident, $method:ident, $ty:ty) => {
        // Implementation for `T OP &CausalTensor<T>`
        impl<'a> $trait<&'a CausalTensor<$ty>> for $ty
        where
            $ty: $trait<$ty, Output = $ty> + Copy, // Explicitly require T op T -> T
        {
            type Output = CausalTensor<$ty>;

            fn $method(self, rhs: &'a CausalTensor<$ty>) -> Self::Output {
                let mut new_data = rhs.data.clone();
                for item in &mut new_data {
                    // This now correctly resolves to the standard library's
                    // implementation for the numeric type.
                    // e.g., for f64, this becomes `*item = self.add(*item);`
                    *item = self.$method(*item);
                }
                CausalTensor {
                    data: new_data,
                    shape: rhs.shape.clone(),
                    strides: rhs.strides.clone(),
                }
            }
        }

        // Implementation for `T OP CausalTensor<T>` (consuming)
        impl $trait<CausalTensor<$ty>> for $ty
        where
            $ty: $trait<$ty, Output = $ty> + Copy,
        {
            type Output = CausalTensor<$ty>;
            fn $method(self, rhs: CausalTensor<$ty>) -> Self::Output {
                self.$method(&rhs)
            }
        }
    };
}

// Helper macro for Tensor-Tensor operations
#[macro_export]
macro_rules! impl_tensor_tensor_op {
    ($trait:ident, $method:ident) => {
        // Base case: &Tensor op &Tensor
        impl<'a, 'b, T> $trait<&'b CausalTensor<T>> for &'a CausalTensor<T>
        where
            T: Copy + Default + PartialOrd + $trait<T, Output = T>,
        {
            type Output = Result<CausalTensor<T>, CausalTensorError>;

            fn $method(self, rhs: &'b CausalTensor<T>) -> Self::Output {
                self.binary_op(rhs, |a, b| a.$method(b))
            }
        }

        // Owned left: Tensor op &Tensor
        impl<'b, T> $trait<&'b CausalTensor<T>> for CausalTensor<T>
        where
            T: Copy + Default + PartialOrd + $trait<T, Output = T>,
        {
            type Output = Result<CausalTensor<T>, CausalTensorError>;

            fn $method(self, rhs: &'b CausalTensor<T>) -> Self::Output {
                (&self).$method(rhs)
            }
        }

        // Owned right: &Tensor op Tensor
        impl<'a, T> $trait<CausalTensor<T>> for &'a CausalTensor<T>
        where
            T: Copy + Default + PartialOrd + $trait<T, Output = T>,
        {
            type Output = Result<CausalTensor<T>, CausalTensorError>;

            fn $method(self, rhs: CausalTensor<T>) -> Self::Output {
                self.$method(&rhs)
            }
        }

        // Owned both: Tensor op Tensor
        impl<T> $trait<CausalTensor<T>> for CausalTensor<T>
        where
            T: Copy + Default + PartialOrd + $trait<T, Output = T>,
        {
            type Output = Result<CausalTensor<T>, CausalTensorError>;

            fn $method(self, rhs: CausalTensor<T>) -> Self::Output {
                (&self).$method(&rhs)
            }
        }
    };
}
