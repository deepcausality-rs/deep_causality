/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

#[test]
fn test_outer_product() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e1 ^ e2 = e12
    let res = e1.outer_product(&e2);
    assert_eq!(res.data()[3], 1.0);

    // e1 ^ e1 = 0
    let res2 = e1.outer_product(&e1);
    assert!(res2.data().iter().all(|&x| x == 0.0));
}

#[test]
fn test_inner_product() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e1 . e2 = 0
    let res = e1.inner_product(&e2);
    assert!(res.data().iter().all(|&x| x == 0.0));

    // e1 . e1 = 1
    let res2 = e1.inner_product(&e1);
    assert_eq!(res2.data()[0], 1.0);
}

#[test]
fn test_geometric_product_euclidean() {
    // Euclidean 2D: e1*e1 = 1, e2*e2 = 1, e1*e2 = e12, e2*e1 = -e12
    let m = Metric::Euclidean(2);

    // e1 (index 1) * e2 (index 2) -> e12 (index 3)
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    let e1e2 = e1.clone() * e2.clone();
    assert_eq!(e1e2.data()[3], 1.0); // e12 component

    // e2 * e1 -> -e12
    let e2e1 = e2 * e1.clone();
    assert_eq!(e2e1.data()[3], -1.0);

    // e1 * e1 -> 1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data()[0], 1.0);
}

#[test]
fn test_geometric_product_minkowski() {
    // Minkowski 2D: e0^2 = 1, e1^2 = -1
    // e0 is index 1, e1 is index 2
    let m = Metric::Minkowski(2);

    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e1 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e0 * e0 = 1
    let e0sq = e0.clone() * e0.clone();
    assert_eq!(e0sq.data()[0], 1.0);

    // e1 * e1 = -1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data()[0], -1.0);
}

#[test]
fn test_geometric_product_pga() {
    // PGA 2D: e0^2 = 0, e1^2 = 1
    let m = Metric::PGA(2);

    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e1 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // e0 * e0 = 0
    let e0sq = e0.clone() * e0;
    assert_eq!(e0sq.data()[0], 0.0);

    // e1 * e1 = 1
    let e1sq = e1.clone() * e1;
    assert_eq!(e1sq.data()[0], 1.0);
}

#[test]
fn test_geometric_product_sparse_high_dim() {
    // Use a dimension > SPARSE_THRESHOLD (e.g., 7)
    let dim = 7;
    let size = 1 << dim; // 128 elements
    let m = Metric::Euclidean(dim);

    // Create sparse vectors: A = e1, B = e2
    let mut data_a = vec![0.0; size];
    data_a[1] = 1.0; // e1
    let a = CausalMultiVector::new(data_a, m).unwrap();

    let mut data_b = vec![0.0; size];
    data_b[2] = 1.0; // e2
    let b = CausalMultiVector::new(data_b, m).unwrap();

    // e1 * e2 = e12 (index 3)
    let product = a * b;

    // The result should be sparse, with only e12 (index 3) having a coefficient of 1.0
    assert_eq!(product.data()[3], 1.0);
    // All other coefficients should be zero
    for (idx, &val) in product.data().iter().enumerate() {
        if idx != 3 {
            assert_eq!(val, 0.0);
        }
    }
}

#[test]
fn test_commutator_lie_euclidean() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();
    let e12 = CausalMultiVector::new(vec![0.0, 0.0, 0.0, 1.0], m).unwrap();

    // [e1, e2] = e1*e2 - e2*e1 = e12 - (-e12) = 2*e12
    let res = e1.commutator_lie(&e2);
    assert_eq!(res.data()[3], 2.0);
    assert_eq!(res.data()[0], 0.0);
    assert_eq!(res.data()[1], 0.0);
    assert_eq!(res.data()[2], 0.0);

    // [e1, e1] = 0
    let res_self = e1.commutator_lie(&e1);
    assert!(res_self.data().iter().all(|&x| x == 0.0));

    // [e12, e1] = e12*e1 - e1*e12 = (-e2) - (e2) = -2e2
    // e12*e1 = (e1^e2)*e1 = (e1e2)e1 = e1(e2e1) = e1(-e1e2) = -(e1e1)e2 = -e2
    // e1*e12 = e1(e1e2) = (e1e1)e2 = e2
    let res_e12_e1 = e12.commutator_lie(&e1);
    assert_eq!(res_e12_e1.data()[2], -2.0);
}

#[test]
fn test_commutator_lie_minkowski() {
    // Minkowski 2D: e0^2 = 1, e1^2 = -1
    let m = Metric::Minkowski(2);
    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap(); // e0 corresponds to index 1
    let e1_b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap(); // e1 corresponds to index 2
    let e01 = CausalMultiVector::new(vec![0.0, 0.0, 0.0, 1.0], m).unwrap(); // e01 corresponds to index 3

    // [e0, e1] = e0*e1 - e1*e0 = e01 - (-e01) = 2*e01
    let res = e0.commutator_lie(&e1_b);
    assert_eq!(res.data()[3], 2.0);
    assert!(
        res.data()
            .iter()
            .enumerate()
            .all(|(idx, &x)| if idx != 3 { x == 0.0 } else { true })
    );

    // [e0, e0] = 0
    let res_self = e0.commutator_lie(&e0);
    assert!(res_self.data().iter().all(|&x| x == 0.0));

    // [e01, e0] = e01*e0 - e0*e01 = (-e1) - (e1) = -2e1
    // e01*e0 = (e0^e1)*e0 = (e0e1)e0 = e0(-e0e1) = -(e0e0)e1 = -e1
    // e0*e01 = e0(e0e1) = (e0e0)e1 = e1
    let res_e01_e0 = e01.commutator_lie(&e0);
    assert_eq!(res_e01_e0.data()[2], -2.0); // e1 is at index 2
}

#[test]
fn test_commutator_geometric_euclidean() {
    let m = Metric::Euclidean(2);
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // {e1, e2} = 0.5 * (e1*e2 - e2*e1) = 0.5 * (e12 - (-e12)) = 0.5 * (2*e12) = e12
    let res = e1.commutator_geometric(&e2);
    assert_eq!(res.data()[3], 1.0);
    assert_eq!(res.data()[0], 0.0);
    assert_eq!(res.data()[1], 0.0);
    assert_eq!(res.data()[2], 0.0);

    // {e1, e1} = 0
    let res_self = e1.commutator_geometric(&e1);
    assert!(res_self.data().iter().all(|&x| x == 0.0));
}

#[test]
fn test_commutator_geometric_minkowski() {
    let m = Metric::Minkowski(2);
    let e0 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let e1_b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], m).unwrap();

    // {e0, e1} = 0.5 * (e0*e1 - e1*e0) = 0.5 * (e01 - (-e01)) = 0.5 * (2*e01) = e01
    let res = e0.commutator_geometric(&e1_b);
    assert_eq!(res.data()[3], 1.0);
    assert!(
        res.data()
            .iter()
            .enumerate()
            .all(|(idx, &x)| if idx != 3 { x == 0.0 } else { true })
    );

    // {e0, e0} = 0
    let res_self = e0.commutator_geometric(&e0);
    assert!(res_self.data().iter().all(|&x| x == 0.0));
}
