/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Supplied-CPDAG vs BOSS-learned comparison — Sock Shop.
//!
//! Runs each committed Sock Shop case twice: once with the supplied
//! **service-map** CPDAG (the reversed call graph), once with `cpdag = None` so
//! **BOSS learns** the structure from the pre-failure data. It prints a
//! side-by-side table — supplied vs learned top-5, how deep the two rankings
//! agree, their Spearman correlation, and the true-fault rank under each.
//!
//! See `examples/verification/README.md` ("Why the rankings agree at the top and
//! diverge below") for what the CPDAG is and why the divergence appears past the
//! matching leading positions.
//!
//! Run: `cargo run -p deep_causality_algorithms --example verification_boss_sockshop`

mod common;

use common::{Report, compare_dataset};
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/verification/brcd/data/sock-shop-2");
    if !dir.exists() {
        eprintln!(
            "=== verification_boss_sockshop — FAILED (no data at {}) ===",
            dir.display()
        );
        std::process::exit(1);
    }
    let mut report = Report::new("BOSS vs supplied CPDAG — Sock Shop");
    compare_dataset(&mut report, &dir, true, 1);
    report.finish();
}
