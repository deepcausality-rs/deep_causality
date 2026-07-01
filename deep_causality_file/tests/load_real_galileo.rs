/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Integration test: the IO-monad loaders parse the real Galileo E14 SP3/CLK products bundled with
//! the chronometric examples. Skips gracefully if the data fixtures are not present.

use deep_causality_file::{read_clock_data, read_gnss_single_satellite, read_orbit_data};
use deep_causality_haft::IoAction;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../examples/chronometric_examples/data/gnss")
}

#[test]
fn loads_e14_clock_and_orbit_from_real_data() {
    let clk = data_dir().join("gbm18770.clk");
    let sp3 = data_dir().join("gbm18770.sp3");
    if !clk.exists() || !sp3.exists() {
        eprintln!(
            "skipping: Galileo fixtures not present at {}",
            data_dir().display()
        );
        return;
    }

    // Lazy IO descriptions — nothing reads until `.run()`.
    let clocks = read_clock_data::<f64>(&clk, "E14").run().expect("clk load");
    let orbits = read_orbit_data::<f64>(&sp3, "E14").run().expect("sp3 load");

    assert!(!clocks.is_empty(), "E14 should have clock samples");
    assert!(!orbits.is_empty(), "E14 should have orbit samples");

    // Clock bias is a small real number (seconds), not the IGS invalid sentinel.
    for c in &clocks {
        assert!(c.bias_s().abs() < 900_000.0);
    }
    // Galileo orbital radius is ~29,600 km (MEO): sanity-bound the parsed ECEF positions.
    for o in &orbits {
        let r_km = o.radius_m() / 1000.0;
        assert!(
            (20_000.0..40_000.0).contains(&r_km),
            "E14 radius {r_km} km out of MEO band"
        );
    }

    // The composed loader (IO-monad and_then) yields the same pair in one run.
    let (c2, o2) = read_gnss_single_satellite::<f64>(&clk, &sp3, "E14")
        .run()
        .expect("composed load");
    assert_eq!(c2.len(), clocks.len());
    assert_eq!(o2.len(), orbits.len());
}
