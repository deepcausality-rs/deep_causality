/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Arrow, ArrowBuilder, Id, Lift, arrow};

fn inc() -> Lift<i32, i32, fn(i32) -> i32> {
    Lift::new((|x: i32| x + 1) as fn(i32) -> i32)
}
fn dbl() -> Lift<i32, i32, fn(i32) -> i32> {
    Lift::new((|x: i32| x * 2) as fn(i32) -> i32)
}
fn add10() -> Lift<i32, i32, fn(i32) -> i32> {
    Lift::new((|x: i32| x + 10) as fn(i32) -> i32)
}

// --- Category laws ---

#[test]
fn test_lift_runs_the_function() {
    let f = Lift::new(|x: i32| x + 1);
    assert_eq!(f.run(41), 42);
}

#[test]
fn test_compose_runs_left_to_right() {
    // (x + 1) then (x * 2)
    assert_eq!(inc().compose(dbl()).run(3), 8);
}

#[test]
fn test_identity_is_left_and_right_unit() {
    assert_eq!(Id::new().compose(inc()).run(5), inc().run(5));
    assert_eq!(inc().compose(Id::new()).run(5), inc().run(5));
}

#[test]
fn test_composition_is_associative() {
    let left = inc().compose(dbl()).compose(add10());
    let right = inc().compose(dbl().compose(add10()));
    for x in [-3, 0, 7, 100] {
        assert_eq!(left.run(x), right.run(x));
    }
}

// --- Strength / monoidal-product laws ---

#[test]
fn test_first_passes_second_component_through() {
    let f = inc().first::<&str>();
    assert_eq!(f.run((2, "kept")), (3, "kept"));
}

#[test]
fn test_second_passes_first_component_through() {
    let f = inc().second::<&str>();
    assert_eq!(f.run(("kept", 2)), ("kept", 3));
}

#[test]
fn test_split_runs_both_arrows_on_a_pair() {
    let f = inc().split(add10());
    assert_eq!(f.run((2, 3)), (3, 13));
}

#[test]
fn test_fanout_feeds_one_input_to_two_arrows() {
    let f = inc().fanout(dbl());
    assert_eq!(f.run(5), (6, 10));
}

#[test]
fn test_product_decomposes_via_first_then_second() {
    // *** = first >>> second
    let split = inc().split(add10());
    let decomposed = inc().first::<i32>().compose(add10().second::<i32>());
    for pair in [(0, 0), (2, 3), (-5, 7)] {
        assert_eq!(split.run(pair), decomposed.run(pair));
    }
}

// --- Multi-input witness (structure stays a captured parameter) ---

#[test]
fn test_multi_input_two_cohorts_combine_into_one_arrow() {
    // A "structural parameter" (here a bias) is captured by the arrows, never flowing in In/Out.
    let bias = 100;
    let normal = Lift::new(move |x: i32| x + bias);
    let anomalous = Lift::new(move |y: i32| y + bias);
    let combine = Lift::new(|(a, b): (i32, i32)| a + b);

    // (i32, i32) -> i32 — the captured `bias` appears in neither In nor Out.
    let pipeline = normal.split(anomalous).compose(combine);
    assert_eq!(pipeline.run((1, 2)), 203); // (1+100) + (2+100)
    // reusable
    assert_eq!(pipeline.run((0, 0)), 200);
}

// --- Builder hides the combinator types ---

#[test]
fn test_builder_chain_equals_explicit_construction() {
    // No Compose/Split/Lift/Morphism named in this expression — only the builder verbs.
    let built = arrow(|x: i32| x + 1).then_fn(|x| x * 2).build();
    let explicit = inc().compose(dbl());
    for x in [-1, 0, 3, 42] {
        assert_eq!(built.run(x), explicit.run(x));
    }
}

#[test]
fn test_builder_run_terminal() {
    assert_eq!(arrow(|x: i32| x + 1).then_fn(|x| x * 2).run(3), 8);
}

#[test]
fn test_builder_par_and_build_yields_reusable_arrow() {
    let pipe = arrow(|x: i32| x + 1)
        .par(add10())
        .then_fn(|(a, b)| a + b)
        .build();
    assert_eq!(pipe.run((2, 3)), 16); // (2+1) + (3+10)
    assert_eq!(pipe.run((0, 0)), 11); // reusable
}

#[test]
fn test_builder_new_wraps_an_existing_arrow() {
    // ArrowBuilder::new wraps a pre-built arrow; the chain continues from there.
    let pipe = ArrowBuilder::new(inc()).then(dbl()).build();
    assert_eq!(pipe.run(3), 8); // (3+1)*2
}

#[test]
fn test_builder_then_composes_with_an_arrow() {
    // `.then(arrow)` is the arrow-typed sibling of `.then_fn(closure)`.
    let pipe = arrow(|x: i32| x + 1).then(dbl()).build();
    assert_eq!(pipe.run(3), 8);
}

#[test]
fn test_builder_fanout_feeds_one_input_to_two_arrows() {
    let pipe = arrow(|x: i32| x + 1)
        .fanout(add10())
        .then_fn(|(a, b)| a + b)
        .build();
    assert_eq!(pipe.run(5), 21); // (5+1) + (5+10)
}

#[test]
fn test_id_default_is_the_identity_arrow() {
    let id: Id<i32> = Id::default();
    assert_eq!(id.run(7), 7);
}
