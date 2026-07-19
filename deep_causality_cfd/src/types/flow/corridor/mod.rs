/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage 3 — corridor **composition** stages that fill the Stage-0 ④ seam (design §"Stage 3"):
//!
//! * [`RegimeClassify`] — the governing-model selector ([2]/[3]): reads the flow's rarefaction
//!   (Knudsen number, from a `"mean_free_path"` field and a configured characteristic length) and the
//!   plasma state (`"n_e"` → [`BlackoutTrigger`] → GNSS-denied), selects the [`GoverningModel`], and
//!   **logs every regime change** into the [`CoupledField`] provenance log.
//! * [`TrajectoryNav`] — the trajectory/navigation stage ([4]): one [`ReentryNavEngine`](crate::ReentryNavEngine) step per
//!   coupling step (KS predict with the ④ aero force as the kick, ESKF correct with GNSS gated by
//!   the classifier's denial flag, through-plasma optical ungated); the nav *state* threads through
//!   the [`CoupledField`], so the stage itself stays immutable.
//! * [`BranchOutcome`] / [`BranchAccumulator`] — the counterfactual bank-angle branch vocabulary
//!   ([5]): a predict-only rollout folds per-step `(heat flux, comms-denied, dt)` samples into the
//!   `(peak heat, thermal load, blackout dwell)` triple and closes with the terminal miss distance.
//!   The rollout *driver* (the alternate-world `run_coupled`) lands with the DSL in Stage 4; this is
//!   the outcome type and its tested reducer the driver feeds.
//! * [`CyberneticCorrect`] — the **bounded-correction gate** ([6]): a [`PhysicsStage`] wrapping a
//!   direct [`CyberneticLoop::control_step`] (not the Effect monad — the committed corrective inner
//!   loop is latency-bound). The loop's Context `C` *is* the verified [`SafetyEnvelope`]; `decide`
//!   clamps the bank-angle Action into it by construction and yields [`BankCorrection::NoSafeAction`]
//!   when no safe action exists, which the gate turns into an `Err` that short-circuits the coupling.
//!   Deterministic (identical inputs → identical action), no Effect-monad allocation on the hot path.
//!
//! The stages live one per module: [`regime`] classifies, [`trajectory_nav`] navigates, [`branch`]
//! scores counterfactual branches and produces the steered aero force, [`envelope`] carries the
//! verified limits, and [`gate`] enforces them.

mod branch;
mod envelope;
mod gate;
mod regime;
mod trajectory_nav;

pub use branch::{BankSteeredLift, BranchAccumulator, BranchOutcome};
pub use envelope::{BankCorrection, BurnEnvelope, SafetyEnvelope};
pub use gate::CyberneticCorrect;
pub use regime::{GoverningModel, MachRegime, RegimeClass, RegimeClassify, ThrustState};
pub use trajectory_nav::TrajectoryNav;

use crate::CfdScalar;

/// The peak (maximum) of a scalar field, or `0` for an empty field.
pub(crate) fn peak<R: CfdScalar>(xs: &[R]) -> R {
    xs.iter()
        .copied()
        .fold(R::zero(), |a, x| if x > a { x } else { a })
}

/// The Euclidean norm of a 3-vector.
pub(crate) fn norm3<R: CfdScalar>(x: [R; 3]) -> R {
    dot3(x, x).sqrt()
}

/// The dot product of two 3-vectors.
pub(crate) fn dot3<R: CfdScalar>(a: [R; 3], b: [R; 3]) -> R {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// A 3-vector scaled by `s`.
pub(crate) fn scale3<R: CfdScalar>(x: [R; 3], s: R) -> [R; 3] {
    [x[0] * s, x[1] * s, x[2] * s]
}

/// The cross product `a × b`.
pub(crate) fn cross3<R: CfdScalar>(a: [R; 3], b: [R; 3]) -> [R; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Clamp `x` into `[lo, hi]`.
pub(crate) fn clamp<R: CfdScalar>(x: R, lo: R, hi: R) -> R {
    if x < lo {
        lo
    } else if x > hi {
        hi
    } else {
        x
    }
}
