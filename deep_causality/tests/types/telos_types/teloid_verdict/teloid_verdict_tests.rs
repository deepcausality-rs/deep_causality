/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{TeloidID, TeloidModal, Verdict};

#[test]
fn test_verdict_new() {
    let outcome = TeloidModal::Obligatory;
    let justification = vec![1, 2, 3];
    let verdict = Verdict::new(outcome, justification.clone());

    assert_eq!(verdict.outcome(), outcome);
    assert_eq!(verdict.justification(), &justification);
}

#[test]
fn test_verdict_new_empty_justification() {
    let outcome = TeloidModal::Impermissible;
    let justification: Vec<TeloidID> = vec![];
    let verdict = Verdict::new(outcome, justification.clone());

    assert_eq!(verdict.outcome(), outcome);
    assert_eq!(verdict.justification(), &justification);
    assert!(verdict.justification().is_empty());
}

#[test]
fn test_verdict_clone() {
    let verdict = Verdict::new(TeloidModal::Optional(10), vec![100, 200]);
    let cloned_verdict = verdict.clone();

    assert_eq!(verdict, cloned_verdict);
    assert_eq!(verdict.outcome(), cloned_verdict.outcome());
    assert_eq!(verdict.justification(), cloned_verdict.justification());
}

#[test]
fn test_verdict_equality() {
    let v1 = Verdict::new(TeloidModal::Obligatory, vec![1, 2]);
    let v2 = Verdict::new(TeloidModal::Obligatory, vec![1, 2]);
    let v3 = Verdict::new(TeloidModal::Impermissible, vec![1, 2]);
    let v4 = Verdict::new(TeloidModal::Obligatory, vec![2, 1]); // Different order

    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
    assert_ne!(v1, v4); // Vec equality is order-sensitive
}

#[test]
fn test_verdict_display() {
    let v_obligatory = Verdict::new(TeloidModal::Obligatory, vec![1, 2]);
    assert_eq!(
        format!("{}", v_obligatory),
        "Verdict: Outcome = Obligatory, Justification = [1, 2]"
    );

    let v_impermissible = Verdict::new(TeloidModal::Impermissible, vec![3]);
    assert_eq!(
        format!("{}", v_impermissible),
        "Verdict: Outcome = Impermissible, Justification = [3]"
    );

    let v_optional = Verdict::new(TeloidModal::Optional(5), vec![4, 5, 6]);
    assert_eq!(
        format!("{}", v_optional),
        "Verdict: Outcome = Optional(5), Justification = [4, 5, 6]"
    );

    let v_empty = Verdict::new(TeloidModal::Obligatory, vec![]);
    assert_eq!(
        format!("{}", v_empty),
        "Verdict: Outcome = Obligatory, Justification = []"
    );
}

#[test]
fn test_verdict_debug() {
    let v = Verdict::new(TeloidModal::Obligatory, vec![1, 2]);
    assert_eq!(
        format!("{:?}", v),
        "Verdict { outcome: Obligatory, justification: [1, 2] }"
    );
}
