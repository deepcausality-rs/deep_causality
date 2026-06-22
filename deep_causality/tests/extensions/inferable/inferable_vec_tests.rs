/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

use deep_causality::utils_test::test_utils::*;

#[test]
fn test_add() {
    let mut col = get_test_inf_vec();
    assert_eq!(2, col.len());

    let f3 = get_test_inferable(3, true);
    col.push(f3);
    assert_eq!(3, col.len());
}

#[test]
fn test_all_inferable() {
    let f = get_test_inferable(3, false);
    let col: Vec<Inference> = Vec::from_iter([f]);
    assert!(col.all_inferable());
}

#[test]
fn test_all_inverse_inferable() {
    let f = get_test_inferable(3, true);
    let col: Vec<Inference> = Vec::from_iter([f]);
    assert!(col.all_inverse_inferable());
}

#[test]
fn test_all_non_inferable() {
    let col = get_test_inf_vec();
    assert!(!col.all_non_inferable());
}

/// A mock `Inferable` whose `is_inferable` and `is_inverse_inferable` both
/// return `true`. A real `Inference` can never be both (observation cannot be
/// simultaneously above and below the threshold), so this mock is the only way
/// to drive the "undecidable, hence non-inferable" `true` arm of
/// `all_non_inferable`.
#[derive(Debug)]
struct UndecidableInferable {
    id: u64,
}

impl Identifiable for UndecidableInferable {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Inferable for UndecidableInferable {
    fn question(&self) -> DescriptionValue {
        DescriptionValue::from("undecidable")
    }
    fn observation(&self) -> NumericalValue {
        1.0
    }
    fn threshold(&self) -> NumericalValue {
        0.5
    }
    fn effect(&self) -> NumericalValue {
        1.0
    }
    fn target(&self) -> NumericalValue {
        1.0
    }
    fn is_inferable(&self) -> bool {
        true
    }
    fn is_inverse_inferable(&self) -> bool {
        true
    }
}

#[test]
fn test_all_non_inferable_true_for_undecidable_item() {
    let col: Vec<UndecidableInferable> = vec![UndecidableInferable { id: 1 }];
    // The single item is both inferable and inverse-inferable, so
    // `all_non_inferable` short-circuits and returns true.
    assert!(col.all_non_inferable());
}

#[test]
fn test_conjoint_delta() {
    let col = get_test_inf_vec();
    // in the synthetic test data,
    // the conjoint delta is 0.0% because all causes explain the observed effects.
    assert_eq!(0.0, col.conjoint_delta());
}

#[test]
fn test_number_inferable() {
    let col = get_test_inf_vec();
    assert_eq!(1.0, col.number_inferable());
}

#[test]
fn test_number_inverse_inferable() {
    let col = get_test_inf_vec();
    assert_eq!(1.0, col.number_inverse_inferable());
}

#[test]
fn test_number_non_inferable() {
    let col = get_test_inf_vec();
    assert_eq!(0.0, col.number_non_inferable());
}

#[test]
fn test_percent_inferable() {
    let col = get_test_inf_vec();
    assert_eq!(50.0, col.percent_inferable())
}

#[test]
fn test_percent_inverse_inferable() {
    let col = get_test_inf_vec();
    assert_eq!(50.0, col.percent_inverse_inferable())
}

#[test]
fn test_percent_non_inferable() {
    let col = get_test_inf_vec();
    assert_eq!(0.0, col.percent_non_inferable())
}

#[test]
fn test_get_all_inferable() {
    let mut col = get_test_inf_vec();
    let f3 = get_test_inferable(3, false);
    col.push(f3);

    let all_inf = col.get_all_inferable();
    assert_eq!(2, all_inf.len());
}

#[test]
fn test_get_all_inverse_inferable() {
    let col = get_test_inf_vec();
    let all_inv_inf = col.get_all_inverse_inferable();
    assert_eq!(1, all_inv_inf.len());
}

#[test]
fn test_get_all_non_inferable() {
    let col = get_test_inf_vec();
    let all_non_inf = col.get_all_non_inferable();
    assert_eq!(0, all_non_inf.len());
}

#[test]
fn test_get_all_items() {
    let col = get_test_inf_vec();
    let all_items = col.get_all_items();

    let exp_len = col.len();
    let actual_len = all_items.len();
    assert_eq!(exp_len, actual_len);
}

#[test]
fn test_len() {
    let col = get_test_inf_vec();
    assert_eq!(2, col.len());
}

#[test]
fn test_is_empty() {
    let col = get_test_inf_vec();
    assert!(!InferableReasoning::is_empty(&col));
}

// --- Per-item Inferable default-method coverage ---

#[test]
fn test_item_conjoint_delta() {
    // Exercises the per-item `Inferable::conjoint_delta` default implementation.
    let inf = get_test_inferable(0, false);
    // conjoint_delta = abs(1.0 - observation)
    let expected = (1.0 - inf.observation()).abs();
    assert_eq!(inf.conjoint_delta(), expected);
}

#[test]
fn test_item_is_inferable_true() {
    let inf = get_test_inferable(0, false);
    assert!(inf.is_inferable());
    assert!(!inf.is_inverse_inferable());
}

#[test]
fn test_item_is_inverse_inferable_true() {
    let inf = get_test_inferable(0, true);
    assert!(inf.is_inverse_inferable());
    assert!(!inf.is_inferable());
}

// --- Early-return branches in all_inferable / all_inverse_inferable ---

#[test]
fn test_all_inferable_returns_false_on_mixed_collection() {
    // The first item is inverse-inferable (not inferable), so `all_inferable`
    // must short-circuit and return false on the first element.
    let col: Vec<Inference> =
        Vec::from_iter([get_test_inferable(0, true), get_test_inferable(1, false)]);
    assert!(!col.all_inferable());
}

#[test]
fn test_all_inverse_inferable_returns_false_on_mixed_collection() {
    // The first item is inferable (not inverse-inferable), so
    // `all_inverse_inferable` must short-circuit and return false.
    let col: Vec<Inference> =
        Vec::from_iter([get_test_inferable(0, false), get_test_inferable(1, true)]);
    assert!(!col.all_inverse_inferable());
}

// --- Empty-collection guard branches ---

#[test]
fn test_empty_collection_metrics_short_circuit() {
    let col: Vec<Inference> = Vec::new();

    // conjoint_delta on an empty collection returns the neutral 1.0.
    assert_eq!(col.conjoint_delta(), 1.0);

    // All percentage metrics return 0.0 on an empty collection.
    assert_eq!(col.percent_inferable(), 0.0);
    assert_eq!(col.percent_inverse_inferable(), 0.0);
    assert_eq!(col.percent_non_inferable(), 0.0);
}
