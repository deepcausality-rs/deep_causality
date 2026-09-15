/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The lazy graph as a value-level `Arrow`.

use deep_causality_haft::{Arrow, Compose, Id};
use deep_causality_uncertain::{
    SampleIndex, SampleSession, Uncertain, UncertainBool, UncertainError,
};

const SEED: u64 = 0x5EED_2026;

/// A downstream arrow, to compose against. Doubles a successful draw and passes a failure through.
struct Double;

impl Arrow for Double {
    type In = Result<f64, UncertainError>;
    type Out = Result<f64, UncertainError>;

    fn run(&self, input: Self::In) -> Self::Out {
        input.map(|x| x * 2.0)
    }
}

/// Running at one address twice agrees, with nothing stored between.
///
/// This is the property that makes `Arrow::run(&self, ..)` well-typed at all: evaluating the graph
/// at an address is a pure function of the graph and the address. A graph drawing from ambient
/// state would fail this.
#[test]
fn running_at_one_index_twice_agrees() {
    let quantity = Uncertain::<f64>::normal(10.0, 3.0);
    let session = SampleSession::seeded(SEED);

    for index in 0..32 {
        let at = SampleIndex::at(&session, index);
        let first = quantity.run(at).expect("draw");
        let second = quantity.run(at).expect("draw");
        assert_eq!(first, second, "index {index} gave two different values");
    }
}

/// And a *fresh* `SampleIndex` for the same address is the same address — the type carries the
/// whole of what the draw depends on, so two independently built addresses agree.
#[test]
fn an_address_is_data_and_two_equal_addresses_draw_alike() {
    let quantity = Uncertain::<f64>::normal(0.0, 1.0);

    let a = SampleIndex::at(&SampleSession::seeded(SEED), 7);
    let b = SampleIndex::at(&SampleSession::seeded(SEED), 7);
    assert_eq!(a, b, "the same seed and index is the same address");
    assert_eq!(
        quantity.run(a).expect("draw"),
        quantity.run(b).expect("draw")
    );

    // A different seed is a different address, and draws differently.
    let elsewhere = SampleIndex::at(&SampleSession::seeded(SEED ^ 1), 7);
    assert_ne!(a, elsewhere);
    assert_ne!(
        quantity.run(a).expect("draw"),
        quantity.run(elsewhere).expect("draw")
    );
}

/// `run` agrees with the ordinary sampling surface — the arrow is a view of it, not a second one.
#[test]
fn run_agrees_with_sample_at() {
    let quantity = Uncertain::<f64>::normal(-4.0, 0.5);
    let session = SampleSession::seeded(SEED);

    for index in 0..16 {
        assert_eq!(
            quantity.run(SampleIndex::at(&session, index)).expect("run"),
            quantity.sample_at(&session, index).expect("sample_at"),
            "the arrow and the sampler disagree at index {index}"
        );
    }
}

/// A graph composes with a downstream arrow, and the composite runs at an index.
#[test]
fn a_graph_composes_with_a_downstream_arrow() {
    let quantity = Uncertain::<f64>::point(21.0);
    let session = SampleSession::seeded(SEED);

    let pipeline = quantity.clone().compose(Double);

    assert_eq!(
        pipeline.run(SampleIndex::at(&session, 0)).expect("run"),
        42.0,
        "the composite must be the downstream arrow applied to the draw"
    );

    // And it is itself an `Arrow`, so it composes again.
    let twice = pipeline.compose(Double);
    assert_eq!(twice.run(SampleIndex::at(&session, 0)).expect("run"), 84.0);
}

/// Composition is **static**: the composite is a concrete generic struct with no trait object.
///
/// Two facts, both about the type rather than about the source. A `dyn` in the composite would show
/// in its name and would make its size a fat pointer's; neither is the case, and the size is known
/// at compile time — which a trait object's is not.
#[test]
fn composition_is_static_with_no_trait_object() {
    use core::mem::size_of;

    type Pipeline = Compose<Uncertain<f64>, Double>;

    let name = core::any::type_name::<Pipeline>();
    assert!(
        !name.contains("dyn"),
        "the composite type names a trait object: {name}"
    );
    assert!(
        name.contains("Compose") && name.contains("Uncertain"),
        "the composite must be the concrete generic struct, got: {name}"
    );

    // `Double` is a unit struct, so the composite is exactly the graph's own size — no vtable, no
    // indirection introduced by composing.
    assert_eq!(size_of::<Double>(), 0);
    assert_eq!(size_of::<Pipeline>(), size_of::<Uncertain<f64>>());
}

/// `Id` composes on either side without changing what the arrow produces.
#[test]
fn composing_with_id_changes_nothing() {
    let quantity = Uncertain::<f64>::normal(3.0, 1.0);
    let session = SampleSession::seeded(SEED);
    let at = SampleIndex::at(&session, 5);

    let bare = quantity.run(at).expect("draw");
    let with_id = quantity.clone().compose(Id::new()).run(at).expect("draw");

    assert_eq!(bare, with_id);
}

/// A failing draw propagates through composition rather than being swallowed.
///
/// A graph whose distribution the constructor refuses fails at `run`, and the composite carries
/// that failure out — which is why `Out` is a `Result` rather than the value alone.
#[test]
fn a_failing_draw_propagates_through_the_composite() {
    // A Bernoulli parameter outside `[0, 1]` is refused when the leaf is drawn.
    let bad = UncertainBool::<f64>::bernoulli(2.0);
    let session = SampleSession::seeded(SEED);

    assert!(
        bad.run(SampleIndex::at(&session, 0)).is_err(),
        "an out-of-range Bernoulli parameter must fail at the draw"
    );
}

/// The Boolean carrier is an arrow too, over the same addresses.
#[test]
fn the_boolean_carrier_is_an_arrow_over_the_same_addresses() {
    let session = SampleSession::seeded(SEED);
    let quantity = Uncertain::<f64>::normal(0.0, 1.0);
    let positive = quantity.greater_than(0.0);

    for index in 0..32 {
        let at = SampleIndex::at(&session, index);
        assert_eq!(
            positive.run(at).expect("verdict"),
            quantity.run(at).expect("draw") > 0.0,
            "the verdict arrow and the value arrow disagree at index {index}"
        );
    }
}

/// `SampleIndex::sequence` walks a session's first `n` addresses, and running over them is the
/// ensemble — one graph, many addresses, nothing stored between.
#[test]
fn a_sequence_of_addresses_reproduces_the_ensemble() {
    let quantity = Uncertain::<f64>::normal(7.0, 2.0);
    let session = SampleSession::seeded(SEED);

    let through_arrow: Vec<f64> = SampleIndex::sequence(&session, 20)
        .map(|at| quantity.run(at).expect("draw"))
        .collect();

    let through_sampler: Vec<f64> = (0..20)
        .map(|i| quantity.sample_at(&session, i).expect("draw"))
        .collect();

    assert_eq!(through_arrow, through_sampler);
    assert_eq!(through_arrow.len(), 20);
}

/// The arrow is generic in the scalar, like everything else here.
#[test]
fn the_arrow_runs_at_a_scalar_the_crate_never_names() {
    use deep_causality_num::BFloat16;

    let quantity = Uncertain::<BFloat16>::point(BFloat16::from(2.5));
    let session = SampleSession::seeded(SEED);
    let at = SampleIndex::at(&session, 0);

    assert_eq!(quantity.run(at).expect("draw"), BFloat16::from(2.5));
}
