/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Phase-2 suite for `entropy` and `conditional_entropy`.
//!
//! Written before the implementation: every test here fails with the phase-1 `unimplemented!`
//! panic until `src/algorithms/entropy.rs` has a body.
//!
//! # Where the expected values come from
//!
//! Shannon entropy in bits is `H = −Σ pᵢ log2 pᵢ`. Every expectation below is one of:
//!
//! * that sum **evaluated by hand** on a distribution whose entries are negative powers of two,
//!   where `log2 2⁻ᵏ = −k` is an integer and the sum reduces to a short exact addition. The
//!   derivation is written out beside each literal so the arithmetic can be rechecked;
//! * a published constant (`ln 2`, `log2 3`), cited at the point of use;
//! * an algebraic invariant (`H(X | Y) = H(X)` under independence, `H ≥ 0`, `H ≤ log2 n`,
//!   `H_nats = H_bits · ln 2`); or
//! * a property over a generated family.
//!
//! No expectation is produced by calling the function under test, none is taken from the three
//! shipped implementations this crate absorbs, and none retypes the implementation's formula —
//! `log2` is never called inside an assertion.
//!
//! # Precision
//!
//! Every numeric check runs at `f32`, `f64` and `Float106` through one generic helper, each with
//! its own tolerance. Two tolerance families are needed:
//!
//! * **exact.** The expectation is a dyadic rational, exactly representable in all three types, so
//!   the tolerance only has to absorb summation order.
//! * **irrational.** The expectation is irrational, so the fixture crosses into `T` from an `f64`
//!   literal ([`lift`]). At `Float106` it is then the *fixture*, not the arithmetic, that caps
//!   agreement at roughly `1e-16`; those tests use the looser family and say so where they use it.
//!
//! # Corner cases — the change's phase-2 table
//!
//! | Row | Case | Covered by |
//! |---|---|---|
//! | A | Empty input | `empty_input_is_refused`, `conditional_empty_inputs_are_refused` |
//! | B | Single element | `single_element_entropy` |
//! | C | Two quantities coincide | `uniform_entropy_is_log2_n` (uniform), `conditional_entropy_of_a_distribution_with_itself_is_zero` |
//! | D | An index expression degenerates | **n/a.** The surface is a flat slice: there is no stride, no `a[i*n + j]`, and no index arithmetic that can collapse. The nearest analogue — two arguments that coincide — is row C and is covered there. |
//! | E | Each documented threshold, both sides | `skip_below_threshold_at_and_either_side`, `by_sum_floor_at_and_either_side` |
//! | F | Zero | `zero_entry_contributes_nothing`, `by_sum_of_an_all_zero_input_is_zero` |
//! | G | Negative | `negative_entry_is_refused`, `conditional_negative_entry_is_refused` |
//! | H | Exact domain boundary (p = 0 or p = 1) | `degenerate_distribution_has_exactly_zero_entropy`, `negative_zero_entry_is_treated_as_zero` |
//! | I | Non-finite | `non_finite_entry_is_refused`, `conditional_non_finite_entry_is_refused` |
//! | J | Overflow / underflow reach at the type's own extremes | `smallest_positive_entry_stays_finite` (`T::epsilon()`, per type), `uniform_entropy_over_a_thousand_outcomes` (summation reach) |
//! | K | Every numeric test at `f32`, `f64` and `Float106` | every `check_*` helper is called from three `#[test]`s with its own tolerance |

use core::fmt::Debug;
use deep_causality_algebra::{Real, RealField};
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::utils_tests::precision::{F32, F64, F106};
use deep_causality_stats::utils_tests::{lift_array, uniform};
use deep_causality_stats::{
    EntropyConfig, LogBase, Normalisation, StatsError, StatsErrorEnum, ZeroPolicy,
    conditional_entropy, entropy,
};

// ---------------------------------------------------------------------------------------------
// Small helpers. Neither calls a function under test nor shares an expression with one.
// ---------------------------------------------------------------------------------------------

fn approx_eq<T: Real + RealField>(actual: T, expected: T, tol: T) -> bool {
    (actual - expected).abs() <= tol
}

fn assert_empty_input(err: &StatsError) {
    assert!(
        matches!(err.0, StatsErrorEnum::EmptyInput(_)),
        "expected StatsErrorEnum::EmptyInput, got {err:?}"
    );
}

fn assert_negative_probability(err: &StatsError) {
    assert!(
        matches!(err.0, StatsErrorEnum::NegativeProbability(_)),
        "expected StatsErrorEnum::NegativeProbability, got {err:?}"
    );
}

fn assert_non_finite(err: &StatsError) {
    assert!(
        matches!(err.0, StatsErrorEnum::NonFiniteInput(_)),
        "expected StatsErrorEnum::NonFiniteInput, got {err:?}"
    );
}

// −∞ is both non-finite and negative. The docs name a typed refusal for each and do not say which
// check runs first, so this suite accepts either variant and states the reading here.
fn assert_non_finite_or_negative(err: &StatsError) {
    assert!(
        matches!(
            err.0,
            StatsErrorEnum::NonFiniteInput(_) | StatsErrorEnum::NegativeProbability(_)
        ),
        "expected NonFiniteInput or NegativeProbability, got {err:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Row C — the uniform distribution, where the answer is an integer number of bits
// ---------------------------------------------------------------------------------------------

/// `H(uniform on n) = −Σⁿ (1/n) log2(1/n) = log2 n`.
///
/// Hand-evaluated at the four `n` that are powers of two, so `log2 n` is an integer:
/// `2 = 2¹ → 1`, `4 = 2² → 2`, `8 = 2³ → 3`, `16 = 2⁴ → 4`. Each `1/n` is exact in binary at
/// every precision, so the sum is `n` copies of `(1/n)·k`, which is `k` exactly.
fn check_uniform_entropy_is_log2_n<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    // (n, hand-evaluated log2 n)
    let cases = [(2usize, 1.0f64), (4, 2.0), (8, 3.0), (16, 4.0)];

    for (n, expected) in cases {
        let p = uniform::<T>(n);
        let h = entropy(&p, &config).expect("a uniform distribution is a valid input");
        assert!(
            approx_eq(h, lift::<T>(expected), tol),
            "H(uniform on {n}) is {expected} bits, got {h:?}"
        );
    }
}

#[test]
fn uniform_entropy_is_log2_n_f32() {
    check_uniform_entropy_is_log2_n::<f32>(lift(F32.native));
}

#[test]
fn uniform_entropy_is_log2_n_f64() {
    check_uniform_entropy_is_log2_n::<f64>(lift(F64.native));
}

#[test]
fn uniform_entropy_is_log2_n_f106() {
    check_uniform_entropy_is_log2_n::<Float106>(lift(F106.native));
}

/// Row J, the summation-reach half: 1024 terms, each `(1/1024)·10`, summing to exactly 10 bits.
///
/// `1024 = 2¹⁰`, so `log2 1024 = 10`. The point is not the value but the reach: every term is
/// `9.765625e-3` while the accumulator climbs to 10, which is where a naive `f32` fold loses
/// digits. The tolerance is per precision for exactly that reason.
fn check_uniform_entropy_over_a_thousand_outcomes<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();
    let p = uniform::<T>(1024);
    let h = entropy(&p, &config).expect("a uniform distribution is a valid input");

    // Hand-evaluated: 1024 = 2^10, so H = log2 1024 = 10 bits.
    assert!(
        approx_eq(h, lift::<T>(10.0), tol),
        "H(uniform on 1024) is 10 bits, got {h:?}"
    );
}

#[test]
fn uniform_entropy_over_a_thousand_outcomes_f32() {
    check_uniform_entropy_over_a_thousand_outcomes::<f32>(lift(F32.reduction));
}

#[test]
fn uniform_entropy_over_a_thousand_outcomes_f64() {
    check_uniform_entropy_over_a_thousand_outcomes::<f64>(lift(F64.reduction));
}

#[test]
fn uniform_entropy_over_a_thousand_outcomes_f106() {
    check_uniform_entropy_over_a_thousand_outcomes::<Float106>(lift(F106.reduction));
}

/// A uniform distribution on three outcomes, where the answer is not a dyadic rational.
///
/// `H(uniform on 3) = log2 3 = 1.584962500721156181453738943947...`, the published binary
/// logarithm of three (OEIS A020857). This is the input where `1/3` is itself inexact in binary,
/// so it exercises a path the powers-of-two cases cannot: the irrational tolerance family applies,
/// and at `Float106` the `f64` fixture caps agreement rather than the arithmetic.
fn check_uniform_entropy_on_three_outcomes<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();
    let p = uniform::<T>(3);
    let h = entropy(&p, &config).expect("a uniform distribution is a valid input");

    // log2 3, published constant (OEIS A020857).
    assert!(
        approx_eq(h, lift::<T>(1.5849625007211562), tol),
        "H(uniform on 3) is log2 3 = 1.5849625007211562 bits, got {h:?}"
    );
}

#[test]
fn uniform_entropy_on_three_outcomes_f32() {
    check_uniform_entropy_on_three_outcomes::<f32>(lift(F32.literal));
}

#[test]
fn uniform_entropy_on_three_outcomes_f64() {
    check_uniform_entropy_on_three_outcomes::<f64>(lift(F64.literal));
}

#[test]
fn uniform_entropy_on_three_outcomes_f106() {
    check_uniform_entropy_on_three_outcomes::<Float106>(lift(F106.literal));
}

// ---------------------------------------------------------------------------------------------
// Rows H, B, F — the domain boundary, one element, and an entry of zero
// ---------------------------------------------------------------------------------------------

