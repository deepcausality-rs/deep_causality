/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The origin-fork counterfactual campaign: `baseline` builds the declared origin world once,
//! `alternate` binds one world per case, `ensemble` fans out the draws, and `march_for` flies them
//! — with one audit file per branch alongside the main spawn/rejoin record.

use crate::CfdFlow;
use crate::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow::coupling::{CoupledField, PhysicsStage};
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow::sweep::sweep;
use crate::types::flow_config::CompressibleMarchConfig;
use deep_causality_core::AlternatableContext;
use deep_causality_par::MaybeParallel;
use deep_causality_physics::PhysicsError;
use std::path::{Path, PathBuf};

use super::*;

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
