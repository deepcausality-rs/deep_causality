/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::Metric;

#[test]
fn test_metric_dimension() {
    assert_eq!(Metric::Euclidean(3).dimension(), 3);
    assert_eq!(Metric::NonEuclidean(2).dimension(), 2);
    assert_eq!(Metric::Minkowski(4).dimension(), 4);
    assert_eq!(Metric::PGA(3).dimension(), 3);
    assert_eq!(Metric::Generic { p: 1, q: 1, r: 1 }.dimension(), 3);
    assert_eq!(
        Metric::Custom {
            dim: 5,
            neg_mask: 0,
            zero_mask: 0
        }
        .dimension(),
        5
    );
}

#[test]
fn test_metric_sign_of_sq() {
    // Euclidean
    let m = Metric::Euclidean(3);
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(2), 1);

    // AntiEuclidean
    let m = Metric::NonEuclidean(3);
    assert_eq!(m.sign_of_sq(0), -1);
    assert_eq!(m.sign_of_sq(2), -1);

    // Minkowski (e0^2 = 1, others -1)
    let m = Metric::Minkowski(4);
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), -1);
    assert_eq!(m.sign_of_sq(3), -1);

    // PGA (e0^2 = 0, others 1)
    let m = Metric::PGA(3);
    assert_eq!(m.sign_of_sq(0), 0);
    assert_eq!(m.sign_of_sq(1), 1);
    assert_eq!(m.sign_of_sq(2), 1);

    // Generic (p=1, q=1, r=1) -> (+, -, 0)
    let m = Metric::Generic { p: 1, q: 1, r: 1 };
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), -1);
    assert_eq!(m.sign_of_sq(2), 0);

    // Custom
    // dim=3, neg_mask=2 (binary 010 -> e1 is neg), zero_mask=4 (binary 100 -> e2 is zero)
    // e0 should be default positive
    let m = Metric::Custom {
        dim: 3,
        neg_mask: 2,
        zero_mask: 4,
    };
    assert_eq!(m.sign_of_sq(0), 1);
    assert_eq!(m.sign_of_sq(1), -1);
    assert_eq!(m.sign_of_sq(2), 0);
}

#[test]
fn test_metric_tensor_product() {
    let m1 = Metric::Euclidean(2);
    let m2 = Metric::Euclidean(3);
    assert_eq!(m1.tensor_product(&m2), Metric::Euclidean(5));

    let m3 = Metric::NonEuclidean(2);
    let m4 = Metric::NonEuclidean(2);
    assert_eq!(m3.tensor_product(&m4), Metric::NonEuclidean(4));

    // Mixed -> Generic metric with accurate signature
    assert_eq!(m1.tensor_product(&m3), Metric::Generic { p: 2, q: 2, r: 0 });
}
#[test]
fn test_metric_display() {
    assert_eq!(format!("{}", Metric::Euclidean(3)), "Euclidean(3)");
    assert_eq!(format!("{}", Metric::NonEuclidean(2)), "NonEuclidean(2)");
    assert_eq!(format!("{}", Metric::Minkowski(4)), "Minkowski(4)");
    assert_eq!(format!("{}", Metric::PGA(3)), "PGA(3)");
    assert_eq!(
        format!("{}", Metric::Generic { p: 1, q: 2, r: 3 }),
        "Generic(1, 2, 3)"
    );
    assert_eq!(
        format!(
            "{}",
            Metric::Custom {
                dim: 5,
                neg_mask: 0,
                zero_mask: 0
            }
        ),
        "Custom(5)"
    );
}