/// Row H. All mass on one outcome: every probability is exactly 0 or exactly 1, and the entropy is
/// exactly zero, not merely near it.
///
/// Hand-evaluated: `−1·log2 1 = 0`, and `lim(p → 0) p log p = 0` is the zero policy's whole
/// subject, so the three zero entries contribute nothing. The sum of exact zeros is an exact zero
/// at every precision, so this is asserted as equality rather than within a tolerance.
fn check_degenerate_distribution_has_exactly_zero_entropy<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let p = lift_array::<T>(&[1.0, 0.0, 0.0, 0.0]);
    let h = entropy(&p, &config).expect("a degenerate distribution is a valid input");
    assert!(h == T::zero(), "H(1, 0, 0, 0) is exactly 0 bits, got {h:?}");

    // The mass need not sit first, and a two-outcome degenerate case is the same statement.
    let q = lift_array::<T>(&[0.0, 1.0]);
    let h_q = entropy(&q, &config).expect("a degenerate distribution is a valid input");
    assert!(h_q == T::zero(), "H(0, 1) is exactly 0 bits, got {h_q:?}");

    // Nats is a positive multiple of bits, so exactly zero is exactly zero in either unit.
    let nats = EntropyConfig::<T>::nats();
    let h_nats = entropy(&p, &nats).expect("a degenerate distribution is a valid input");
    assert!(
        h_nats == T::zero(),
        "H(1, 0, 0, 0) is exactly 0 nats, got {h_nats:?}"
    );
}

#[test]
fn degenerate_distribution_has_exactly_zero_entropy_f32() {
    check_degenerate_distribution_has_exactly_zero_entropy::<f32>();
}

#[test]
fn degenerate_distribution_has_exactly_zero_entropy_f64() {
    check_degenerate_distribution_has_exactly_zero_entropy::<f64>();
}

#[test]
fn degenerate_distribution_has_exactly_zero_entropy_f106() {
    check_degenerate_distribution_has_exactly_zero_entropy::<Float106>();
}

/// Row B. One element.
///
/// Two readings are pinned, so the single-element case is not tested only where it vanishes:
///
/// * `[1.0]` is the sole outcome of a certain experiment: `−1·log2 1 = 0`, exactly zero.
/// * `[0.25]` under `Normalisation::None` is a caller handing in an entry that does not sum to
///   one; the documented behaviour of `None` is to use the input as given, so the answer is the
///   plain sum `−0.25·log2 0.25 = 0.25·2 = 0.5`. That is the non-vanishing pin: a dropped factor
///   or a wrong sign passes the `[1.0]` case and fails this one.
fn check_single_element_entropy<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let certain = lift_array::<T>(&[1.0]);
    let h = entropy(&certain, &config).expect("one certain outcome is a valid input");
    assert!(h == T::zero(), "H(1) is exactly 0 bits, got {h:?}");

    // Hand-evaluated: −0.25 · log2 0.25 = −0.25 · (−2) = 0.5.
    let quarter = lift_array::<T>(&[0.25]);
    let h_quarter = entropy(&quarter, &config).expect("a one-entry slice is a valid input");
    assert!(
        approx_eq(h_quarter, lift::<T>(0.5), tol),
        "the sum −Σ p log2 p over [0.25] is 0.5, got {h_quarter:?}"
    );
}

#[test]
fn single_element_entropy_f32() {
    check_single_element_entropy::<f32>(lift(F32.native));
}

#[test]
fn single_element_entropy_f64() {
    check_single_element_entropy::<f64>(lift(F64.native));
}

#[test]
fn single_element_entropy_f106() {
    check_single_element_entropy::<Float106>(lift(F106.native));
}

/// Row F. An entry of exactly zero contributes nothing under `SkipZero`.
///
/// Hand-evaluated: `H(0.5, 0.5) = 0.5·1 + 0.5·1 = 1` bit, because `log2 0.5 = −1`. Adding an
/// outcome of probability zero leaves the distribution unchanged, so the answer is still 1 bit —
/// and it is still 1 bit with three of them. The invariant beside the literal is that the two
/// inputs agree.
fn check_zero_entry_contributes_nothing<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let without = lift_array::<T>(&[0.5, 0.5]);
    let with_one = lift_array::<T>(&[0.5, 0.5, 0.0]);
    let with_three = lift_array::<T>(&[0.0, 0.5, 0.0, 0.5, 0.0]);

    let h_without = entropy(&without, &config).expect("a fair coin is a valid input");
    let h_with_one = entropy(&with_one, &config).expect("a zero entry is a valid input");
    let h_with_three = entropy(&with_three, &config).expect("zero entries are a valid input");

    // Hand-evaluated: −(0.5 log2 0.5 + 0.5 log2 0.5) = 0.5 + 0.5 = 1 bit.
    assert!(
        approx_eq(h_without, lift::<T>(1.0), tol),
        "H(0.5, 0.5) is 1 bit, got {h_without:?}"
    );
    assert!(
        approx_eq(h_with_one, lift::<T>(1.0), tol),
        "a zero outcome adds nothing, so H(0.5, 0.5, 0) is 1 bit, got {h_with_one:?}"
    );
    assert!(
        approx_eq(h_with_three, lift::<T>(1.0), tol),
        "three zero outcomes add nothing, so H is 1 bit, got {h_with_three:?}"
    );
}

#[test]
fn zero_entry_contributes_nothing_f32() {
    check_zero_entry_contributes_nothing::<f32>(lift(F32.native));
}

#[test]
fn zero_entry_contributes_nothing_f64() {
    check_zero_entry_contributes_nothing::<f64>(lift(F64.native));
}

#[test]
fn zero_entry_contributes_nothing_f106() {
    check_zero_entry_contributes_nothing::<Float106>(lift(F106.native));
}

/// Negative zero is zero, not a negative entry.
///
/// The reading asserted here, since the docs speak of "a negative entry" without naming a sign
/// bit: negativity is decided by the ordering, and `−0.0 == 0.0` holds in it. So `−0.0` is skipped
/// as a zero rather than refused, and the answer is the 1 bit of a fair coin, hand-evaluated as
/// `0.5·1 + 0.5·1`.
fn check_negative_zero_entry_is_treated_as_zero<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();
    let p = lift_array::<T>(&[0.5, 0.5, -0.0]);

    let h = entropy(&p, &config).expect("negative zero is not a negative probability");
    assert!(
        approx_eq(h, lift::<T>(1.0), tol),
        "H(0.5, 0.5, −0.0) is 1 bit, got {h:?}"
    );
}

#[test]
fn negative_zero_entry_is_treated_as_zero_f32() {
    check_negative_zero_entry_is_treated_as_zero::<f32>(lift(F32.native));
}

#[test]
fn negative_zero_entry_is_treated_as_zero_f64() {
    check_negative_zero_entry_is_treated_as_zero::<f64>(lift(F64.native));
}

#[test]
fn negative_zero_entry_is_treated_as_zero_f106() {
    check_negative_zero_entry_is_treated_as_zero::<Float106>(lift(F106.native));
}

// ---------------------------------------------------------------------------------------------
// The base: the axis on which two shipped implementations disagree about the unit of the answer
// ---------------------------------------------------------------------------------------------

/// The two bases differ by exactly `ln 2`: `H_nats = H_bits · ln 2` on the same input.
///
/// The input is `[0.5, 0.25, 0.25]`, whose entropy in bits is hand-evaluated as
/// `0.5·1 + 0.25·2 + 0.25·2 = 0.5 + 0.5 + 0.5 = 1.5`, using `log2 2⁻¹ = −1` and `log2 2⁻² = −2`.
///
/// `ln 2 = 0.693147180559945309417232121458...` is the published constant (OEIS A002162), so the
/// answer in nats is `1.5 · ln 2 = 1.039720770839917964125848182187...`, again by hand.
///
/// Both literals are asserted independently, and the ratio between the two calls is asserted
/// beside them as the base-conversion invariant. Either literal alone would catch a base swap; the
/// invariant catches a conversion applied in the wrong direction.
fn check_the_two_bases_differ_by_ln_two<T>(tol_exact: T, tol_irrational: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let p = lift_array::<T>(&[0.5, 0.25, 0.25]);

    let bits = EntropyConfig::<T>::bits();
    let nats = EntropyConfig::<T>::nats();

    let h_bits = entropy(&p, &bits).expect("a distribution is a valid input");
    let h_nats = entropy(&p, &nats).expect("a distribution is a valid input");

    // Hand-evaluated: 0.5·1 + 0.25·2 + 0.25·2 = 1.5 bits.
    assert!(
        approx_eq(h_bits, lift::<T>(1.5), tol_exact),
        "H(0.5, 0.25, 0.25) is 1.5 bits, got {h_bits:?}"
    );

    // Hand-evaluated: 1.5 × 0.693147180559945309 = 1.039720770839917964 nats. The literal is
    // that value in its shortest form that round-trips through `f64`.
    assert!(
        approx_eq(h_nats, lift::<T>(1.039_720_770_839_918), tol_irrational),
        "H(0.5, 0.25, 0.25) is 1.039720770839917964 nats, got {h_nats:?}"
    );

    // ln 2 = 0.693147180559945309417232121458..., taken from the standard library's named
    // constant rather than retyped as digits (OEIS A002162 carries the same expansion).
    let ln_two = lift::<T>(core::f64::consts::LN_2);
    assert!(
        approx_eq(h_nats, h_bits * ln_two, tol_irrational),
        "nats = bits × ln 2, got {h_nats:?} against {h_bits:?} × ln 2"
    );

    // The two are genuinely different numbers, not the same answer relabelled: they differ by
    // 1.5 − 1.0397207708399180 = 0.460279229160082, which is far outside any tolerance here.
    assert!(
        (h_bits - h_nats).abs() > lift::<T>(0.4),
        "bits and nats are a factor of ln 2 apart, got {h_bits:?} and {h_nats:?}"
    );
}

