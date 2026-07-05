/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The campaign level of the CfdFlow DSL: a study is a family of cases run to a verdict.
//!
//! A study opens at [`CfdFlow::study`](crate::CfdFlow::study) and flows through a family of
//! typestate phases, each riding inside a [`StudyEffect`](crate::StudyEffect) so errors
//! short-circuit (tagged with the failing verb) and warnings accumulate into the final verdict.
//! Each verb is defined only on the phase where it is meaningful, so a mis-ordered study does not
//! compile: there is no `record` or `gates` before rows exist, and no `verdict` before a gating
//! sequence is applied.
//!
//! This module carries the pointwise / swept path (`cases → sweep → record → gates → verdict`);
//! the march, reduction, and counterfactual phases extend it in sibling impls.

mod gate_seq;
mod verdict;
mod view;

pub use gate_seq::{GateFn, GateSeq};
pub use verdict::{GateOutcome, Verdict};
pub use view::StudyView;

use crate::traits::Marchable;
use crate::types::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow::sweep::sweep;
use deep_causality_file::{FromTableRow, TableRow, TableScalar, read_rows, read_table};
use deep_causality_haft::IoAction;
use deep_causality_par::MaybeParallel;
use deep_causality_physics::PhysicsError;
use std::path::Path;

/// The study entry: a titled campaign awaiting its case axis. Opened by
/// [`CfdFlow::study`](crate::CfdFlow::study).
pub struct StudyDef {
    title: String,
}

impl StudyDef {
    pub(crate) fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
        }
    }

    /// The study title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// An in-memory case list as the study's case axis.
    pub fn cases<T>(self, cases: Vec<T>) -> StudyEffect<Cases<T>> {
        StudyEffect::pure(Cases {
            title: self.title,
            cases,
        })
    }

    /// Read one named column from a table as the case axis (a schedule of scalars).
    pub fn read<R: TableScalar>(self, path: impl AsRef<Path>, column: &str) -> StudyEffect<Cases<R>> {
        let title = self.title;
        let loaded = read_table::<R>(path).run().and_then(|t| t.column(column));
        match loaded {
            Ok(cases) => StudyEffect::pure(Cases { title, cases }),
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("read", e))),
        }
    }

    /// Read a typed test matrix as the case axis (one case per file row, columns matched to the
    /// row type's schema by name).
    pub fn matrix<T: FromTableRow>(self, path: impl AsRef<Path>) -> StudyEffect<Cases<T>> {
        let title = self.title;
        match read_rows::<T>(path).run() {
            Ok(cases) => StudyEffect::pure(Cases { title, cases }),
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("matrix", e))),
        }
    }
}

// ── Phase states (plain data; the run config / rows travel inside the state) ──────────────────

/// The typed case axis: the study's cases, awaiting a binder or a sweep.
pub struct Cases<T> {
    title: String,
    cases: Vec<T>,
}

/// The reduced result rows (with any prior refinement rounds), awaiting record / gates.
pub struct Swept<Row> {
    title: String,
    cases_len: usize,
    rows: Vec<Row>,
    rounds: Vec<Vec<Row>>,
}

/// The cases plus a shared apparatus built once (a fitted shock, a calibration), awaiting a
/// rig-sharing sweep.
pub struct Prepared<T, A> {
    title: String,
    cases: Vec<T>,
    apparatus: A,
}

/// One case bound to a solver config, awaiting the march.
pub struct Configured<T, C> {
    title: String,
    cases: Vec<T>,
    configs: Vec<C>,
}

/// One report per case, awaiting the reduction to rows.
pub struct Marched<T, C, R: CfdScalar> {
    title: String,
    cases_len: usize,
    cases: Vec<T>,
    configs: Vec<C>,
    reports: Vec<Report<R>>,
}

/// What a reduction reads for one case: the case, its config, and its report.
pub struct CaseRun<'a, T, C, R: CfdScalar> {
    case: &'a T,
    config: &'a C,
    report: &'a Report<R>,
}

impl<'a, T, C, R: CfdScalar> CaseRun<'a, T, C, R> {
    /// The swept case value.
    pub fn case(&self) -> &T {
        self.case
    }

    /// The solver config this case marched.
    pub fn config(&self) -> &C {
        self.config
    }

    /// The report the march produced.
    pub fn report(&self) -> &Report<R> {
        self.report
    }
}

/// The rows plus the accumulated gate verdict, awaiting more gates or the terminal verdict.
pub struct Judged<Row> {
    title: String,
    cases_len: usize,
    rows: Vec<Row>,
    rounds: Vec<Vec<Row>>,
    verdict: Verdict,
}

// ── Cases: the pointwise sweep + observe ──────────────────────────────────────────────────────

