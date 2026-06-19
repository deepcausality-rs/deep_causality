/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Real-world verification — Sock Shop.
//!
//! Replays the committed Sock Shop cases (the already-preprocessed `df_obs` and
//! `df_a`, the CPDAG, and the ranking the authoritative Python BRCD produced) and
//! checks that the Rust port reproduces the same top root cause for each case.
//! See `examples/verification/README.md` for how the data was captured.
//!
//! Run: `cargo run -p deep_causality_algorithms --example verification_sockshop`

mod common;

use common::{Report, verify_dataset};
use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("verification/brcd/data/sock-shop-2");

    if !dir.exists() {
        eprintln!(
            "=== verification_sockshop — FAILED (no data at {}) ===",
            dir.display()
        );
        eprintln!(
            "The committed reference dataset is missing. See examples/verification/README.md."
        );
        std::process::exit(1);
    }

    let mut report = Report::new("Sock Shop");
    // Reference config: node_transform=none, transform_parents=True, k=1.
    verify_dataset(&mut report, &dir, true, 1);
    report.finish();
}