#[test]
fn the_two_bases_differ_by_ln_two_f32() {
    check_the_two_bases_differ_by_ln_two::<f32>(lift(F32.native), lift(F32.literal));
}

#[test]
fn the_two_bases_differ_by_ln_two_f64() {
    check_the_two_bases_differ_by_ln_two::<f64>(lift(F64.native), lift(F64.literal));
}

#[test]
fn the_two_bases_differ_by_ln_two_f106() {
    check_the_two_bases_differ_by_ln_two::<Float106>(lift(F106.native), lift(F106.literal));
}

/// Setting the base through `with_base` is the same statement as choosing the constructor.
///
/// The value is the uniform four-outcome case, hand-evaluated as `log2 4 = 2` bits because
/// `4 = 2²`; in nats the same distribution is `2 · ln 2 = 1.386294361119890618834464242916...`,
/// twice the published `ln 2` (OEIS A002162).
fn check_with_base_selects_the_unit<T>(tol_exact: T, tol_irrational: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let p = uniform::<T>(4);

    let as_bits = EntropyConfig::<T>::nats().with_base(LogBase::Bits);
    let as_nats = EntropyConfig::<T>::bits().with_base(LogBase::Nats);

    let h_bits = entropy(&p, &as_bits).expect("a uniform distribution is a valid input");
    let h_nats = entropy(&p, &as_nats).expect("a uniform distribution is a valid input");

    // Hand-evaluated: 4 = 2^2, so H = log2 4 = 2 bits.
    assert!(
        approx_eq(h_bits, lift::<T>(2.0), tol_exact),
        "H(uniform on 4) is 2 bits, got {h_bits:?}"
    );
    // Hand-evaluated: 2 × 0.693147180559945309 = 1.386294361119890618 nats.
    assert!(
        approx_eq(h_nats, lift::<T>(1.3862943611198906), tol_irrational),
        "H(uniform on 4) is 1.3862943611198906 nats, got {h_nats:?}"
    );
}

#[test]
fn with_base_selects_the_unit_f32() {
    check_with_base_selects_the_unit::<f32>(lift(F32.native), lift(F32.literal));
}

#[test]
fn with_base_selects_the_unit_f64() {
    check_with_base_selects_the_unit::<f64>(lift(F64.native), lift(F64.literal));
}

#[test]
fn with_base_selects_the_unit_f106() {
    check_with_base_selects_the_unit::<Float106>(lift(F106.native), lift(F106.literal));
}

// ---------------------------------------------------------------------------------------------
// The zero policy: the axis on which the shipped implementations put the cutoff in two places
// ---------------------------------------------------------------------------------------------

/// The two zero policies disagree on an entry that is positive but below the threshold.
///
/// The input is the dyadic distribution `[0.5, 0.25, 0.125, 0.0625, 0.0625]`, which sums to
/// exactly one. Hand-evaluated with `log2 2⁻ᵏ = −k`:
///
/// * `SkipZero` includes every strictly positive entry:
///   `0.5·1 + 0.25·2 + 0.125·3 + 0.0625·4 + 0.0625·4 = 0.5 + 0.5 + 0.375 + 0.25 + 0.25 = 1.875`.
/// * `SkipBelow(0.1)` omits the two entries at `0.0625`, which are positive and below `0.1`:
///   `0.5 + 0.5 + 0.375 = 1.375`.
///
/// The two answers differ by exactly `0.5`, which is the point of the parameter.
///
/// `Normalisation::None` is used deliberately so the two axes stay independent: the threshold is
/// compared against the entries as given, with no normalisation step in between to argue about.
fn check_the_two_zero_policies_differ<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let p = lift_array::<T>(&[0.5, 0.25, 0.125, 0.0625, 0.0625]);

    let skip_zero = EntropyConfig::<T>::bits();
    let skip_below = EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(0.1)));

    let h_skip_zero = entropy(&p, &skip_zero).expect("a distribution is a valid input");
    let h_skip_below = entropy(&p, &skip_below).expect("a distribution is a valid input");

    // Hand-evaluated above: 0.5 + 0.5 + 0.375 + 0.25 + 0.25 = 1.875 bits.
    assert!(
        approx_eq(h_skip_zero, lift::<T>(1.875), tol),
        "SkipZero keeps every positive entry: 1.875 bits, got {h_skip_zero:?}"
    );
    // Hand-evaluated above: 0.5 + 0.5 + 0.375 = 1.375 bits.
    assert!(
        approx_eq(h_skip_below, lift::<T>(1.375), tol),
        "SkipBelow(0.1) drops the two 0.0625 entries: 1.375 bits, got {h_skip_below:?}"
    );

    // Hand-evaluated: 1.875 − 1.375 = 0.5, the two dropped terms of 0.25 each.
    assert!(
        approx_eq(h_skip_zero - h_skip_below, lift::<T>(0.5), tol),
        "the policies differ by the two dropped terms, 0.5 bits, got {h_skip_zero:?} and {h_skip_below:?}"
    );
}

#[test]
fn the_two_zero_policies_differ_f32() {
    check_the_two_zero_policies_differ::<f32>(lift(F32.native));
}

#[test]
fn the_two_zero_policies_differ_f64() {
    check_the_two_zero_policies_differ::<f64>(lift(F64.native));
}

#[test]
fn the_two_zero_policies_differ_f106() {
    check_the_two_zero_policies_differ::<Float106>(lift(F106.native));
}

/// Row E for `ZeroPolicy::SkipBelow`: at the threshold, just below it and just above it.
///
/// The same dyadic input `[0.5, 0.25, 0.125, 0.0625, 0.0625]` is measured at three thresholds
/// placed around the value `0.0625` of its two smallest entries. The documented rule is "omit
/// entries at or below `threshold`", so:
///
/// * threshold `0.06`, below the entries: both are kept, `H = 1.875` (hand-evaluated as
///   `0.5 + 0.5 + 0.375 + 0.25 + 0.25`);
/// * threshold `0.0625`, exactly at the entries: both are omitted — this is the side the word
///   "at" decides — `H = 1.375` (`0.5 + 0.5 + 0.375`);
/// * threshold `0.07`, above the entries: both are omitted, `H = 1.375`.
///
/// A `<` written where the documented `<=` belongs shows up only in the middle case.
fn check_skip_below_threshold_at_and_either_side<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let p = lift_array::<T>(&[0.5, 0.25, 0.125, 0.0625, 0.0625]);

    let below = EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(0.06)));
    let at = EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(0.0625)));
    let above = EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(0.07)));

    let h_below = entropy(&p, &below).expect("a distribution is a valid input");
    let h_at = entropy(&p, &at).expect("a distribution is a valid input");
    let h_above = entropy(&p, &above).expect("a distribution is a valid input");

    assert!(
        approx_eq(h_below, lift::<T>(1.875), tol),
        "a threshold under the entries keeps them: 1.875 bits, got {h_below:?}"
    );
    assert!(
        approx_eq(h_at, lift::<T>(1.375), tol),
        "an entry exactly at the threshold is omitted: 1.375 bits, got {h_at:?}"
    );
    assert!(
        approx_eq(h_above, lift::<T>(1.375), tol),
        "a threshold over the entries omits them: 1.375 bits, got {h_above:?}"
    );
}

#[test]
fn skip_below_threshold_at_and_either_side_f32() {
    check_skip_below_threshold_at_and_either_side::<f32>(lift(F32.native));
}

#[test]
fn skip_below_threshold_at_and_either_side_f64() {
    check_skip_below_threshold_at_and_either_side::<f64>(lift(F64.native));
}

#[test]
fn skip_below_threshold_at_and_either_side_f106() {
    check_skip_below_threshold_at_and_either_side::<Float106>(lift(F106.native));
}

/// The same disagreement at the scale the corner-case table names: an entry near machine residue.
///
/// The input is `[0.5, 0.5, 2⁻²⁰]`. Under `Normalisation::None` the input is used as given, and it
/// deliberately sums to `1 + 2⁻²⁰` rather than to one: the question here is which terms enter the
/// sum, not whether the caller normalised.
///
/// Hand-evaluated with `2⁻²⁰ = 9.5367431640625e-7` and `log2 2⁻²⁰ = −20`:
///
/// * `SkipZero`: `0.5·1 + 0.5·1 + 20·2⁻²⁰ = 1 + 20/1048576 = 1.000019073486328125`.
///   That value is exact in `f32` too — `20/1048576` is exactly 160 ulps at 1.0.
/// * `SkipBelow(1e-3)`: the tiny entry is positive but below the threshold, so it is dropped and
///   the answer is exactly `1.0`.
fn check_an_entry_below_epsilon_separates_the_policies<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    // 2^-20, written as its exact decimal expansion.
    let p = lift_array::<T>(&[0.5, 0.5, 9.5367431640625e-7]);

    let skip_zero = EntropyConfig::<T>::bits();
    let skip_below = EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(1e-3)));

    let h_skip_zero = entropy(&p, &skip_zero).expect("a positive entry is a valid input");
    let h_skip_below = entropy(&p, &skip_below).expect("a positive entry is a valid input");

    // Hand-evaluated above: 1 + 20 × 2^-20 = 1 + 20/1048576 = 1.000019073486328125 exactly. The
    // literal is that value in its shortest form that round-trips through `f64`; both name the
    // same number, since 20/1048576 = 5·2^-18 is exact in binary.
    assert!(
        approx_eq(h_skip_zero, lift::<T>(1.000_019_073_486_328_1), tol),
        "SkipZero includes the 2^-20 entry: 1.000019073486328125 bits, got {h_skip_zero:?}"
    );
    // Hand-evaluated: the tiny entry is dropped, leaving 0.5·1 + 0.5·1 = 1 bit.
    assert!(
        approx_eq(h_skip_below, lift::<T>(1.0), tol),
        "SkipBelow(1e-3) drops the 2^-20 entry: 1 bit, got {h_skip_below:?}"
    );

    // And the two results are genuinely different: they differ by 20 × 2^-20 = 1.9073486328125e-5,
    // which is more than an order of magnitude above the tightest tolerance used at f32.
    assert!(
        h_skip_zero > h_skip_below,
        "the included term is positive, so SkipZero is the larger: got {h_skip_zero:?} and {h_skip_below:?}"
    );
}

