/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The checks a verification makes: each a claim of the paper, and whether QCL reproduces
//! it.

/// The checks made so far.
#[derive(Default)]
pub struct Report {
    checks: Vec<(String, bool)>,
}

impl Report {
    /// Records `claim`, prints it as `PASS` or `FAIL` with `detail`, the numbers behind it.
    pub fn check(&mut self, claim: &str, pass: bool, detail: String) {
        println!(
            "  {} {claim}\n       {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        self.checks.push((claim.to_string(), pass));
    }

    /// Prints the tally for `target`, and returns an error naming the failed claims, if any.
    pub fn finish(self, target: &str) -> Result<(), String> {
        let failed: Vec<&str> = self
            .checks
            .iter()
            .filter(|(_, pass)| !pass)
            .map(|(claim, _)| claim.as_str())
            .collect();
        println!(
            "{target}: {} of {} checks pass",
            self.checks.len() - failed.len(),
            self.checks.len()
        );
        if failed.is_empty() {
            Ok(())
        } else {
            Err(format!("{target} failed: {}", failed.join("; ")))
        }
    }
}
