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

use crate::CfdFlow;
use crate::traits::Marchable;
use crate::types::CfdScalar;
use crate::types::flow::CompressiblePause;
use crate::types::flow::Report;
use crate::types::flow::coupling::{CoupledField, PhysicsStage};
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow::sweep::sweep;
use crate::types::flow_config::CompressibleMarchConfig;
use deep_causality_core::AlternatableContext;
use deep_causality_file::{FromTableRow, TableRow, TableScalar, read_rows, read_table};
use deep_causality_haft::IoAction;
use deep_causality_par::MaybeParallel;
use deep_causality_physics::PhysicsError;
use std::path::{Path, PathBuf};

/// The study entry: a titled campaign awaiting its case axis. Opened by
/// [`CfdFlow::study`](crate::CfdFlow::study).
pub struct StudyDef {
    title: String,
    audit: Option<PathBuf>,
}

impl StudyDef {
    pub(crate) fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            audit: None,
        }
    }

    /// The study title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Attach a disk audit-log base path (the campaign-level `save_log` verb, before the cases).
    /// The coupled campaign (`march_for`) writes one file per branch under a concurrent fan-out —
    /// `<base>.sweep-N.<world>.draw-D.log`, each by one thread, flushed stepwise — plus a
    /// `<base>.main.log` naming every spawn and rejoin. Without it the run is unchanged.
    #[must_use]
    pub fn save_log(mut self, base_path: impl AsRef<Path>) -> Self {
        self.audit = Some(base_path.as_ref().to_path_buf());
        self
    }

    /// An in-memory case list as the study's case axis.
    pub fn cases<T>(self, cases: Vec<T>) -> StudyEffect<Cases<T>> {
        StudyEffect::pure(Cases {
            title: self.title,
            cases,
            audit: self.audit,
        })
    }

    /// Read one named column from a table as the case axis (a schedule of scalars).
    pub fn read<R: TableScalar>(
        self,
        path: impl AsRef<Path>,
        column: &str,
    ) -> StudyEffect<Cases<R>> {
        let title = self.title;
        let audit = self.audit;
        let loaded = read_table::<R>(path).run().and_then(|t| t.column(column));
        match loaded {
            Ok(cases) => StudyEffect::pure(Cases {
                title,
                cases,
                audit,
            }),
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("read", e))),
        }
    }

    /// Read a typed test matrix as the case axis (one case per file row, columns matched to the
    /// row type's schema by name).
    pub fn matrix<T: FromTableRow>(self, path: impl AsRef<Path>) -> StudyEffect<Cases<T>> {
        let title = self.title;
        let audit = self.audit;
        match read_rows::<T>(path).run() {
            Ok(cases) => StudyEffect::pure(Cases {
                title,
                cases,
                audit,
            }),
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("matrix", e))),
        }
    }
}

// ── Phase states (plain data; the run config / rows travel inside the state) ──────────────────

/// The typed case axis: the study's cases, awaiting a binder or a sweep.
pub struct Cases<T> {
    title: String,
    cases: Vec<T>,
    /// The campaign-level audit base path (`save_log`), carried to the coupled `march_for`. Only
    /// the origin-counterfactual coupled path consumes it; the other binders drop it.
    audit: Option<PathBuf>,
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

/// The cases plus the shared fork point — a paused trajectory this study's branches continue from.
/// Produced by [`fork`](StudyEffect::fork); its only verb is [`branch`](StudyEffect::branch).
pub struct ForkStudy<'p, 'c, T, R: CfdScalar + 'c, S> {
    title: String,
    cases: Vec<T>,
    pause: &'p CompressiblePause<'c, R, S>,
}

/// One branch world per case at the fork, awaiting the continued march. Produced by
/// [`branch`](StudyEffect::branch); its only verb is [`continue_for`](StudyEffect::continue_for).
pub struct Branched<'p, 'c, T, R: CfdScalar + 'c, S> {
    title: String,
    cases: Vec<T>,
    worlds: Vec<CompressibleMarchConfig<R>>,
    pause: &'p CompressiblePause<'c, R, S>,
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

// ── The event-fork counterfactual path: fork → branch → continue_for ──────────────────────────