#[test]
fn an_entry_below_epsilon_separates_the_policies_f32() {
    check_an_entry_below_epsilon_separates_the_policies::<f32>(lift(F32.native));
}

#[test]
fn an_entry_below_epsilon_separates_the_policies_f64() {
    check_an_entry_below_epsilon_separates_the_policies::<f64>(lift(F64.native));
}

#[test]
fn an_entry_below_epsilon_separates_the_policies_f106() {
    check_an_entry_below_epsilon_separates_the_policies::<Float106>(lift(F106.native));
}

/// Row J, the underflow half, taken at each type's own extreme rather than at a fixed constant.
///
/// The input is `[1.0, T::epsilon()]`, whose second entry is `1.19e-7` at `f32`, `2.22e-16` at
/// `f64` and `4.93e-32` at `Float106`. The term `ε·log2(1/ε)` is therefore about `2.8e-6`,
/// `1.2e-14` and `5.1e-30` respectively — three different orders, which is why this is a bound
/// rather than a literal.
///
/// What is asserted: the answer is finite, it is not negative, and it stays below `1e-4`, a bound
/// satisfied at every precision because the largest of the three terms is the `f32` one at
/// `2.8e-6`. An implementation that lets `log2 ε` reach the sum unmultiplied returns something
/// near `−23`, `−52` or `−104` and fails all three parts.
fn check_smallest_positive_entry_stays_finite<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();
    let p = [T::one(), T::epsilon()];

    let h = entropy(&p, &config).expect("a positive entry is a valid input");

    assert!(h.is_finite(), "the entropy stays finite, got {h:?}");
    assert!(h >= T::zero(), "the entropy is not negative, got {h:?}");
    assert!(
        h <= lift::<T>(1e-4),
        "ε · log2(1/ε) is at most 2.8e-6 at f32 and smaller at the wider types, got {h:?}"
    );
}

#[test]
fn smallest_positive_entry_stays_finite_f32() {
    check_smallest_positive_entry_stays_finite::<f32>();
}

#[test]
fn smallest_positive_entry_stays_finite_f64() {
    check_smallest_positive_entry_stays_finite::<f64>();
}

#[test]
fn smallest_positive_entry_stays_finite_f106() {
    check_smallest_positive_entry_stays_finite::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// Normalisation: the axis on which one shipped implementation divides by the sum and one does not
// ---------------------------------------------------------------------------------------------

/// `BySum` on an input summing to 2.0 is the same answer as the normalised input under `None`.
///
/// The input is `[1.0, 0.5, 0.25, 0.25]`, whose sum is `1 + 0.5 + 0.25 + 0.25 = 2.0` exactly.
/// Dividing by it gives `[0.5, 0.25, 0.125, 0.125]`, hand-evaluated with `log2 2⁻ᵏ = −k` as
/// `0.5·1 + 0.25·2 + 0.125·3 + 0.125·3 = 0.5 + 0.5 + 0.375 + 0.375 = 1.75` bits.
///
/// Both the literal and the agreement between the two configurations are asserted: the literal
/// fixes the value, the agreement fixes that `BySum` divides by the sum rather than, say, by the
/// count.
fn check_by_sum_matches_the_prenormalised_input<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let unnormalised = lift_array::<T>(&[1.0, 0.5, 0.25, 0.25]);
    let prenormalised = lift_array::<T>(&[0.5, 0.25, 0.125, 0.125]);

    let by_sum =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() });
    let none = EntropyConfig::<T>::bits();

    let h_by_sum = entropy(&unnormalised, &by_sum).expect("a positive input is valid");
    let h_none = entropy(&prenormalised, &none).expect("a distribution is a valid input");

    // Hand-evaluated above: 0.5 + 0.5 + 0.375 + 0.375 = 1.75 bits.
    assert!(
        approx_eq(h_by_sum, lift::<T>(1.75), tol),
        "BySum over an input summing to 2.0 is 1.75 bits, got {h_by_sum:?}"
    );
    assert!(
        approx_eq(h_none, lift::<T>(1.75), tol),
        "the same distribution given normalised is 1.75 bits, got {h_none:?}"
    );
    assert!(
        approx_eq(h_by_sum, h_none, tol),
        "normalising by the sum reproduces the prenormalised input, got {h_by_sum:?} and {h_none:?}"
    );

    // Without normalisation the same unnormalised input is a different number, so the test cannot
    // pass by ignoring the parameter. Hand-evaluated:
    // −(1·log2 1) − 0.5·log2 0.5 − 0.25·log2 0.25 − 0.25·log2 0.25 = 0 + 0.5 + 0.5 + 0.5 = 1.5.
    let h_unnormalised = entropy(&unnormalised, &none).expect("a positive input is valid");
    assert!(
        approx_eq(h_unnormalised, lift::<T>(1.5), tol),
        "the same slice read as given is 1.5 bits, got {h_unnormalised:?}"
    );
}

#[test]
fn by_sum_matches_the_prenormalised_input_f32() {
    check_by_sum_matches_the_prenormalised_input::<f32>(lift(F32.native));
}

#[test]
fn by_sum_matches_the_prenormalised_input_f64() {
    check_by_sum_matches_the_prenormalised_input::<f64>(lift(F64.native));
}

#[test]
fn by_sum_matches_the_prenormalised_input_f106() {
    check_by_sum_matches_the_prenormalised_input::<Float106>(lift(F106.native));
}

/// Row E for `Normalisation::BySum`: the floor at the sum, below it and above it.
///
/// The input is `[0.25, 0.25]`, whose sum is exactly `0.5`. The documented rule is that a sum "at
/// or below `floor`" carries no mass and the result is zero.
///
/// * floor `0.25`, below the sum: the input normalises to `[0.5, 0.5]`, whose entropy is
///   hand-evaluated as `0.5·1 + 0.5·1 = 1` bit;
/// * floor `0.5`, exactly at the sum: zero — this is the side the word "at" decides;
/// * floor `0.6`, above the sum: zero.
///
/// The first case is what keeps the other two from being a test that only pins a vanishing
/// quantity.
fn check_by_sum_floor_at_and_either_side<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let p = lift_array::<T>(&[0.25, 0.25]);

    let under =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: lift(0.25) });
    let at =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: lift(0.5) });
    let over =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: lift(0.6) });

    let h_under = entropy(&p, &under).expect("a positive input is valid");
    let h_at = entropy(&p, &at).expect("a sum at the floor is a valid input");
    let h_over = entropy(&p, &over).expect("a sum below the floor is a valid input");

    // Hand-evaluated: [0.25, 0.25] / 0.5 = [0.5, 0.5], and 0.5·1 + 0.5·1 = 1 bit.
    assert!(
        approx_eq(h_under, lift::<T>(1.0), tol),
        "a sum above the floor normalises to a fair coin: 1 bit, got {h_under:?}"
    );
    assert!(
        h_at == T::zero(),
        "a sum exactly at the floor carries no mass: exactly 0, got {h_at:?}"
    );
    assert!(
        h_over == T::zero(),
        "a sum below the floor carries no mass: exactly 0, got {h_over:?}"
    );
}

#[test]
fn by_sum_floor_at_and_either_side_f32() {
    check_by_sum_floor_at_and_either_side::<f32>(lift(F32.native));
}

#[test]
fn by_sum_floor_at_and_either_side_f64() {
    check_by_sum_floor_at_and_either_side::<f64>(lift(F64.native));
}

#[test]
fn by_sum_floor_at_and_either_side_f106() {
    check_by_sum_floor_at_and_either_side::<Float106>(lift(F106.native));
}

/// Row F, the other half: an input of all zeros under `BySum` sums to zero.
///
/// With `floor = 0` the sum is at the floor, so the documented answer is zero rather than a
/// division by zero. An implementation that divides first returns `NaN` and fails the equality;
/// one that returns the fold's identity without checking passes here and fails the case above,
/// which is why the two travel together.
fn check_by_sum_of_an_all_zero_input_is_zero<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let p = lift_array::<T>(&[0.0, 0.0, 0.0]);
    let config =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() });

    let h = entropy(&p, &config).expect("an all-zero input carries no mass, it is not an error");
    assert!(
        h == T::zero(),
        "an input with no mass has exactly zero entropy, got {h:?}"
    );
    assert!(!h.is_nan(), "no division by the zero sum, got {h:?}");
}

#[test]
fn by_sum_of_an_all_zero_input_is_zero_f32() {
    check_by_sum_of_an_all_zero_input_is_zero::<f32>();
}

#[test]
fn by_sum_of_an_all_zero_input_is_zero_f64() {
    check_by_sum_of_an_all_zero_input_is_zero::<f64>();
}

