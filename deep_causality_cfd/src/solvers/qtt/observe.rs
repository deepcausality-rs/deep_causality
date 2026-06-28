/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tensor-train-native observable extraction for the QTT 2-D incompressible rollout.
//!
//! The headline diagnostics are computed **directly on the velocity trains** — kinetic energy and the
//! divergence residual from the train `norm` (and the projector's `divergence`), and the maximum bond
//! dimension from the cores — so no dense field is materialized. Only the maximum speed needs the
//! pointwise field, so it dequantizes. The functions are free functions over the trains, usable
//! without the CfdFlow DSL.

use crate::tensor_bridge::{QttProjector2d, dequantize_2d};
use crate::types::CfdScalar;
use deep_causality_num::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensorTrain, TensorTrain};

/// Kinetic energy `½(‖u‖² + ‖v‖²)` from the train norms — the `‖·‖` is the Frobenius/L2 norm over the
/// `2^Lx · 2^Ly` grid coefficients, so this is the (unweighted) discrete kinetic energy. No dequantize.
///
/// # Errors
/// Propagates train-norm errors.
pub fn kinetic_energy<R>(
    u: &CausalTensorTrain<R>,
    v: &CausalTensorTrain<R>,
) -> Result<R, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let nu = u.norm()?;
    let nv = v.norm()?;
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    Ok(half * (nu * nu + nv * nv))
}

/// The divergence residual `‖∇·(u, v)‖` (Frobenius/L2 over the grid) — the projector forms the
/// divergence train, then its norm is taken. No dequantize.
///
/// # Errors
/// Propagates the projector's divergence and the train-norm errors.
pub fn divergence_residual<R>(
    projector: &QttProjector2d<R>,
    u: &CausalTensorTrain<R>,
    v: &CausalTensorTrain<R>,
) -> Result<R, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let div = projector.divergence(u, v)?;
    Ok(div.norm()?)
}

/// The maximum bond dimension across both velocity trains — the compression / rank metric. Each
/// rank-3 core `[r_left, phys, r_right]` contributes its right bond `shape()[2]`.
pub fn max_bond<R>(u: &CausalTensorTrain<R>, v: &CausalTensorTrain<R>) -> usize
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    u.cores()
        .iter()
        .chain(v.cores().iter())
        .map(|c| c.shape()[2])
        .max()
        .unwrap_or(1)
}

/// The maximum speed `max √(u² + v²)` over the dequantized `2^Lx × 2^Ly` grid.
///
/// # Errors
/// [`PhysicsError`] from dequantizing either train.
pub fn max_speed<R>(
    u: &CausalTensorTrain<R>,
    v: &CausalTensorTrain<R>,
    lx: usize,
    ly: usize,
) -> Result<R, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let ud = dequantize_2d(u, lx, ly)?;
    let vd = dequantize_2d(v, lx, ly)?;
    let (us, vs) = (ud.as_slice(), vd.as_slice());
    let mut max_sq = R::zero();
    for (a, b) in us.iter().zip(vs.iter()) {
        let sq = *a * *a + *b * *b;
        if sq > max_sq {
            max_sq = sq;
        }
    }
    Ok(max_sq.sqrt())
}
