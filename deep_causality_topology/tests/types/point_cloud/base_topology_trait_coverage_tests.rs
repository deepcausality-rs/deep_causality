/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, PointCloud};

// `PointCloud::len` / `PointCloud::is_empty` inherent methods shadow the
// `BaseTopology` trait methods, so a plain `pc.len()` call never exercises the
// trait body. These reach the trait `len` and `is_empty` via fully-qualified
// syntax.
#[test]
fn test_point_cloud_base_topology_trait_qualified_non_empty() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    assert_eq!(<PointCloud<f64, f64> as BaseTopology>::dimension(&pc), 0);
    assert_eq!(<PointCloud<f64, f64> as BaseTopology>::len(&pc), 3);
    assert!(!<PointCloud<f64, f64> as BaseTopology>::is_empty(&pc));
    assert_eq!(
        <PointCloud<f64, f64> as BaseTopology>::num_elements_at_grade(&pc, 0),
        Some(3)
    );
    assert_eq!(
        <PointCloud<f64, f64> as BaseTopology>::num_elements_at_grade(&pc, 1),
        None
    );
}

// NOTE: an *empty* `PointCloud` cannot be constructed through the public API —
// `PointCloud::new` rejects an empty `points` tensor with
// `InvalidInput("PointCloud `points` cannot be empty or have invalid shape")`
// (see constructors/constructors_impl.rs). The trait `len` / `is_empty` bodies
// are pure delegations to the inherent methods with no emptiness-dependent
// branching, so the non-empty test above already exercises every line of the
// trait impl. The empty-case path is therefore unreachable and intentionally
// not tested here.
