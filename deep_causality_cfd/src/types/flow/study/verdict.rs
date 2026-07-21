/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`Verdict`]: the value a study resolves to. It is data, not a process effect — the DSL never
//! exits or prints; the caller maps `passed()` to an exit code and prints the `Display` rendering.
//!
//! A verdict carries the gate outcomes (label, pass flag, detail) and the study's accumulated
//! non-fatal warnings. [`merge`](Verdict::merge) composes two verdicts into one, so a mixed
//! program (a campaign verdict plus a trajectory leg verdict) still ends in a single report.

use crate::types::EvidenceClass;
use crate::types::flow::study_warning::StudyWarning;
use core::fmt;

/// One gate's outcome within a verdict.
#[derive(Debug, Clone)]
pub struct GateOutcome {
    label: String,
    passed: bool,
    detail: String,
    evidence: EvidenceClass,
}

impl GateOutcome {
    pub(crate) fn new(
        label: impl Into<String>,
        passed: bool,
        detail: impl Into<String>,
        evidence: EvidenceClass,
    ) -> Self {
        Self {
            label: label.into(),
            passed,
            detail: detail.into(),
            evidence,
        }
    }

    /// The gate's label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Whether the gate passed.
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// The gate's detail line.
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Where this gate's bound came from — whether clearing it is evidence about the physics or
    /// only evidence of non-regression.
    pub fn evidence(&self) -> EvidenceClass {
        self.evidence
    }
}

/// The resolved outcome of a study: the gate outcomes and the accumulated non-fatal warnings.
#[derive(Debug, Clone)]
pub struct Verdict {
    title: String,
    outcomes: Vec<GateOutcome>,
    warnings: Vec<StudyWarning>,
}

impl Verdict {
    /// Build a verdict from a titled set of gate outcomes, with no warnings yet.
    pub(crate) fn new(title: impl Into<String>, outcomes: Vec<GateOutcome>) -> Self {
        Self {
            title: title.into(),
            outcomes,
            warnings: Vec::new(),
        }
    }

    /// Attach the study's accumulated warnings (called once at `verdict()`).
    pub(crate) fn with_warnings(mut self, warnings: Vec<StudyWarning>) -> Self {
        self.warnings = warnings;
        self
    }

    /// The study title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Every gate outcome, in order.
    pub fn outcomes(&self) -> &[GateOutcome] {
        &self.outcomes
    }

    /// The accumulated non-fatal diagnostics; never part of pass/fail.
    pub fn warnings(&self) -> &[StudyWarning] {
        &self.warnings
    }

    /// Whether every gate passed. An empty gate set passes vacuously.
    pub fn passed(&self) -> bool {
        self.outcomes.iter().all(GateOutcome::passed)
    }

    /// Compose two verdicts into one: a mixed program's campaign and trajectory verdicts combine
    /// their outcomes and warnings under a joined title.
    pub fn merge(mut self, other: Verdict) -> Verdict {
        self.title = format!("{} + {}", self.title, other.title);
        self.outcomes.extend(other.outcomes);
        self.warnings.extend(other.warnings);
        self
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--- {} ---", self.title)?;
        for o in &self.outcomes {
            let tag = if o.passed { "PASS" } else { "FAIL" };
            writeln!(f, "  [{tag}] [{}] {}: {}", o.evidence, o.label, o.detail)?;
        }
        for w in &self.warnings {
            writeln!(f, "  [WARN] {}", w.message())?;
        }
        if self.passed() {
            write!(f, "=== All gates passed: {}. ===", self.title)
        } else {
            write!(
                f,
                "=== Gate REGRESSION in {}: see the FAIL lines. ===",
                self.title
            )
        }
    }
}
