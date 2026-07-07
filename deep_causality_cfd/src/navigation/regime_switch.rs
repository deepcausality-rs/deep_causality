/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Encke↔Cowell **integrator regime switch** (Gap-3 B4) — the `grmhd/select_metric` pattern applied
//! to the trajectory integrator. The indicator `ε = a_aero / a_grav` (the g-load) is computed **from
//! state**; it is compared to **config** thresholds to choose the integrator:
//!
//! * `ε < ε_switch` — the **perturbed-conformal** (Encke-like) split: the exact KS matrix-exponential
//!   core + a small between-step aero kick, branch-cheap. The perturbative regime (coast, blackout onset).
//! * `ε ≥ ε_switch` — **direct** (Cowell) integration, accurate where aero dominates (peak dynamic
//!   pressure). Branch-cheapness is given up *observably*, by an explicit coupling decision.
//!
//! The switch carries **hysteresis** (a Schmitt trigger — separate enter/exit thresholds) so it does not
//! chatter near `ε_switch`. In the overlap band the two integrators agree (the KS Strang split is
//! 2nd-order against a direct solve), so the handover is seamless.

use deep_causality_algebra::RealField;
use deep_causality_physics::PhysicsError;

/// Which trajectory integrator the regime detector has selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegratorRegime {
    /// Encke-like perturbed-conformal split (exact KS core + between-step aero kick) — `ε` small.
    PerturbedConformal,
    /// Cowell direct integration — `ε` large (aero-dominated).
    Direct,
}

/// The regime indicator `ε = a_aero / a_grav = |a_aero| / (GM/r²)` — the g-load, computed from state.
///
/// # Errors
/// [`PhysicsError::Singularity`] on a non-positive radius.
pub fn aero_gravity_ratio<R: RealField>(
    aero_accel: [R; 3],
    radius: R,
    gm: R,
) -> Result<R, PhysicsError> {
    if radius <= R::zero() {
        return Err(PhysicsError::Singularity(
            "aero/gravity ratio needs a positive radius".into(),
        ));
    }
    let a_aero = (aero_accel[0] * aero_accel[0]
        + aero_accel[1] * aero_accel[1]
        + aero_accel[2] * aero_accel[2])
        .sqrt();
    let a_grav = gm / (radius * radius);
    Ok(a_aero / a_grav)
}

/// A hysteresis (Schmitt-trigger) integrator switch: it flips to [`IntegratorRegime::Direct`] when the
/// g-load `ε` rises above `enter_direct`, and back to [`IntegratorRegime::PerturbedConformal`] only when
/// `ε` falls below the lower `exit_direct` — the dead band prevents chatter around `ε_switch`.
#[derive(Clone, Copy, Debug)]
pub struct RegimeSwitch<R> {
    exit_direct: R,
    enter_direct: R,
    regime: IntegratorRegime,
}

impl<R: RealField> RegimeSwitch<R> {
    /// A switch with the lower (`exit_direct`) and upper (`enter_direct`) g-load thresholds and an initial
    /// regime. Requires `exit_direct ≤ enter_direct` (the hysteresis band).
    pub fn new(exit_direct: R, enter_direct: R, initial: IntegratorRegime) -> Self {
        Self {
            exit_direct,
            enter_direct,
            regime: initial,
        }
    }

    /// Feed the current g-load `ε` and return the (possibly updated) regime, applying hysteresis.
    pub fn select(&mut self, epsilon: R) -> IntegratorRegime {
        self.regime = match self.regime {
            IntegratorRegime::PerturbedConformal => {
                if epsilon > self.enter_direct {
                    IntegratorRegime::Direct
                } else {
                    IntegratorRegime::PerturbedConformal
                }
            }
            IntegratorRegime::Direct => {
                if epsilon < self.exit_direct {
                    IntegratorRegime::PerturbedConformal
                } else {
                    IntegratorRegime::Direct
                }
            }
        };
        self.regime
    }

    /// The current regime.
    pub fn regime(&self) -> IntegratorRegime {
        self.regime
    }
}