#[test]
fn by_sum_of_an_all_zero_input_is_zero_f106() {
    check_by_sum_of_an_all_zero_input_is_zero::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// Properties over generated families
// ---------------------------------------------------------------------------------------------

/// Among the distributions on `n` outcomes, entropy is maximised by the uniform one.
///
/// The maximum for `n = 4` is `log2 4 = 2` bits, hand-evaluated because `4 = 2²`. Every other
/// member of the family is strictly below it, and the margin `0.05` is comfortably inside the
/// closest case: the nearest member here is `[0.5, 0.25, 0.125, 0.125]` at `1.75` bits, which is
/// the value hand-evaluated in `check_by_sum_matches_the_prenormalised_input`.
///
/// This is the classic maximum-entropy statement, so it catches a sign error, a missing negation
/// or a base swap in a way no single value does.
fn check_uniform_maximises_entropy<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let uniform_four = uniform::<T>(4);
    let h_uniform = entropy(&uniform_four, &config).expect("a uniform distribution is valid");

    // Hand-evaluated: 4 = 2^2, so the maximum is log2 4 = 2 bits.
    assert!(
        approx_eq(h_uniform, lift::<T>(2.0), tol),
        "H(uniform on 4) is 2 bits, got {h_uniform:?}"
    );

    let family = [
        lift_array::<T>(&[0.5, 0.25, 0.125, 0.125]),
        lift_array::<T>(&[0.7, 0.1, 0.1, 0.1]),
        lift_array::<T>(&[0.4, 0.3, 0.2, 0.1]),
        lift_array::<T>(&[0.97, 0.01, 0.01, 0.01]),
        lift_array::<T>(&[0.5, 0.5, 0.0, 0.0]),
        lift_array::<T>(&[1.0, 0.0, 0.0, 0.0]),
    ];

    for p in family.iter() {
        let h = entropy(p, &config).expect("a distribution on four outcomes is valid");
        assert!(
            h <= lift::<T>(2.0) + tol,
            "no distribution on four outcomes exceeds log2 4 = 2 bits, got {h:?}"
        );
        assert!(
            h < lift::<T>(1.95),
            "a non-uniform distribution is strictly under the uniform maximum, got {h:?}"
        );
    }
}

#[test]
fn uniform_maximises_entropy_f32() {
    check_uniform_maximises_entropy::<f32>(lift(F32.native));
}

#[test]
fn uniform_maximises_entropy_f64() {
    check_uniform_maximises_entropy::<f64>(lift(F64.native));
}

#[test]
fn uniform_maximises_entropy_f106() {
    check_uniform_maximises_entropy::<Float106>(lift(F106.native));
}

/// Entropy is never negative on a distribution.
///
/// Every member of the family sums to one, so each `pᵢ ≤ 1`, each `log2 pᵢ ≤ 0`, and each term
/// `−pᵢ log2 pᵢ` is non-negative. The bound is the invariant; no value is pinned here.
fn check_entropy_is_never_negative<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let bits = EntropyConfig::<T>::bits();
    let nats = EntropyConfig::<T>::nats();

    let family = [
        lift_array::<T>(&[0.5, 0.5]),
        lift_array::<T>(&[1.0, 0.0]),
        lift_array::<T>(&[0.9, 0.1]),
        lift_array::<T>(&[0.6, 0.4]),
        uniform::<T>(3),
        uniform::<T>(5),
        lift_array::<T>(&[0.5, 0.25, 0.125, 0.125]),
        lift_array::<T>(&[0.25, 0.25, 0.25, 0.125, 0.125]),
    ];

    for p in family.iter() {
        for config in [&bits, &nats] {
            let h = entropy(p, config).expect("a distribution is a valid input");
            assert!(
                h + tol >= T::zero(),
                "entropy of a distribution is never negative, got {h:?}"
            );
            assert!(
                h.is_finite(),
                "entropy of a distribution is finite, got {h:?}"
            );
        }
    }
}

#[test]
fn entropy_is_never_negative_f32() {
    check_entropy_is_never_negative::<f32>(lift(F32.native));
}

#[test]
fn entropy_is_never_negative_f64() {
    check_entropy_is_never_negative::<f64>(lift(F64.native));
}

#[test]
fn entropy_is_never_negative_f106() {
    check_entropy_is_never_negative::<Float106>(lift(F106.native));
}

// ---------------------------------------------------------------------------------------------
// Rows A, G, I — the refusals
// ---------------------------------------------------------------------------------------------

/// Row A. Empty input is refused, under every configuration.
///
/// The `BySum` case is the one worth stating: an empty slice sums to zero, which is at or below
/// any non-negative floor, so an implementation that normalises before validating returns zero and
/// reports success. The docstring refuses empty input, so the refusal comes first.
fn check_empty_input_is_refused<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let empty: [T; 0] = [];

    let configs = [
        EntropyConfig::<T>::bits(),
        EntropyConfig::<T>::nats(),
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(1e-3))),
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() }),
    ];

    for config in configs.iter() {
        let err = entropy(&empty, config).expect_err("an empty input has no distribution");
        assert_empty_input(&err);
    }
}

#[test]
fn empty_input_is_refused_f32() {
    check_empty_input_is_refused::<f32>();
}

#[test]
fn empty_input_is_refused_f64() {
    check_empty_input_is_refused::<f64>();
}

#[test]
fn empty_input_is_refused_f106() {
    check_empty_input_is_refused::<Float106>();
}

/// Row G. A negative entry is refused, and no zero policy interprets it.
///
/// The `SkipBelow` case is the one that matters: a negative number is below any positive
/// threshold, so an implementation that filters with the threshold rather than validating first
/// silently drops it and returns a plausible answer. The docstring says every function in the
/// crate rejects a negative entry, so the refusal survives the policy.
fn check_negative_entry_is_refused<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let with_negative = lift_array::<T>(&[0.5, -0.1, 0.6]);
    let mostly_negative = lift_array::<T>(&[-0.5, 1.5]);
    // Just below zero at the type's own scale, rather than at a fixed constant.
    let barely_negative = [T::one(), -T::epsilon()];

    let configs = [
        EntropyConfig::<T>::bits(),
        EntropyConfig::<T>::nats(),
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(1e-3))),
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() }),
    ];

    for config in configs.iter() {
        for p in [
            with_negative.as_slice(),
            mostly_negative.as_slice(),
            barely_negative.as_slice(),
        ] {
            let err = entropy(p, config).expect_err("a negative entry is not a probability");
            assert_negative_probability(&err);
        }
    }
}

#[test]
fn negative_entry_is_refused_f32() {
    check_negative_entry_is_refused::<f32>();
}

#[test]
fn negative_entry_is_refused_f64() {
    check_negative_entry_is_refused::<f64>();
}

#[test]
fn negative_entry_is_refused_f106() {
    check_negative_entry_is_refused::<Float106>();
}

/// Row I. A non-finite entry is refused.
///
/// The reading asserted, since the entropy docstring names only the empty and negative cases: the
/// crate declares `NonFiniteInput` for "an input carried a non-finite value where the statistic has
/// no meaning for one", and `NaN` is exactly that — it is neither a probability nor comparable
/// against a threshold, so every ordering test against it is false and it would otherwise pass
/// through both zero policies untouched and poison the sum.
///
/// `−∞` is both non-finite and negative; the docs do not say which check runs first, so either
/// typed refusal is accepted there.
fn check_non_finite_entry_is_refused<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let with_nan = [lift::<T>(0.5), T::nan(), lift::<T>(0.5)];
    let err = entropy(&with_nan, &config).expect_err("NaN is not a probability");
    assert_non_finite(&err);

    let with_inf = [lift::<T>(0.5), lift::<T>(f64::INFINITY)];
    let err = entropy(&with_inf, &config).expect_err("+∞ is not a probability");
    assert_non_finite(&err);

    let with_neg_inf = [lift::<T>(0.5), lift::<T>(f64::NEG_INFINITY)];
    let err = entropy(&with_neg_inf, &config).expect_err("−∞ is not a probability");
    assert_non_finite_or_negative(&err);

    // The same under normalisation, where a non-finite entry would otherwise reach the sum first.
    let by_sum =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() });
    let err = entropy(&with_nan, &by_sum).expect_err("NaN is not a probability");
    assert_non_finite(&err);
}

#[test]
fn non_finite_entry_is_refused_f32() {
    check_non_finite_entry_is_refused::<f32>();
}

#[test]
fn non_finite_entry_is_refused_f64() {
    check_non_finite_entry_is_refused::<f64>();
}

#[test]
fn non_finite_entry_is_refused_f106() {
    check_non_finite_entry_is_refused::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// Conditional entropy: H(X | Y) = H(X, Y) − H(Y)
// ---------------------------------------------------------------------------------------------

/// Under independence, conditioning tells you nothing: `H(X | Y) = H(X)`.
///
/// `X` has three outcomes with `p = [0.5, 0.25, 0.25]` and `Y` is a fair coin, `q = [0.5, 0.5]`.
/// Independence makes the joint the outer product, laid out row-major over `(x, y)`:
///
/// ```text
///   [0.5·0.5, 0.5·0.5, 0.25·0.5, 0.25·0.5, 0.25·0.5, 0.25·0.5]
/// = [0.25,    0.25,    0.125,    0.125,    0.125,    0.125   ]
/// ```
///
/// Hand-evaluated with `log2 2⁻ᵏ = −k`:
///
/// * `H(X, Y) = 0.25·2 + 0.25·2 + 4 × (0.125·3) = 0.5 + 0.5 + 1.5 = 2.5` bits;
/// * `H(Y) = 0.5·1 + 0.5·1 = 1` bit;
/// * `H(X | Y) = 2.5 − 1 = 1.5` bits, which is `H(X) = 0.5·1 + 0.25·2 + 0.25·2 = 1.5`.
///
/// The literal `1.5` is the closed form; the equality with `H(X)` is the independence invariant.
/// This case is deliberately not uniform, so the value is pinned somewhere the several candidate
/// formulas do not coincide.
fn check_conditional_entropy_equals_the_marginal_under_independence<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let joint = lift_array::<T>(&[0.25, 0.25, 0.125, 0.125, 0.125, 0.125]);
    let conditioning = lift_array::<T>(&[0.5, 0.5]);
    let marginal_x = lift_array::<T>(&[0.5, 0.25, 0.25]);

    let h_cond =
        conditional_entropy(&joint, &conditioning, &config).expect("a joint and its marginal");
    let h_x = entropy(&marginal_x, &config).expect("a distribution is a valid input");

    // Hand-evaluated above: 2.5 − 1 = 1.5 bits.
    assert!(
        approx_eq(h_cond, lift::<T>(1.5), tol),
        "H(X | Y) under independence is H(X) = 1.5 bits, got {h_cond:?}"
    );
    assert!(
        approx_eq(h_cond, h_x, tol),
        "H(X | Y) = H(X) under independence, got {h_cond:?} against {h_x:?}"
    );
}

