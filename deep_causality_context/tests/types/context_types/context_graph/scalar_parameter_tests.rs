/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The acceptance test for the scalar parameter.
//!
//! A context is built at the narrowest real field the workspace ships and at the widest, and its
//! coordinates are read back from both. Neither scalar is named anywhere in
//! `deep_causality_context/src`, so a type passing here does so through the blanket bound rather
//! than through an entry made for it.

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Coordinate, Data, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, RelationKind, TimeScale,
};
use deep_causality_num::{BFloat16, Float106};

/// Builds a two-node context at scalar `R`, joins the nodes, and returns the space node's `x`.
///
/// Generic in `R`, so the same body serves every scalar: what the test asserts about one it
/// asserts about all of them by construction.
fn build_and_read<R>(x: R, y: R, z: R) -> (R, usize)
where
    R: deep_causality_algebra::RealField,
{
    let mut context: Context<
        Data<u64>,
        EuclideanSpace<R>,
        EuclideanTime<R>,
        EuclideanSpacetime<R>,
    > = Context::with_capacity(1, "context at one scalar", 4);

    let space = context
        .add_node(Contextoid::new(
            1,
            ContextoidType::Spaceoid(EuclideanSpace::new(1, x, y, z)),
        ))
        .expect("failed to add the space node");

    let time = context
        .add_node(Contextoid::new(
            2,
            ContextoidType::Tempoid(EuclideanTime::new(2, TimeScale::Second, z)),
        ))
        .expect("failed to add the time node");

    context
        .add_edge(space, time, RelationKind::SpaceTemporal)
        .expect("failed to join the two nodes");

    let node = context.get_node(space).expect("the space node is missing");
    let read_back = match node.vertex_type() {
        ContextoidType::Spaceoid(s) => *s.coordinate(0).expect("axis 0 is missing"),
        _ => panic!("the node at this index is not a space node"),
    };

    (read_back, context.number_of_nodes())
}

#[test]
fn test_context_at_the_narrowest_shipped_real_field() {
    // BFloat16 keeps 8 bits of significand. The values are chosen to be exact in it, so the
    // assertion tests the plumbing rather than the rounding.
    let (x, nodes) = build_and_read(
        BFloat16::from(1.5f32),
        BFloat16::from(2.0f32),
        BFloat16::from(4.0f32),
    );

    assert_eq!(x, BFloat16::from(1.5f32));
    assert_eq!(nodes, 2);
}

#[test]
fn test_context_at_the_widest_shipped_real_field() {
    // Float106 keeps ~106 bits of significand, against BFloat16's 8.
    let (x, nodes) = build_and_read(
        Float106::from(1.5f64),
        Float106::from(2.0f64),
        Float106::from(4.0f64),
    );

    assert_eq!(x, Float106::from(1.5f64));
    assert_eq!(nodes, 2);
}

#[test]
fn test_the_two_scalars_disagree_where_precision_decides() {
    // A negative control for the two tests above: if the scalar were not actually reaching the
    // node, both would carry the same value and this would fail.
    //
    // 0.1 is representable in neither, but Float106 holds it to ~31 decimal digits and BFloat16
    // to about two. Reading each back through a context and widening both to f64 has to show the
    // gap, or the parameter is not doing anything.
    let (narrow, _) = build_and_read(
        BFloat16::from(0.1f32),
        BFloat16::from(0.0f32),
        BFloat16::from(0.0f32),
    );
    let (wide, _) = build_and_read(
        Float106::from(0.1f64),
        Float106::from(0.0f64),
        Float106::from(0.0f64),
    );

    let narrow_error = (f64::from(narrow) - 0.1).abs();
    let wide_error = (f64::from(wide) - 0.1).abs();

    assert!(
        narrow_error > wide_error,
        "the narrow scalar should carry more error than the wide one: {narrow_error} vs {wide_error}"
    );
}
