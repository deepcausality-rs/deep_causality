/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

pub(crate) mod astro;
pub(crate) mod condensed;
pub(crate) mod constants;
pub(crate) mod dynamics;
pub(crate) mod electromagnetism;
pub(crate) mod error;
pub(crate) mod fluids;
pub(crate) mod materials;
pub(crate) mod mhd;
pub(crate) mod nuclear;
pub(crate) mod photonics;
pub(crate) mod quantum;
pub(crate) mod relativity;
mod theories;
pub(crate) mod thermodynamics;
pub(crate) mod units;
pub(crate) mod waves;

pub use astro::*;
pub use condensed::*;
pub use constants::*;
pub use dynamics::*;
pub use electromagnetism::*;
pub use error::*;
pub use fluids::*;
pub use materials::*;
pub use mhd::*;
pub use nuclear::*;
pub use photonics::*;
pub use quantum::*;
pub use relativity::*;
pub use thermodynamics::*;
pub use units::*;
pub use waves::*;

// Re-export metric types and conventions from deep_causality_metric
pub use deep_causality_metric::{
    // Convention trait and wrappers
    EastCoastMetric,
    LorentzianMetric,
    // Constants
    MINKOWSKI_4D,
    // Core types
    Metric,
    MetricError,
    PARTICLE_MINKOWSKI_4D,
    // Domain-specific aliases
    ParticleMetric,
    PhysicsMetric,
    RELATIVITY_MINKOWSKI_4D,
    RelativityMetric,
    WestCoastMetric,
};
