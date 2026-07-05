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

use crate::types::CfdScalar;

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

    /// Begin composing a marching workflow from **any** config family — the single trajectory
    /// entry. The config type selects the solver at compile time through
    /// [`MarchDispatch`](crate::MarchDispatch) and opens the family's pipeline (a
    /// [`MarchPipeline`](crate::MarchPipeline) for the DEC march, a
    /// [`DuctMarchRun`](crate::DuctMarchRun) for the duct, a [`QttMarchRun`](crate::QttMarchRun)
    /// for the QTT march, a [`CompressibleMarchRun`](crate::CompressibleMarchRun) for the coupled
    /// host, and the uncertain pipeline under the `uncertain` path). The geometry-bearing families
    /// lend their manifold next via `.on(..)`.
    pub fn march<R, Cfg>(config: &Cfg) -> Cfg::Pipeline<'_>
    where
        R: CfdScalar,
        Cfg: crate::traits::MarchDispatch<R>,
    {
        config.pipeline()
    }
}
