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

use crate::CfdScalar;
use crate::solvers::qtt::compressible::{EulerStateTt2d, ideal_gas_pressure_2d};
use crate::tensor_bridge::{QttProjector2d, dequantize_2d, quantize_2d};
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain, TensorTrain, Truncation};

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

/// The penalization **heat** integral over the immersed body: `Q = (1/η) ∫ χ_body ⊙ (T_w − T) dV`,
/// the volumetric rate at which the penalization term exchanges heat with the fluid to hold the body
/// at `t_wall`. The same contraction shape as [`drag_lift`], with temperature in place of velocity.
///
/// **This is not a surface flux.** Its dimensions are `[T]·[L]²/[t]` — a temperature-weighted volume
/// integral over the masked body, carrying no gradient, no conductivity and no wall normal. Fourier's
/// law is `q = −k·∂T/∂n`, a per-area quantity on the wall surface, and no scaling converts a volume
/// integral into one. The name says `integral` for that reason, and `wall_heat_flux` is deliberately
/// left free for an actual Fourier-law implementation.
///
/// **What blocks that implementation is geometry, not machinery.** Gradient operators, a diffusivity
/// and the mask are all present, so `−k ∫ ∇T·∇χ dV` (the smoothed-mask surface-delta form) could be
/// written today. It would not mean anything: volume penalization has no wall surface, only a mask
/// smoothed over `SMOOTH_CELLS·dx`. For the `tanh` mask this crate uses, `|∇χ|` peaks at `1/(2w)` and
/// `T` relaxes across the same width, so the result scales as `k·ΔT/w` — inversely with a purely
/// numerical parameter. The audit already measured the *drag* moving 6.1x across that sweep, and drag
/// is a volume integral, which averages; a wall-normal derivative amplifies instead. The result would
/// be more blur-governed than the drag while carrying a name that invites an absolute reading.
///
/// Spectral differentiation does not rescue it either: the mask transitions over ~2 grid points, near
/// Nyquist, so an FFT-based gradient rings at the interface rather than sharpening it. Computing wall
/// fluxes from the penalization source — what this function does — is the standard volume-penalization
/// answer precisely because the interface gradient is unreliable on a smeared mask.
///
/// A genuine `wall_heat_flux` belongs on the **DEC cut-cell path**, where the surface is real:
/// `CutFaceFragment` carries a `(D−1)`-area and an outward unit normal, and
/// [`viscous_surface_force`](crate::solvers::dec::surface_force) already integrates
/// `∮ μ(∇u+∇uᵀ)·n dA` over those fragments. `q = −k ∮ ∇T·n dA` is the same contraction with a scalar
/// in place of the tensor, so the diagnostic is close to a transcription of that one.
///
/// **The one missing part is a temperature field on that path** — the DEC solver marches velocity
/// only and has no scalar transport. Supplying it (advection–diffusion of `T`, with a Dirichlet wall
/// temperature on the cut fragments) is the prerequisite, not Gap-2 and not the cut geometry. Gap-2
/// supplies a *better* temperature field (real `k(T)`, chemistry); it is not what unblocks a flux.
///
/// The quantity is still useful as it stands: it is the thermal analogue of the penalization force
/// integral, and same-configuration ratios built on it (as [`preserved_drag_fraction`] does for
/// force) are meaningful. **Neutral** — the seam the Gap-2 reacting energy equation replaces.
///
/// # Errors
/// Propagates the train-contraction errors.
pub fn penalization_heat_integral<R>(
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

/// The forebody-strip **pressure** contraction of an evolved compressible state: the pressure is
/// recovered pointwise from the conserved components (`p = (γ−1)(E − ½|m|²/ρ)`, the ideal-gas
/// closure), re-quantized, and contracted against the strip mask via the train `inner` product
/// and the cell volume — `F = ∫ χ_strip · p dV`, no cut-cell surface or boundary-fiber
/// reconstruction. This is the *compressible* sibling of the incompressible penalization-force
/// contraction: the integrand is the field's own pressure (the preserved aerodynamic drag the
/// Jarvinen–Adams dataset measured), **not** the forcing deficit.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] if the density leaves the positive cone; propagates
/// codec / contraction errors.
#[allow(clippy::too_many_arguments)]
pub fn strip_pressure_force<R>(
    strip: &CausalTensorTrain<R>,
    state: &EulerStateTt2d<R>,
    gamma: R,
    lx: usize,
    ly: usize,
    dx: R,
    dy: R,
    trunc: &Truncation<R>,
) -> Result<R, PhysicsError>
where
    R: CfdScalar + ConjugateScalar<Real = R>,
{
    let rho = dequantize_2d(&state[0], lx, ly)?;
    let mx = dequantize_2d(&state[1], lx, ly)?;
    let my = dequantize_2d(&state[2], lx, ly)?;
    let e = dequantize_2d(&state[3], lx, ly)?;
    let n = rho.as_slice().len();
    let mut p = Vec::with_capacity(n);
    for (((&r, &a), &b), &en) in rho
        .as_slice()
        .iter()
        .zip(mx.as_slice())
        .zip(my.as_slice())
        .zip(e.as_slice())
    {
        if r <= R::zero() || !r.is_finite() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "strip_pressure_force: density must stay positive".into(),
            ));
        }
        p.push(ideal_gas_pressure_2d(r, a, b, en, gamma));
    }
    let (nx, ny) = (1usize << lx, 1usize << ly);
    let p_tt = quantize_2d(&CausalTensor::new(p, alloc::vec![nx, ny])?, trunc)?;
    Ok(strip.inner(&p_tt)? * dx * dy)
}

/// The **preserved-drag fraction**: the powered (plume-imprinted) run's contracted forebody
/// force over the unpowered baseline's, from the same configuration — the dimensionless
/// quantity the Jarvinen–Adams correlation tabulates (`C_A,F / C_A0`). A same-configuration
/// ratio, so the harness's common geometry biases cancel.
///
/// # Errors
/// [`PhysicsError::Singularity`] if the unpowered baseline force is not finite or vanishes
/// (there is no drag to preserve a fraction of).
pub fn preserved_drag_fraction<R>(powered: R, unpowered: R) -> Result<R, PhysicsError>
where
    R: CfdScalar,
{
    if !unpowered.is_finite() || unpowered == R::zero() {
        return Err(PhysicsError::Singularity(
            "preserved_drag_fraction: the unpowered baseline force must be finite and nonzero"
                .into(),
        ));
    }
    Ok(powered / unpowered)
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
