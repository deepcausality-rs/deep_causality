// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use ultragraph::prelude::*;

#[test]
fn test_new() {
    let g = ultragraph::new::<u8>();
    assert!(g.is_empty());
}

#[test]
fn test_with_capacity() {
    let g = ultragraph::with_capacity::<u8>(100);
    assert!(g.is_empty());
}

#[test]
fn test_new_with_matrix_storage() {
    let g = ultragraph::new_with_matrix_storage::<u8>(100);
    assert!(g.is_empty());
}

#[test]
fn test_default() {
    let g = ultragraph::default::<u8>();
    assert!(g.is_empty());
}
