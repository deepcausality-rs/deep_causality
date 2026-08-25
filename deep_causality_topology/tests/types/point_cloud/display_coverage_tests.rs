/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

// When the `points` tensor is 1-dimensional, `shape().get(1)` is `None`, so the
// Display impl falls back to `unwrap_or(&0)` for the "Point Dimensions" line.
#[test]
fn test_point_cloud_display_one_dimensional_shape_falls_back_to_zero() {
    // Shape [2]: one axis only -> get(1) is None.
    let points = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let display_str = format!("{}", pc);

    assert!(display_str.contains("PointCloud:"));
    assert!(display_str.contains("Point Dimensions: 0"));
}

// ============================================================================
// Writer-error propagation
//
// Each `writeln!` in the Display impl ends in `?`. A sink that fails partway
// through must make the whole format call fail.
// ============================================================================

use core::fmt::{self, Write};

/// Accepts `budget` writes and reports an error on the next one.
struct FailAfter {
    budget: usize,
}

impl Write for FailAfter {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        if self.budget == 0 {
            return Err(fmt::Error);
        }
        self.budget -= 1;
        Ok(())
    }
}

#[test]
fn test_point_cloud_display_propagates_every_writer_failure() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let mut first_ok = None;
    for budget in 0..4096 {
        let mut sink = FailAfter { budget };
        if fmt::write(&mut sink, format_args!("{}", pc)).is_ok() {
            first_ok = Some(budget);
            break;
        }
    }
    let first_ok = first_ok.expect("a large enough budget formats the whole value");
    assert!(first_ok > 0, "the value needs at least one write");

    for budget in 0..first_ok {
        let mut sink = FailAfter { budget };
        assert!(
            fmt::write(&mut sink, format_args!("{}", pc)).is_err(),
            "a sink that fails after {budget} writes must yield Err"
        );
    }
}
