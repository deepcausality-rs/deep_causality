/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Core CFD trait seams: `FluidTheory` (a Navier–Stokes regime), `Marcher`
//! (a per-step advance), and the solver/coupling traits the CfdFlow DSL composes.

mod fluid_theory;
mod marcher;
mod solver;

pub use fluid_theory::FluidTheory;
pub use marcher::Marcher;
pub use solver::Solver;
