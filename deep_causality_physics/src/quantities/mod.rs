/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! All physics quantities, consolidated from kernel-local files and the former
//! `units` module. Every public type is re-exported flat from this module and
//! from the crate root so consumers always import via `deep_causality_physics::TypeName`
//! — imports remain stable regardless of where within `quantities/` a type lives.

// ── Cross-domain primitives ───────────────────────────────────────────────
/// Dimensionless scalars: Ratio, PhaseAngle, Probability.
pub mod dimensionless;
/// SI base and derived scalar quantities used across multiple domains.
pub mod si_primitives;

// ── Domain-specific quantities ────────────────────────────────────────────
pub mod chronometric;
pub mod condensed;
pub mod dynamics;
pub mod em;
pub mod fluids;
pub mod hypersonic;
pub mod materials;
pub mod mhd;
pub mod nuclear;
pub mod photonics;
pub mod propulsion;
pub mod relativity;
pub mod thermodynamics;

// ── Physical-unit types (DEC fluid solver forms) ──────────────────────────
pub(crate) mod fluid_dynamics;

// ── Flat re-exports ───────────────────────────────────────────────────────
// Everything below is accessible as `crate::TypeName` via lib.rs::quantities::*.

pub use dimensionless::*;
pub use si_primitives::*;

pub use chronometric::*;
pub use condensed::*;
pub use dynamics::*;
pub use em::*;
pub use fluids::*;
pub use hypersonic::*;
pub use materials::*;
pub use mhd::*;
pub use nuclear::*;
pub use photonics::*;
pub use propulsion::*;
pub use relativity::*;
pub use thermodynamics::*;

pub use fluid_dynamics::body_force_one_form::BodyForceOneForm;
pub use fluid_dynamics::pressure_zero_form::PressureZeroForm;
pub use fluid_dynamics::solenoidal_field::SolenoidalField;
pub use fluid_dynamics::velocity_one_form::VelocityOneForm;
pub use fluid_dynamics::vorticity_two_form::VorticityTwoForm;
