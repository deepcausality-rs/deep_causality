/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Core CFD trait seams: `FluidTheory` (a Navier–Stokes regime), `Marcher`
//! (a per-step advance), and the solver/coupling traits the CfdFlow DSL composes.

mod cfd_scalar;
mod fluid_theory;
mod marchable;
mod marcher;
mod metric_provider;
mod metric_provider_3d;
mod solver;

pub use cfd_scalar::CfdScalar;
pub use fluid_theory::FluidTheory;
pub use marchable::{MarchDispatch, Marchable};
pub use marcher::Marcher;
pub use metric_provider::MetricProvider;
pub use metric_provider_3d::MetricProvider3d;
pub use solver::Solver;