impl<T> StudyEffect<Cases<T>>
where
    T: MaybeParallel,
{
    /// The pointwise sweep: one case, one result row. Order-preserving and first-error-wins,
    /// concurrent under the `parallel` feature and inline otherwise, with identical results in
    /// both. A study that reaches [`Swept`] has therefore produced a row for every case.
    pub fn sweep<Row, F>(self, f: F) -> StudyEffect<Swept<Row>>
    where
        Row: MaybeParallel,
        F: Fn(&T) -> Result<Row, PhysicsError> + MaybeParallel,
    {
        self.and_then(|c| {
            let cases_len = c.cases.len();
            match sweep(&c.cases, f) {
                Ok(rows) => StudyEffect::pure(Swept {
                    title: c.title,
                    cases_len,
                    rows,
                    rounds: Vec::new(),
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("sweep", e))),
            }
        })
    }

    /// Observe the cases without consuming the phase (the print seam); runs only on a live study.
    pub fn inspect(self, f: impl FnOnce(&[T])) -> Self {
        self.map(|c| {
            f(&c.cases);
            c
        })
    }
}

impl<T> StudyEffect<Cases<T>> {
    /// Build the shared apparatus once — a fitted shock, a calibration — typed into the phase so
    /// every case sees the same rig by reference. Runs exactly once, before any case.
    pub fn prepare<A>(
        self,
        rig: impl FnOnce() -> Result<A, PhysicsError>,
    ) -> StudyEffect<Prepared<T, A>> {
        self.and_then(|c| match rig() {
            Ok(apparatus) => StudyEffect::pure(Prepared {
                title: c.title,
                cases: c.cases,
                apparatus,
            }),
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("prepare", e))),
        })
    }

    /// Bind each case to an owned solver config — the configuration/execution seam, where
    /// `model_config` enters the grammar. Config construction runs sequentially, first-error-wins.
    pub fn case<C>(
        self,
        f: impl Fn(&T) -> Result<C, PhysicsError>,
    ) -> StudyEffect<Configured<T, C>> {
        self.and_then(|c| {
            let mut configs = Vec::with_capacity(c.cases.len());
            for case in &c.cases {
                match f(case) {
                    Ok(cfg) => configs.push(cfg),
                    Err(e) => {
                        return StudyEffect::from_result(Err(StudyError::in_stage("case", e)));
                    }
                }
            }
            StudyEffect::pure(Configured {
                title: c.title,
                cases: c.cases,
                configs,
            })
        })
    }
}

// ── Prepared: the rig-sharing sweep ───────────────────────────────────────────────────────────

impl<T, A> StudyEffect<Prepared<T, A>>
where
    T: MaybeParallel,
    A: MaybeParallel,
{
    /// The apparatus-sharing sweep: every case sees the same rig by reference. Order-preserving,
    /// first-error-wins, concurrent under `parallel`.
    pub fn sweep<Row, F>(self, f: F) -> StudyEffect<Swept<Row>>
    where
        Row: MaybeParallel,
        F: Fn(&A, &T) -> Result<Row, PhysicsError> + MaybeParallel,
    {
        self.and_then(|p| {
            let cases_len = p.cases.len();
            let out = sweep(&p.cases, |case| f(&p.apparatus, case));
            match out {
                Ok(rows) => StudyEffect::pure(Swept {
                    title: p.title,
                    cases_len,
                    rows,
                    rounds: Vec::new(),
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("sweep", e))),
            }
        })
    }

    /// Observe the cases without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[T])) -> Self {
        self.map(|p| {
            f(&p.cases);
            p
        })
    }
}

// ── Configured: march every case ──────────────────────────────────────────────────────────────

impl<T, C> StudyEffect<Configured<T, C>>
where
    T: MaybeParallel,
    C: MaybeParallel,
{
    /// March every configured case. The config type selects the solver family (`C: Marchable`);
    /// the sweep guarantee — order-preserving, first-error-wins, concurrent under `parallel` —
    /// applies to the fan-out.
    pub fn march<R>(self) -> StudyEffect<Marched<T, C, R>>
    where
        R: CfdScalar,
        C: Marchable<R>,
        Report<R>: MaybeParallel,
    {
        self.and_then(|cfg| {
            let cases_len = cfg.cases.len();
            let idx: Vec<usize> = (0..cfg.configs.len()).collect();
            match sweep(&idx, |&i| cfg.configs[i].march()) {
                Ok(reports) => StudyEffect::pure(Marched {
                    title: cfg.title,
                    cases_len,
                    cases: cfg.cases,
                    configs: cfg.configs,
                    reports,
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("march", e))),
            }
        })
    }

    /// Observe the configured cases without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[T])) -> Self {
        self.map(|cfg| {
            f(&cfg.cases);
            cfg
        })
    }
}

// ── Marched: reduce reports to rows ───────────────────────────────────────────────────────────

