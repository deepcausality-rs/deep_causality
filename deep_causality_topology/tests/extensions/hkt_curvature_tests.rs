/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::RiemannMap;
use deep_causality_metric::Metric;
use deep_causality_topology::{
    CurvatureSymmetry, CurvatureTensor, CurvatureTensorWitness, TensorVector,
};

#[test]
fn test_geodesic_deviation_flat() {
    let flat: CurvatureTensor<
        f64,
        TensorVector<f64>,
        TensorVector<f64>,
        TensorVector<f64>,
        TensorVector<f64>,
    > = CurvatureTensor::flat(4);

    let u = TensorVector::<f64>::basis(4, 0);
    let v = TensorVector::<f64>::basis(4, 1);
    let w = TensorVector::<f64>::new(&[1.0, 2.0, 3.0, 4.0]);

    // Fully qualified path required due to HKT trait complexity
    let deviation: TensorVector<f64> = <CurvatureTensorWitness<f64> as RiemannMap<
        CurvatureTensorWitness<f64>,
    >>::curvature(flat, u, v, w);

    // Flat spacetime has zero geodesic deviation
    assert!(deviation.data.iter().all(|&x: &f64| x.abs() < f64::EPSILON));
}

#[test]
fn test_tensor_vector_operations() {
    let v = TensorVector::<f64>::new(&[1.0, 2.0, 3.0]);
    assert_eq!(v.dim(), 3);

    let basis = TensorVector::<f64>::basis(4, 2);
    assert_eq!(basis.data[2], 1.0);
    assert_eq!(basis.data[0], 0.0);
}

#[test]
fn test_curved_tensor_contraction() {
    // Create a simple non-flat curvature tensor
    let tensor: CurvatureTensor<
        f64,
        TensorVector<f64>,
        TensorVector<f64>,
        TensorVector<f64>,
        TensorVector<f64>,
    > = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::None,
        |d, a, b, c| {
            if d == 0 && a == 0 && b == 1 && c == 0 {
                1.0 // R^0_010 = 1
            } else {
                0.0
            }
        },
    );

    let u = TensorVector::<f64>::new(&[1.0, 0.0]);
    let v = TensorVector::<f64>::new(&[0.0, 1.0]);
    let w = TensorVector::<f64>::new(&[1.0, 0.0]);

    let result: TensorVector<f64> = <CurvatureTensorWitness<f64> as RiemannMap<
        CurvatureTensorWitness<f64>,
    >>::curvature(tensor, u, v, w);

    // R(u,v)w with R^0_010 = 1 should give [1, 0]
    // u=0 -> a=0
    // v=1 -> b=1
    // w=0 -> c=0
    // Sum R^d_010 * 1 * 1 * 1
    // d=0 -> 1.0
    // d=1 -> 0.0

    // Using explicit tolerance check
    assert!((result.data[0] - 1.0).abs() < f64::EPSILON);
    assert!(result.data[1].abs() < f64::EPSILON);
}

#[test]
fn test_scatter_vectors() {
    // Test basic S-matrix placeholder logic
    // Interaction tensor with 1.0 everywhere
    let tensor: CurvatureTensor<
        f64,
        TensorVector<f64>,
        TensorVector<f64>,
        TensorVector<f64>,
        TensorVector<f64>,
    > = CurvatureTensor::from_generator(
        2,
        Metric::Euclidean(2),
        CurvatureSymmetry::None,
        |_, _, _, _| 1.0,
    );

    let in1 = TensorVector::<f64>::new(&[1.0, 0.0]);
    let in2 = TensorVector::<f64>::new(&[1.0, 0.0]);

    let (out1, out2): (TensorVector<f64>, TensorVector<f64>) =
        <CurvatureTensorWitness<f64> as RiemannMap<CurvatureTensorWitness<f64>>>::scatter(
            tensor, in1, in2,
        );

    // If all components 1.0:
    // amplitude = 1.0 * 1.0 * 1.0 = 1.0 (since only a=0,b=0 nonzero)
    // out1[c] += 1.0 * 0.5 * (dim=2 for d) = 1.0
    // out2[d] += 1.0 * 0.5 * (dim=2 for c) = 1.0

    assert!((out1.data[0] - 1.0).abs() < 1e-6);
    assert!((out2.data[0] - 1.0).abs() < 1e-6);
}
