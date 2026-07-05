/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The acceptance-gate builder: the `[PASS]`/`[FAIL]` block every self-verifying program
//! prints, as one type.
//!
//! `Gates` accumulates labeled checks and prints them on [`finish`](Gates::finish), exactly
//! the lines the existing gate blocks print and nothing more: no process exit (the caller
//! keeps that), no number formatting (details arrive pre-formatted, so precision display
//! stays at the caller's boundary), no timing, no color.

/// A titled set of acceptance gates. Build with [`new`](Self::new), add checks with
/// [`gate`](Self::gate), print and read the verdict with [`finish`](Self::finish).
#[derive(Debug)]
pub struct Gates {
    title: String,
    entries: Vec<(String, bool, String)>,
}

impl Gates {
    /// A new, empty gate set under `title`.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            entries: Vec::new(),
        }
    }

    /// Add one gate: a label, whether it passed, and the pre-formatted detail line. Consumes and
    /// returns the builder, so the result must be kept — a bare `.gate(..);` would drop the entry.
    #[must_use]
    pub fn gate(mut self, label: impl Into<String>, pass: bool, detail: impl Into<String>) -> Self {
        self.entries.push((label.into(), pass, detail.into()));
        self
    }

    /// Print the gate lines and the verdict; return true when every gate passed. An empty
    /// gate set passes vacuously and says so in its verdict line. The returned pass/fail is the
    /// block's primary outcome (the caller maps it to an exit code), so it must not be discarded.
    #[must_use]
    pub fn finish(self) -> bool {
        println!("--- {} ---", self.title);
        let mut all = true;
        for (label, pass, detail) in &self.entries {
            println!(
                "  [{}] {label}: {detail}",
                if *pass { "PASS" } else { "FAIL" }
            );
            all &= *pass;
        }
        if self.entries.is_empty() {
            println!("=== {}: no gates registered. ===", self.title);
        } else if all {
            println!("=== All gates passed: {}. ===", self.title);
        } else {
            println!(
                "=== Gate REGRESSION in {}: see the FAIL lines above. ===",
                self.title
            );
        }
        all
    }
}
