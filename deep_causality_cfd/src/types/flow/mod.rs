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

mod blackout;
mod cfd_flow;
mod corridor;
mod coupling;
mod frequency;
#[cfg(feature = "std")]
mod io;
mod march_run;
mod mms;
mod operator_study;
mod qtt_march_run;
mod report;
#[cfg(feature = "std")]
mod uncertain_march_run;
mod verify;
mod zones;

pub use blackout::{
    BlackoutState, BlackoutTrigger, EosStage, IonizationStage, RecoveryTemperatureStage,
    ler_relax_scalar, ler_step,
};
pub use cfd_flow::{CfdFlow, fail};
pub use corridor::{
    BankCorrection, BranchAccumulator, BranchOutcome, CyberneticCorrect, GoverningModel,
    RegimeClass, RegimeClassify, SafetyEnvelope,
};
pub use coupling::{
    AeroBlackoutStub, AeroForceCoupling, CoupledField, Coupling, PhysicsStage, StepContext,
    ThermalRelax, ViscosityArrhenius,
};
pub use frequency::{dominant_frequency, strouhal_number};
#[cfg(feature = "std")]
pub use io::write_xy_csv;
pub use march_run::{MarchPipeline, MarchRun, StepView};
pub use mms::{MmsBuilder, Regime};
pub use operator_study::{Operator, OperatorStudyBuilder};
pub use qtt_march_run::{QttMarchRun, QttStepView};
pub use report::Report;
#[cfg(feature = "std")]
pub use uncertain_march_run::{UncertainMarchPipeline, UncertainMarchRun, UncertainStepView};
pub use verify::VerifyRun;
