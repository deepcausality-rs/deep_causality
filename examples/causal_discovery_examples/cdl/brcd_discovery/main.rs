/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # BRCD with a supplied CPDAG (real Sock Shop data)
//!
//! Runs the BRCD root-cause lineage on a committed real-world case from the RCAEval
//! Sock Shop benchmark (`carts_cpu_1`): 44 service metrics, a normal and an
//! anomalous window, and the supplied service-call CPDAG. This mirrors the
//! `verification_sockshop` reference run (`BrcdConfig::continuous`, `k = 1`), so the
//! CDL pipeline and the algorithm agree: the top-ranked root cause is
//! `shipping_latency` (column index 42), matching the case's `expected.txt`.
//!
//! Data is loaded relative to the crate root via `CARGO_MANIFEST_DIR`, so the
//! example runs from any working directory / machine.

use deep_causality_discovery::*;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. The entire CDL pipeline runs at this precision.
pub type FloatType = f64;

fn main() {
    // 1. Real Sock Shop case, loaded relative to the crate root.
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/data/sock-shop-2/carts_cpu_1");
    let normal_path = format!("{base}/normal.csv");
    let anomalous_path = format!("{base}/anomalous.csv");
    let cpdag_path = format!("{base}/cpdag.csv");

    // 2. Build the run config. The reference config is the continuous family with
    //    a single root cause (`BrcdConfig::continuous`), which is exactly the default.
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(&normal_path)
        .with_anomalous_path(&anomalous_path)
        .with_brcd_config(BrcdConfig::<FloatType>::continuous(0))
        .with_cpdag_path(&cpdag_path)
        .build()
        .expect("Sock Shop carts_cpu_1 data files exist");

    // 3. Run the BRCD lineage. Loading happens inside the pipeline. The top-ranked
    //    candidate should be column 42 (shipping_latency), per the case's expected.txt.
    CdlBuilder::build_brcd(&config)
        .brcd_load_input()
        .brcd_discover()
        .brcd_analyze()
        .finalize()
        .print_results();
}
