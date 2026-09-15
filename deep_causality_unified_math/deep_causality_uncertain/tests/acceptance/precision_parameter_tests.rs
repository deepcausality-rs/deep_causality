/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The acceptance test for "precision is a parameter".
//!
//! Everything here is instantiated at `f32` and at `BFloat16` — two scalars that
//! `deep_causality_uncertain/src` names **nowhere**. No line was added to the crate to make them
//! work, and that is the claim under test: a scalar joins by satisfying the algebra, not by
//! appearing in a list.
//!
//! A count of deleted files would not decide this. What decides it is whether the whole surface —
//! construction, both carriers, arithmetic, comparison, logic, the verdict algebras, the presence
//! gate, Monte-Carlo statistics and the QMC path — runs at a scalar nobody wrote down.
//!
//! # Reading the tolerances
//!
//! `BFloat16` keeps `f32`'s full exponent range and trades significand: 7 stored bits, an epsilon
//! of `7.8125e-3`, roughly two decimal digits. So a `BFloat16` assertion is stated as a *relative*
//! tolerance of a few percent, and one at `f32` as a much tighter one. A wider tolerance here is
//! the format speaking, not the estimator: where an estimator defect is at issue, the assertion is
//! on a quantity the format can represent exactly, and the wrongness would be orders of magnitude
//! larger than the resolution.

use deep_causality_num::BFloat16;
use deep_causality_uncertain::{
    MaybeUncertain, SampleSession, Uncertain, UncertainBool, UncertainScalar,
};

use deep_causality_algebra::Verdict;

/// The crate's scalar bound, plus what an assertion needs.
///
/// `UncertainScalar` does not imply `Debug` — `Real` in `deep_causality_algebra` does not, and
/// requiring it of every scalar to serve `assert_eq!` would be the test wagging the library. So the
/// test states it, once, for itself.
trait TestScalar: UncertainScalar + core::fmt::Debug {}
impl<T: UncertainScalar + core::fmt::Debug> TestScalar for T {}

/// A literal, at whichever scalar the caller is working in.
fn at<R: TestScalar>(v: f64) -> R {
    R::from_f64(v).expect("a small literal converts to every scalar")
}

/// A scalar, back at `f64` for an assertion message and a comparison.
fn f64_of<R: TestScalar>(v: R) -> f64 {
    v.to_f64().expect("every scalar lowers to f64")
}

/// `|actual − expected| <= |expected| * relative`, reported with both numbers.
fn assert_close<R: TestScalar>(actual: R, expected: f64, relative: f64, what: &str) {
    let actual = f64_of(actual);
    let tolerance = expected.abs() * relative;
    assert!(
        (actual - expected).abs() <= tolerance,
        "{what}: expected {expected} ± {tolerance}, got {actual}"
    );
}

// ---------------------------------------------------------------------------------------------
// The body of the acceptance test, written once and instantiated per scalar.
// ---------------------------------------------------------------------------------------------

/// A certain value survives the round trip unchanged, at any scalar.
fn point_is_exact<R: TestScalar>() {
    let session = SampleSession::seeded(7);
    let x: R = at(2.5);
    let u = Uncertain::<R>::point(x);
    assert_eq!(u.sample_at(&session, 0).expect("draw"), x);
    assert_eq!(u.sample_at(&session, 99).expect("draw"), x);
    // `value()` reads the root without drawing.
    assert_eq!(u.value(), x);
}

/// A normal distribution's first two moments come back, estimated at the scalar itself.
///
/// The mean of N(100, 5) is representable at `BFloat16` to within 0.5, so a tolerance of a few
/// percent is more than a hundred times the resolution — the assertion would still catch an
/// estimator that stagnates, which is what it is here to do.
fn normal_moments_are_recovered<R: TestScalar>(relative: f64) {
    let session = SampleSession::seeded(4242);
    let u = Uncertain::<R>::normal(at(100.0), at(5.0));

    let mean = u.expected_value(&session, 2000).expect("mean");
    assert_close(mean, 100.0, relative, "mean of N(100, 5)");

    let sd = u.standard_deviation(&session, 2000).expect("sd");
    assert_close(sd, 5.0, relative.max(0.15), "sd of N(100, 5)");
}

/// Every draw of a uniform lands inside its half-open range.
fn uniform_draws_stay_in_range<R: TestScalar>() {
    let mut session = SampleSession::seeded(11);
    let low: R = at(-3.0);
    let high: R = at(4.0);
    let u = Uncertain::<R>::uniform(low, high);

    let samples = u.take_samples(&mut session, 500).expect("draws");
    assert_eq!(samples.len(), 500);
    for s in samples {
        assert!(
            s >= low && s < high,
            "draw {} left [{}, {})",
            f64_of(s),
            f64_of(low),
            f64_of(high)
        );
    }
}

