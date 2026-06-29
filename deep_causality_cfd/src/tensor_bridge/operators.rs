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
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrain, TensorTrainOperator,
    Truncation,
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

/// A single identity MPO core `[1, 2, 2, 1]` (`out == in`).
fn identity_core<R: ConjugateScalar>() -> CausalTensor<R> {
    build_core::<R, _>(1, 1, |_cl, o, i, _cr| o == i)
}

/// Lifts a 1-D operator to act on the **leading** modes of an `(L + m)`-mode field, identity on the
/// trailing `m` modes (`op ⊗ I_m`). The shared bonds are all 1, so the cores concatenate directly.
fn lift_leading<R>(
    op: &CausalTensorTrainOperator<R>,
    m: usize,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let mut cores = op.cores().to_vec();
    cores.extend((0..m).map(|_| identity_core::<R>()));
    Ok(CausalTensorTrainOperator::from_cores(cores)?)
}

/// Lifts a 1-D operator to act on the **trailing** modes, identity on the leading `m` (`I_m ⊗ op`).
fn lift_trailing<R>(
    op: &CausalTensorTrainOperator<R>,
    m: usize,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let mut cores: Vec<CausalTensor<R>> = (0..m).map(|_| identity_core::<R>()).collect();
    cores.extend(op.cores().to_vec());
    Ok(CausalTensorTrainOperator::from_cores(cores)?)
}

/// `∂ₓ` on a `2^Lx × 2^Ly` field (serial x-then-y mode layout): `gradient_1d(x) ⊗ I_y`.
///
/// # Errors
/// Propagates 1-D operator and lift errors.
pub fn gradient_x<R>(
    lx: usize,
    ly: usize,
    dx: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lift_leading(&gradient::<R>(lx, dx, trunc)?, ly)
}

/// `∂ᵧ` on a `2^Lx × 2^Ly` field: `I_x ⊗ gradient_1d(y)`.
///
/// # Errors
/// Propagates 1-D operator and lift errors.
pub fn gradient_y<R>(
    lx: usize,
    ly: usize,
    dy: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lift_trailing(&gradient::<R>(ly, dy, trunc)?, lx)
}

/// 2-D periodic Laplacian `∂²ₓ + ∂²ᵧ` on a `2^Lx × 2^Ly` field (the five-point stencil), recompressed.
///
/// # Errors
/// Propagates 1-D operator, lift, and rounding errors.
pub fn laplacian_2d<R>(
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let lap_x = lift_leading(&laplacian::<R>(lx, dx, trunc)?, ly)?;
    let lap_y = lift_trailing(&laplacian::<R>(ly, dy, trunc)?, lx)?;
    Ok(lap_x.add(&lap_y)?.round(trunc)?)
}

// ---------------------------------------------------------------------------
// 3-D operators (Tier-B): serial x-then-y-then-z mode layout. The y-axis lives
// in the middle block, so a single `op ⊗ I` / `I ⊗ op` lift is not enough — a
// general block lift puts identity cores before *and* after the operator.
// ---------------------------------------------------------------------------

/// Lifts a 1-D operator to act on a contiguous mode block, identity on `lead` modes before and
/// `trail` modes after (`I_lead ⊗ op ⊗ I_trail`). The shared bonds are all 1, so cores concatenate.
fn lift_block<R>(
    op: &CausalTensorTrainOperator<R>,
    lead: usize,
    trail: usize,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let mut cores: Vec<CausalTensor<R>> = (0..lead).map(|_| identity_core::<R>()).collect();
    cores.extend(op.cores().to_vec());
    cores.extend((0..trail).map(|_| identity_core::<R>()));
    Ok(CausalTensorTrainOperator::from_cores(cores)?)
}

/// `∂ₓ` on a `2^Lx × 2^Ly × 2^Lz` field: `gradient_1d(x) ⊗ I_{y,z}`.
///
/// # Errors
/// Propagates 1-D operator and lift errors.
pub fn gradient_x_3d<R>(
    lx: usize,
    ly: usize,
    lz: usize,
    dx: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lift_block(&gradient::<R>(lx, dx, trunc)?, 0, ly + lz)
}

/// `∂ᵧ` on a `2^Lx × 2^Ly × 2^Lz` field: `I_x ⊗ gradient_1d(y) ⊗ I_z` (the middle block).
///
/// # Errors
/// Propagates 1-D operator and lift errors.
pub fn gradient_y_3d<R>(
    lx: usize,
    ly: usize,
    lz: usize,
    dy: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lift_block(&gradient::<R>(ly, dy, trunc)?, lx, lz)
}

/// `∂_z` on a `2^Lx × 2^Ly × 2^Lz` field: `I_{x,y} ⊗ gradient_1d(z)`.
///
/// # Errors
/// Propagates 1-D operator and lift errors.
pub fn gradient_z_3d<R>(
    lx: usize,
    ly: usize,
    lz: usize,
    dz: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    lift_block(&gradient::<R>(lz, dz, trunc)?, lx + ly, 0)
}

/// 3-D periodic Laplacian `∂²ₓ + ∂²ᵧ + ∂²_z` on a `2^Lx × 2^Ly × 2^Lz` field (the seven-point
/// stencil), recompressed.
///
/// # Errors
/// Propagates 1-D operator, lift, and rounding errors.
pub fn laplacian_3d<R>(
    lx: usize,
    ly: usize,
    lz: usize,
    dx: R,
    dy: R,
    dz: R,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrainOperator<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let lap_x = lift_block(&laplacian::<R>(lx, dx, trunc)?, 0, ly + lz)?;
    let lap_y = lift_block(&laplacian::<R>(ly, dy, trunc)?, lx, lz)?;
    let lap_z = lift_block(&laplacian::<R>(lz, dz, trunc)?, lx + ly, 0)?;
    Ok(lap_x.add(&lap_y)?.add(&lap_z)?.round(trunc)?)
}

/// Divergence `∇·F = ∂ₓFₓ + ∂ᵧFᵧ + ∂_zF_z` of a 3-D vector field given as three component trains and
/// the three pre-built gradient operators (built once, reused each step by the marcher), recompressed.
///
/// # Errors
/// Propagates apply, add, and rounding errors.
pub fn divergence_3d<R>(
    fx: &CausalTensorTrain<R>,
    fy: &CausalTensorTrain<R>,
    fz: &CausalTensorTrain<R>,
    grad_x: &CausalTensorTrainOperator<R>,
    grad_y: &CausalTensorTrainOperator<R>,
    grad_z: &CausalTensorTrainOperator<R>,
    trunc: &Truncation<R>,
) -> Result<CausalTensorTrain<R>, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let dfx = grad_x.apply(fx, trunc)?;
    let dfy = grad_y.apply(fy, trunc)?;
    let dfz = grad_z.apply(fz, trunc)?;
    Ok(dfx.add(&dfy)?.add(&dfz)?.round(trunc)?)
}
