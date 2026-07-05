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
use crate::types::flow::{CompressibleMarchRun, MarchPipeline, PhysicsStage, QttMarchRun};
use crate::types::flow_config::{CompressibleMarchConfig, MarchConfig, QttMarchConfig};
use deep_causality_num::ConjugateScalar;

/// The CfdFlow DSL entry point.
pub struct CfdFlow;

impl CfdFlow {
    /// Open a named **study** — the campaign level of the language, a family of cases run to a
    /// verdict. The name titles the verdict report. The returned [`StudyDef`](crate::StudyDef)
    /// takes the case axis (`cases`, `read`, or `matrix`).
    ///
    /// A study reads as one expression whose every stage is a domain verb:
    ///
    /// ```
    /// use deep_causality_cfd::{CfdFlow, GateSeq, StudyView};
    /// use deep_causality_file::TableRow;
    /// use deep_causality_physics::PhysicsError;
    ///
    /// #[derive(Clone)]
    /// struct Row { x: f64 }
    /// impl TableRow for Row {
    ///     type Scalar = f64;
    ///     const SCHEMA: &'static [(&'static str, &'static str)] = &[("x", "-")];
    ///     fn cells(&self) -> Vec<f64> { vec![self.x] }
    /// }
    /// fn positive(v: &StudyView<'_, Row>) -> (bool, String) {
    ///     (v.rows().iter().all(|r| r.x > 0.0), format!("{} rows", v.rows().len()))
    /// }
    ///
    /// let verdict = CfdFlow::study("demo")
    ///     .cases(vec![1.0_f64, 2.0])
    ///     .sweep(|x: &f64| Ok::<Row, PhysicsError>(Row { x: *x }))
    ///     .gates(GateSeq::new("demo").gate("positive", positive))
    ///     .verdict()
    ///     .unwrap();
    /// assert!(verdict.passed());
    /// ```
    ///
    /// # Phase discipline (these do not compile)
    ///
    /// The phases are typestates: a verb exists only where it is meaningful, so a mis-ordered
    /// study is a compile error.
    ///
    /// `verdict` before any gate — `verdict` is on the judged phase, not the swept phase:
    /// ```compile_fail
    /// use deep_causality_cfd::CfdFlow;
    /// use deep_causality_physics::PhysicsError;
    /// let _ = CfdFlow::study("x")
    ///     .cases(vec![1.0_f64])
    ///     .sweep(|p: &f64| Ok::<f64, PhysicsError>(*p))
    ///     .verdict();
    /// ```
    ///
    /// `gates` before rows exist — `gates` is not on the cases phase:
    /// ```compile_fail
    /// use deep_causality_cfd::{CfdFlow, GateSeq};
    /// let _ = CfdFlow::study("x")
    ///     .cases(vec![1.0_f64])
    ///     .gates(GateSeq::new("g"));
    /// ```
    ///
    /// `record` before a sweep — `record` is not on the cases phase:
    /// ```compile_fail
    /// use deep_causality_cfd::CfdFlow;
    /// let _ = CfdFlow::study("x")
    ///     .cases(vec![1.0_f64])
    ///     .record("out.csv");
    /// ```
    ///
    /// `reduce` before a march — `reduce` is on the marched phase, not the configured phase:
    /// ```compile_fail
    /// use deep_causality_cfd::CfdFlow;
    /// use deep_causality_physics::PhysicsError;
    /// let _ = CfdFlow::study("x")
    ///     .cases(vec![1.0_f64])
    ///     .case(|p: &f64| Ok::<f64, PhysicsError>(*p))
    ///     .reduce(|_| Ok::<f64, PhysicsError>(0.0));
    /// ```
    ///
    /// A gating sequence built for another study's rows — `GateSeq<i32>` cannot judge a study of
    /// `f64` rows:
    /// ```compile_fail
    /// use deep_causality_cfd::{CfdFlow, GateSeq, StudyView};
    /// use deep_causality_physics::PhysicsError;
    /// fn g(_: &StudyView<'_, i32>) -> (bool, String) { (true, String::new()) }
    /// let seq: GateSeq<i32> = GateSeq::new("g").gate("x", g);
    /// let _ = CfdFlow::study("x")
    ///     .cases(vec![1.0_f64])
    ///     .sweep(|p: &f64| Ok::<f64, PhysicsError>(*p))
    ///     .gates(seq);
    /// ```
    pub fn study(title: &str) -> crate::types::flow::StudyDef {
        crate::types::flow::StudyDef::new(title)
    }

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

    /// Begin composing a **compressible coupled march** from a
    /// [`CompressibleMarchConfig`](crate::CompressibleMarchConfig): the corridor's evolved-state
    /// carrier, hosting the same coupled loop, pause/fork machinery, and counterfactual
    /// vocabulary as the QTT host.
    pub fn compressible_march<R: CfdScalar + ConjugateScalar<Real = R>>(
        config: &CompressibleMarchConfig<R>,
    ) -> CompressibleMarchRun<'_, R> {
        CompressibleMarchRun::new(config)
    }

    /// Begin composing a **quasi-1-D duct march** from a
    /// [`DuctConfig`](crate::DuctConfig): the compressible nozzle path.
    /// Borrows the config and yields a runnable
    /// [`DuctMarchRun`](crate::DuctMarchRun); there is no geometry stage (the
    /// duct carries its own area profile).
    pub fn duct_march<R: CfdScalar>(
        config: &crate::types::flow_config::DuctConfig<R>,
    ) -> crate::types::flow::DuctMarchRun<'_, R> {
        crate::types::flow::DuctMarchRun::new(config)
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
