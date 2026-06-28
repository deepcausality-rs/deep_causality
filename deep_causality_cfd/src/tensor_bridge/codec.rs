/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use alloc::format;
use alloc::vec;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, Tensor, TensorTrain, Truncation};

/// Encodes a length-`2^L` periodic 1-D field as an `L`-mode **quantized** tensor train (QTT).
///
/// The field is reshaped to `L` binary modes (physical dimension 2 each) and factored by TT-SVD under
/// `trunc`. The ordering is **most-significant-bit first** (mode 0 is the coarsest scale): for a
/// row-major length-`2^L` buffer this is simply the natural reshape to `[2; L]`, so multiscale
/// structure lands in the low bonds. A smooth field compresses to a small bond dimension.
///
/// # Errors
/// [`PhysicsError::DimensionMismatch`] if the field length is zero or not a power of two; propagates
/// TT-SVD errors.
pub fn quantize<R>(
    field: &CausalTensor<R>,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let n = field.as_slice().len();
    if n == 0 || !n.is_power_of_two() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "quantize requires a power-of-two field length, got {n}"
        )));
    }
    let l = n.trailing_zeros() as usize;
    let modes = vec![2usize; l];
    let reshaped = field.reshape(&modes)?;
    Ok(CausalTensorTrain::from_dense(&reshaped, trunc)?)
}

/// Recovers the dense length-`2^L` field from its quantized tensor train (inverse of [`quantize`]).
///
/// # Errors
/// Propagates densification/reshape errors.
pub fn dequantize<R>(train: &CausalTensorTrain<R>) -> Result<CausalTensor<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let dense = train.to_dense()?;
    let n: usize = dense.shape().iter().product();
    Ok(dense.reshape(&[n])?)
}

/// Encodes a `2^Lx × 2^Ly` periodic field (shape `[Nx, Ny]`) as an `(Lx + Ly)`-mode QTT: the leading
/// `Lx` modes are the x-bits, the trailing `Ly` the y-bits (MSB-first per axis, the natural row-major
/// reshape). Axis operators built by lifting (see `gradient_x`/`gradient_y`) act on the matching block.
///
/// # Errors
/// [`PhysicsError::DimensionMismatch`] if the field is not 2-D or either extent is not a power of two.
pub fn quantize_2d<R>(
    field: &CausalTensor<R>,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let shape = field.shape();
    if shape.len() != 2 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "quantize_2d requires a 2-D field, got {} dims",
            shape.len()
        )));
    }
    let (nx, ny) = (shape[0], shape[1]);
    if nx == 0 || ny == 0 || !nx.is_power_of_two() || !ny.is_power_of_two() {
        return Err(PhysicsError::DimensionMismatch(format!(
            "quantize_2d requires power-of-two extents, got {nx} x {ny}"
        )));
    }
    let modes = vec![2usize; nx.trailing_zeros() as usize + ny.trailing_zeros() as usize];
    let reshaped = field.reshape(&modes)?;
    Ok(CausalTensorTrain::from_dense(&reshaped, trunc)?)
}

/// Recovers the dense `[2^Lx, 2^Ly]` field from its quantized tensor train (inverse of [`quantize_2d`]).
/// `lx`/`ly` give the per-axis mode split.
///
/// # Errors
/// Propagates densification/reshape errors.
pub fn dequantize_2d<R>(
    train: &CausalTensorTrain<R>,
    lx: usize,
    ly: usize,
) -> Result<CausalTensor<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let dense = train.to_dense()?;
    Ok(dense.reshape(&[1usize << lx, 1usize << ly])?)
}
