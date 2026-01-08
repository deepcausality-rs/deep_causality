/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{PointCloud, PointCloudWitness};

#[test]
fn test_point_cloud_functor() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    // Map: x -> x * 10
    // We use PointCloudWitness::<f64> because coordinates are f64
    let mapped_pc = PointCloudWitness::<f64>::fmap(pc, |x| x * 10.0);

    assert_eq!(mapped_pc.metadata().as_slice(), &[10.0, 20.0]);
    assert_eq!(mapped_pc.len(), 2);
}

#[test]
fn test_point_cloud_extract() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![10.0, 20.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 1).unwrap(); // Cursor at 1

    let val = PointCloudWitness::<f64>::extract(&pc);
    assert_eq!(val, 20.0);
}

#[test]
fn test_point_cloud_extend() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    // Extend: sum of metadata + cursor index
    let extended_pc = PointCloudWitness::<f64>::extend(&pc, |w: &PointCloud<f64, f64>| {
        let val = PointCloudWitness::<f64>::extract(w);
        val + (w.cursor() as f64)
    });

    // Index 0: val 1 + 0 = 1
    // Index 1: val 2 + 1 = 3
    // Index 2: val 3 + 2 = 5
    assert_eq!(extended_pc.metadata().as_slice(), &[1.0, 3.0, 5.0]);
}