/// Arithmetic composes, and a value used twice in one expression is one draw rather than two.
///
/// `x - x` is the discriminating case: under a shared-leaf memo it is exactly zero at every index,
/// and under two independent draws it is the difference of two samples of N(0, 1) — which is zero
/// with probability zero.
fn arithmetic_shares_one_draw<R: TestScalar>() {
    let session = SampleSession::seeded(2024);
    let x = Uncertain::<R>::normal(at(0.0), at(1.0));

    let difference = x.clone() - x.clone();
    for index in 0..32 {
        assert_eq!(
            difference.sample_at(&session, index).expect("draw"),
            R::zero(),
            "x - x drew twice at index {index}"
        );
    }

    // And a genuine composition still moves.
    let doubled = x.clone() + x.clone();
    let single = x.sample_at(&session, 3).expect("draw");
    assert_eq!(
        doubled.sample_at(&session, 3).expect("draw"),
        single + single
    );
}

/// A comparison yields the Boolean carrier over the same scalar, and its probability is stated
/// in that scalar.
fn comparison_yields_the_boolean_carrier<R: TestScalar>(relative: f64) {
    let session = SampleSession::seeded(31337);
    let u = Uncertain::<R>::normal(at(0.0), at(1.0));

    // P(x > 0) = 1/2 by symmetry.
    let positive: UncertainBool<R> = u.greater_than(R::zero());
    let p = positive.estimate_probability(&session, 4000).expect("p");
    assert_close(p, 0.5, relative.max(0.1), "P(N(0,1) > 0)");

    // A certain value compares certainly, at every index.
    let five = Uncertain::<R>::point(at(5.0));
    assert!(
        five.greater_than(at(4.0))
            .sample_at(&session, 0)
            .expect("draw")
    );
    assert!(
        !five
            .greater_than(at(6.0))
            .sample_at(&session, 0)
            .expect("draw")
    );
    assert!(five.within_range(at(4.0), at(6.0)).value_at(&session));
    assert!(!five.within_range(at(1.0), at(2.0)).value_at(&session));
    assert!(five.approx_eq(at(5.0), at(0.5)).value_at(&session));
}

/// A Bernoulli leaf is a distribution like any other, and its parameter is stated at `R`.
fn bernoulli_recovers_its_parameter<R: TestScalar>(absolute: f64) {
    let session = SampleSession::seeded(909);
    let b = UncertainBool::<R>::bernoulli(at(0.25));
    let p = f64_of(b.estimate_probability(&session, 4000).expect("p"));
    assert!(
        (p - 0.25).abs() <= absolute,
        "P(true) of a Bernoulli(0.25): expected 0.25 ± {absolute}, got {p}"
    );

    // The endpoints are exact, whatever the scalar: `Bernoulli` holds `p` as fixed point.
    let never = UncertainBool::<R>::bernoulli(R::zero());
    let always = UncertainBool::<R>::bernoulli(R::one());
    for index in 0..64 {
        assert!(!never.sample_at(&session, index).expect("draw"));
        assert!(always.sample_at(&session, index).expect("draw"));
    }
}

/// The logical operators live on the Boolean carrier and work at every scalar.
fn logic_works_on_the_boolean_carrier<R: TestScalar>() {
    let session = SampleSession::seeded(5);
    let t = UncertainBool::<R>::point(true);
    let f = UncertainBool::<R>::point(false);

    assert!((t.clone() & t.clone()).value_at(&session));
    assert!(!(t.clone() & f.clone()).value_at(&session));
    assert!((t.clone() | f.clone()).value_at(&session));
    assert!(!(f.clone() | f.clone()).value_at(&session));
    assert!((t.clone() ^ f.clone()).value_at(&session));
    assert!(!(t.clone() ^ t.clone()).value_at(&session));
    assert!((!f.clone()).value_at(&session));
    assert!(!(!t.clone()).value_at(&session));
}

/// Both verdict algebras exist at this scalar, and they are the two different algebras the split
/// was made to keep.
fn both_verdict_algebras_hold<R: TestScalar>() {
    let session = SampleSession::seeded(64);

    // Boolean class on the Boolean carrier.
    let top = <UncertainBool<R> as Verdict>::top();
    let bottom = <UncertainBool<R> as Verdict>::bottom();
    assert!(top.clone().value_at(&session));
    assert!(!bottom.clone().value_at(&session));
    assert!(!top.clone().meet(bottom.clone()).value_at(&session));
    assert!(top.clone().join(bottom.clone()).value_at(&session));
    assert!(bottom.clone().complement().value_at(&session));

    // MV class on `[0, 1]` on the real carrier — `min`, `max`, `1 − p`.
    let one = <Uncertain<R> as Verdict>::top();
    let zero = <Uncertain<R> as Verdict>::bottom();
    let quarter = Uncertain::<R>::point(at(0.25));

    assert_eq!(
        quarter
            .clone()
            .meet(one.clone())
            .sample_at(&session, 0)
            .expect("draw"),
        at::<R>(0.25)
    );
    assert_eq!(
        quarter
            .clone()
            .join(zero.clone())
            .sample_at(&session, 0)
            .expect("draw"),
        at::<R>(0.25)
    );
    assert_eq!(
        quarter
            .clone()
            .complement()
            .sample_at(&session, 0)
            .expect("draw"),
        at::<R>(0.75)
    );
}

