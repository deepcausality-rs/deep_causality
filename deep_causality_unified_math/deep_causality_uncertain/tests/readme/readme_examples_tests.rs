/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every ```rust block of the crate README, compiled and run.
//!
//! The README is not pulled in by `include_str!`, so its examples are not doctests — and
//! that is precisely how every one of them came to name an API that no longer exists. This
//! file is the guard: each block below is a README block verbatim, wrapped in a test, so an
//! example that stops compiling fails here rather than misleading a reader.
//!
//! Blocks marked `rust,ignore` in the README are the ones needing a crate this one does
//! not depend on; they are excluded here for the same reason they are excluded there.

// The blocks are the README's verbatim, and illustrative code names things it does not go on to
// use. Binding `let _ = …` to every one of them would make the README worse to read in order to
// make this file quieter, which is the wrong trade: the README is the artifact, this is the guard.
#![allow(unused_variables, unused_mut)]

#[test]
fn readme_example_1() {
    use deep_causality_uncertain::{Uncertain, UncertainBool};

    // A precise, known value.
    let precise = Uncertain::<f64>::point(10.0);

    // A sensor reading with Gaussian noise.
    let reading = Uncertain::<f64>::normal(25.0, 0.5);

    // A value somewhere in a range.
    let jitter = Uncertain::<f64>::uniform(-1.0, 1.0);

    // An uncertain truth value: a Bernoulli trial.
    let coin = UncertainBool::<f64>::bernoulli(0.7);
}

#[test]
fn readme_example_2() {
    use deep_causality_num::{BFloat16, Float106};
    use deep_causality_uncertain::Uncertain;

    let wide = Uncertain::<Float106>::normal(Float106::from(0.0), Float106::from(1.0));
    let narrow = Uncertain::<BFloat16>::normal(BFloat16::from(0.0), BFloat16::from(1.0));
    let single = Uncertain::<f32>::normal(0.0, 1.0);
}

#[test]
fn readme_example_3() {
    use deep_causality_uncertain::{SampleSession, Uncertain};

    let reading = Uncertain::<f64>::normal(25.0, 0.5);

    // Reproducible: the same seed and index give the same value, in this process or a later one.
    let session = SampleSession::seeded(42);
    let a = reading.sample_at(&session, 0).unwrap();
    let b = reading.sample_at(&session, 0).unwrap();
    assert_eq!(a, b);

    // Advancing draws a fresh index each time.
    let mut walking = SampleSession::seeded(42);
    let first = reading.sample_next(&mut walking).unwrap();
    let second = reading.sample_next(&mut walking).unwrap();
    assert_ne!(first, second);

    // No session of your own: nothing about the value can be reproduced afterwards, and the name
    // says so.
    let once = reading.sample_from_entropy().unwrap();
    let _ = (a, b, first, second, once);
}

#[test]
fn readme_example_4() {
    use deep_causality_uncertain::{SampleSession, Uncertain};

    let session = SampleSession::seeded(7);
    let x = Uncertain::<f64>::normal(10.0, 2.0);
    let y = Uncertain::<f64>::normal(3.0, 0.5);

    let total = x.clone() + y.clone();
    let scaled = x.clone() * Uncertain::point(2.0);

    // A quantity used twice in one expression is one draw, not two.
    let difference = x.clone() - x.clone();
    assert_eq!(difference.sample_at(&session, 0).unwrap(), 0.0);
    let _ = (total, scaled);
}

#[test]
fn readme_example_5() {
    use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool};

    let session = SampleSession::seeded(7);
    let reading = Uncertain::<f64>::normal(100.0, 5.0);

    // Every comparison yields an `UncertainBool<R>` over the same graph.
    let too_high: UncertainBool<f64> = reading.greater_than(105.0);
    let in_band = reading.within_range(95.0, 105.0);
    let near = reading.approx_eq(100.0, 1.0);

    let target = Uncertain::<f64>::normal(98.0, 3.0);
    let exceeds_target = reading.gt_uncertain(&target);

    let p: f64 = too_high.estimate_probability(&session, 1000).unwrap();
    println!("P(reading > 105) = {:.1}%", p * 100.0);
    let _ = (in_band, near, exceeds_target);
}

#[test]
fn readme_example_6() {
    use deep_causality_uncertain::{SampleSession, Uncertain};

    let session = SampleSession::seeded(7);
    let celsius = Uncertain::<f64>::normal(25.0, 2.0);

    let fahrenheit = celsius.map(|c| c * 1.8 + 32.0);
    let is_hot = celsius.map_to_bool(|c| c > 30.0);

    let mean: f64 = fahrenheit.expected_value(&session, 1000).unwrap();
    let p: f64 = is_hot.estimate_probability(&session, 1000).unwrap();
    println!("{mean:.2} F, P(hot) = {:.1}%", p * 100.0);
}

#[test]
fn readme_example_7() {
    use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool};

    let session = SampleSession::seeded(7);
    let heavy_traffic = UncertainBool::<f64>::bernoulli(0.7);
    let via_main = Uncertain::<f64>::normal(30.0, 5.0);
    let via_back = Uncertain::<f64>::normal(45.0, 2.0);

    let travel_time = Uncertain::conditional(heavy_traffic, via_back, via_main);
    let mean: f64 = travel_time.expected_value(&session, 1000).unwrap();
    println!("Expected travel time: {mean:.1} minutes");
}

#[test]
fn readme_example_8() {
    use deep_causality_uncertain::{SampleSession, Uncertain};

    let session = SampleSession::seeded(7);
    let price = Uncertain::<f64>::normal(150.0, 10.0);

    let mean: f64 = price.expected_value(&session, 1000).unwrap();
    let spread: f64 = price.standard_deviation(&session, 1000).unwrap();

    // Quasi-Monte-Carlo: a digitally shifted Sobol sequence, reproducible from its own seed and
    // faster-converging on low-dimension static graphs.
    let qmc_mean: f64 = price.expected_value_qmc(1024, 0xABCD).unwrap();

    println!("{mean:.2} +/- {spread:.2}  (QMC mean {qmc_mean:.2})");
}

#[test]
fn readme_example_9() {
    use deep_causality_uncertain::{SampleSession, UncertainBool};

    let session = SampleSession::seeded(7);
    let healthy = UncertainBool::<f64>::bernoulli(0.9);

    // threshold, confidence, indifference region, sample budget.
    if healthy.to_bool(&session, 0.5, 0.95, 0.05, 1000).unwrap() {
        println!("System is healthy at 95% confidence.");
    }

    // "More likely than not", with those defaults filled in.
    if healthy.implicit_conditional(&session).unwrap() {
        println!("System is more likely than not healthy.");
    }
}

#[test]
fn readme_example_10() {
    use deep_causality_uncertain::{MaybeUncertain, SampleSession, Uncertain};

    let mut session = SampleSession::seeded(7);

    // Certainly present, uncertain in value.
    let present = MaybeUncertain::<f64>::from_uncertain(Uncertain::normal(10.0, 2.0));
    assert!(present.sample(&mut session).unwrap().is_some());

    // Certainly absent.
    let absent = MaybeUncertain::<f64>::always_none();
    assert!(absent.sample(&mut session).unwrap().is_none());

    // Present with probability 0.7.
    let intermittent =
        MaybeUncertain::<f64>::from_bernoulli_and_uncertain(0.7, Uncertain::normal(5.0, 1.0));

    // Absence propagates through arithmetic: one absent operand makes the sum absent.
    let sum = present.clone() + absent.clone();
    assert!(sum.sample(&mut session).unwrap().is_none());

    // Collapse to a plain `Uncertain<R>` only if the evidence of presence clears the gate.
    let gate = SampleSession::seeded(8);
    match intermittent.lift_to_uncertain(&gate, 0.6, 0.95, 0.05, 1000) {
        Ok(value) => {
            let mean: f64 = value.expected_value(&gate, 1000).unwrap();
            println!("Present; expected value {mean:.2}");
        }
        Err(e) => println!("Not enough evidence of presence: {e}"),
    }
}

#[test]
fn readme_example_11() {
    use deep_causality_haft::Arrow;
    use deep_causality_uncertain::{SampleIndex, SampleSession, Uncertain};

    let session = SampleSession::seeded(7);
    let quantity = Uncertain::<f64>::normal(10.0, 2.0);

    let at = SampleIndex::at(&session, 0);
    assert_eq!(quantity.run(at).unwrap(), quantity.run(at).unwrap());
}
