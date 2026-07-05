/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`GateSeq`]: a named, ordered gating sequence built once as a value and inserted whole into a
//! study. A complex workflow declares sequence A and sequence B as two named values (in
//! `model.rs`) and places each at its stage; the sequence is the reviewable definition of what
//! the study must satisfy.
//!
//! Each gate is a plain `fn` pointer (static dispatch, no boxing), higher-ranked in the view
//! lifetime (feasibility note F1) so one sequence checks a view of any lifetime. Sequences are
//! row-typed, so a sequence built for one study's rows cannot be inserted into a study whose rows
//! it does not understand — that mismatch is a compile error.

use crate::types::flow::study::verdict::{GateOutcome, Verdict};
use crate::types::flow::study::view::StudyView;

/// A gate check: reads a study view of any lifetime, returns `(passed, detail)`.
pub type GateFn<Row> = for<'a> fn(&StudyView<'a, Row>) -> (bool, String);

/// A named, ordered sequence of gate checks over a study's `Row` type.
pub struct GateSeq<Row> {
    title: String,
    gates: Vec<(&'static str, GateFn<Row>)>,
}

impl<Row> GateSeq<Row> {
    /// Open a named gating sequence.
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            gates: Vec::new(),
        }
    }

    /// Append a labeled gate check. The check is a free function `fn(&StudyView<Row>) ->
    /// (bool, String)`.
    pub fn gate(mut self, label: &'static str, check: GateFn<Row>) -> Self {
        self.gates.push((label, check));
        self
    }

    /// The sequence title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Run every gate against `view`, collecting the outcomes into a [`Verdict`] (without
    /// warnings; the campaign attaches those at `verdict()`).
    pub fn check(&self, view: &StudyView<'_, Row>) -> Verdict {
        let outcomes = self
            .gates
            .iter()
            .map(|(label, check)| {
                let (passed, detail) = check(view);
                GateOutcome::new(*label, passed, detail)
            })
            .collect();
        Verdict::new(self.title.clone(), outcomes)
    }
}
