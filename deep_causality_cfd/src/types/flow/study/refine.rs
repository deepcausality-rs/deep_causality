/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The refinement round: `refine` re-forks the same pause, so a second, finer case axis derived
//! from the coarse rows flies the identical onset state, with the prior rounds carried.

use crate::CfdScalar;
use crate::types::flow::CompressiblePause;
use crate::types::flow::Report;
use crate::types::flow::coupling::PhysicsStage;
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow_config::CompressibleMarchConfig;
use deep_causality_par::MaybeParallel;
use deep_causality_physics::PhysicsError;

use super::*;

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