#[test]
fn conditional_entropy_equals_the_marginal_under_independence_f32() {
    check_conditional_entropy_equals_the_marginal_under_independence::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_equals_the_marginal_under_independence_f64() {
    check_conditional_entropy_equals_the_marginal_under_independence::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_equals_the_marginal_under_independence_f106() {
    check_conditional_entropy_equals_the_marginal_under_independence::<Float106>(lift(F106.native));
}

/// The same statement where both variables are uniform, which is row C: two fair coins.
///
/// The joint is uniform on four cells, `[0.25, 0.25, 0.25, 0.25]`, and `Y` is `[0.5, 0.5]`.
/// Hand-evaluated: `H(X, Y) = log2 4 = 2` bits and `H(Y) = log2 2 = 1` bit, so
/// `H(X | Y) = 2 − 1 = 1` bit, which is `H(X)` for a fair coin.
///
/// Several formulas agree at the uniform point, which is why the non-uniform case above exists;
/// this one is here because the uniform point is where a stride or a count is most likely to be
/// wrong without showing.
fn check_conditional_entropy_under_uniform_independence<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let joint = uniform::<T>(4);
    let conditioning = uniform::<T>(2);

    let h_cond =
        conditional_entropy(&joint, &conditioning, &config).expect("a joint and its marginal");

    // Hand-evaluated above: 2 − 1 = 1 bit.
    assert!(
        approx_eq(h_cond, lift::<T>(1.0), tol),
        "H(X | Y) for two independent fair coins is 1 bit, got {h_cond:?}"
    );
}

#[test]
fn conditional_entropy_under_uniform_independence_f32() {
    check_conditional_entropy_under_uniform_independence::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_under_uniform_independence_f64() {
    check_conditional_entropy_under_uniform_independence::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_under_uniform_independence_f106() {
    check_conditional_entropy_under_uniform_independence::<Float106>(lift(F106.native));
}

/// Under deterministic dependence, conditioning tells you everything: `H(X | Y) = 0`.
///
/// Two cases, both with `X = Y` so the joint is diagonal:
///
/// * a fair coin: the joint is `[0.5, 0, 0, 0.5]` over the 2 × 2 cells, `H(X, Y) = 1` bit
///   (hand-evaluated as `0.5·1 + 0.5·1`, the zeros contributing nothing), and `H(Y) = 1` bit, so
///   the difference is `0`;
/// * a uniform four-outcome variable: the joint is the 4 × 4 diagonal of `0.25`s,
///   `H(X, Y) = log2 4 = 2` bits, `H(Y) = 2` bits, difference `0`.
///
/// Every entry is a negative power of two, so both entropies are exact at every precision and the
/// difference is exactly zero; the tolerance is here only for a summation order that reassociates.
fn check_conditional_entropy_is_zero_under_deterministic_dependence<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let coin_joint = lift_array::<T>(&[0.5, 0.0, 0.0, 0.5]);
    let coin_marginal = lift_array::<T>(&[0.5, 0.5]);
    let h_coin = conditional_entropy(&coin_joint, &coin_marginal, &config)
        .expect("a diagonal joint and its marginal");
    assert!(
        approx_eq(h_coin, T::zero(), tol),
        "X = Y leaves no uncertainty: 0 bits, got {h_coin:?}"
    );

    let four_joint = lift_array::<T>(&[
        0.25, 0.0, 0.0, 0.0, //
        0.0, 0.25, 0.0, 0.0, //
        0.0, 0.0, 0.25, 0.0, //
        0.0, 0.0, 0.0, 0.25,
    ]);
    let four_marginal = uniform::<T>(4);
    let h_four = conditional_entropy(&four_joint, &four_marginal, &config)
        .expect("a diagonal joint and its marginal");
    assert!(
        approx_eq(h_four, T::zero(), tol),
        "X = Y on four outcomes leaves no uncertainty: 0 bits, got {h_four:?}"
    );
}

#[test]
fn conditional_entropy_is_zero_under_deterministic_dependence_f32() {
    check_conditional_entropy_is_zero_under_deterministic_dependence::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_is_zero_under_deterministic_dependence_f64() {
    check_conditional_entropy_is_zero_under_deterministic_dependence::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_is_zero_under_deterministic_dependence_f106() {
    check_conditional_entropy_is_zero_under_deterministic_dependence::<Float106>(lift(F106.native));
}

/// Row C, the degenerate-argument form: the two slices are the same slice.
///
/// `H(X | X) = H(X) − H(X) = 0` whatever `X` is, so this is an identity over a family rather than
/// one value. It is the case where a swapped subtraction, a doubled term, or a second call
/// measured under a different configuration shows up as a non-zero answer.
fn check_conditional_entropy_of_a_distribution_with_itself_is_zero<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let family = [
        lift_array::<T>(&[0.5, 0.5]),
        lift_array::<T>(&[0.5, 0.25, 0.125, 0.125]),
        lift_array::<T>(&[1.0, 0.0]),
        uniform::<T>(3),
        uniform::<T>(8),
    ];

    for p in family.iter() {
        let h = conditional_entropy(p, p, &config).expect("a distribution is a valid input");
        assert!(
            approx_eq(h, T::zero(), tol),
            "H(X | X) is 0 for every X, got {h:?}"
        );
    }
}

#[test]
fn conditional_entropy_of_a_distribution_with_itself_is_zero_f32() {
    check_conditional_entropy_of_a_distribution_with_itself_is_zero::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_of_a_distribution_with_itself_is_zero_f64() {
    check_conditional_entropy_of_a_distribution_with_itself_is_zero::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_of_a_distribution_with_itself_is_zero_f106() {
    check_conditional_entropy_of_a_distribution_with_itself_is_zero::<Float106>(lift(F106.native));
}

/// Row B for the conditioning argument: conditioning on a certain variable changes nothing.
///
/// `Y` has one outcome of probability one, so `H(Y) = 0` and `H(X | Y) = H(X, Y) = H(X)`.
/// The joint is `[0.5, 0.25, 0.125, 0.125]`, hand-evaluated as
/// `0.5·1 + 0.25·2 + 0.125·3 + 0.125·3 = 0.5 + 0.5 + 0.375 + 0.375 = 1.75` bits.
fn check_conditional_entropy_on_a_certain_variable<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let joint = lift_array::<T>(&[0.5, 0.25, 0.125, 0.125]);
    let certain = lift_array::<T>(&[1.0]);

    let h = conditional_entropy(&joint, &certain, &config).expect("a certain conditioning");

    // Hand-evaluated above: 1.75 − 0 = 1.75 bits.
    assert!(
        approx_eq(h, lift::<T>(1.75), tol),
        "conditioning on a certain variable leaves H(X) = 1.75 bits, got {h:?}"
    );
}

#[test]
fn conditional_entropy_on_a_certain_variable_f32() {
    check_conditional_entropy_on_a_certain_variable::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_on_a_certain_variable_f64() {
    check_conditional_entropy_on_a_certain_variable::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_on_a_certain_variable_f106() {
    check_conditional_entropy_on_a_certain_variable::<Float106>(lift(F106.native));
}

/// Conditional entropy is never negative, over a family of joints and their true marginals.
///
/// `H(X | Y) ≥ 0` because conditioning cannot create uncertainty: `H(X, Y) ≥ H(Y)`. Each pair
/// below is a genuine joint together with the marginal obtained by summing the joint's cells for
/// each value of `Y` — the sums are written out in the comments, and they are additions of
/// literals, not a call to anything under test.
fn check_conditional_entropy_is_never_negative<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let bits = EntropyConfig::<T>::bits();
    let nats = EntropyConfig::<T>::nats();

    let pairs = [
        // Two fair coins; Y's marginal is 0.25 + 0.25 = 0.5 for each of its two values.
        (uniform::<T>(4), lift_array::<T>(&[0.5, 0.5])),
        // X = Y, a fair coin; the marginal is again 0.5 and 0.5.
        (
            lift_array::<T>(&[0.5, 0.0, 0.0, 0.5]),
            lift_array::<T>(&[0.5, 0.5]),
        ),
        // Three by two, independent; Y's marginal is 0.25 + 0.125 + 0.125 = 0.5 for each value.
        (
            lift_array::<T>(&[0.25, 0.25, 0.125, 0.125, 0.125, 0.125]),
            lift_array::<T>(&[0.5, 0.5]),
        ),
        // Eight uniform cells over a four-valued Y; each Y value takes 0.125 + 0.125 = 0.25.
        (uniform::<T>(8), uniform::<T>(4)),
        // A dependent joint: Y's marginal is 0.5 + 0.25 = 0.75 and 0.125 + 0.125 = 0.25.
        (
            lift_array::<T>(&[0.5, 0.25, 0.125, 0.125]),
            lift_array::<T>(&[0.75, 0.25]),
        ),
    ];

    for (joint, conditioning) in pairs.iter() {
        for config in [&bits, &nats] {
            let h = conditional_entropy(joint, conditioning, config)
                .expect("a joint and its marginal are valid input");
            assert!(
                h + tol >= T::zero(),
                "conditioning cannot create uncertainty, so H(X | Y) ≥ 0, got {h:?}"
            );
            assert!(h.is_finite(), "H(X | Y) is finite here, got {h:?}");
        }
    }
}

#[test]
fn conditional_entropy_is_never_negative_f32() {
    check_conditional_entropy_is_never_negative::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_is_never_negative_f64() {
    check_conditional_entropy_is_never_negative::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_is_never_negative_f106() {
    check_conditional_entropy_is_never_negative::<Float106>(lift(F106.native));
}

/// Both halves of the difference are measured under the same config, so the answer is in one unit.
///
/// The independent 3 × 2 case again, now in nats. In bits the answer is `1.5` (derived in
/// `check_conditional_entropy_equals_the_marginal_under_independence`), so in nats it is
/// `1.5 · ln 2 = 1.039720770839917964...` with `ln 2 = 0.693147180559945309...`, the published
/// constant (OEIS A002162). Irrational tolerance family, so `Float106` is capped by the `f64`
/// fixture rather than by the arithmetic.
///
/// An implementation that measured the joint in one base and the conditioning in the other would
/// return `2.5·ln 2 − 1 = 0.733` or `2.5 − ln 2 = 1.807`; neither is within any tolerance of the
/// expected value.
fn check_conditional_entropy_in_nats<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::nats();

    let joint = lift_array::<T>(&[0.25, 0.25, 0.125, 0.125, 0.125, 0.125]);
    let conditioning = lift_array::<T>(&[0.5, 0.5]);

    let h = conditional_entropy(&joint, &conditioning, &config).expect("a joint and its marginal");

    // Hand-evaluated: 1.5 × 0.693147180559945309 = 1.039720770839917964 nats. The literal is
    // that value in its shortest form that round-trips through `f64`.
    assert!(
        approx_eq(h, lift::<T>(1.039_720_770_839_918), tol),
        "H(X | Y) is 1.039720770839917964 nats, got {h:?}"
    );
}

#[test]
fn conditional_entropy_in_nats_f32() {
    check_conditional_entropy_in_nats::<f32>(lift(F32.literal));
}

#[test]
fn conditional_entropy_in_nats_f64() {
    check_conditional_entropy_in_nats::<f64>(lift(F64.literal));
}

#[test]
fn conditional_entropy_in_nats_f106() {
    check_conditional_entropy_in_nats::<Float106>(lift(F106.literal));
}

/// Under `BySum` each slice is normalised by its own sum, not by a shared one.
///
/// The joint `[0.5, 0.5, 0.25, 0.25, 0.25, 0.25]` sums to `2.0` and normalises to
/// `[0.25, 0.25, 0.125, 0.125, 0.125, 0.125]`, whose entropy is hand-evaluated as
/// `0.25·2 + 0.25·2 + 4 × (0.125·3) = 0.5 + 0.5 + 1.5 = 2.5` bits. The conditioning `[1.0, 1.0]`
/// sums to `2.0` and normalises to `[0.5, 0.5]`, whose entropy is `0.5·1 + 0.5·1 = 1` bit. So
/// `H(X | Y) = 2.5 − 1 = 1.5` bits.
///
/// Normalising both by the joint's sum instead would give `2.5 − 0` (the conditioning would become
/// `[0.5, 0.5]` only by coincidence here — its own sum happens to match, so the arrangement is
/// checked again below with sums that differ).
fn check_conditional_entropy_normalises_each_slice_by_its_own_sum<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() });

    let joint = lift_array::<T>(&[0.5, 0.5, 0.25, 0.25, 0.25, 0.25]);
    let conditioning = lift_array::<T>(&[1.0, 1.0]);

    let h = conditional_entropy(&joint, &conditioning, &config).expect("a joint and its marginal");
    // Hand-evaluated above: 2.5 − 1 = 1.5 bits.
    assert!(
        approx_eq(h, lift::<T>(1.5), tol),
        "each slice normalised by its own sum gives 1.5 bits, got {h:?}"
    );

    // The two sums now differ: the joint sums to 4.0 and the conditioning to 1.0.
    // The joint [1, 1, 0.5, 0.5, 0.5, 0.5] / 4 = [0.25, 0.25, 0.125, 0.125, 0.125, 0.125], entropy
    // 2.5 bits as above; the conditioning [0.5, 0.5] is already normalised, entropy 1 bit. So the
    // answer is again 1.5 bits, and an implementation sharing one sum between the two slices gets
    // the conditioning wrong by a factor of four and misses it.
    let joint_four = lift_array::<T>(&[1.0, 1.0, 0.5, 0.5, 0.5, 0.5]);
    let conditioning_one = lift_array::<T>(&[0.5, 0.5]);
    let h_mixed = conditional_entropy(&joint_four, &conditioning_one, &config)
        .expect("a joint and its marginal");
    assert!(
        approx_eq(h_mixed, lift::<T>(1.5), tol),
        "slices with different sums each normalise by their own: 1.5 bits, got {h_mixed:?}"
    );
}

#[test]
fn conditional_entropy_normalises_each_slice_by_its_own_sum_f32() {
    check_conditional_entropy_normalises_each_slice_by_its_own_sum::<f32>(lift(F32.native));
}

#[test]
fn conditional_entropy_normalises_each_slice_by_its_own_sum_f64() {
    check_conditional_entropy_normalises_each_slice_by_its_own_sum::<f64>(lift(F64.native));
}

#[test]
fn conditional_entropy_normalises_each_slice_by_its_own_sum_f106() {
    check_conditional_entropy_normalises_each_slice_by_its_own_sum::<Float106>(lift(F106.native));
}

/// Row A for both arguments. An empty joint or an empty conditioning is refused.
///
/// The reading asserted: `conditional_entropy` is the difference of two entropies "measured under
/// the same config", so each argument meets the same refusal `entropy` gives an empty slice. An
/// empty conditioning is not a variable with zero entropy; it is no variable at all.
fn check_conditional_empty_inputs_are_refused<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();
    let empty: [T; 0] = [];
    let joint = lift_array::<T>(&[0.25, 0.25, 0.25, 0.25]);
    let conditioning = lift_array::<T>(&[0.5, 0.5]);

    let err = conditional_entropy(&empty, &conditioning, &config)
        .expect_err("an empty joint has no distribution");
    assert_empty_input(&err);

    let err = conditional_entropy(&joint, &empty, &config)
        .expect_err("an empty conditioning is no variable");
    assert_empty_input(&err);

    let err = conditional_entropy(&empty, &empty, &config).expect_err("both are empty");
    assert_empty_input(&err);
}

#[test]
fn conditional_empty_inputs_are_refused_f32() {
    check_conditional_empty_inputs_are_refused::<f32>();
}

#[test]
fn conditional_empty_inputs_are_refused_f64() {
    check_conditional_empty_inputs_are_refused::<f64>();
}

#[test]
fn conditional_empty_inputs_are_refused_f106() {
    check_conditional_empty_inputs_are_refused::<Float106>();
}

/// Row G for both arguments. A negative entry in either slice is refused.
fn check_conditional_negative_entry_is_refused<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let configs = [
        EntropyConfig::<T>::bits(),
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(1e-3))),
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: T::zero() }),
    ];

    let good_joint = lift_array::<T>(&[0.25, 0.25, 0.25, 0.25]);
    let good_conditioning = lift_array::<T>(&[0.5, 0.5]);
    let bad_joint = lift_array::<T>(&[0.5, -0.25, 0.5, 0.25]);
    let bad_conditioning = lift_array::<T>(&[1.5, -0.5]);

    for config in configs.iter() {
        let err = conditional_entropy(&bad_joint, &good_conditioning, config)
            .expect_err("a negative entry in the joint is not a probability");
        assert_negative_probability(&err);

        let err = conditional_entropy(&good_joint, &bad_conditioning, config)
            .expect_err("a negative entry in the conditioning is not a probability");
        assert_negative_probability(&err);
    }
}

