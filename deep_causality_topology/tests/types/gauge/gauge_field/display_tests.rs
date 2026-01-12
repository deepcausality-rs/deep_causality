/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{GaugeField, GaugeGroup};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct TestGroup;

impl GaugeGroup for TestGroup {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "TestGroup"
    }
}

#[test]
fn test_gauge_field_display() {
    use deep_causality_topology::{Manifold, Simplex, SimplicialComplexBuilder};

    let metric = Metric::Minkowski(4);

    // Construct a 1-simplex (Edge [0, 1])
    // Vertices: 2 (0, 1)
    // Simplices: 3 ([0], [1], [0,1])

    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let _num_points = 2; // Vertices usually determine 'num_points' for gauge field

    // If base.len() = 3
    let num_elements = 3;
    let conn_data = vec![0.0; num_elements * 4];
    let conn = CausalTensor::new(conn_data, vec![num_elements, 4, 1]).unwrap();

    let fs_data = vec![0.0; num_elements * 16];
    let fs = CausalTensor::new(fs_data, vec![num_elements, 4, 4, 1]).unwrap();

    // Manifold data: 3 elements
    let mf_data = vec![0.0; 3];
    let mf_tensor = CausalTensor::new(mf_data, vec![3]).unwrap();

    // Try to create manifold.
    // If 1-simplex isn't a valid manifold, we'll get error again.
    // But it should be.
    let base = Manifold::<f64, f64>::new(complex, mf_tensor, 0).unwrap();

    let gf = GaugeField::<TestGroup, f64, f64>::new(base, metric, conn, fs).unwrap();

    let output = format!("{}", gf);
    assert!(output.contains("GaugeField<TestGroup>"));
    assert!(output.contains("metric=Minkowski(4)"));
    assert!(output.contains("conn="));
    assert!(output.contains("field_strength="));
}