impl<T> StudyEffect<Cases<T>> {
    /// Declare the shared fork point: a paused trajectory this study's branches continue from. The
    /// case axis becomes the command that distinguishes the branches; every branch will resume
    /// this pause's state bit-identically and O(1) copy-on-write. `fork` itself never fails — a
    /// broken pause is carried through and surfaces as each branch's error at
    /// [`continue_for`](StudyEffect::continue_for).
    pub fn fork<'p, 'c, R, S>(
        self,
        pause: &'p CompressiblePause<'c, R, S>,
    ) -> StudyEffect<ForkStudy<'p, 'c, T, R, S>>
    where
        R: CfdScalar,
    {
        self.map(|c| ForkStudy {
            title: c.title,
            cases: c.cases,
            pause,
        })
    }
}

impl<'p, 'c, T, R, S> StudyEffect<ForkStudy<'p, 'c, T, R, S>>
where
    R: CfdScalar,
{
    /// Bind each case to a branch world at the fork. Guarantee: every branch resumes the pause's
    /// state bit-identically, alternated into its own world (the `!!ContextAlternation!!` audit
    /// marker) and sharing the paused state copy-on-write. World construction runs sequentially,
    /// first-error-wins — the config/execution seam, exactly like [`case`](StudyEffect::case).
    pub fn branch(
        self,
        f: impl Fn(&T) -> Result<CompressibleMarchConfig<R>, PhysicsError>,
    ) -> StudyEffect<Branched<'p, 'c, T, R, S>> {
        self.and_then(|fs| {
            let mut worlds = Vec::with_capacity(fs.cases.len());
            for case in &fs.cases {
                match f(case) {
                    Ok(w) => worlds.push(w),
                    Err(e) => {
                        return StudyEffect::from_result(Err(StudyError::in_stage("branch", e)));
                    }
                }
            }
            StudyEffect::pure(Branched {
                title: fs.title,
                cases: fs.cases,
                worlds,
                pause: fs.pause,
            })
        })
    }
}

impl<'p, 'c, T, R, S> StudyEffect<Branched<'p, 'c, T, R, S>>
where
    T: MaybeParallel,
    R: CfdScalar,
    S: PhysicsStage<2, R>,
    CompressibleMarchConfig<R>: MaybeParallel,
    Report<R>: MaybeParallel,
    CompressiblePause<'c, R, S>: MaybeParallel,
{
    /// Fly every branch from the shared fork for `steps` coupled steps, concurrently under the
    /// `parallel` feature and inline otherwise, reports in case order. Lowers onto
    /// `CompressiblePause::continue_branches`; each branch takes its single copy-on-write clone
    /// at its first write. The result is a [`Marched`] exactly like the march path, so `reduce`
    /// reads each branch through a [`CaseRun`] — the branch world is the case's config.
    pub fn continue_for(
        self,
        steps: usize,
    ) -> StudyEffect<Marched<T, CompressibleMarchConfig<R>, R>> {
        self.and_then(|b| {
            let cases_len = b.cases.len();
            let refs: Vec<&CompressibleMarchConfig<R>> = b.worlds.iter().collect();
            match b.pause.continue_branches(&refs, steps) {
                Ok(reports) => StudyEffect::pure(Marched {
                    title: b.title,
                    cases_len,
                    cases: b.cases,
                    configs: b.worlds,
                    reports,
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("continue_for", e))),
            }
        })
    }
}

// ── The refinement round: refine → branch → continue_for → reduce_all, prior rounds carried ────
//
// A second event-fork round from the *same* shared fork point, its cases derived from the first
// round's rows. `refine` re-attaches the fork point that `reduce_all` dropped and threads the
// finished round into the next `Swept`'s `rounds`, so the final `StudyView` exposes both rounds
// (the gates read `view.rows()` for the fine round and `view.rounds()` for the coarse one).

/// After [`refine`](StudyEffect::refine): the next round's cases, the shared fork point re-attached,
/// and the prior round's rows carried across the (possible) case-type change. Its only verb is
/// [`branch`](StudyEffect::branch).
pub struct Refining<'p, 'c, T2, Row, R: CfdScalar + 'c, S> {
    title: String,
    cases: Vec<T2>,
    prior_rounds: Vec<Vec<Row>>,
    pause: &'p CompressiblePause<'c, R, S>,
}