#[test]
fn conditional_negative_entry_is_refused_f32() {
    check_conditional_negative_entry_is_refused::<f32>();
}

#[test]
fn conditional_negative_entry_is_refused_f64() {
    check_conditional_negative_entry_is_refused::<f64>();
}

#[test]
fn conditional_negative_entry_is_refused_f106() {
    check_conditional_negative_entry_is_refused::<Float106>();
}

/// Row I for both arguments. A non-finite entry in either slice is refused.
///
/// Same reading as for `entropy`: `NaN` and `+∞` are `NonFiniteInput`, and `−∞`, being both
/// non-finite and negative, is accepted as either typed refusal. Without the check the difference
/// of two entropies can also hide the damage — `∞ − ∞` is `NaN`, but `H(X, Y)` polluted to `∞`
/// against a finite `H(Y)` returns `∞` and looks like an answer.
fn check_conditional_non_finite_entry_is_refused<T>()
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let config = EntropyConfig::<T>::bits();

    let good_joint = lift_array::<T>(&[0.25, 0.25, 0.25, 0.25]);
    let good_conditioning = lift_array::<T>(&[0.5, 0.5]);

    let nan_joint = [lift::<T>(0.25), T::nan(), lift::<T>(0.25), lift::<T>(0.25)];
    let err = conditional_entropy(&nan_joint, &good_conditioning, &config)
        .expect_err("NaN in the joint is not a probability");
    assert_non_finite(&err);

    let nan_conditioning = [lift::<T>(0.5), T::nan()];
    let err = conditional_entropy(&good_joint, &nan_conditioning, &config)
        .expect_err("NaN in the conditioning is not a probability");
    assert_non_finite(&err);

    let inf_joint = [
        lift::<T>(0.25),
        lift::<T>(f64::INFINITY),
        lift::<T>(0.25),
        lift::<T>(0.25),
    ];
    let err = conditional_entropy(&inf_joint, &good_conditioning, &config)
        .expect_err("+∞ in the joint is not a probability");
    assert_non_finite(&err);

    let neg_inf_conditioning = [lift::<T>(0.5), lift::<T>(f64::NEG_INFINITY)];
    let err = conditional_entropy(&good_joint, &neg_inf_conditioning, &config)
        .expect_err("−∞ in the conditioning is not a probability");
    assert_non_finite_or_negative(&err);
}

