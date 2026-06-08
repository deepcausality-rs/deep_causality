/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{BrcdConfig, CdlBuilder, CdlConfigBuilder};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_csv(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn test_state_accessor_exposes_configured_state() {
    let normal = write_csv("x,y\n1.0,2.0\n3.0,4.0\n");
    let anomalous = write_csv("x,y\n5.0,6.0\n7.0,8.0\n");
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .build()
        .expect("files exist");

    let effect = CdlBuilder::build_brcd(&config);
    let cdl = effect.inner.unwrap();
    // The `state()` accessor returns the carried configured state.
    assert_eq!(
        cdl.state().config.normal_path(),
        normal.path().to_str().unwrap()
    );
}
