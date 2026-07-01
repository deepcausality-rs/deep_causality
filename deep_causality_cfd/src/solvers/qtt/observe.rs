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

/// The penalization-force integral `(1/η) ∫ χ_body ⊙ (a − a_body) dV` over the grid — a single
/// tensor-train contraction (`inner` of the mask with the field deficit), no surface reconstruction.
/// This is the momentum (or heat) the Brinkman penalization exchanges with the body.
fn penalization_integral<R>(
    mask: &CausalTensorTrain<R>,
    a: &CausalTensorTrain<R>,
    a_body: R,
    eta: R,
    cell_volume: R,
) -> Result<R, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    // a − a_body (the deficit); for a_body = 0 this is just `a`.
    let deficit = if a_body == R::zero() {
        mask.inner(a)?
    } else {
        mask.inner(&a.add_scalar(R::zero() - a_body)?)?
    };
    Ok(deficit * cell_volume / eta)
}

/// Drag and lift coefficients on the immersed body, from the **penalization-force contraction**:
/// the force the fluid exerts on the body is the penalization momentum integral `F = (1/η) ∫ χ_body ⊙
/// (u − u_body) dV` per component, nondimensionalized as `C_d = F_x / (½ ρ U² D)` (ρ = 1). A pure
/// tensor-train contraction — no cut-cell surface or boundary fiber.
///
/// # Errors
/// Propagates the train-contraction errors.
#[allow(clippy::too_many_arguments)]
pub fn drag_lift<R>(
    mask: &CausalTensorTrain<R>,
    u: &CausalTensorTrain<R>,
    v: &CausalTensorTrain<R>,
    ubx: R,
    uby: R,
    eta: R,
    dx: R,
    dy: R,
    u_ref: R,
    d_ref: R,
) -> Result<(R, R), PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let cell_volume = dx * dy;
    let fx = penalization_integral(mask, u, ubx, eta, cell_volume)?;
    let fy = penalization_integral(mask, v, uby, eta, cell_volume)?;
    let half = R::from_f64(0.5).expect("0.5 lifts into every real field");
    let denom = half * u_ref * u_ref * d_ref;
    Ok((fx / denom, fy / denom))
}

/// Wall heat flux on the immersed body, from the penalization **heat** integral `Q = (1/η) ∫ χ_body ⊙
/// (T_w − T) dV` (the heat the wall exchanges with the fluid to hold the body at `t_wall`). The same
/// contraction shape as [`drag_lift`]. **Neutral** — the seam the Gap-2 reacting energy equation replaces.
///
/// # Errors
/// Propagates the train-contraction errors.
pub fn wall_heat_flux<R>(
    mask: &CausalTensorTrain<R>,
    temp: &CausalTensorTrain<R>,
    t_wall: R,
    eta: R,
    dx: R,
    dy: R,
) -> Result<R, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    // Q = (1/η) ∫ χ_body (T_w − T) dV = −[(1/η) ∫ χ_body (T − T_w) dV].
    let q = penalization_integral(mask, temp, t_wall, eta, dx * dy)?;
    Ok(R::zero() - q)
}

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