/// The presence gate runs at the scalar, and absence propagates through arithmetic.
fn maybe_uncertain_gates_at_the_scalar<R: TestScalar>() {
    let mut session = SampleSession::seeded(77);

    let present = MaybeUncertain::<R>::from_value(at(3.0));
    assert_eq!(present.sample(&mut session).expect("draw"), Some(at(3.0)));

    let absent = MaybeUncertain::<R>::always_none();
    assert_eq!(absent.sample(&mut session).expect("draw"), None);

    // Conjunctive presence: one absent operand makes the sum absent.
    let sum = present.clone() + absent.clone();
    assert_eq!(sum.sample(&mut session).expect("draw"), None);

    // And two present operands add.
    let two = MaybeUncertain::<R>::from_value(at(2.0));
    let total = present.clone() + two;
    assert_eq!(total.sample(&mut session).expect("draw"), Some(at(5.0)));

    // The SPRT gate lifts a certainly-present value and refuses a certainly-absent one.
    let gate = SampleSession::seeded(78);
    let lifted = present
        .lift_to_uncertain(&gate, at(0.5), at(0.95), at(0.05), 200)
        .expect("a certainly-present value lifts");
    assert_eq!(lifted.sample_at(&gate, 0).expect("draw"), at::<R>(3.0));

    assert!(
        absent
            .lift_to_uncertain(&gate, at(0.5), at(0.95), at(0.05), 200)
            .is_err(),
        "a certainly-absent value must not lift"
    );
}

/// The quasi-Monte-Carlo path runs at the scalar too, on both carriers.
fn qmc_runs_at_the_scalar<R: TestScalar>(relative: f64) {
    let u = Uncertain::<R>::normal(at(10.0), at(2.0));
    let mean = u.expected_value_qmc(1024, 99).expect("qmc mean");
    assert_close(mean, 10.0, relative.max(0.05), "QMC mean of N(10, 2)");

    let b = UncertainBool::<R>::bernoulli(at(0.5));
    let p = f64_of(b.estimate_probability_qmc(1024, 99).expect("qmc p"));
    assert!(
        (p - 0.5).abs() <= 0.1,
        "QMC P(true) of a Bernoulli(0.5): expected 0.5 ± 0.1, got {p}"
    );
}

/// A seed reproduces its draws, at any scalar.
fn a_seed_reproduces_its_draws<R: TestScalar>() {
    let u = Uncertain::<R>::normal(at(0.0), at(1.0));
    let first: Vec<R> = (0..64)
        .map(|i| u.sample_at(&SampleSession::seeded(2026), i).expect("draw"))
        .collect();
    let second: Vec<R> = (0..64)
        .map(|i| u.sample_at(&SampleSession::seeded(2026), i).expect("draw"))
        .collect();
    assert_eq!(first, second);

    // A different seed is a different sequence. (Not every element need differ; the whole
    // sequence matching would mean the seed was not reaching the draw.)
    let other: Vec<R> = (0..64)
        .map(|i| u.sample_at(&SampleSession::seeded(2027), i).expect("draw"))
        .collect();
    assert_ne!(first, other);
}

/// One convenience for the assertions above: a Boolean carrier's draw at index 0.
trait ValueAt<R> {
    fn value_at(&self, session: &SampleSession) -> bool;
}

impl<R: TestScalar> ValueAt<R> for UncertainBool<R> {
    fn value_at(&self, session: &SampleSession) -> bool {
        self.sample_at(session, 0).expect("draw")
    }
}

// ---------------------------------------------------------------------------------------------
// f32 — a scalar the crate names nowhere.
// ---------------------------------------------------------------------------------------------

#[test]
fn f32_point_is_exact() {
    point_is_exact::<f32>();
}

#[test]
fn f32_normal_moments_are_recovered() {
    normal_moments_are_recovered::<f32>(0.02);
}

#[test]
fn f32_uniform_draws_stay_in_range() {
    uniform_draws_stay_in_range::<f32>();
}

#[test]
fn f32_arithmetic_shares_one_draw() {
    arithmetic_shares_one_draw::<f32>();
}

