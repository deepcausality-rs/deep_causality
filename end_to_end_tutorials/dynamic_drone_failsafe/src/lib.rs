/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The world every part of the dynamic drone fail-safe tutorial flies in: a mountain slope under a
//! high-voltage line, the inspection drone, the faults that befall it, and the judgement of where
//! it came down. The controller in each part sees this world only through the drone's
//! [`Telemetry`] and acts on it only through a [`Command`].

mod constants;
mod types;

pub use constants::{FLIGHT_LIMIT_S, FLIGHT_START_HOUR};
pub use types::command::Command;
pub use types::drone::Drone;
pub use types::guidance::Guidance;
pub use types::outcome::Outcome;
pub use types::patch_reading::PatchReading;
pub use types::quantity::Quantity;
pub use types::surface::Surface;
pub use types::telemetry::Telemetry;
pub use types::terrain::Terrain;
pub use types::touchdown::Touchdown;

/// The working precision of the whole tutorial.
pub type FloatType = f32;
