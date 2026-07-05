/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The study warning channel: non-fatal diagnostics a campaign accumulates as it runs (a
//! `force_load` snapshot override, a clamped refinement candidate, a solver fallback). Warnings
//! never gate — they ride the [`StudyEffect`](crate::types::flow::StudyEffect) alongside the
//! value and render in the final [`Verdict`](crate::Verdict); gates gate.
//!
//! Mirrors the Causal Discovery Language's warning log so the two grammars share one shape.

use deep_causality_haft::{LogAddEntry, LogAppend, LogEffect, LogSize};

/// One non-fatal study diagnostic, classified by where it arose.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StudyWarning {
    /// A data-loading diagnostic (a `force_load` snapshot override, a tolerated gap).
    Data(String),
    /// A case-execution diagnostic (a clamped candidate, a solver fallback).
    Case(String),
    /// Any other non-fatal note.
    Generic(String),
}

impl StudyWarning {
    /// The diagnostic message text.
    pub fn message(&self) -> &str {
        match self {
            StudyWarning::Data(s) | StudyWarning::Case(s) | StudyWarning::Generic(s) => s,
        }
    }
}

impl From<&str> for StudyWarning {
    fn from(s: &str) -> Self {
        StudyWarning::Generic(s.to_string())
    }
}

/// The accumulated study warnings, in the order they were recorded.
#[derive(Debug, Clone, Default)]
pub struct StudyWarningLog {
    entries: Vec<StudyWarning>,
}

impl StudyWarningLog {
    /// The recorded warnings, in order.
    pub fn entries(&self) -> &[StudyWarning] {
        &self.entries
    }

    /// Record one classified warning.
    pub fn push(&mut self, warning: StudyWarning) {
        self.entries.push(warning);
    }
}

impl LogAddEntry for StudyWarningLog {
    fn add_entry(&mut self, message: &str) {
        self.entries.push(StudyWarning::from(message));
    }
}

impl LogAppend for StudyWarningLog {
    fn append(&mut self, other: &mut Self) {
        self.entries.append(&mut other.entries);
    }
}

impl LogSize for StudyWarningLog {
    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl LogEffect for StudyWarningLog {}
