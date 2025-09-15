/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// surd_utils are private and thus cannot be tested in the test folder.
// While a lot gets tested through the public API, these tests cover some rare corner cases.

use crate::surd::surd_utils;

#[test]
fn test_diff_empty() {
    let data = Vec::<f64>::new();

    let diff = surd_utils::diff(data.as_slice());

    assert!(diff.is_empty());
}

#[test]
fn test_combinations_r_empty() {
    let data = Vec::<f64>::new();
    let r = 0;

    let result = surd_utils::combinations(data.as_slice(), r);

    assert_eq!(result.len(), 1);
    assert!(result[0].is_empty());
}

#[test]
#[should_panic]
fn test_combinations_r_exceeds_pool() {
    let data = Vec::<f64>::new();
    let r = 3;
    // Triggers panic: Cannot choose r elements from a pool smaller than r.
    surd_utils::combinations(data.as_slice(), r);
}