#[test]
fn f32_comparison_yields_the_boolean_carrier() {
    comparison_yields_the_boolean_carrier::<f32>(0.05);
}

#[test]
fn f32_bernoulli_recovers_its_parameter() {
    bernoulli_recovers_its_parameter::<f32>(0.03);
}

#[test]
fn f32_logic_works_on_the_boolean_carrier() {
    logic_works_on_the_boolean_carrier::<f32>();
}

#[test]
fn f32_both_verdict_algebras_hold() {
    both_verdict_algebras_hold::<f32>();
}

#[test]
fn f32_maybe_uncertain_gates_at_the_scalar() {
    maybe_uncertain_gates_at_the_scalar::<f32>();
}

#[test]
fn f32_qmc_runs_at_the_scalar() {
    qmc_runs_at_the_scalar::<f32>(0.02);
}

#[test]
fn f32_a_seed_reproduces_its_draws() {
    a_seed_reproduces_its_draws::<f32>();
}

// ---------------------------------------------------------------------------------------------
// BFloat16 — the narrowest scalar in the workspace, and also named nowhere.
// ---------------------------------------------------------------------------------------------

#[test]
fn bf16_point_is_exact() {
    point_is_exact::<BFloat16>();
}

#[test]
fn bf16_normal_moments_are_recovered() {
    // Five percent. `BFloat16`'s spacing at 100 is 0.5, so this is a hundred times the format's
    // own resolution: it passes on a correct estimate and fails on one that stagnates.
    normal_moments_are_recovered::<BFloat16>(0.05);
}

#[test]
fn bf16_uniform_draws_stay_in_range() {
    uniform_draws_stay_in_range::<BFloat16>();
}

#[test]
fn bf16_arithmetic_shares_one_draw() {
    arithmetic_shares_one_draw::<BFloat16>();
}

#[test]
fn bf16_comparison_yields_the_boolean_carrier() {
    comparison_yields_the_boolean_carrier::<BFloat16>(0.1);
}

#[test]
fn bf16_bernoulli_recovers_its_parameter() {
    bernoulli_recovers_its_parameter::<BFloat16>(0.03);
}

#[test]
fn bf16_logic_works_on_the_boolean_carrier() {
    logic_works_on_the_boolean_carrier::<BFloat16>();
}

#[test]
fn bf16_both_verdict_algebras_hold() {
    both_verdict_algebras_hold::<BFloat16>();
}

#[test]
fn bf16_maybe_uncertain_gates_at_the_scalar() {
    maybe_uncertain_gates_at_the_scalar::<BFloat16>();
}

#[test]
fn bf16_qmc_runs_at_the_scalar() {
    qmc_runs_at_the_scalar::<BFloat16>(0.05);
}

#[test]
fn bf16_a_seed_reproduces_its_draws() {
    a_seed_reproduces_its_draws::<BFloat16>();
}

// ---------------------------------------------------------------------------------------------
// The two scalars that shipped before the retrofit still work, unchanged.
// ---------------------------------------------------------------------------------------------

#[test]
fn f64_still_works() {
    point_is_exact::<f64>();
    normal_moments_are_recovered::<f64>(0.02);
    arithmetic_shares_one_draw::<f64>();
    both_verdict_algebras_hold::<f64>();
    maybe_uncertain_gates_at_the_scalar::<f64>();
    a_seed_reproduces_its_draws::<f64>();
}

#[test]
fn f106_still_works() {
    use deep_causality_num::Float106;
    point_is_exact::<Float106>();
    normal_moments_are_recovered::<Float106>(0.02);
    arithmetic_shares_one_draw::<Float106>();
    both_verdict_algebras_hold::<Float106>();
    maybe_uncertain_gates_at_the_scalar::<Float106>();
    a_seed_reproduces_its_draws::<Float106>();
}

/// One scalar drives both carriers and a `MaybeUncertain`, and all three agree at one index.
///
/// This is the shape the carrier split exists for: the presence channel is `UncertainBool<R>`,
/// the value channel is `Uncertain<R>`, and they share the scalar because they share the tree.
#[test]
fn bf16_one_graph_two_carriers_one_index() {
    let session = SampleSession::seeded(4711);
    let x = Uncertain::<BFloat16>::normal(at(10.0), at(1.0));
    let is_big: UncertainBool<BFloat16> = x.greater_than(at(10.0));

    for index in 0..64 {
        let drawn = x.sample_at(&session, index).expect("draw");
        let verdict = is_big.sample_at(&session, index).expect("draw");
        assert_eq!(
            verdict,
            drawn > at::<BFloat16>(10.0),
            "the comparison disagreed with the draw it was taken from, at index {index}"
        );
    }
}
