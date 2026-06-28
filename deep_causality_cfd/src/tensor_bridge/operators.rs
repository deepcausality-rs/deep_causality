/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

/// Builds a rank-4 MPO core `[rl, 2, 2, rr]` (row-major `[carry_left, out, in, carry_right]`) by
/// setting `1` where the boolean `fill` holds.
fn build_core<R, F>(rl: usize, rr: usize, fill: F) -> CausalTensor<R>
where
    R: ConjugateScalar,
    F: Fn(usize, usize, usize, usize) -> bool,
{
    let mut data = vec![R::zero(); rl * 2 * 2 * rr];
    for cl in 0..rl {
        for o in 0..2 {
            for i in 0..2 {
                for cr in 0..rr {
                    if fill(cl, o, i, cr) {
                        data[((cl * 2 + o) * 2 + i) * rr + cr] = R::one();
                    }
                }
            }
        }
    }
    CausalTensor::new(data, vec![rl, 2, 2, rr]).unwrap()
}

/// The periodic grid-shift operator `S₊` (cyclic `+1`) on a `2^L` grid, as a bond-2 MPO built by hand.
///
/// Hand-built so it never forms the dense `2^L × 2^L` matrix (the whole point of the QTT layer). It is
/// the binary ripple-carry increment with **most-significant-bit-first** mode ordering: the carry flows
/// from the LSB mode (right) toward the MSB mode (left) along the rank-2 bond — per bit,
/// `out = in XOR carry_in`, `carry_out = in AND carry_in`. The MSB mode drops the overflow carry
/// (cyclic, mod `2^L`); the LSB mode injects `carry_in = 1`.
///
/// # Errors
/// [`PhysicsError::DimensionMismatch`] if `l == 0`; propagates core-validation errors.
pub fn shift_plus<R>(l: usize) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    if l == 0 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "shift operator requires l >= 1, got {l}"
        )));
    }
    if l == 1 {
        // A single mode is both MSB (drop carry-out) and LSB (carry-in = 1): out = NOT in.
        let c = build_core::<R, _>(1, 1, |_cl, o, i, _cr| o == (i ^ 1));
        return Ok(CausalTensorTrainOperator::from_cores(vec![c])?);
    }
    let mut cores = Vec::with_capacity(l);
    for k in 0..l {
        let c = if k == 0 {
            // MSB: left boundary drops the overflow carry; out = in XOR carry_in.
            build_core::<R, _>(1, 2, |_cl, o, i, cr| o == (i ^ cr))
        } else if k == l - 1 {
            // LSB: right boundary fixes carry_in = 1; out = NOT in, carry_out = in.
            build_core::<R, _>(2, 1, |cl, o, i, _cr| o == (i ^ 1) && cl == i)
        } else {
            // Interior: out = in XOR carry_in, carry_out = in AND carry_in.
            build_core::<R, _>(2, 2, |cl, o, i, cr| o == (i ^ cr) && cl == (i & cr))
        };
        cores.push(c);
    }
    Ok(CausalTensorTrainOperator::from_cores(cores)?)
}

/// `S₋` (cyclic `−1`) — the transpose (hence inverse) of [`shift_plus`].
pub fn shift_minus<R>(l: usize) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    Ok(shift_plus::<R>(l)?.transpose())
}

/// Centered first-difference operator `∂ₓ ≈ (u[k+1] − u[k−1])/(2Δx)` on a periodic `2^L` grid.
///
/// With the index-shift convention `(S₊·u)[k] = u[k−1]` (and `(S₋·u)[k] = u[k+1]`), the forward
/// centered difference is `(S₋ − S₊)/(2Δx)`.
///
/// # Errors
/// Propagates shift-build and rounding errors.
pub fn gradient<R>(
    l: usize,
    dx: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let two = R::one() + R::one();
    let half_inv_dx = R::one() / (two * dx);
    let g = shift_minus::<R>(l)?
        .sub(&shift_plus::<R>(l)?)?
        .scale(half_inv_dx);
    Ok(g.round(trunc)?)
}

/// Second-difference (Laplacian) operator `∂²ₓ ≈ (S₊ + S₋ − 2·I)/Δx²` on a periodic `2^L` grid.
///
/// # Errors
/// Propagates shift-build and rounding errors.
pub fn laplacian<R>(
    l: usize,
    dx: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let two = R::one() + R::one();
    let inv_dx2 = R::one() / (dx * dx);
    let id = CausalTensorTrainOperator::<R>::identity(&vec![2usize; l]);
    let lap = shift_plus::<R>(l)?
        .add(&shift_minus::<R>(l)?)?
        .sub(&id.scale(two))?
        .scale(inv_dx2);
    Ok(lap.round(trunc)?)
}
