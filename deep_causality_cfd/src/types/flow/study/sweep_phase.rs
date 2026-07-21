/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The pointwise and swept paths: the case sweep and its rig-sharing sibling, and the verb that
//! marches every configured case.

use crate::CfdScalar;
use crate::traits::Marchable;
use crate::types::flow::Report;
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow::sweep::sweep;
use deep_causality_par::MaybeParallel;
use deep_causality_physics::PhysicsError;

use super::*;

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
