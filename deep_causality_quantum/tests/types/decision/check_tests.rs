/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The decision form. The shape is `CommutatorCheck`'s, generalised over the item, and the two
//! obligations it carries are that a boolean is derivable but never the channel, and that a
//! report of nothing examined is visible as one.

use deep_causality_quantum::{Check, CheckItem, CheckReport, CheckVerdict, Factorization};

fn check(item: CheckItem, measured: f64, threshold: f64) -> Check<f64> {
    Check::new(item, measured, threshold)
}

#[test]
fn test_a_passing_decision_reports_its_distance_from_the_edge() {
    // Eleven items, the largest margin 0.87: the report says "passed, thirteen percent from the
    // edge", which is what `true` cannot say.
    let checks: Vec<Check<f64>> = (0..11)
        .map(|i| {
            let m = if i == 4 { 0.87 } else { 0.05 * i as f64 };
            check(CheckItem::Index(i), m, 1.0)
        })
        .collect();
    let report = CheckReport::from_checks(checks);
    assert_eq!(report.examined(), 11);
    assert_eq!(report.worst_margin(), Some(0.87));
    assert_eq!(report.worst().unwrap().item, CheckItem::Index(4));
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    assert!(report.accepted());
    assert!(report.first_rejection().is_none());
}

#[test]
fn test_the_zero_threshold_convention_follows_the_shipped_check() {
    // With a zero threshold the margin is the measured quantity itself when positive, zero when
    // both are zero, and a positive measurement rejects. This is `quantum_markov_check`'s branch.
    let exact = check(CheckItem::Whole, 0.0, 0.0);
    assert_eq!(exact.margin, 0.0);
    assert!(exact.accepted);

    let off = check(CheckItem::Whole, 0.3, 0.0);
    assert_eq!(off.margin, 0.3);
    assert!(!off.accepted);

    // And with a positive threshold the margin is the ratio, accepting at or below one.
    let edge = check(CheckItem::Whole, 2.0, 2.0);
    assert_eq!(edge.margin, 1.0);
    assert!(edge.accepted);
    let over = check(CheckItem::Whole, 3.0, 2.0);
    assert_eq!(over.margin, 1.5);
    assert!(!over.accepted);
}

#[test]
fn test_a_report_that_examined_nothing_is_vacuous_not_accepted() {
    let report = CheckReport::<f64>::vacuous();
    assert_eq!(report.examined(), 0);
    assert!(report.is_vacuous());
    assert_eq!(report.worst_margin(), None);
    // The derivable boolean says "nothing failed", which is true and not enough.
    assert!(report.accepted());
    // The verdict says what happened.
    assert_eq!(report.verdict(), CheckVerdict::Vacuous);
}

#[test]
fn test_a_rejection_carries_the_record_a_pass_carries() {
    let report = CheckReport::from_checks(vec![
        check(CheckItem::Pair(0, 1), 0.5, 1.0),
        check(CheckItem::Pair(0, 2), 4.0, 1.0),
    ]);
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    assert!(!report.accepted());
    let rejecting = report.first_rejection().expect("one pair rejected");
    assert_eq!(rejecting.item, CheckItem::Pair(0, 2));
    assert_eq!(rejecting.measured, 4.0);
    assert_eq!(rejecting.threshold, 1.0);
    assert_eq!(rejecting.margin, 4.0);
    // Examined counts up to and including the rejecting item.
    assert_eq!(report.examined(), 2);
    assert_eq!(report.worst_margin(), Some(4.0));
}

#[test]
fn test_examined_can_exceed_the_record_count() {
    // A trace-preservation check compares every entry of a d × d residual and records one defect.
    let report = CheckReport::new(vec![check(CheckItem::Whole, 1e-15, 1e-12)], 16);
    assert_eq!(report.examined(), 16);
    assert_eq!(report.checks().len(), 1);
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
}

