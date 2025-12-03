/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::{Simplex, Skeleton};
use std::vec;

#[test]
fn test_skeleton_new() {
    let simplices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let skeleton = Skeleton::new(0, simplices.clone());
    assert_eq!(skeleton.dim(), 0);
    assert_eq!(skeleton.simplices(), &simplices);
}

#[test]
fn test_skeleton_getters() {
    let s0 = Simplex::new(vec![0]);
    let s1 = Simplex::new(vec![1]);
    let simplices = vec![s0.clone(), s1.clone()];
    let skeleton = Skeleton::new(0, simplices.clone());

    assert_eq!(skeleton.dim(), 0);
    assert_eq!(skeleton.simplices(), &simplices);
}

#[test]
fn test_skeleton_get_index() {
    let s0 = Simplex::new(vec![0]);
    let s1 = Simplex::new(vec![1]);
    let s2 = Simplex::new(vec![2]);
    let simplices = vec![s0.clone(), s1.clone(), s2.clone()];
    let skeleton = Skeleton::new(0, simplices);

    assert_eq!(skeleton.get_index(&s0), Some(0));
    assert_eq!(skeleton.get_index(&s1), Some(1));
    assert_eq!(skeleton.get_index(&s2), Some(2));

    let s_non_existent = Simplex::new(vec![3]);
    assert_eq!(skeleton.get_index(&s_non_existent), None);

    let s_different_dim = Simplex::new(vec![0, 1]);
    assert_eq!(skeleton.get_index(&s_different_dim), None);
}

#[test]
fn test_skeleton_display() {
    let s0 = Simplex::new(vec![0]);
    let s1 = Simplex::new(vec![1]);
    let simplices = vec![s0, s1];
    let skeleton = Skeleton::new(0, simplices);

    assert_eq!(
        format!("{}", skeleton),
        "Skeleton(Dim: 0, Num Simplices: 2)"
    );
}

#[test]
fn test_skeleton_empty() {
    let skeleton = Skeleton::new(0, vec![]);
    assert_eq!(skeleton.dim(), 0);
    assert!(skeleton.simplices().is_empty());
    assert_eq!(skeleton.get_index(&Simplex::new(vec![0])), None);
}
