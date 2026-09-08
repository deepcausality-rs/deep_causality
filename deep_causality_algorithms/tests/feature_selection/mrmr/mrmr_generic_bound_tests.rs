/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! That mRMR is reachable from a caller that knows only the algebra tower.
//!
//! The selector used to be bounded on `deep_causality_num::Float`, which is the numeric crate's
//! own float capability and not part of the algebra tower every other consumer is written
//! against. A caller generic over `RealField + FromPrimitive` could not reach it without adopting
//! `Float` too, which is how a low-level trait leaks upward through a workspace.
//!
//! These cases are as much a compile check as a behaviour one: the helpers below are generic and
//! name **no** `Float` bound, so the suite stops building if that bound ever returns. The
//! assertions then confirm the same call answers identically at two precisions, which is the
//! property the tower exists to provide.

use deep_causality_algebra::RealField;
use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_num::{Float106, FromPrimitive, lift};
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;

/// A caller written against the tower alone: `RealField + FromPrimitive`, plus the container
/// bounds the selector's own signature asks for. No `Float`.
fn select_generic<T>(
    raw: &[f64],
    rows: usize,
    cols: usize,
    want: usize,
    target: usize,
) -> Vec<usize>
where
    T: RealField + FromPrimitive + MaybeParallel,
{
    let data: Vec<T> = raw.iter().map(|&v| lift::<T>(v)).collect();
    let tensor = CausalTensor::new(data, vec![rows, cols]).expect("a well-formed matrix");
    let mut picked: Vec<usize> = mrmr_features_selector::<T, T>(&tensor, want, target)
        .expect("a full-rank design selects")
        .iter()
        .map(|(index, _)| *index)
        .collect();
    picked.sort_unstable();
    picked
}

/// The same call through `Option<T>`, which is the shape the discovery pipeline passes.
fn select_generic_optional<T>(
    raw: &[f64],
    rows: usize,
    cols: usize,
    want: usize,
    target: usize,
) -> Vec<usize>
where
    T: RealField + FromPrimitive + MaybeParallel,
{
    let data: Vec<Option<T>> = raw.iter().map(|&v| Some(lift::<T>(v))).collect();
    let tensor = CausalTensor::new(data, vec![rows, cols]).expect("a well-formed matrix");
    let mut picked: Vec<usize> = mrmr_features_selector::<Option<T>, T>(&tensor, want, target)
        .expect("a full-rank design selects")
        .iter()
        .map(|(index, _)| *index)
        .collect();
    picked.sort_unstable();
    picked
}

/// F0, F1, F2, target over four rows. F2 is an exact multiple of F0, so the two tie on relevance
/// and only the selected *set* is determined — the same reason the sibling suite sorts before
/// asserting.
const RAW: [f64; 16] = [
    1.0, 2.0, 3.0, 1.6, //
    2.0, 4.1, 6.0, 3.5, //
    3.0, 6.2, 9.0, 5.5, //
    4.0, 8.1, 12.0, 7.5,
];

#[test]
fn test_mrmr_is_reachable_from_a_tower_only_caller() {
    assert_eq!(select_generic::<f64>(&RAW, 4, 4, 2, 3), vec![0, 2]);
    assert_eq!(select_generic::<Float106>(&RAW, 4, 4, 2, 3), vec![0, 2]);
}

#[test]
fn test_mrmr_is_reachable_through_option_from_a_tower_only_caller() {
    assert_eq!(select_generic_optional::<f64>(&RAW, 4, 4, 2, 3), vec![0, 2]);
    assert_eq!(
        select_generic_optional::<Float106>(&RAW, 4, 4, 2, 3),
        vec![0, 2]
    );
}
