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
use crate::types::flow::{MarchPipeline, PhysicsStage, QttMarchRun};
use crate::types::flow_config::{MarchConfig, QttMarchConfig};
use deep_causality_num::ConjugateScalar;

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

    /// Begin composing a **QTT 2-D incompressible march** from a
    /// [`QttMarchConfig`](crate::QttMarchConfig) — the tensor-train sibling of [`march`](Self::march).
    /// Borrows the config and yields a runnable [`QttMarchRun`]; there is no geometry stage (the QTT
    /// solver carries no borrowed manifold).
    pub fn qtt_march<R: CfdScalar + ConjugateScalar<Real = R>>(
        config: &QttMarchConfig<R>,
    ) -> QttMarchRun<'_, R> {
        QttMarchRun::new(config)
    }

    /// Begin composing a **sensor-fed uncertain-inflow march** from an
    /// [`UncertainMarchConfig`](crate::UncertainMarchConfig). Borrows the config; the geometry is
    /// lent next via [`UncertainMarchPipeline::on`](crate::UncertainMarchPipeline::on).
    #[cfg(feature = "std")]
    pub fn uncertain_march<R: CfdScalar + deep_causality_uncertain::ProbabilisticType>(
        config: &crate::types::flow_config::UncertainMarchConfig<R>,
    ) -> crate::types::flow::UncertainMarchPipeline<'_, R> {
        crate::types::flow::UncertainMarchPipeline::new(config)
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
