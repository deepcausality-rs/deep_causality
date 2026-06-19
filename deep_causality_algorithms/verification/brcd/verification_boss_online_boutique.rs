/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Supplied-CPDAG vs BOSS-learned comparison — Online Boutique.
//!
//! Runs each committed Online Boutique case twice: once with the supplied
//! **service-map** CPDAG (the reversed call graph), once with `cpdag = None` so
//! **BOSS learns** the structure, then prints the side-by-side table.
//!
//! ## Heads-up: this can be intractable on the large cases
//!
//! The supplied service-map CPDAG is **fully directed**, so BRCD does no
//! cut-configuration enumeration (`mec_size = 1`) and runs instantly. A BOSS
//! **learned** CPDAG, however, contains **undirected** edges, and BRCD enumerates
//! `2^(undirected edges incident on the candidate)` cut configurations — the cost
//! is **exponential in the local undirected degree** (the paper's own complexity
//! result, Appendix E). On Online Boutique's larger CPU cases (~50 variables,
//! ~2100 rows) BOSS learns a CPDAG with a high-undirected-degree hub, so the
//! learned-CPDAG run does **not** complete in reasonable time.
//!
//! This is itself a reason a curated service map (fully directed) is preferred
//! over a learned CPDAG. For a comparison that completes, use the Sock Shop
//! example, which is what the README table is built from:
//!
//! ```text
//! cargo run --release -p deep_causality_algorithms --example verification_boss_sockshop
//! ```
//!
//! Run (may not finish on the large cases):
//! `cargo run --release -p deep_causality_algorithms --example verification_boss_online_boutique`

mod common;

use common::{Report, compare_dataset};
use std::path::PathBuf;

fn main() {
    let dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("verification/brcd/data/online-boutique");
    if !dir.exists() {
        eprintln!(
            "=== verification_boss_online_boutique — FAILED (no data at {}) ===",
            dir.display()
        );
        std::process::exit(1);
    }
    eprintln!(
        "NOTE: the learned-CPDAG run is exponential in the local undirected degree \
         and may not complete on the large Online Boutique cases. See the module docs; \
         the README table is built from the Sock Shop comparison, which completes."
    );
    let mut report = Report::new("BOSS vs supplied CPDAG — Online Boutique");
    compare_dataset(&mut report, &dir, true, 1);
    report.finish();
}
