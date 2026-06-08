/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # BRCD without a CPDAG (BOSS-learned, real Sock Shop data)
//!
//! Same real-world Sock Shop case (`carts_cpu_1`) as the supplied-CPDAG example,
//! but no CPDAG is provided. The loader leaves the CPDAG `None`, and the BRCD
//! driver learns the structure from the normal (observational) window via BOSS
//! before ranking. Data is loaded relative to the crate root via
//! `CARGO_MANIFEST_DIR`.
//!
//! Note: the in-repo BOSS uses a corrected score sign, so the learned structure
//! (and hence the ranking) need not reproduce the supplied-CPDAG result — this
//! example demonstrates the BOSS path on real telemetry, not exact agreement.

use deep_causality_discovery::*;

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. The entire CDL pipeline runs at this precision.
pub type FloatType = deep_causality_num::Float106;

fn main() {
    // Real Sock Shop case, loaded relative to the crate root. No CPDAG file: the
    // structure is learned via BOSS inside brcd_run.
    let base = concat!(env!("CARGO_MANIFEST_DIR"), "/data/sock-shop-2/carts_cpu_1");
    let normal_path = format!("{base}/normal.csv");
    let anomalous_path = format!("{base}/anomalous.csv");

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(&normal_path)
        .with_anomalous_path(&anomalous_path)
        .with_brcd_config(BrcdConfig::<FloatType>::continuous(0))
        .build()
        .expect("Sock Shop carts_cpu_1 data files exist");

    CdlBuilder::build_brcd(&config)
        .brcd_load_input()
        .brcd_discover()
        .brcd_analyze()
        .finalize()
        .print_results();
}