/// The refinement round's branch worlds, prior rounds carried. Its only verb is
/// [`continue_for`](StudyEffect::continue_for).
pub struct RefineBranched<'p, 'c, T2, Row, R: CfdScalar + 'c, S> {
    title: String,
    cases: Vec<T2>,
    worlds: Vec<CompressibleMarchConfig<R>>,
    prior_rounds: Vec<Vec<Row>>,
    pause: &'p CompressiblePause<'c, R, S>,
}

/// The refinement round's continued reports, prior rounds carried. Its only verb is
/// [`reduce_all`](StudyEffect::reduce_all), which lands on [`Swept`] with `rounds` populated.
pub struct RefineMarched<T2, Row, R: CfdScalar> {
    title: String,
    cases_len: usize,
    cases: Vec<T2>,
    configs: Vec<CompressibleMarchConfig<R>>,
    reports: Vec<Report<R>>,
    prior_rounds: Vec<Vec<Row>>,
}

impl<Row> StudyEffect<Swept<Row>> {
    /// Open the next refinement round: derive the round's cases from the rows so far, re-attaching
    /// the shared fork point so the branches continue from the *same* paused trajectory (the fork
    /// point that `reduce_all` dropped). The just-finished round is retained and surfaces on the
    /// final [`StudyView::rounds`], so a gate can require the refined round to at least match it.
    pub fn refine<'p, 'c, T2, R, S>(
        self,
        pause: &'p CompressiblePause<'c, R, S>,
        f: impl FnOnce(&[Row]) -> Result<Vec<T2>, PhysicsError>,
    ) -> StudyEffect<Refining<'p, 'c, T2, Row, R, S>>
    where
        R: CfdScalar,
    {
        self.and_then(|s| match f(&s.rows) {
            Ok(cases) => {
                let mut prior_rounds = s.rounds;
                prior_rounds.push(s.rows);
                StudyEffect::pure(Refining {
                    title: s.title,
                    cases,
                    prior_rounds,
                    pause,
                })
            }
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("refine", e))),
        })
    }
}

impl<'p, 'c, T2, Row, R, S> StudyEffect<Refining<'p, 'c, T2, Row, R, S>>
where
    R: CfdScalar,
{
    /// Bind each refinement case to a branch world at the shared fork — the same seam as the first
    /// round's [`branch`](StudyEffect::branch), first-error-wins.
    pub fn branch(
        self,
        f: impl Fn(&T2) -> Result<CompressibleMarchConfig<R>, PhysicsError>,
    ) -> StudyEffect<RefineBranched<'p, 'c, T2, Row, R, S>> {
        self.and_then(|r| {
            let mut worlds = Vec::with_capacity(r.cases.len());
            for case in &r.cases {
                match f(case) {
                    Ok(w) => worlds.push(w),
                    Err(e) => {
                        return StudyEffect::from_result(Err(StudyError::in_stage("branch", e)));
                    }
                }
            }
            StudyEffect::pure(RefineBranched {
                title: r.title,
                cases: r.cases,
                worlds,
                prior_rounds: r.prior_rounds,
                pause: r.pause,
            })
        })
    }
}

impl<'p, 'c, T2, Row, R, S> StudyEffect<RefineBranched<'p, 'c, T2, Row, R, S>>
where
    T2: MaybeParallel,
    R: CfdScalar,
    S: PhysicsStage<2, R>,
    CompressibleMarchConfig<R>: MaybeParallel,
    Report<R>: MaybeParallel,
    CompressiblePause<'c, R, S>: MaybeParallel,
{
    /// Fly the refinement round from the shared fork, exactly like the first round's
    /// [`continue_for`](StudyEffect::continue_for); the prior rounds ride along to the reduction.
    pub fn continue_for(self, steps: usize) -> StudyEffect<RefineMarched<T2, Row, R>> {
        self.and_then(|b| {
            let cases_len = b.cases.len();
            let refs: Vec<&CompressibleMarchConfig<R>> = b.worlds.iter().collect();
            match b.pause.continue_branches(&refs, steps) {
                Ok(reports) => StudyEffect::pure(RefineMarched {
                    title: b.title,
                    cases_len,
                    cases: b.cases,
                    configs: b.worlds,
                    reports,
                    prior_rounds: b.prior_rounds,
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("continue_for", e))),
            }
        })
    }
}