impl<T, C, R> StudyEffect<Marched<T, C, R>>
where
    T: MaybeParallel,
    C: MaybeParallel,
    R: CfdScalar,
    Report<R>: MaybeParallel,
{
    /// Data reduction: one report, one row. The reduction reads the case, its config, and its
    /// report through a [`CaseRun`]. Order-preserving, first-error-wins, concurrent.
    pub fn reduce<Row, F>(self, f: F) -> StudyEffect<Swept<Row>>
    where
        Row: MaybeParallel,
        F: Fn(&CaseRun<'_, T, C, R>) -> Result<Row, PhysicsError> + MaybeParallel,
    {
        self.and_then(|m| {
            let idx: Vec<usize> = (0..m.reports.len()).collect();
            let out = sweep(&idx, |&i| {
                let run = CaseRun {
                    case: &m.cases[i],
                    config: &m.configs[i],
                    report: &m.reports[i],
                };
                f(&run)
            });
            match out {
                Ok(rows) => StudyEffect::pure(Swept {
                    title: m.title,
                    cases_len: m.cases_len,
                    rows,
                    rounds: Vec::new(),
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("reduce", e))),
            }
        })
    }

    /// Observe the reports without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[Report<R>])) -> Self {
        self.map(|m| {
            f(&m.reports);
            m
        })
    }
}

impl<T, C, R> StudyEffect<Marched<T, C, R>>
where
    R: CfdScalar,
{
    /// The collective reduction: sees every case's run at once and returns one row per case in
    /// order. For cross-case references — a shared aim point derived from one case that every
    /// other case scores against. Runs sequentially (it needs the whole set), unlike `reduce`.
    pub fn reduce_all<Row, F>(self, f: F) -> StudyEffect<Swept<Row>>
    where
        F: FnOnce(&[CaseRun<'_, T, C, R>]) -> Result<Vec<Row>, PhysicsError>,
    {
        self.and_then(|m| {
            let result = {
                let runs: Vec<CaseRun<'_, T, C, R>> = (0..m.reports.len())
                    .map(|i| CaseRun {
                        case: &m.cases[i],
                        config: &m.configs[i],
                        report: &m.reports[i],
                    })
                    .collect();
                f(&runs)
            };
            match result {
                Ok(rows) => StudyEffect::pure(Swept {
                    title: m.title,
                    cases_len: m.cases_len,
                    rows,
                    rounds: Vec::new(),
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("reduce_all", e))),
            }
        })
    }
}

// ── Swept: record, gates, observe ─────────────────────────────────────────────────────────────

impl<Row> StudyEffect<Swept<Row>> {
    /// Write the rows through the typed table writer; the schema and precision come from the row
    /// type. Recording precedes judgment, so the table exists even when the gates then fail.
    pub fn record(self, path: impl AsRef<Path>) -> Self
    where
        Row: TableRow + Clone,
    {
        let path = path.as_ref().to_path_buf();
        self.and_then(|s| {
            use deep_causality_file::write_rows;
            use deep_causality_haft::IoAction;
            match write_rows(&path, s.rows.clone()).run() {
                Ok(()) => StudyEffect::pure(s),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("record", e))),
            }
        })
    }

    /// Insert a whole gating sequence, opening judgment. The sequence is row-typed, so a sequence
    /// built for another study's rows does not compile here.
    pub fn gates(self, seq: GateSeq<Row>) -> StudyEffect<Judged<Row>> {
        self.map(|s| {
            let view = StudyView::new(&s.rows, &s.rounds, s.cases_len, &s.title);
            let verdict = seq.check(&view);
            Judged {
                title: s.title,
                cases_len: s.cases_len,
                rows: s.rows,
                rounds: s.rounds,
                verdict,
            }
        })
    }

    /// Observe the rows without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[Row])) -> Self {
        self.map(|s| {
            f(&s.rows);
            s
        })
    }
}

// ── Judged: more gates, observe, terminal verdict ─────────────────────────────────────────────

impl<Row> StudyEffect<Judged<Row>> {
    /// A complex workflow inserts its later gating sequences the same way; the outcomes merge.
    pub fn gates(self, seq: GateSeq<Row>) -> Self {
        self.map(|mut j| {
            let view = StudyView::new(&j.rows, &j.rounds, j.cases_len, &j.title);
            let more = seq.check(&view);
            j.verdict = j.verdict.merge(more);
            j
        })
    }

    /// Observe the rows without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[Row])) -> Self {
        self.map(|j| {
            f(&j.rows);
            j
        })
    }

    /// Terminal: resolve the study to its [`Verdict`], attaching the accumulated warnings. No
    /// printing, no exit. A carried error surfaces as `Err`, naming the verb that failed.
    pub fn verdict(self) -> Result<Verdict, StudyError> {
        let (result, warnings) = self.into_parts();
        result.map(|j| j.verdict.with_warnings(warnings.entries().to_vec()))
    }
}
