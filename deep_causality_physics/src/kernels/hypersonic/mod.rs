/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Hypersonic reacting-air Park two-temperature kernels for the Gap-2 (Tier-A)
//! plasma-blackout slice: vibrational relaxation, Arrhenius rates, Saha / Park-2T
//! ionization, the Rankine–Hugoniot temperature jump, and the recovery-temperature
//! reconstruction. Pure pointwise kernels; the LER between-step stages that drive
//! them live in `deep_causality_cfd`.

pub mod finite_rate;
pub mod ionization;
pub mod shock;
pub mod thermochemistry;
pub mod wrappers;

pub use crate::quantities::hypersonic::*;
pub use finite_rate::*;
pub use ionization::*;
pub use shock::*;
pub use thermochemistry::*;
pub use wrappers::*;