impl<T2, Row, R> StudyEffect<RefineMarched<T2, Row, R>>
where
    R: CfdScalar,
{
    /// The collective reduction of the refinement round — one row per case, seeing every branch at
    /// once (the shared aim point stays the coarse round's, so the rounds remain comparable). Lands
    /// on [`Swept`] with `rounds` carrying the prior round(s), for the gates to compare against.
    pub fn reduce_all<F>(self, f: F) -> StudyEffect<Swept<Row>>
    where
        F: FnOnce(
            &[CaseRun<'_, T2, CompressibleMarchConfig<R>, R>],
        ) -> Result<Vec<Row>, PhysicsError>,
    {
        self.and_then(|m| {
            let result = {
                let runs: Vec<CaseRun<'_, T2, CompressibleMarchConfig<R>, R>> =
                    (0..m.reports.len())
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
                    rounds: m.prior_rounds,
                }),
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("reduce_all", e))),
            }
        })
    }

    /// Observe the refinement round's reports without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[Report<R>])) -> Self {
        self.map(|m| {
            f(&m.reports);
            m
        })
    }
}

// ── The origin-fork counterfactual path: baseline → alternate → ensemble → couple → march_for ──
//
// The weather campaign: every case is a whole atmosphere alternated from one declared baseline
// world, flown from scratch through the coupled physics, optionally an ensemble of receiver-noise
// draws. Unlike the event-fork path (which resumes a shared pause), each origin case runs its own
// coupled march on the compressible carrier from a fresh field.

/// The cases plus the declared baseline world. Produced by [`baseline`](StudyEffect::baseline); its
/// only verb is [`alternate`](StudyEffect::alternate).
pub struct Counterfactual<T, R: CfdScalar> {
    title: String,
    cases: Vec<T>,
    baseline: CompressibleMarchConfig<R>,
    audit: Option<PathBuf>,
}

/// One world per case, each alternated from the baseline, plus the ensemble draw multiplicity.
/// Produced by [`alternate`](StudyEffect::alternate); verbs [`ensemble`](StudyEffect::ensemble) and
/// [`couple`](StudyEffect::couple).
pub struct Alternated<T, R: CfdScalar> {
    title: String,
    cases: Vec<T>,
    baseline: CompressibleMarchConfig<R>,
    worlds: Vec<CompressibleMarchConfig<R>>,
    draws: usize,
    audit: Option<PathBuf>,
}

/// The alternated campaign with its coupling stack factory attached. Produced by
/// [`couple`](StudyEffect::couple); its only verb is [`march_for`](StudyEffect::march_for).
pub struct CoupledCampaign<T, R: CfdScalar, S, FC> {
    title: String,
    cases: Vec<T>,
    baseline: CompressibleMarchConfig<R>,
    worlds: Vec<CompressibleMarchConfig<R>>,
    draws: usize,
    coupling: FC,
    audit: Option<PathBuf>,
    _stack: core::marker::PhantomData<fn() -> S>,
}

/// One report per case and draw (flat, case-major), awaiting the ensemble reduction. Produced by
/// [`march_for`](StudyEffect::march_for); verbs [`reduce_ensemble`](StudyEffect::reduce_ensemble)
/// and `inspect`.
pub struct EnsembleMarched<T, R: CfdScalar> {
    title: String,
    cases: Vec<T>,
    draws: usize,
    reports: Vec<Report<R>>,
}

impl<T> StudyEffect<Cases<T>> {
    /// Declare the validated origin world the study's counterfactuals alternate from — built once,
    /// before any case. The baseline itself flies unmarked; every case whose alternated world
    /// differs from it carries the `!!ContextAlternation!!` audit marker.
    pub fn baseline<R>(
        self,
        origin: impl FnOnce() -> Result<CompressibleMarchConfig<R>, PhysicsError>,
    ) -> StudyEffect<Counterfactual<T, R>>
    where
        R: CfdScalar,
    {
        self.and_then(|c| match origin() {
            Ok(baseline) => StudyEffect::pure(Counterfactual {
                title: c.title,
                cases: c.cases,
                baseline,
                audit: c.audit,
            }),
            Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("baseline", e))),
        })
    }
}

