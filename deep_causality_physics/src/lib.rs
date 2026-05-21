/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;
extern crate core;

pub(crate) mod constants;
pub(crate) mod error;
pub(crate) mod kernels;
pub mod theories;
pub(crate) mod units;

pub use crate::constants::*;

pub use crate::error::physics_error::{PhysicsError, PhysicsErrorEnum};

pub use crate::kernels::astro::*;
pub use crate::kernels::chronometric::*;
pub use crate::kernels::condensed::*;
pub use crate::kernels::dynamics::*;
pub use crate::kernels::em::*;
pub use crate::kernels::fluids::*;
pub use crate::kernels::materials::*;
pub use crate::kernels::mhd::*;
pub use crate::kernels::nuclear::*;
pub use crate::kernels::photonics::*;
pub use crate::kernels::quantum::*;
pub use crate::kernels::relativity::*;
pub use crate::kernels::thermodynamics::*;
pub use crate::kernels::waves::*;

pub use crate::units::energy::Energy;
pub use crate::units::index_of_refraction::IndexOfRefraction;
pub use crate::units::probability::Probability;
pub use crate::units::ratio::Ratio;
pub use crate::units::temperature::Temperature;
pub use crate::units::time::Time;

pub use crate::theories::*;

// Re-export metric types and conventions from deep_causality_metric
pub use deep_causality_metric::{
    // Convention trait and wrappers
    EastCoastMetric,
    LorentzianMetric,
    MINKOWSKI_4D,
    Metric,
    // Constants
    MetricError,
    // Domain-specific aliases
    PARTICLE_MINKOWSKI_4D,
    // Core types
    ParticleMetric,
    PhysicsMetric,
    RELATIVITY_MINKOWSKI_4D,
    RelativityMetric,
    WestCoastMetric,
};
