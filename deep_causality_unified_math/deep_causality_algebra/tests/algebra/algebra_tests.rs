/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Algebra, Module, MulSemigroup};

/// Test that semigroup traits are implemented for the real field.
#[test]
fn test_semigroup_impls() {
    fn require_mul_semigroup<T: MulSemigroup>() {}

    require_mul_semigroup::<f64>();
}

/// Cover the Module::scale and Module::scale_mut blanket-impl helpers.
#[test]
fn test_module_scale_helpers() {
    let mut x: f64 = 3.0;
    let scaled = <f64 as Module<f64>>::scale(&x, 4.0);
    assert_eq!(scaled, 12.0);
    <f64 as Module<f64>>::scale_mut(&mut x, 2.0);
    assert_eq!(x, 6.0);
}

/// Cover the Algebra::sqr default helper.
#[test]
fn test_algebra_sqr() {
    let x: f64 = 5.0;
    assert_eq!(<f64 as Algebra<f64>>::sqr(&x), 25.0);
}
