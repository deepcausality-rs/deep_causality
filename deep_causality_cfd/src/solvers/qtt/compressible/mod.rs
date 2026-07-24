/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B compressible shock-capturing QTT solver family. Stage 2 provides the 1-D conservative Euler
//! flux (ideal gas + Rusanov), gated by the Sod exact-Riemann solution; later stages add the
//! body-fitted coordinate coupling, IMEX time integration, and shock fitting.

mod euler_1d;
mod fitting;
mod forcing;
mod imex;
mod marcher_2d;
mod marcher_3d;
mod marcher_3d_fitted;

pub use euler_1d::{CompressibleEuler1d, EulerState, ideal_gas_pressure};
pub use fitting::{
    FittedNormalShock, Park2tClosure, PostShockState, REDUCED_MASS_AMU, StagnationOutcome,
    reduced_mass_amu,
};
pub use forcing::ForcingRegion;
pub use imex::{AcousticImex1d, conservation_round, positivity_floor};
pub use marcher_2d::{CompressibleMarcher2d, EulerState2d, EulerStateTt2d, ideal_gas_pressure_2d};
pub use marcher_3d::{CompressibleMarcher3d, EulerState3d, EulerStateTt3d};
pub use marcher_3d_fitted::CompressibleMarcher3dFitted;

use crate::CfdScalar;
use deep_causality_physics::PhysicsError;

/// Reject a non-hyperbolic pressure before it enters the flux.
///
/// The ideal-gas EOS is hyperbolic only for `p > 0`; a state with `E < ½ρ|u|²` yields `p ≤ 0`, at
/// which the acoustic wave speed `c = √(γp/ρ)` has no real value and the Rusanov flux computed from
/// `p` is not an approximation to the equations being solved. All four marchers
/// (`euler_1d`, `marcher_2d`, `marcher_3d`, `marcher_3d_fitted`) call this so the check — and its
/// diagnostic — stay identical across the family rather than drifting between four copies.
///
/// This **replaces** the previous per-site `let tiny = R::from_f64(1e-300)…` floor, which was used
/// only for the wave speed while the unfloored `p` still entered `f[1]`/`f[3]`. With the state
/// rejected here, `p` is positive everywhere downstream and needs no floor — so the precision trap in
/// that literal (`f64 → f32` is an infallible cast, so `from_f64(1e-300)` returns `Some(0.0)` at
/// `f32` and the guard silently vanished) is removed by construction rather than repaired. The
/// rejection is a comparison, identical at every supported precision.
///
/// # Errors
/// [`PhysicsError::PhysicalInvariantBroken`] naming the pressure value and the offending cell index.
pub(crate) fn require_positive_pressure<R: CfdScalar>(
    p: R,
    cell: usize,
) -> Result<(), PhysicsError> {
    if p <= R::zero() || !p.is_finite() {
        return Err(PhysicsError::PhysicalInvariantBroken(alloc::format!(
            "compressible flux: pressure must stay positive (ideal-gas EOS is not hyperbolic at \
             p <= 0), got p = {p:?} at cell {cell}"
        )));
    }
    Ok(())
}
