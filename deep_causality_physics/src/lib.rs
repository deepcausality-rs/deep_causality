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
pub mod quantities;
pub mod theories;
#[cfg(feature = "alloc")]
pub mod utils_tests;

pub use crate::constants::*;

pub use crate::error::physics_error::{PhysicsError, PhysicsErrorEnum};

pub use crate::kernels::astro::*;
pub use crate::kernels::chronometric::*;
pub use crate::kernels::condensed::*;
pub use crate::kernels::dynamics::*;
pub use crate::kernels::em::*;
pub use crate::kernels::fluids::*;
pub use crate::kernels::hypersonic::*;
pub use crate::kernels::materials::*;
pub use crate::kernels::mhd::*;
pub use crate::kernels::nuclear::*;
pub use crate::kernels::photonics::*;
pub use crate::kernels::propulsion::*;
pub use crate::kernels::quantum::*;
pub use crate::kernels::relativity::*;
pub use crate::kernels::thermodynamics::*;
pub use crate::kernels::waves::*;

// All physics quantities and physical-unit types — exported flat at the
// crate root so `deep_causality_physics::TypeName` always resolves
// regardless of where the type lives within `quantities/`.
pub use crate::quantities::*;

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
