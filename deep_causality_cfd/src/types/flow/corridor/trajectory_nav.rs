/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The trajectory/navigation stage (\[4\]): one nav-engine step per coupling step.

use super::super::coupling::{CoupledField, PhysicsStage, StepContext};
use crate::CfdScalar;
use crate::navigation::ImuModel;
use alloc::vec::Vec;
use deep_causality_haft::LogAddEntry;
use deep_causality_physics::PhysicsError;

/// The trajectory/navigation stage (\[4\]): one [`ReentryNavEngine`](crate::ReentryNavEngine) step per coupling step — KS
/// predict with the ④ aero-force channel as the perturbation kick, then the ESKF measurement fold.
///
/// The nav *state* threads through the [`CoupledField`] (the stage takes the engine out, advances
/// it, and puts it back — stages stay immutable, so the same stage value drives every step and a
/// forked field carries its own engine). A no-op if no engine has been seeded with
/// [`CoupledField::set_nav`].
///
/// Measurements are **consumed** (taken out of the field, one-shot) from named scalar fields a
/// sensor stage publishes each step — a publisher that goes quiet leaves no stale fix behind to
/// be re-folded as fresh:
/// * `"gnss_fix"` (3 cells, a measured Cartesian position) — folded **only when GNSS is available**;
///   the gate is the classifier's [`RegimeClass::gnss_denied`] flag on the field (no classifier ⇒
///   available). This is the ④ blackout gating of the corridor. A denied-step fix is consumed
///   unread.
/// * `"optical_fix"` (3 cells) — the through-plasma optical fix, folded regardless of denial.
///
/// Each step it publishes `"nav_position"` (3 cells) and `"nav_position_variance"` (1 cell) — the
/// dead-reckoning drift / reacquisition-collapse witnesses — and logs the transition between aided
/// and dead-reckoning navigation to the provenance log (transitions only, not every step).
///
/// With [`with_imu`](Self::with_imu) the predict integrates the **IMU-sensed** specific force
/// (the ④ aero plus the accelerometer bias) instead of the true value — the real INS
/// dead-reckoning error mechanism — and the ESKF `Q` comes from the IMU's grade.
#[derive(Debug, Clone, Copy)]
pub struct TrajectoryNav<R: CfdScalar> {
    process_noise: [R; 17],
    gnss_variance: R,
    optical_variance: R,
    imu: Option<ImuModel<R>>,
}

impl<R: CfdScalar> TrajectoryNav<R> {
    /// A trajectory stage with the ESKF `Q` diagonal `process_noise` and the per-axis measurement
    /// variances of the GNSS and through-plasma optical fixes. The predict integrates the true ④
    /// specific force; use [`with_imu`](Self::with_imu) for a sensed one.
    pub fn new(process_noise: [R; 17], gnss_variance: R, optical_variance: R) -> Self {
        Self {
            process_noise,
            gnss_variance,
            optical_variance,
            imu: None,
        }
    }

    /// Sense the ④ specific force through `imu` before the predict (accelerometer bias included),
    /// and take the ESKF `Q` diagonal from the IMU's grade.
    pub fn with_imu(mut self, imu: ImuModel<R>) -> Self {
        self.process_noise = imu.process_noise();
        self.imu = Some(imu);
        self
    }
}

/// Read a 3-cell fix field, if present and well-formed.
fn fix3<R: CfdScalar>(xs: Option<&[R]>) -> Option<[R; 3]> {
    match xs {
        Some([x, y, z]) => Some([*x, *y, *z]),
        _ => None,
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for TrajectoryNav<R> {
    fn apply(
        &self,
        ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(mut engine) = field.take_nav() else {
            return Ok(());
        };
        // Predict: KS drift + the ④ aero acceleration as the Strang kick (zero if no producer
        // ran), sensed through the IMU when one is configured.
        let aero = field.aero_force().unwrap_or([R::zero(); 3]);
        let sensed = match &self.imu {
            Some(imu) => imu.sense_specific_force(aero),
            None => aero,
        };
        if let Err(e) = engine.predict(ctx.dt(), sensed, self.process_noise) {
            // Thread the engine back before short-circuiting, so the pause/fork state stays whole.
            field.set_nav(engine);
            return Err(e);
        }

        // Correct: GNSS gated by the classifier's denial flag; optical rides through the plasma.
        // Each fix is a **consumed, one-shot** measurement: taking it out of the field means a
        // publisher that goes quiet leaves nothing behind, so a stale fix is never re-folded as
        // fresh on a later step (which would hold the filter in aided mode and over-collapse the
        // covariance). A denied-step GNSS fix is consumed unread — the broadcast the receiver
        // could not use is gone, not latched for reacquisition.
        let gnss = field.take_scalar("gnss_fix");
        let optical = field.take_scalar("optical_fix");
        let denied = field.regime().map(|r| r.gnss_denied).unwrap_or(false);
        let mut aided = false;
        if !denied && let Some(fix) = fix3(gnss.as_deref()) {
            engine.correct_position(fix, self.gnss_variance);
            aided = true;
        }
        if let Some(fix) = fix3(optical.as_deref()) {
            engine.correct_position(fix, self.optical_variance);
            aided = true;
        }

        // Log aided <-> dead-reckoning transitions only (the mode is carried on the field).
        let mode = if aided { R::one() } else { R::zero() };
        let prev = field.scalar("nav_mode").and_then(|m| m.first().copied());
        if prev != Some(mode) {
            let label = if aided {
                "nav: aided (position fix folded)"
            } else {
                "nav: dead reckoning (no usable fix)"
            };
            field.log_mut().add_entry(label);
        }
        field.set_scalar("nav_mode", Vec::from([mode]));

        // Publish the drift / reacquisition witnesses and thread the engine back.
        field.set_scalar("nav_position", engine.position().to_vec());
        field.set_scalar(
            "nav_position_variance",
            Vec::from([engine.position_variance()]),
        );
        field.set_nav(engine);
        Ok(())
    }
}
