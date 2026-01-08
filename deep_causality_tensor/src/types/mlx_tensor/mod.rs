/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::any::TypeId;

/// Wrapper around `mlx_rs::Array` for type-safe tensor operations.
///
/// This wrapper preserves the element type information at the type level
/// while storing the actual MLX array internally.
#[derive(Clone)]
pub struct MlxTensor<T> {
    /// The underlying MLX array
    pub(crate) array: mlx_rs::Array,
    /// Phantom data for type safety
    _marker: core::marker::PhantomData<T>,
}

impl<T> MlxTensor<T> {
    /// Creates a new MlxTensor from an MLX array.
    pub fn new(array: mlx_rs::Array) -> Self {
        Self {
            array,
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns a reference to the underlying MLX array.
    pub fn as_array(&self) -> &mlx_rs::Array {
        &self.array
    }

    /// Consumes self and returns the underlying MLX array.
    pub fn into_array(self) -> mlx_rs::Array {
        self.array
    }
}

impl<T: 'static> MlxTensor<T> {
    /// Returns the MLX data type corresponding to T.
    ///
    /// # Panics
    /// Panics if T is not a supported MLX data type.
    pub fn get_mlx_dtype() -> mlx_rs::Dtype {
        let type_id = TypeId::of::<T>();

        if type_id == TypeId::of::<f32>() {
            mlx_rs::Dtype::Float32
        } else if type_id == TypeId::of::<bool>() {
            mlx_rs::Dtype::Bool
        } else if type_id == TypeId::of::<i8>() {
            mlx_rs::Dtype::Int8
        } else if type_id == TypeId::of::<i16>() {
            mlx_rs::Dtype::Int16
        } else if type_id == TypeId::of::<i32>() {
            mlx_rs::Dtype::Int32
        } else if type_id == TypeId::of::<i64>() {
            mlx_rs::Dtype::Int64
        } else if type_id == TypeId::of::<u8>() {
            mlx_rs::Dtype::Uint8
        } else if type_id == TypeId::of::<u16>() {
            mlx_rs::Dtype::Uint16
        } else if type_id == TypeId::of::<u32>() {
            mlx_rs::Dtype::Uint32
        } else if type_id == TypeId::of::<u64>() {
            mlx_rs::Dtype::Uint64
        } else if type_id == TypeId::of::<mlx_rs::complex64>() {
            mlx_rs::Dtype::Complex64
        } else if type_id == TypeId::of::<f64>() {
            // Downcast f64 to f32 as MLX on Metal doesn't support f64
            mlx_rs::Dtype::Float32
        } else {
            panic!(
                "Unsupported type for MlxTensor: {}",
                std::any::type_name::<T>()
            );
        }
    }
}

// Implement Send + Sync for MlxTensor since MLX arrays are thread-safe
unsafe impl<T: Send> Send for MlxTensor<T> {}
unsafe impl<T: Sync> Sync for MlxTensor<T> {}
