/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! End-to-end BRCD pipeline: supplied-CPDAG and BOSS-fallback paths produce a
//! report, and a dimension mismatch surfaces as a pipeline error.

use deep_causality_discovery::{BrcdConfig, CdlBuilder, CdlConfigBuilder, CdlError};
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

fn write_csv(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

fn write_cpdag() -> NamedTempFile {
    write_csv("# vertices=3\nsrc,dst,mark_src,mark_dst\n0,1,Tail,Tail\n1,2,Tail,Tail\n")
}

#[test]
fn test_brcd_full_pipeline_supplied_cpdag() {
    let normal = write_chain(0.0, 0x1234_5678);
    let anomalous = write_chain(4.0, 0x9abc_def0);
    let cpdag = write_cpdag();

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_path(cpdag.path().to_str().unwrap())
        .build()
        .expect("files exist");

    let report_effect = CdlBuilder::build_brcd(&config)
        .brcd_load_input()
        .brcd_discover()
        .brcd_analyze()
        .finalize();

    assert!(report_effect.inner.is_ok());
    let report = report_effect.inner.unwrap();
    assert!(report.feature_selection.is_none());
    let rendered = format!("{}", report);
    assert!(rendered.contains("ROOT-CAUSE DISCOVERY (BRCD)"));
    assert!(!rendered.contains("FEATURE SELECTION"));
}

#[test]
fn test_brcd_full_pipeline_boss_fallback() {
    let normal = write_chain(0.0, 0x1234_5678);
    let anomalous = write_chain(4.0, 0x9abc_def0);

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .build()
        .expect("files exist");

    let report_effect = CdlBuilder::build_brcd(&config)
        .brcd_load_input()
        .brcd_discover()
        .brcd_analyze()
        .finalize();

    assert!(report_effect.inner.is_ok());
}

#[test]
fn test_brcd_dimension_mismatch_is_error() {
    // normal has 3 columns, anomalous has 2 → load fails inside the pipeline.
    let normal = write_csv("x,y,z\n1.0,2.0,3.0\n4.0,5.0,6.0\n");
    let anomalous = write_csv("x,y\n1.0,2.0\n4.0,5.0\n");

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .build()
        .expect("files exist");

    let effect = CdlBuilder::build_brcd(&config).brcd_load_input();
    assert!(matches!(effect.inner, Err(CdlError::BrcdLoadError(_))));
}
