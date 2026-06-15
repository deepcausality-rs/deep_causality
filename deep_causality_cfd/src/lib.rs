/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # deep_causality_cfd
//!
//! Computational fluid dynamics solvers and the **Flow** DSL for DeepCausality.
//!
//! This crate consolidates the fluid-dynamics theories and the DEC-native
//! Navier–Stokes solver (migrated out of `deep_causality_physics`) behind a
//! composable, precision-generic interface, and lifts them into the `Flow`
//! domain-specific language — peer to `CausalFlow` and `CausalDiscovery`.
//!
//! Physics errors (`PhysicsError`), physics quantities (the typed DEC forms and
//! quantity newtypes), and the pointwise governing kernels stay consolidated in
//! `deep_causality_physics`; this crate imports them rather than duplicating them.
//!
//! Precision is a parameter: every theory and solver is generic over a real
//! scalar (`CfdScalar`) with no `f64` downcast. Composition is static (no `dyn`),
//! built on the `deep_causality_haft` HKT/algebra foundation. CPU parallelism is
//! opt-in via the `parallel` feature and rides the `MaybeParallel` bound.

extern crate alloc;

mod errors;
mod extensions;
mod solvers;
mod theories;
mod traits;
mod types;

// Physics types this crate's public API exposes — the typed DEC forms and
// physics-quantity newtypes (which stay in `deep_causality_physics`) plus
// `PhysicsError` — re-exported so CFD code can import them from one crate.
pub use deep_causality_physics::PhysicsError;
pub use deep_causality_physics::quantities::*;

// Core CFD trait seams and value types.
pub use crate::traits::{FluidTheory, Marcher};
pub use crate::types::{Ambient, CfdScalar};

// Fluid-dynamics theories: the DEC-native FluidTheory realization, and the
// classical pointwise NS regime evaluators + their causal wrappers.
pub use crate::theories::DecIncompressible;
pub use crate::theories::regimes::*;

// Solver configuration + type-state builder.
pub use crate::solvers::{
    DecNs, DecNsConfig, DecNsConfigNeedsTimeStep, DecNsConfigNeedsViscosity, DecNsConfigReady,
};

// Public API of the migrated DEC-native Navier–Stokes solver. Re-exported flat at
// the crate root per the workspace convention.
pub use crate::solvers::dec::*;
