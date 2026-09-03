/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! SensorTraceSet accessors and gap representation.

use deep_causality_file::read_sensor_trace;
use deep_causality_haft::IoAction;
use std::fs;

#[test]
fn channel_lookup_and_alignment() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("trace.csv");
    fs::write(&path, "t,a,b\n0.0,1.0,\n1.0,,2.0\n").expect("write");
    let trace = read_sensor_trace::<f64>(&path).run().expect("parses");
    assert_eq!(trace.timestamps().len(), 2);
    for c in trace.channels() {
        assert_eq!(c.samples().len(), trace.timestamps().len(), "aligned");
    }
    assert!(trace.channel("a").is_some());
    assert!(trace.channel("z").is_none());
}
