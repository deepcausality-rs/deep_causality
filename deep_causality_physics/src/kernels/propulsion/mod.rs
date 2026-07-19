/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Propulsion kernels for the plasma-retropulsion descent (Stage 1 of the
//! retropulsion build order): rocket performance, nozzle exit state, the
//! supersonic-retropropulsion similarity numbers and Jarvinen–Adams drag
//! correlation, the analytic plume-boundary geometry, and the powered-descent
//! closed forms. Pure pointwise kernels; the stages that drive them
//! (`RetroThrust`, `PlumeObstruction`, guidance, envelope) live in
//! `deep_causality_cfd`.

pub mod descent;
pub mod nozzle;
pub mod performance;
pub mod plume;
pub mod srp;
pub mod wrappers;

pub use crate::quantities::propulsion::*;
pub use descent::*;
pub use nozzle::*;
pub use performance::*;
pub use plume::*;
pub use srp::*;
pub use wrappers::*;
