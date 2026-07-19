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

mod alias;
mod coordinate;
mod navigation;
mod solvers;
mod tensor_bridge;
mod theories;
mod traits;
mod types;

// The CFD ↔ tensor-network (QTT) bridge: quantized field codec and finite-difference MPO assembly.
pub use crate::alias::physical_gradient_3_d::PhysicalGradient3d;
// Physics types this crate's public API exposes — the typed DEC forms and
// physics-quantity newtypes (which stay in `deep_causality_physics`) plus
// `PhysicsError` — re-exported so CFD code can import them from one crate.
pub use deep_causality_physics::PhysicsError;
pub use deep_causality_physics::quantities::*;

// Core CFD trait seams and value types.
pub use crate::traits::{
    CfdScalar, FluidTheory, MarchDispatch, Marchable, Marcher, MetricProvider, MetricProvider3d,
    Solver,
};
pub use crate::types::{Ambient, KeyedInterpolation, KeyedTable};

// The CFD ↔ tensor-network (QTT) bridge: quantized field codec and finite-difference MPO assembly.
pub use crate::coordinate::{
    BlendedMap, BlendedMapConfig, BodyFittedCoordinate, BodyFittedCoordinate3d, CartesianIdentity,
    CartesianIdentity3d,
};
pub use crate::tensor_bridge::{
    AcousticCoreInverse, AcousticCoreInverse2d, AcousticCoreInverse3d, QttProjector2d,
    body_mask_2d, dequantize, dequantize_2d, dequantize_3d, divergence_3d, gradient, gradient_x,
    gradient_x_3d, gradient_y, gradient_y_3d, gradient_z_3d, laplacian, laplacian_2d, laplacian_3d,
    mask_from_fn, plume_mask_2d, quantize, quantize_2d, quantize_3d, shift_minus, shift_plus,
};

// GNSS-denial navigation (aerospace-engineering estimation layer composing the physics kernels):
// the error-state Kalman engine, synthetic INS sensors, and the Encke↔Cowell integrator regime switch.
pub use crate::navigation::{
    ImuModel, InsErrorState, IntegratorRegime, NAV_STATES, NavFilter, ReentryNavEngine,
    RegimeSwitch, aero_gravity_ratio, nav_transition_matrix,
};

// The CfdFlow DSL facade (owned case descriptions materialized at run).
// Workflow composition — the CfdFlow DSL (the "how").
// State snapshot and resume: pack/unpack a running state, one-line save and cross-workflow
// load, plus the file-crate container types a consumer needs.
pub use crate::types::flow::state_snapshot::{
    NamedTtFields, load_resume_state, pack_resume, pack_tt_fields, save_resume_state,
    unpack_resume, unpack_tt_fields,
};
pub use deep_causality_file::{
    BitCodec, ScalarTypeTag, SnapshotPackage, SnapshotSection, SnapshotTier, fingerprint64,
    force_load_snapshot, load_snapshot, save_snapshot,
};
// Typed table IO: the study grammar's `read`/`matrix`/`record` lower onto these, and a
// verification harness or example writing a probe trace uses `write_rows` directly. Re-exported
// so a CfdFlow program imports table IO from one crate.
pub use deep_causality_file::{
    FromTableRow, NumericTable, TableColumn, TableRow, TableScalar, read_rows, read_table,
    write_rows, write_table,
};

pub use crate::types::flow::{
    AeroBlackoutStub, AeroForceCoupling, BankCorrection, BankSteeredLift, BlackoutState,
    BlackoutTrigger, BranchAccumulator, BranchOutcome, BurnEnvelope, CfdFlow, CompressibleFork,
    CompressibleMarchRun, CompressiblePause, CoupledField, CoupledMarch, Coupling,
    CyberneticCorrect, DuctMarchRun, EosStage, FiniteRateIonizationStage, Gates, GoverningModel,
    IonizationStage, MarchFork, MarchPause, MarchPipeline, MarchRun, MarchState, MmsBuilder,
    Operator, OperatorStudyBuilder, PhysicsStage, PropulsionStub, QttMarchRun, QttStepView,
    ReadyMarch, RecoveryTemperatureStage, Regime, RegimeClass, RegimeClassify, Report,
    SafetyEnvelope, StepContext, StepView, StudyEffect, StudyEffectWitness, StudyError,
    StudyWarning, StudyWarningLog, ThermalRelax, TrajectoryNav,
};
pub use crate::types::flow::{
    Alternated, Branched, CaseRun, Cases, Configured, Counterfactual, CoupledCampaign,
    EnsembleMarched, ForkStudy, GateFn, GateOutcome, GateSeq, Judged, Marched, Prepared,
    RefineBranched, RefineMarched, Refining, StudyDef, StudyView, Swept, Verdict, VerifyRun,
    VibrationalLagStage, ViscosityArrhenius, dominant_frequency, ler_relax_scalar, ler_step,
    strouhal_number, sweep,
};
// Configuration — CfdConfigBuilder + the owned config containers / scenario types (the "what").
pub use crate::types::flow_config::{
    AtmosphereRow, Body, CfdConfigBuilder, CompressibleMarchConfig, CompressibleMarchConfigBuilder,
    DescentSchedule, DuctAreaProfile, DuctConfig, DuctInlet, DuctStop, Grading, Manufactured,
    ManufacturedSample, MarchConfig, MarchConfigBuilder, MarchStop, Mesh, Observe, QttBody,
    QttMarchConfig, QttMarchConfigBuilder, QttObserve, ReferenceScales, Seed, TaylorGreen,
    VerifyConfig, VerifyConfigBuilder,
};
// IO effect: the `IoAction` trait (from haft), so a `CfdFlow` program can describe and run file
// output through one crate. The typed table writers (`write_rows` / `record`) are the language's
// write surface; the retired `write_xy_csv` and the `write_csv` re-export are gone (a formatted
// string dump imports core's `write_csv` directly).
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
    AcousticImex1d, CompressibleEuler1d, CompressibleMarcher2d, CompressibleMarcher3d,
    CompressibleMarcher3dFitted, DecNs, DecNsConfig, DecNsConfigNeedsTimeStep,
    DecNsConfigNeedsViscosity, DecNsConfigReady, EulerState, EulerState2d, EulerState3d,
    EulerStateTt2d, EulerStateTt3d, FittedNormalShock, ForcingRegion, Park2tClosure,
    PostShockState, QttImmersed2d, QttIncompressible2d, QttLinear1d, StagnationOutcome,
    conservation_round, ideal_gas_pressure, ideal_gas_pressure_2d, positivity_floor,
};

// QTT rollout observable extraction (tensor-train-native diagnostics + surface observables).
pub use crate::solvers::{
    divergence_residual, drag_lift, kinetic_energy, max_bond, max_speed, preserved_drag_fraction,
    strip_pressure_force, wall_heat_flux,
};

// Public API of the Navier–Stokes solver.
pub use crate::solvers::dec::*;
