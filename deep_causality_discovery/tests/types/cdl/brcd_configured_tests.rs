/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{BrcdConfig, CdlBuilder, CdlConfigBuilder};
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

fn write_cpdag() -> NamedTempFile {
    let s = "# deep_causality MixedGraph v1; vertices=3\nsrc,dst,mark_src,mark_dst\n0,1,Tail,Tail\n1,2,Tail,Tail\n";
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(s.as_bytes()).unwrap();
    f
}

#[test]
fn test_brcd_load_input_with_cpdag() {
    let normal = write_chain(0.0, 0x1234);
    let anomalous = write_chain(4.0, 0x9abc);
    let cpdag = write_cpdag();

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .with_cpdag_path(cpdag.path().to_str().unwrap())
        .build()
        .expect("files exist");

    let effect = CdlBuilder::build_brcd(&config).brcd_load_input();
    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    assert_eq!(cdl.state.input.normal().shape(), &[120, 3]);
    assert_eq!(cdl.state.input.anomalous().shape(), &[120, 3]);
    assert!(cdl.state.input.cpdag().is_some());
}

#[test]
fn test_brcd_load_input_without_cpdag_is_none() {
    let normal = write_chain(0.0, 0x1234);
    let anomalous = write_chain(4.0, 0x9abc);

    let config = CdlConfigBuilder::build_brcd_config()
        .with_normal_path(normal.path().to_str().unwrap())
        .with_anomalous_path(anomalous.path().to_str().unwrap())
        .with_brcd_config(BrcdConfig::<f64>::continuous(7))
        .build()
        .expect("files exist");

    let effect = CdlBuilder::build_brcd(&config).brcd_load_input();
    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    assert!(cdl.state.input.cpdag().is_none());
}
