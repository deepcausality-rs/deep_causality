/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # deep_causality_cfd
//!
//! Computational fluid dynamics solvers and the **CfdFlow** DSL for DeepCausality.
//!
//! This crate consolidates the fluid-dynamics theories and the DEC-native
//! Navier–Stokes solver behind a composable, precision-generic interface,
//! and lifts them into the `CfdFlow` domain-specific language
//!
//! Physics errors (`PhysicsError`), physics quantities (the typed DEC forms and
//! quantity newtypes), and the pointwise governing kernels stay consolidated in
//! `deep_causality_physics`; this crate imports them rather than duplicating them.
//!
//! Precision is a parameter: every theory and solver is generic over a real
//! scalar (`CfdScalar`). Composition is static (no `dyn`),
//! built on the `deep_causality_haft` HKT/algebra foundation.
//!
//! CPU parallelism is opt-in via the `parallel` feature
//! and rides the `MaybeParallel` bound.

extern crate alloc;

mod solvers;
mod tensor_bridge;
mod theories;
mod traits;
mod types;

// Physics types this crate's public API exposes — the typed DEC forms and
// physics-quantity newtypes (which stay in `deep_causality_physics`) plus
// `PhysicsError` — re-exported so CFD code can import them from one crate.
pub use deep_causality_physics::PhysicsError;
pub use deep_causality_physics::quantities::*;

// Core CFD trait seams and value types.
pub use crate::traits::{FluidTheory, Marcher, Solver};
pub use crate::types::{Ambient, CfdScalar};

// The CFD ↔ tensor-network (QTT) bridge: quantized field codec and finite-difference MPO assembly.
pub use crate::tensor_bridge::{
    QttProjector2d, dequantize, dequantize_2d, gradient, gradient_x, gradient_y, laplacian,
    laplacian_2d, quantize, quantize_2d, shift_minus, shift_plus,
};

// The CfdFlow DSL facade (owned case descriptions materialized at run).
// Workflow composition — the CfdFlow DSL (the "how").
pub use crate::types::flow::{
    CfdFlow, CoupledField, Coupling, MarchPipeline, MarchRun, MmsBuilder, Operator,
    OperatorStudyBuilder, PhysicsStage, QttMarchRun, QttStepView, Regime, Report, StepContext,
    StepView, ThermalRelax, VerifyRun, ViscosityArrhenius, dominant_frequency, fail,
    strouhal_number,
};
// Configuration — CfdConfigBuilder + the owned config containers / scenario types (the "what").
pub use crate::types::flow_config::{
    Body, CfdConfigBuilder, Grading, Manufactured, ManufacturedSample, MarchConfig,
    MarchConfigBuilder, MarchStop, Mesh, Observe, QttMarchConfig, QttMarchConfigBuilder,
    QttObserve, Seed, TaylorGreen, VerifyConfig, VerifyConfigBuilder,
};
// IO effect: the `IoAction` trait (from haft), the core `write_csv` file action, and the CFD CSV
// helper, so a `CfdFlow` example can describe and run file output through one crate.
#[cfg(feature = "std")]
pub use crate::types::flow::write_xy_csv;
#[cfg(feature = "std")]
pub use deep_causality_core::write_csv;
#[cfg(feature = "std")]
pub use deep_causality_haft::IoAction;

// The sensor-fed uncertain-inflow march (std-only: consumes `deep_causality_uncertain`).
#[cfg(feature = "std")]
pub use crate::types::flow::{UncertainMarchPipeline, UncertainMarchRun, UncertainStepView};
#[cfg(feature = "std")]
pub use crate::types::flow_config::{UncertainMarchConfig, UncertainMarchConfigBuilder};

// Fluid-dynamics theories: the DEC-native FluidTheory realization, the pointwise NS
// regime evaluators (`*_rhs`), and their causal-effect wrappers (`*_rhs_effect`).
pub use crate::theories::*;

// Solver configuration + type-state builder.
pub use crate::solvers::{
    DecNs, DecNsConfig, DecNsConfigNeedsTimeStep, DecNsConfigNeedsViscosity, DecNsConfigReady,
    QttIncompressible2d, QttLinear1d,
};

// QTT rollout observable extraction (tensor-train-native diagnostics).
pub use crate::solvers::{divergence_residual, kinetic_energy, max_bond, max_speed};

// Public API of the Navier–Stokes solver.
pub use crate::solvers::dec::*;