#[test]
fn test_vacuity_survives_a_fold() {
    // Folding a vacuous report into one that examined items sums the counts and draws no margin
    // from the vacuous member, because it contributed no record.
    let examined = CheckReport::from_checks(vec![
        check(CheckItem::Index(0), 0.2, 1.0),
        check(CheckItem::Index(1), 0.6, 1.0),
        check(CheckItem::Index(2), 0.4, 1.0),
    ]);
    let folded = CheckReport::vacuous().fold(examined.clone());
    assert_eq!(folded.examined(), 3);
    assert_eq!(folded.worst_margin(), Some(0.6));
    assert_eq!(folded.verdict(), CheckVerdict::Accepted);

    let both_vacuous = CheckReport::<f64>::vacuous().fold(CheckReport::vacuous());
    assert_eq!(both_vacuous.verdict(), CheckVerdict::Vacuous);

    // And a rejection anywhere rejects the whole.
    let rejected = examined.fold(CheckReport::from_checks(vec![check(
        CheckItem::Index(9),
        5.0,
        1.0,
    )]));
    assert_eq!(rejected.examined(), 4);
    assert_eq!(rejected.verdict(), CheckVerdict::Rejected);
}

#[test]
fn test_factorization_defaults_to_rederived_and_an_inherited_member_taints_a_fold() {
    let own = CheckReport::from_checks(vec![check(CheckItem::Pair(0, 1), 0.1, 1.0)]);
    assert_eq!(own.factorization(), Factorization::Rederived);
    let inherited = CheckReport::from_checks(vec![check(CheckItem::Pair(1, 2), 0.1, 1.0)])
        .with_factorization(Factorization::Inherited);
    assert_eq!(inherited.factorization(), Factorization::Inherited);
    assert_eq!(
        own.clone().fold(inherited.clone()).factorization(),
        Factorization::Inherited
    );
    assert_eq!(
        inherited.fold(own).factorization(),
        Factorization::Inherited
    );
}

#[test]
fn test_the_worst_record_is_the_one_closest_to_rejecting() {
    // Margins may be negative, as they are for a comfortably positive spectrum measured as −λ;
    // the worst is still the largest.
    let report = CheckReport::from_checks(vec![
        check(CheckItem::Index(0), -0.9, 1.0),
        check(CheckItem::Index(1), -0.1, 1.0),
        check(CheckItem::Index(2), -0.5, 1.0),
    ]);
    assert_eq!(report.worst().unwrap().item, CheckItem::Index(1));
    assert_eq!(report.worst_margin(), Some(-0.1));
}

// `at_least`: the margin agrees with the verdict, slack included, and nothing measured orders worst.

#[test]
fn test_at_least_margin_agrees_with_the_verdict_under_slack() {
    // Below the floor on its own, admitted by the slack: accepted, and the margin says so.
    let c = Check::<f64>::at_least(CheckItem::Whole, 4.9, 5.0, 0.2);
    assert!(c.accepted);
    assert!(c.margin < 1.0);
    assert!((c.margin - 5.0 / 5.1).abs() < 1e-12);
    // Short even with the slack: rejected, margin above one.
    let c = Check::<f64>::at_least(CheckItem::Whole, 4.0, 5.0, 0.2);
    assert!(!c.accepted);
    assert!(c.margin > 1.0);
    // A zero threshold accepts with a zero margin, as `new` does.
    let c = Check::<f64>::at_least(CheckItem::Whole, 0.0, 0.0, 0.0);
    assert!(c.accepted);
    assert_eq!(c.margin, 0.0);
}

#[test]
fn test_at_least_orders_a_non_positive_effective_measurement_as_a_rejection() {
    let rejected = Check::<f64>::at_least(CheckItem::Whole, -2.0, 1.0, 0.5);
    assert!(!rejected.accepted);
    assert!(rejected.margin.is_infinite() && rejected.margin.is_sign_positive());
}

fn nothing_measured_orders_worst<R>()
where
    R: deep_causality_algebra::RealField + deep_causality_num::FromPrimitive + core::fmt::Debug,
{
    let lift = |x: f64| R::from_f64(x).expect("representable");
    let zero = Check::<R>::at_least(CheckItem::Pair(0, 1), lift(0.0), lift(5.0), lift(0.0));
    let tiny = Check::<R>::at_least(CheckItem::Pair(0, 2), lift(1e-9), lift(5.0), lift(0.0));
    assert!(zero.margin.is_infinite(), "{:?}", zero.margin);
    assert!(!zero.accepted && !tiny.accepted);
    let report = CheckReport::from_checks(vec![tiny, zero]);
    assert_eq!(
        report.worst().expect("two records").item,
        CheckItem::Pair(0, 1)
    );
}

#[test]
fn test_nothing_measured_orders_worst_at_every_scalar() {
    nothing_measured_orders_worst::<f32>();
    nothing_measured_orders_worst::<f64>();
    nothing_measured_orders_worst::<deep_causality_num::Float106>();
}
