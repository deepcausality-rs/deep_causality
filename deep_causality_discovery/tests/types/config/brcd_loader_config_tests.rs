/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{BrcdConfig, CdlConfigBuilder};
use std::io::Write;
use tempfile::NamedTempFile;

fn temp() -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(b"x,y\n1.0,2.0\n").unwrap();
    f
}

#[test]
fn test_brcd_loader_config_getters_with_cpdag() {
    let n = temp();
    let a = temp();
    let c = temp();
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(n.path().to_str().unwrap())
        .with_anomalous_path(a.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_path(c.path().to_str().unwrap())
        .build()
        .expect("files exist");

    assert_eq!(config.normal_path(), n.path().to_str().unwrap());
    assert_eq!(config.anomalous_path(), a.path().to_str().unwrap());
    assert_eq!(config.cpdag_path().map(|s| s.as_str()), c.path().to_str());
    assert_eq!(config.brcd_config().num_root_causes, 1);
    assert!(format!("{}", config).contains("BrcdLoaderConfig"));
}

#[test]
fn test_brcd_loader_config_cpdag_none_by_default() {
    let n = temp();
    let a = temp();
    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(n.path().to_str().unwrap())
        .with_anomalous_path(a.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(0))
        .build()
        .expect("files exist");
    assert!(config.cpdag_path().is_none());
}
