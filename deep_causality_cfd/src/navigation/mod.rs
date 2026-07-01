/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! GNSS-denial **navigation** (Gap-3 B1+B2+B3+B4) — the aerospace-engineering estimation layer of the
//! plasma-blackout corridor. It *composes* the physics kernels (`deep_causality_physics`: the KS
//! conformal propagator, the relativistic forward-clock kernel, the KS constraint projection) into an
//! error-state Kalman navigation engine, synthetic sensors, and the Encke↔Cowell integrator regime
//! switch. This is engineering (INS/ESKF/guidance), not a force of nature, so it lives in the CFD /
//! aerospace crate, not in `deep_causality_physics`.

pub(crate) mod eskf;
pub(crate) mod ins_error_state;
pub(crate) mod nav_sensors;
pub(crate) mod reentry_nav;
pub(crate) mod regime_switch;

pub use eskf::*;
pub use ins_error_state::*;
pub use nav_sensors::*;
pub use reentry_nav::*;
pub use regime_switch::*;