#[test]
fn conditional_non_finite_entry_is_refused_f32() {
    check_conditional_non_finite_entry_is_refused::<f32>();
}

#[test]
fn conditional_non_finite_entry_is_refused_f64() {
    check_conditional_non_finite_entry_is_refused::<f64>();
}

#[test]
fn conditional_non_finite_entry_is_refused_f106() {
    check_conditional_non_finite_entry_is_refused::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// The zero policy at the machine epsilon
// ---------------------------------------------------------------------------------------------

/// `SkipZero` keeps an entry that is positive but smaller than the machine epsilon.
///
/// The policy test above uses entries at `0.0625`, which every precision holds comfortably. That
/// separates `SkipZero` from `SkipBelow(0.1)`, but it does not pin *where* `SkipZero`'s own cutoff
/// sits: an implementation that skipped at `epsilon` rather than at zero passes it unchanged. This
/// case is what distinguishes them.
///
/// The distribution is `[1, δ]` with `δ = epsilon / 1000`. Its entropy is
///
/// ```text
/// H = −1·log2(1) − δ·log2(δ) = 0 + δ·log2(1/δ)
/// ```
///
/// because `1` contributes exactly nothing — `log2(1) = 0` — so the whole answer is the `δ` term.
/// Keeping `δ` gives a small positive number; dropping it gives exactly zero. The two are
/// distinguishable however small `δ` is, which is what makes this checkable at a width where the
/// term itself is far below the tolerance of any comparison against the total.
///
/// The input does not sum to one. That is deliberate and the function does not require it: with
/// `Normalisation::None` the caller states the entries are the distribution, and normalising here
/// would move `δ` back above the cutoff and destroy the case.
fn check_skip_zero_keeps_a_sub_epsilon_entry<T>(epsilon: f64)
where
    T: Real + RealField + FromPrimitive + Default + Debug,
{
    let delta = epsilon / 1000.0;
    let p = lift_array::<T>(&[1.0, delta]);
    let h = entropy(&p, &EntropyConfig::<T>::bits()).expect("a positive entry is a valid input");

    assert!(
        h > T::zero(),
        "SkipZero keeps an entry below the machine epsilon: δ = {delta:e} contributes \
         δ·log2(1/δ) > 0, got {h:?}. Exactly zero here means the cutoff was moved off zero."
    );

    // And the entry is genuinely below the epsilon, so the case tests what it claims to.
    assert!(
        lift::<T>(delta) > T::zero() && lift::<T>(delta) < T::epsilon(),
        "the fixture must be positive and below the machine epsilon to separate the two cutoffs"
    );
}

#[test]
fn skip_zero_keeps_a_sub_epsilon_entry_f32() {
    check_skip_zero_keeps_a_sub_epsilon_entry::<f32>(F32.epsilon);
}

#[test]
fn skip_zero_keeps_a_sub_epsilon_entry_f64() {
    check_skip_zero_keeps_a_sub_epsilon_entry::<f64>(F64.epsilon);
}

#[test]
fn skip_zero_keeps_a_sub_epsilon_entry_f106() {
    check_skip_zero_keeps_a_sub_epsilon_entry::<Float106>(F106.epsilon);
}

// ---------------------------------------------------------------------------------------------
// A zero policy that omits nothing, and a normalising sum that leaves the type
// ---------------------------------------------------------------------------------------------

/// `SkipBelow` with a *negative* threshold.
///
/// Provenance: hand evaluation of `H = −Σ pᵢ log2 pᵢ` on `[1/2, 1/2, 0]`. The two halves give
/// `−2·(1/2)·log2(1/2) = 2·(1/2)·1 = 1` bit; the zero entry gives `lim(p → 0) p·log p = 0`, which
/// is the limit the zero policy exists to encode and is what makes `H` continuous at the corner of
/// the simplex. The threshold does not enter the answer: it names which entries are omitted, and
/// omitting an entry that contributes zero changes nothing.
///
/// A negative threshold is the reading of "omit entries at or below the threshold" that omits
/// none of them — the entries are probabilities, so none is negative and none is at or below it.
/// The exact zeros are then *kept*, and `0 · ln 0` is `0 · (−∞)`, which is a `NaN` and not the
/// limit. The whole answer is lost to it: a `NaN` added to the running sum stays there, so an
/// otherwise ordinary distribution comes back with no entropy at all rather than with one bit.
///
/// The `-0.0` case is the same corner reached from the other side: it is not less than zero, so
/// the input passes the negative-probability check, and it is not greater than a negative
/// threshold either.
fn check_negative_threshold_keeps_the_limit_at_zero<T>(tol: T)
where
    T: Real + RealField + FromPrimitive + Debug + Default,
{
    for threshold in [-0.5, -1e-30, -0.0] {
        let config =
            EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift(threshold)));

        let p = lift_array::<T>(&[0.5, 0.5, 0.0]);
        let h = entropy(&p, &config).expect("a distribution with a zero entry has an entropy");
        assert!(
            approx_eq(h, lift::<T>(1.0), tol),
            "SkipBelow({threshold}) on [1/2, 1/2, 0] gave {h:?}, not the 1 bit the two halves carry"
        );

        // Several zeros, and a zero in the leading position, so no ordering hides the corner.
        let padded = lift_array::<T>(&[0.0, 0.5, 0.0, 0.5, 0.0]);
        let h = entropy(&padded, &config).expect("padding with zeros does not remove the entropy");
        assert!(
            approx_eq(h, lift::<T>(1.0), tol),
            "SkipBelow({threshold}) on [0, 1/2, 0, 1/2, 0] gave {h:?}, not 1 bit"
        );
    }
}

#[test]
fn negative_threshold_keeps_the_limit_at_zero_f32() {
    check_negative_threshold_keeps_the_limit_at_zero::<f32>(lift(F32.native));
}

#[test]
fn negative_threshold_keeps_the_limit_at_zero_f64() {
    check_negative_threshold_keeps_the_limit_at_zero::<f64>(lift(F64.native));
}

#[test]
fn negative_threshold_keeps_the_limit_at_zero_f106() {
    check_negative_threshold_keeps_the_limit_at_zero::<Float106>(lift(F106.native));
}

/// `Normalisation::BySum` where the weights are finite and their sum is not.
///
/// Provenance: hand evaluation of `H = −Σ pᵢ log2 pᵢ` on the normalised weights, plus the scale
/// invariance that `BySum` exists to provide — `H(w) = H(c·w)` for every `c > 0`, because the
/// weights are divided by their own sum before the surprisal is taken.
///
/// * `k` equal weights normalise to the uniform distribution on `k`, whose entropy is `log2 k`.
///   Two weights give exactly 1 bit; three give `log2 3 = 1.584962500721156` (the published
///   constant, cited here rather than computed).
///
/// The weights are each the type's own maximum, so every one of them is an ordinary number of the
/// type and their sum is `+∞`. Dividing by that sum sends every entry to zero, and an entropy of
/// zero is then returned for the distribution that carries the *most* entropy a `k`-outcome
/// distribution can — the exact opposite of the answer.
fn check_by_sum_where_the_sum_overflows<T>(max_finite: f64, tol: T)
where
    T: Real + RealField + FromPrimitive + Debug + Default,
{
    let config =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: lift(0.0) });

    let two = lift_array::<T>(&[max_finite, max_finite]);
    let h = entropy(&two, &config).expect("two equal weights are a distribution");
    assert!(
        approx_eq(h, lift::<T>(1.0), tol),
        "two equal weights at MAX gave {h:?}, not the 1 bit of a fair coin"
    );

    let three = lift_array::<T>(&[max_finite, max_finite, max_finite]);
    let h = entropy(&three, &config).expect("three equal weights are a distribution");
    // log2 3 = 1.584962500721156, a published constant.
    assert!(
        approx_eq(h, lift::<T>(1.584_962_500_721_156), tol),
        "three equal weights at MAX gave {h:?}, not log2 3"
    );

    // Unequal weights, so the answer is not the uniform one by accident: 1 and 3 parts of the
    // mass are 1/4 and 3/4, and H(1/4, 3/4) = 2/4 + (3/4)·log2(4/3) = 0.8112781244591328.
    let uneven = lift_array::<T>(&[max_finite / 3.0, max_finite]);
    let h = entropy(&uneven, &config).expect("two unequal weights are a distribution");
    assert!(
        approx_eq(h, lift::<T>(0.811_278_124_459_132_8), tol),
        "weights in the ratio 1:3 gave {h:?}, not H(1/4, 3/4)"
    );
}

#[test]
fn by_sum_where_the_sum_overflows_f32() {
    check_by_sum_where_the_sum_overflows::<f32>(F32.max_finite, lift(F32.native));
}

#[test]
fn by_sum_where_the_sum_overflows_f64() {
    check_by_sum_where_the_sum_overflows::<f64>(F64.max_finite, lift(F64.native));
}

#[test]
fn by_sum_where_the_sum_overflows_f106() {
    check_by_sum_where_the_sum_overflows::<Float106>(F106.max_finite, lift(F106.literal));
}
