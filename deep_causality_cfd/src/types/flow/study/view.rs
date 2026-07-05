/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`StudyView`]: the borrowed view a gate reads — the reduced rows, every prior refinement
//! round, the scheduled case count, and the study title. A gate check is a plain function of a
//! view, so gates are free functions in `model.rs` that never own the study's data.

/// A read-only view of a study at judgment time.
pub struct StudyView<'s, Row> {
    rows: &'s [Row],
    rounds: &'s [Vec<Row>],
    cases_len: usize,
    title: &'s str,
}

impl<'s, Row> StudyView<'s, Row> {
    pub(crate) fn new(
        rows: &'s [Row],
        rounds: &'s [Vec<Row>],
        cases_len: usize,
        title: &'s str,
    ) -> Self {
        Self {
            rows,
            rounds,
            cases_len,
            title,
        }
    }

    /// The current (final-round) reduced rows.
    pub fn rows(&self) -> &[Row] {
        self.rows
    }

    /// Every prior refinement round's rows, oldest first (empty for a single-round study).
    pub fn rounds(&self) -> &[Vec<Row>] {
        self.rounds
    }

    /// The number of cases scheduled for the current round.
    pub fn cases_len(&self) -> usize {
        self.cases_len
    }

    /// The study title.
    pub fn title(&self) -> &str {
        self.title
    }
}
