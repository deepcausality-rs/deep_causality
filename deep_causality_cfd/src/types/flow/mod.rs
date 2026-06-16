/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The **Flow** DSL facade: owned case descriptions materialized at run.
//!
//! A case carries only owned specs (mesh, solver config, seed, observe set); `run`
//! builds the manifold + solver as locals, executes, and returns an owned `Report`
//! — borrows never escape (design D2). The facade spans three solver kinds sharing
//! one `Report`: the marching solver (here), and the MMS-verification and
//! operator-accuracy solvers (added next).

mod body;
mod coupling;
mod frequency;
mod march_builder;
mod march_case;
mod mesh;
mod mms;
mod observe;
mod operator_study;
mod report;
mod seed;
mod zones;

pub use body::Body;
pub use coupling::{
    Coupling, CoupledField, PhysicsStage, StepContext, ThermalRelax, ViscosityArrhenius,
};
pub use frequency::{dominant_frequency, strouhal_number};
pub use march_builder::{Flow, MarchBuilder};
pub use march_case::MarchCase;
pub use mesh::Mesh;
pub use mms::{MmsBuilder, Regime};
pub use observe::Observe;
pub use operator_study::{Operator, OperatorStudyBuilder};
pub use report::Report;
pub use seed::Seed;
