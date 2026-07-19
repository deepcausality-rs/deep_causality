/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reduction and judgement: the verbs that turn marched reports into rows, record and gate them,
//! and close the study with its terminal verdict.

use crate::CfdScalar;
use crate::types::flow::Report;
use crate::types::flow::study_effect::StudyEffect;
use crate::types::flow::study_error::StudyError;
use crate::types::flow::sweep::sweep;
use deep_causality_file::TableRow;
use deep_causality_par::MaybeParallel;
use deep_causality_physics::PhysicsError;
use std::path::Path;

use super::*;

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