impl<T, R> StudyEffect<Counterfactual<T, R>>
where
    R: CfdScalar,
{
    /// Bind each case to a world alternated from the baseline. Guarantee: every case whose world
    /// differs from the baseline (by name) flies alternated and carries the alternation marker; the
    /// baseline case flies unmarked. World construction runs sequentially, first-error-wins.
    pub fn alternate(
        self,
        f: impl Fn(&T) -> Result<CompressibleMarchConfig<R>, PhysicsError>,
    ) -> StudyEffect<Alternated<T, R>> {
        self.and_then(|cf| {
            let mut worlds = Vec::with_capacity(cf.cases.len());
            for case in &cf.cases {
                match f(case) {
                    Ok(w) => worlds.push(w),
                    Err(e) => {
                        return StudyEffect::from_result(Err(StudyError::in_stage("alternate", e)));
                    }
                }
            }
            StudyEffect::pure(Alternated {
                title: cf.title,
                cases: cf.cases,
                baseline: cf.baseline,
                worlds,
                draws: 1,
                audit: cf.audit,
            })
        })
    }
}

impl<T, R> StudyEffect<Alternated<T, R>>
where
    R: CfdScalar,
{
    /// Raise the sweep multiplicity: each case flies `draws` times, the draw index threaded to the
    /// coupling, so the ensemble reduction sees the whole draw set per case.
    pub fn ensemble(self, draws: usize) -> Self {
        self.map(|mut a| {
            a.draws = draws.max(1);
            a
        })
    }

    /// Attach the coupled-physics stack per case and draw — the multiphysics seam. `f` receives the
    /// case and the draw index (0 unless [`ensemble`](StudyEffect::ensemble) raised the
    /// multiplicity) and returns the composed stack.
    pub fn couple<S, FC>(self, f: FC) -> StudyEffect<CoupledCampaign<T, R, S, FC>>
    where
        S: PhysicsStage<2, R>,
        FC: Fn(&T, usize) -> S,
    {
        self.map(|a| CoupledCampaign {
            title: a.title,
            cases: a.cases,
            baseline: a.baseline,
            worlds: a.worlds,
            draws: a.draws,
            coupling: f,
            audit: a.audit,
            _stack: core::marker::PhantomData,
        })
    }
}

