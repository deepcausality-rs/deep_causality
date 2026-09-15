/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The provided methods on [`Rng`], at their boundaries.

use deep_causality_num::{BFloat16, Float106, lift};
use deep_causality_rand::{Rng, RngCore, Uniform, UniformDistributionError, Xoshiro256};

/// A generator whose every word is zero — the lower boundary of the word range.
///
/// A real generator reaches it with probability `2^-64`, so no sampling test finds what it finds.
struct ZeroRng;

impl RngCore for ZeroRng {
    fn next_u32(&mut self) -> u32 {
        0
    }
    fn next_u64(&mut self) -> u64 {
        0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }
}

impl Rng for ZeroRng {}

/// A generator whose every word is the largest — the upper boundary.
struct MaxRng;

impl RngCore for MaxRng {
    fn next_u32(&mut self) -> u32 {
        u32::MAX
    }
    fn next_u64(&mut self) -> u64 {
        u64::MAX
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0xFF);
    }
}

impl Rng for MaxRng {}

// ---------------------------------------------------------------------------------------------
// random_bool at its two endpoints
// ---------------------------------------------------------------------------------------------

#[test]
fn an_impossible_event_never_happens() {
    // `p = 0.0` means the event does not occur, and no word may make it occur. The formula this
    // replaced compared `word / u64::MAX <= p`, which is `0 <= 0` on a zero word — so a
    // probability-zero event fired once in every `2^64` draws. No sampling test reaches that;
    // a zero generator reaches it on the first draw.
    assert!(
        !ZeroRng.random_bool(0.0),
        "an event of probability zero occurred"
    );

    let mut g = Xoshiro256::from_seed(0x5EED_2026);
    for _ in 0..10_000 {
        assert!(!g.random_bool(0.0), "an event of probability zero occurred");
    }
}

#[test]
fn a_certain_event_always_happens() {
    // The other endpoint, which the largest word must not spoil.
    assert!(
        MaxRng.random_bool(1.0),
        "an event of probability one did not occur"
    );

    let mut g = Xoshiro256::from_seed(0x5EED_2026);
    for _ in 0..10_000 {
        assert!(
            g.random_bool(1.0),
            "an event of probability one did not occur"
        );
    }
}

#[test]
fn the_probability_is_honoured_in_between() {
    let mut g = Xoshiro256::from_seed(0x5EED_2026);
    let n = 200_000;
    let hits = (0..n).filter(|_| g.random_bool(0.25)).count() as f64 / n as f64;
    assert!(
        (hits - 0.25).abs() < 0.005,
        "p = 0.25 fired {hits} of the time"
    );
}

#[test]
fn the_probability_is_taken_at_the_callers_scalar() {
    // The parameter is a probability, not an `f64`. A caller working at `f32` or `Float106`
    // states it in their own scalar and gets the same behaviour at the endpoints.
    assert!(!ZeroRng.random_bool(0.0f32));
    assert!(MaxRng.random_bool(1.0f32));
    assert!(!ZeroRng.random_bool(Float106::from(0.0)));
    assert!(MaxRng.random_bool(Float106::from(1.0)));

    let mut g = Xoshiro256::from_seed(0x5EED_2026);
    let n = 100_000;
    let hits = (0..n).filter(|_| g.random_bool(0.25f32)).count() as f64 / n as f64;
    assert!(
        (hits - 0.25).abs() < 0.01,
        "p = 0.25 at f32 fired {hits} of the time"
    );
}

#[test]
#[should_panic(expected = "outside range")]
fn a_probability_above_one_is_refused() {
    Xoshiro256::from_seed(1).random_bool(1.5);
}

#[test]
#[should_panic(expected = "outside range")]
fn a_negative_probability_is_refused() {
    Xoshiro256::from_seed(1).random_bool(-0.1);
}

/// A range with distinct endpoints can be empty at a narrow scalar, and `random_range` panics on
/// it while `Uniform::new` returns an error.
///
/// `BFloat16`'s spacing at a thousand is 8, so `1000.0` and `1001.0` are the same number there and
/// the range between them holds nothing. This is a fact about the scalar rather than about either
/// function, and the two answer it differently on purpose: the infallible convenience refuses by
/// panicking, and the fallible constructor refuses by returning.
///
/// Pinned because a caller whose bounds are close together only meets this at a width where the
/// endpoints collapse, which is exactly where it is least expected.
#[test]
fn test_a_range_the_scalar_collapses_is_empty() {
    let low = lift::<BFloat16>(1000.0);
    let high = lift::<BFloat16>(1001.0);
    assert_eq!(
        low, high,
        "the fixture is wrong: these endpoints must collapse at BFloat16"
    );

    assert!(
        matches!(
            Uniform::new(low, high),
            Err(UniformDistributionError::EmptyRange)
        ),
        "Uniform::new must refuse a collapsed range by returning"
    );

    let panicked = std::panic::catch_unwind(|| {
        let mut rng = Xoshiro256::from_seed(1);
        rng.random_range(low..high)
    });
    assert!(
        panicked.is_err(),
        "random_range must refuse a collapsed range by panicking"
    );

    // The same endpoints one power of two lower, where the spacing is 4 and the range is not
    // empty — so the refusal above is about these values and not about the type.
    let low = lift::<BFloat16>(1000.0);
    let wide = lift::<BFloat16>(1016.0);
    assert!(low < wide, "1000 and 1016 are distinct at BFloat16");
    let mut rng = Xoshiro256::from_seed(1);
    let drawn: BFloat16 = rng.random_range(low..wide);
    assert!(drawn >= low && drawn < wide, "the draw lies in the range");
}
