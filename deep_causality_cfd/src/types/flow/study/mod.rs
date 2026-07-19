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
//!
//! The verbs are grouped by the path they belong to: [`sweep_phase`] carries the pointwise and
//! march-every-case paths, [`event_fork`] the fork/branch counterfactual, [`refine`] its refinement
//! round, [`origin_fork`] the baseline/alternate/ensemble campaign, and [`reduce`] the reduction
//! and terminal verdict every path converges on.

mod event_fork;
mod gate_seq;
mod origin_fork;
mod reduce;
mod refine;
mod sweep_phase;
mod verdict;
mod view;

pub use gate_seq::{GateFn, GateSeq};
pub use origin_fork::{Alternated, Counterfactual, CoupledCampaign, EnsembleMarched};
pub use refine::{RefineBranched, RefineMarched, Refining};
pub use verdict::{GateOutcome, Verdict};
pub use view::StudyView;

use crate::CfdScalar;
use crate::types::flow::CompressiblePause;
use crate::types::flow::Report;
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow_config::CompressibleMarchConfig;
use deep_causality_file::{FromTableRow, TableScalar, read_rows, read_table};
use deep_causality_haft::IoAction;
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
