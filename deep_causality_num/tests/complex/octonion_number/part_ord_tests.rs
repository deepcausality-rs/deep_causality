/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::cmp::Ordering;
use deep_causality_num::Octonion;

#[test]
fn test_partial_ord_equal() {
    let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o2 = o1;
    assert_eq!(o1.partial_cmp(&o2), Some(Ordering::Equal));
}

#[test]
fn test_partial_ord_less() {
    let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o2 = Octonion::new(2.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0); // o2.s > o1.s
    assert_eq!(o1.partial_cmp(&o2), Some(Ordering::Less));

    let o3 = Octonion::new(1.0, 3.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0); // o3.e1 > o1.e1
    assert_eq!(o1.partial_cmp(&o3), Some(Ordering::Less));
}

#[test]
fn test_partial_ord_greater() {
    let o1 = Octonion::new(2.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0); // o1.s > o2.s
    let o2 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    assert_eq!(o1.partial_cmp(&o2), Some(Ordering::Greater));

    let o3 = Octonion::new(1.0, 4.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0); // o3.e1 > o1.e1
    let o4 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    assert_eq!(o3.partial_cmp(&o4), Some(Ordering::Greater));
}

#[test]
fn test_partial_ord_nan() {
    let o1 = Octonion::new(1.0, f64::NAN, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o2 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    assert_eq!(o1.partial_cmp(&o2), None);

    let o3 = Octonion::new(f64::NAN, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    assert_eq!(o3.partial_cmp(&o2), None);
}
