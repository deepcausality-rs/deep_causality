/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CfdFlow` — the **CfdFlow** DSL entry point (workflow composition — the "how").
//!
//! It composes a workflow from a [`MarchConfig`](crate::MarchConfig) container built by the
//! [`CfdConfigBuilder`](crate::CfdConfigBuilder) configuration layer, lends a caller-owned geometry
//! (B1), and runs — returning an owned [`Report`](crate::Report). Mirrors the Discovery
//! `CdlConfigBuilder` → `CdlBuilder` split (`CfdConfigBuilder` builds, `CfdFlow` composes/runs).

use crate::solvers::dec::BoundaryZone;
use crate::types::CfdScalar;
use crate::types::flow::{MarchPipeline, PhysicsStage};
use crate::types::flow_config::MarchConfig;

/// The CfdFlow DSL entry point.
pub struct CfdFlow;

impl CfdFlow {
    /// Begin composing a marching workflow from a [`MarchConfig`](crate::MarchConfig) container.
    /// Borrows the config; the geometry is lent next via
    /// [`MarchPipeline::on`](crate::MarchPipeline::on).
    pub fn march<const D: usize, R: CfdScalar, Z: BoundaryZone<D, R>, C: PhysicsStage<D, R>>(
        config: &MarchConfig<D, R, Z, C>,
    ) -> MarchPipeline<'_, D, R, Z, C> {
        MarchPipeline::new(config)
    }
}

/// Print a stage-failure `context` and its error on stderr, then exit the process non-zero — a
/// fail-fast convenience for the example / CLI `main`s that drive a `CfdFlow` workflow, so the
/// happy path stays a flat pipeline:
///
/// ```ignore
/// use deep_causality_cfd::{CfdFlow, fail};
/// let case = config::march_config(n, steps).unwrap_or_else(|e| fail("configuration", e));
/// let manifold = case.materialize().unwrap_or_else(|e| fail("geometry", e));
/// let report = CfdFlow::march(&case).on(&manifold).run().unwrap_or_else(|e| fail("pipeline", e));
/// ```
pub fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
