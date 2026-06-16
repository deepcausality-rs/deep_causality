/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CfdConfigBuilder` — the single entry for **configuration** (the "what").
//!
//! It separates *configuration* from *workflow composition*: every solver gets its own config
//! object (and, in later phases, every parameterized coupling its own config builder) here, while
//! the [`CfdFlow`](crate::CfdFlow) DSL composes those configs onto a geometry and runs (the "how").
//! Mirrors the Discovery `CdlConfigBuilder` → `CdlBuilder` split.

use crate::solvers::{DecNs, DecNsConfigNeedsViscosity};
use crate::types::CfdScalar;
use crate::types::flow_config::MarchConfigBuilder;

/// The configuration entry point. Each method starts a dedicated, validated config builder for one
/// solver (and, later, one parameterized coupling) or a marching-case container.
pub struct CfdConfigBuilder;

impl CfdConfigBuilder {
    /// Start the DEC incompressible Navier–Stokes **solver** configuration. Required, in order:
    /// `viscosity`, `time_step`; then optional knobs; then `build()` → `DecNsConfig`.
    pub fn dec_ns() -> DecNsConfigNeedsViscosity {
        DecNs::config()
    }

    /// Start a **marching-case container** configuration (mesh + solver + seed + observe + zones +
    /// coupling). The mesh pins the dimension `D` and precision `R`; `build()` → `MarchConfig`,
    /// which the [`CfdFlow`](crate::CfdFlow) DSL composes onto a geometry and runs.
    pub fn march<const D: usize, R: CfdScalar>(
        name: impl Into<String>,
    ) -> MarchConfigBuilder<D, R, (), ()> {
        MarchConfigBuilder::new(name)
    }
}