impl<T, R, S, FC> StudyEffect<CoupledCampaign<T, R, S, FC>>
where
    T: MaybeParallel,
    R: CfdScalar,
    S: PhysicsStage<2, R> + MaybeParallel,
    FC: Fn(&T, usize) -> S + MaybeParallel,
    CompressibleMarchConfig<R>: MaybeParallel,
    Report<R>: MaybeParallel,
{
    /// Fly every (case, draw) for a fixed horizon on the compressible carrier from a fresh field,
    /// concurrently under `parallel`, reports in case-major then draw order. Each case whose world
    /// differs from the baseline is alternated into its world (the `!!ContextAlternation!!` marker);
    /// the fresh initial field `field` is built per run (the F2 seam — it lives in neither the
    /// config nor the stack). First-error-wins.
    pub fn march_for(
        self,
        steps: usize,
        field: impl Fn() -> CoupledField<R> + MaybeParallel,
    ) -> StudyEffect<EnsembleMarched<T, R>> {
        self.and_then(|c| {
            let draws = c.draws;
            // Audit fan-out: the main file names every branch file it spawns before the concurrent
            // round; each branch writes its own file, exclusively, flushed stepwise inside the run.
            if let Some(base) = &c.audit {
                let mut spawn = String::from("fan-out sweep-1: branches spawned");
                for i in 0..c.cases.len() {
                    for d in 0..draws {
                        spawn.push_str(&alloc::format!(
                            "\n  spawn {}",
                            audit_branch_path(base, i, c.worlds[i].name(), d).display()
                        ));
                    }
                }
                if let Err(e) =
                    crate::types::flow::audit::append_line(audit_main_path(base), &spawn)
                {
                    return StudyEffect::from_result(Err(StudyError::in_stage("save_log", e)));
                }
            }
            let idx: Vec<usize> = (0..c.cases.len() * draws).collect();
            let out = sweep(&idx, |&k| {
                let (i, draw) = (k / draws, k % draws);
                let world = &c.worlds[i];
                let stack = (c.coupling)(&c.cases[i], draw);
                let mut run = CfdFlow::march(&c.baseline);
                if world.name() != c.baseline.name() {
                    run = run.alternate_context(world);
                }
                // One thread, one file: this branch flushes its own audit file, named by world+draw.
                if let Some(base) = &c.audit {
                    run = run.save_log(audit_branch_path(base, i, world.name(), draw));
                }
                run.couple(stack).from_field(field()).run_for(steps)
            });
            match out {
                Ok(reports) => {
                    if let Some(base) = &c.audit {
                        let mut rejoin = String::from("rejoin sweep-1: branch outcomes");
                        for (k, report) in reports.iter().enumerate() {
                            let (i, d) = (k / draws, k % draws);
                            let marked = report
                                .effect_log()
                                .map(|l| alloc::format!("{l}").contains("!!ContextAlternation!!"))
                                .unwrap_or(false);
                            rejoin.push_str(&alloc::format!(
                                "\n  rejoin {}.draw-{d}: complete ({})",
                                c.worlds[i].name(),
                                if marked { "alternated" } else { "baseline" },
                            ));
                        }
                        if let Err(e) =
                            crate::types::flow::audit::append_line(audit_main_path(base), &rejoin)
                        {
                            return StudyEffect::from_result(Err(StudyError::in_stage(
                                "save_log", e,
                            )));
                        }
                    }
                    StudyEffect::pure(EnsembleMarched {
                        title: c.title,
                        cases: c.cases,
                        draws,
                        reports,
                    })
                }
                Err(e) => StudyEffect::from_result(Err(StudyError::in_stage("march_for", e))),
            }
        })
    }
}

/// The per-branch audit file path: `<base>.sweep-1.case-<NN>-<world>.draw-<draw>.log`. The case
/// index makes the file unique per branch even when two cases share a world name (so one branch
/// can never truncate another's log), and the world token is reduced to a filesystem-safe form so
/// a name carrying path separators cannot redirect the write outside the base directory.
fn audit_branch_path(base: &Path, case: usize, world: &str, draw: usize) -> PathBuf {
    let token: String = world
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    PathBuf::from(alloc::format!(
        "{}.sweep-1.case-{case:02}-{token}.draw-{draw}.log",
        base.to_string_lossy()
    ))
}

/// The main audit file path: `<base>.main.log` (the fan-out spawn/rejoin narration).
fn audit_main_path(base: &Path) -> PathBuf {
    PathBuf::from(alloc::format!("{}.main.log", base.to_string_lossy()))
}

impl<T, R> StudyEffect<EnsembleMarched<T, R>>
where
    R: CfdScalar,
{
    /// The ensemble reduction: one case and its whole draw set in, one row out (means, scatters,
    /// worst draws computed where the data is). Order-preserving over cases, first-error-wins.
    pub fn reduce_ensemble<Row, F>(self, f: F) -> StudyEffect<Swept<Row>>
    where
        F: Fn(&T, &[Report<R>]) -> Result<Row, PhysicsError>,
    {
        self.and_then(|m| {
            let cases_len = m.cases.len();
            let mut rows = Vec::with_capacity(cases_len);
            for i in 0..cases_len {
                let draws = &m.reports[i * m.draws..(i + 1) * m.draws];
                match f(&m.cases[i], draws) {
                    Ok(row) => rows.push(row),
                    Err(e) => {
                        return StudyEffect::from_result(Err(StudyError::in_stage(
                            "reduce_ensemble",
                            e,
                        )));
                    }
                }
            }
            StudyEffect::pure(Swept {
                title: m.title,
                cases_len,
                rows,
                rounds: Vec::new(),
            })
        })
    }

    /// Observe the flat (case, draw) reports without consuming the phase.
    pub fn inspect(self, f: impl FnOnce(&[Report<R>])) -> Self {
        self.map(|m| {
            f(&m.reports);
            m
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
