/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **CfdFlow** DSL (workflow composition — the "how").
//!
//! `CfdFlow` composes a workflow from a [`MarchConfig`](crate::MarchConfig) container (built by the
//! [`CfdConfigBuilder`](crate::CfdConfigBuilder) configuration layer in
//! [`flow_config`](crate::types::flow_config)): it lends a caller-owned geometry (B1), couples
//! physics, marches, and returns an owned [`Report`] — borrows never escape `run` (design D2). The
//! facade spans three solver kinds sharing one `Report`: the marching solver (here), and the
//! MMS-verification and operator-accuracy solvers.

#[cfg(feature = "std")]
mod audit;
mod blackout;
mod carrier;
mod cfd_flow;
mod compressible_march_run;
mod corridor;
mod coupled_march;
mod coupling;
mod duct_march_run;
mod finite_rate_ionization;
mod flight_sensors;
mod frequency;
mod gates;
mod march_run;
mod march_state;
mod mms;
mod operator_study;
mod qtt_march_pause;
mod qtt_march_run;
mod report;
mod retropulsion;
pub mod state_snapshot;
mod study;
mod study_effect;
mod study_error;
mod study_warning;
mod sweep;
mod throttle_guidance;
#[cfg(feature = "std")]
mod uncertain_march_run;
mod verify;
mod zones;

#[cfg(feature = "std")]
pub use audit::LogSink;
pub use blackout::{
    BlackoutState, BlackoutTrigger, EosStage, IonizationStage, RecoveryTemperatureStage,
    VibrationalLagStage, ler_relax_scalar, ler_step,
};
pub use cfd_flow::CfdFlow;
pub use compressible_march_run::{CompressibleFork, CompressibleMarchRun, CompressiblePause};
pub use corridor::{
    BankCorrection, BankSteeredLift, BranchAccumulator, BranchOutcome, BurnEnvelope,
    CyberneticCorrect, GoverningModel, MachRegime, REGIME_TRANSITIONS_FIELD, RegimeClass,
    RegimeClassify, SafetyEnvelope, ThrustState, TrajectoryNav,
};
pub use coupled_march::{CoupledMarch, LEG_RE_SEEDS_FIELD, ReadyMarch};
pub use coupling::{
    AeroBlackoutStub, AeroForceCoupling, CoupledField, Coupling, PhysicsStage, StepContext,
    ThermalRelax, ViscosityArrhenius,
};
pub use duct_march_run::DuctMarchRun;
pub use finite_rate_ionization::FiniteRateIonizationStage;
pub use flight_sensors::FlightSensors;
pub use frequency::{dominant_frequency, strouhal_number};
pub use gates::Gates;
pub use march_run::{MarchPipeline, MarchRun, StepView};
pub use march_state::MarchState;
pub use mms::{MmsBuilder, Regime};
pub use operator_study::{Operator, OperatorStudyBuilder};
pub use qtt_march_pause::{MarchFork, MarchPause};
pub use qtt_march_run::{QttMarchRun, QttStepView};
pub use report::{ForkEconomics, Report};
pub use retropulsion::{PlumeNozzle, PlumeObstruction, PropulsionStub, RetroThrust};
pub use study::{
    Alternated, Branched, CaseRun, Cases, Configured, Counterfactual, CoupledCampaign,
    EnsembleMarched, ForkStudy, GateFn, GateOutcome, GateSeq, Judged, Marched, Prepared,
    RefineBranched, RefineMarched, Refining, StudyDef, StudyView, Swept, Verdict,
};
pub use study_effect::{StudyEffect, StudyEffectWitness};
pub use study_error::StudyError;
pub use study_warning::{StudyWarning, StudyWarningLog};
pub use sweep::sweep;
pub use throttle_guidance::{
    IGNITION_COMMIT_AIDED_FIELD, IGNITION_COMMIT_MACH_FIELD, IGNITION_COMMIT_Q_FIELD,
    IGNITION_COMMIT_SIGMA_FIELD, IGNITION_COMMIT_STEP_FIELD, IGNITION_LATCH_FIELD,
    IgnitionCorridor, STOPPING_BURN_FIELD, ThrottleGuidance,
};
#[cfg(feature = "std")]
pub use uncertain_march_run::{UncertainMarchPipeline, UncertainMarchRun, UncertainStepView};
pub use verify::VerifyRun;
