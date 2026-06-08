/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{BrcdConfig, CdlBuilder, CdlConfigBuilder, CdlDiscoveryOutcome};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_chain(intercept: f64, seed: u64) -> NamedTempFile {
    let mut state = seed | 1;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        ((state >> 11) as f64 / (1u64 << 53) as f64) * 2.0 - 1.0
    };
    let mut csv = String::from("x,y,z\n");
    for _ in 0..120 {
        let x = next();
        let y = intercept + 1.5 * x + next();
        let z = 2.0 * y + next();
        csv.push_str(&format!("{:.6},{:.6},{:.6}\n", x, y, z));
    }
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(csv.as_bytes()).unwrap();
    f
}

#[test]
fn test_brcd_analyze_converges_to_brcd_outcome() {
    let normal = write_chain(0.0, 0x1234_5678);
    let anomalous = write_chain(4.0, 0x9abc_def0);

    // No CPDAG: BOSS learns one. Exercises the analyze step on the converged state.
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .build()
        .expect("files exist");

    let effect = CdlBuilder::build_brcd(&config)
        .brcd_load_input()
        .brcd_discover()
        .brcd_analyze();

    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    // BRCD has no feature-selection step.
    assert!(cdl.state.feature_selection.is_none());
    assert!(matches!(cdl.state.outcome, CdlDiscoveryOutcome::Brcd(_)));
    // The analysis produced at least one rendered line.
    assert!(!cdl.state.analysis.0.is_empty());
}
