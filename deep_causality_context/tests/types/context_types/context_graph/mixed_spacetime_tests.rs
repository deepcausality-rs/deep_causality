/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Contexts whose spacetime varies, and contexts that have none.
//!
//! The type parameters are spelled out rather than hidden behind a bundle. A reader of these
//! signatures can see which space, time and spacetime the context holds without opening another
//! type to find out.

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, EuclideanSpacetime,
    EuclideanTime, FloatType, LorentzianSpacetime, MetricSignature, NoSpaceTime, RelationKind,
    Root, SpaceKind, SpaceTimeKind, TangentSpacetime, TimeKind, TimeScale,
};
use deep_causality_metric::Metric;

/// A context over the variant enums: any spatial, temporal or spacetime variant may be stored.
type VariantContext =
    Context<Data<FloatType>, SpaceKind<FloatType>, TimeKind<FloatType>, SpaceTimeKind<FloatType>>;

/// A context with a clock and no spatial extent. `NoSpaceTime` is zero-sized, so naming the
/// absence costs nothing, and it beats naming a spatial type the graph never holds.
type ClockContext = Context<
    Data<FloatType>,
    NoSpaceTime<FloatType>,
    EuclideanTime<FloatType>,
    NoSpaceTime<FloatType>,
>;

fn metric_at(context: &VariantContext, index: usize) -> Metric {
    match context.get_node(index).unwrap().vertex_type() {
        ContextoidType::SpaceTempoid(st) => st.metric(),
        _ => panic!("expected a spacetime node at index {index}"),
    }
}

// =============================================================================
// A regime change inside one context
// =============================================================================

#[test]
fn test_one_context_holds_two_spacetime_variants() {
    // What a regime-changing model needs: a probe reasoning one way far from a mass and another
    // way close to it is one context whose spacetime varies, not two contexts.
    //
    // Far from the mass a coordinate spacetime carries a flat metric; close in it is numerically
    // evolved, so the tangent type carries a tensor per point. Both report the same signature,
    // because the manifold is Lorentzian throughout.
    let mut context: VariantContext = Context::with_capacity(1, "two regimes", 4);

    let far = context
        .add_node(Contextoid::new(
            1,
            ContextoidType::SpaceTempoid(SpaceTimeKind::Lorentzian(LorentzianSpacetime::new(
                1,
                0.0,
                0.0,
                0.0,
                0.0,
                TimeScale::Second,
            ))),
        ))
        .expect("failed to add the coordinate node");

    let near = context
        .add_node(Contextoid::new(
            2,
            ContextoidType::SpaceTempoid(SpaceTimeKind::Tangent(TangentSpacetime::new(
                2, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0,
            ))),
        ))
        .expect("failed to add the tangent node");

    context
        .add_edge(far, near, RelationKind::SpaceTemporal)
        .expect("failed to join the two regimes");

    assert_eq!(context.number_of_nodes(), 2);
    assert_eq!(
        context.get_edge(far, near),
        Some(&RelationKind::SpaceTemporal)
    );

    // Each is readable back as its own variant, so the enum carries the distinction rather than
    // flattening it.
    match context.get_node(far).unwrap().vertex_type() {
        ContextoidType::SpaceTempoid(SpaceTimeKind::Lorentzian(_)) => {}
        other => panic!("expected a coordinate spacetime, found {other}"),
    }
    match context.get_node(near).unwrap().vertex_type() {
        ContextoidType::SpaceTempoid(SpaceTimeKind::Tangent(_)) => {}
        other => panic!("expected a tangent spacetime, found {other}"),
    }

    // Here the two agree, which is what makes the regimes one manifold rather than two.
    assert_eq!(metric_at(&context, far), Metric::Lorentzian(4));
    assert_eq!(metric_at(&context, near), Metric::Lorentzian(4));
}

#[test]
fn test_one_context_reports_two_different_signatures() {
    // The case with no answer if a context declared one signature of its own: a Newtonian node
    // and a relativistic node held together, each reporting its own.
    let mut context: VariantContext = Context::with_capacity(1, "mixed signatures", 4);

    let newtonian = context
        .add_node(Contextoid::new(
            1,
            ContextoidType::SpaceTempoid(SpaceTimeKind::Euclidean(EuclideanSpacetime::new(
                1,
                0.0,
                0.0,
                0.0,
                0.0,
                TimeScale::Second,
            ))),
        ))
        .expect("failed to add the Newtonian node");

    let relativistic = context
        .add_node(Contextoid::new(
            2,
            ContextoidType::SpaceTempoid(SpaceTimeKind::Lorentzian(LorentzianSpacetime::new(
                2,
                0.0,
                0.0,
                0.0,
                0.0,
                TimeScale::Second,
            ))),
        ))
        .expect("failed to add the relativistic node");

    assert_eq!(metric_at(&context, newtonian), Metric::Euclidean(4));
    assert_eq!(metric_at(&context, relativistic), Metric::Lorentzian(4));
    assert_ne!(
        metric_at(&context, newtonian),
        metric_at(&context, relativistic)
    );
}

// =============================================================================
// A context with no spatial extent
// =============================================================================

#[test]
fn test_a_clock_context_accepts_root_data_and_time() {
    let mut context: ClockContext = Context::with_capacity(1, "clock only", 4);

    let root = context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("failed to add the root");
    let datum = context
        .add_node(Contextoid::new(
            2,
            ContextoidType::Datoid(Data::new(2, 42.0)),
        ))
        .expect("failed to add the datum");
    let clock = context
        .add_node(Contextoid::new(
            3,
            ContextoidType::Tempoid(EuclideanTime::new(3, TimeScale::Second, 1.0)),
        ))
        .expect("failed to add the clock");

    context
        .add_edge(root, datum, RelationKind::Datial)
        .expect("failed to join root and datum");
    context
        .add_edge(root, clock, RelationKind::Temporal)
        .expect("failed to join root and clock");

    assert_eq!(context.number_of_nodes(), 3);
    assert_eq!(context.number_of_edges(), 2);
}

#[test]
fn test_naming_the_absence_of_space_costs_nothing() {
    // Both empty slots are zero-sized, so a context that holds no spatial node pays nothing for
    // saying so in its own type.
    assert_eq!(size_of::<NoSpaceTime<FloatType>>(), 0);
}
