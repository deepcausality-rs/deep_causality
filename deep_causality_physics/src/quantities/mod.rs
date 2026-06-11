/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! All physics quantities, consolidated from kernel-local files and the former
//! `units` module. Every public type is re-exported flat from this module and
//! from the crate root, so consumers always import via `deep_causality_physics::TypeName`.

// --- SI base and derived primitives used across multiple domains ---
pub mod si_primitives;

// --- Domain-specific quantities (moved from kernels) ---
pub mod chronometric;
pub mod condensed;
pub mod dynamics;
pub mod em;
pub mod fluids;
pub mod materials;
pub mod mhd;
pub mod nuclear;
pub mod photonics;
pub mod quantum;
pub mod relativity;
pub mod thermodynamics;

// --- Physical-unit types (moved from the former `units` module) ---
pub mod energy;
pub(crate) mod fluid_dynamics;
pub mod index_of_refraction;
pub mod probability;
pub mod ratio;
pub mod temperature;
pub mod time;

// Flat re-exports so `crate::TypeName` always resolves regardless of where
// a type lives within this hierarchy.
pub use si_primitives::*;
pub use chronometric::*;
pub use condensed::*;
pub use dynamics::*;
pub use em::*;
pub use energy::*;
pub use fluids::*;
pub use materials::*;
pub use mhd::*;
pub use nuclear::*;
pub use photonics::*;
pub use quantum::*;
pub use relativity::*;
pub use thermodynamics::*;

pub use fluid_dynamics::body_force_one_form::BodyForceOneForm;
pub use fluid_dynamics::pressure_zero_form::PressureZeroForm;
pub use fluid_dynamics::solenoidal_field::SolenoidalField;
pub use fluid_dynamics::velocity_one_form::VelocityOneForm;
pub use fluid_dynamics::vorticity_two_form::VorticityTwoForm;

pub use index_of_refraction::*;
pub use probability::*;
pub use ratio::*;
pub use temperature::*;
pub use time::*;
