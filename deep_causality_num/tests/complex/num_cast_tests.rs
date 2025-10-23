/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, ComplexNumber, NumCast};

#[test]
fn test_num_cast_f64() {
    let c = <Complex<f64> as NumCast>::from(1.5).unwrap();
    assert_eq!(c.re(), 1.5);
    assert_eq!(c.im(), 0.0);
}
