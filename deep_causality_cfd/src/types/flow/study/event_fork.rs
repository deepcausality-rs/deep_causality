/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The event-fork counterfactual path: `fork` a paused march, `branch` each case onto its own
//! world, and `continue_for` a bounded number of further coupled steps.

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
