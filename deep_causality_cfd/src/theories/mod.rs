/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fluid-dynamics theories: the Navier–Stokes regimes (incompressible,
//! compressible, Euler, Stokes) and the DEC-native incompressible rate, each a
//! `FluidTheory` realization reused across solvers. The pointwise regime
//! evaluators stay in `deep_causality_physics` and are reached through this layer
//! for verification solvers.

mod dec_incompressible;
pub mod regimes;

pub use dec_incompressible::DecIncompressible;
