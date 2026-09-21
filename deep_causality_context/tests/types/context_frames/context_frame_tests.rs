/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    BaseFrame, ClockFrame, Context, ContextFrame, Contextoid, ContextoidType, ContextuableGraph,
    Coordinate, Data, EuclideanSpacetime, EuclideanTime, FloatType, LorentzianSpacetime,
    MetricSignature, MetricTensor4D, NoSpaceTime, RelationKind, Root, SpaceTimeKind,
    TangentSpacetime, Temporal, TimeScale, UniformFrame,
};
use deep_causality_metric::{Metric, MetricFamily, detect_convention, is_lorentzian};

// =============================================================================
// The frame carries a family, and no signature
// =============================================================================

#[test]
fn test_metric_family_resolves_without_a_context() {
    // A family is a choice about the model rather than a property of a node — a node can say it
    // is Lorentzian, but not that it is Schwarzschild at a given mass — so it stays on the frame.
    assert_eq!(BaseFrame::metric_family(), Some(MetricFamily::Flat));
    assert_eq!(ClockFrame::metric_family(), Some(MetricFamily::Flat));
    assert_eq!(UniformFrame::metric_family(), None);
}

#[test]
fn test_the_signature_comes_from_the_node() {
    // Not from the frame. Two spacetime types, two signatures, neither declared by a context.
    let newtonian = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let relativistic = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    assert_eq!(newtonian.metric(), Metric::Euclidean(4));
    assert_eq!(relativistic.metric(), Metric::Lorentzian(4));
    assert_ne!(newtonian.metric(), relativistic.metric());
}

// =============================================================================
// A node signature composes with the metric crate's existing checks
// =============================================================================

#[test]
fn test_a_node_signature_feeds_the_existing_convention_checks() {
    // Those checks take a `Metric`, and a node now supplies one, so the two compose with nothing
    // new in between.
    let relativistic = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    assert_eq!(detect_convention(&relativistic.metric()), Some(true));
    assert!(is_lorentzian(&relativistic.metric()));
}

#[test]
fn test_a_newtonian_node_has_no_convention() {
    // A Euclidean signature is neither east nor west coast, and the check says so rather than
    // guessing one.
    let newtonian = EuclideanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);

    assert_eq!(detect_convention(&newtonian.metric()), None);
    assert!(!is_lorentzian(&newtonian.metric()));
}

#[test]
fn test_a_tangent_spacetime_keeps_its_signature_while_its_tensor_changes() {
    // A signature does not vary under continuous evolution. Replacing every component of the
    // stored tensor must not change the answer, which is why the node derives it from what it is
    // rather than reading it off the field.
    let mut evolved = TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    let before = evolved.metric();

    evolved.update_metric_tensor([
        [-4.0, 0.0, 0.0, 0.0],
        [0.0, 2.0, 0.0, 0.0],
        [0.0, 0.0, 2.0, 0.0],
        [0.0, 0.0, 0.0, 2.0],
    ]);

    assert_eq!(evolved.metric(), before);
    assert!(is_lorentzian(&evolved.metric()));
}

#[test]
fn test_an_empty_spacetime_reports_a_zero_dimensional_signature() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();

    assert_eq!(empty.metric(), Metric::Euclidean(0));
    assert_eq!(empty.dimension(), 0);
}

// =============================================================================
// A regime change inside one context
// =============================================================================

type UniformCtx = Context<
    Data<FloatType>,
    <UniformFrame as ContextFrame>::Space,
    <UniformFrame as ContextFrame>::Time,
    <UniformFrame as ContextFrame>::SpaceTime,
>;

fn metric_at(context: &UniformCtx, index: usize) -> Metric {
    match context.get_node(index).unwrap().vertex_type() {
        ContextoidType::SpaceTempoid(st) => st.metric(),
        _ => panic!("expected a spacetime node at index {index}"),
    }
}

#[test]
fn test_one_context_holds_two_spacetime_variants() {
    // What a regime-changing model needs: a probe reasoning one way far from a mass and another
    // way close to it is one context whose spacetime varies, not two contexts.
    //
    // Far from the mass the metric is flat and a coordinate spacetime carries it; close in it is
    // numerically evolved, so the tangent type carries a tensor per point. Both report the same
    // signature, because the manifold is Lorentzian throughout.
    let mut context: UniformCtx = Context::with_capacity(1, "two regimes", 4);

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

    // Both are readable back as their own variants, so the enum carries the distinction rather
    // than flattening it.
    match context.get_node(far).unwrap().vertex_type() {
        ContextoidType::SpaceTempoid(SpaceTimeKind::Lorentzian(_)) => {}
        other => panic!("expected a coordinate spacetime, found {other}"),
    }
    match context.get_node(near).unwrap().vertex_type() {
        ContextoidType::SpaceTempoid(SpaceTimeKind::Tangent(_)) => {}
        other => panic!("expected a tangent spacetime, found {other}"),
    }

    // Each node answers for itself. Here they agree, which is what makes the two regimes one
    // manifold rather than two.
    assert_eq!(metric_at(&context, far), Metric::Lorentzian(4));
    assert_eq!(metric_at(&context, near), Metric::Lorentzian(4));
}

#[test]
fn test_one_context_reports_two_different_signatures() {
    // The case that has no answer if a context declares one signature: a Newtonian node and a
    // relativistic node held together, each reporting its own.
    let mut context: UniformCtx = Context::with_capacity(1, "mixed signatures", 4);

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

#[test]
fn test_a_fixed_frame_names_one_spacetime_type() {
    // The trade a fixed frame makes: its SpaceTime is a concrete type, so a context on it holds
    // that type and no other, and the signature its nodes report never varies.
    fn spacetime_of<F: ContextFrame>() -> &'static str {
        core::any::type_name::<F::SpaceTime>()
    }

    assert!(spacetime_of::<BaseFrame>().contains("EuclideanSpacetime"));
    assert!(spacetime_of::<UniformFrame>().contains("SpaceTimeKind"));
}

// =============================================================================
// A frame with no spatial part
// =============================================================================

type ClockCtx = Context<
    Data<FloatType>,
    <ClockFrame as ContextFrame>::Space,
    <ClockFrame as ContextFrame>::Time,
    <ClockFrame as ContextFrame>::SpaceTime,
>;

#[test]
fn test_a_clock_context_accepts_root_data_and_time() {
    let mut context: ClockCtx = Context::with_capacity(1, "clock only", 4);

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
fn test_the_absence_of_space_is_visible_in_the_frame() {
    // Stated, not implied: the frame's own members name the empty type.
    fn space_of<F: ContextFrame>() -> &'static str {
        core::any::type_name::<F::Space>()
    }

    assert!(space_of::<ClockFrame>().contains("NoSpaceTime"));
    assert!(!space_of::<UniformFrame>().contains("NoSpaceTime"));
}

#[test]
fn test_a_clock_frame_costs_nothing_for_the_space_it_does_not_have() {
    assert_eq!(size_of::<<ClockFrame as ContextFrame>::Space>(), 0);
    assert_eq!(size_of::<<ClockFrame as ContextFrame>::SpaceTime>(), 0);
}

#[test]
fn test_a_clock_frame_still_has_a_real_clock() {
    // The frame lacks a position, not a time. Its Time member is a real clock, and the scalar
    // bound does not reach it.
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(empty.dimension(), 0);

    let clock = EuclideanTime::new(1, TimeScale::Second, 2.5);
    assert_eq!(clock.time_unit(), 2.5);
}
