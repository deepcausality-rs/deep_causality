/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The one set of per-precision parameters every numeric suite uses.
//!
//! A suite that ran at `f64` alone would pass with a tolerance that is unreachable at `f32` and
//! ten orders too loose at `Float106`. This table is what lets the same assertions be meaningful
//! at all three.
//!
//! # One table, no local copies
//!
//! Every tolerance in every suite comes from here. A suite does **not** declare its own
//! `TOL_F32`/`TOL_F64`/`TOL_F106`: six suites did, under five different names, with values that
//! disagreed for no stated reason, and a reader had no way to tell which was considered correct.
//!
//! # The rows are computation kinds, not suites
//!
//! The rows differ because the *operations* differ, not because the callers do. A direct reduction
//! keeps more digits than a linear solve, at every precision, and any suite doing either picks the
//! matching row. Choose by asking what the assertion is comparing:
//!
//! | Row | Use it when |
//! |---|---|
//! | [`Prec::native`] | both sides computed in `T`, over a handful of operations |
//! | [`Prec::literal`] | the expectation crossed in as an `f64` decimal literal |
//! | [`Prec::reduction`] | a sum or fold over roughly a thousand terms |
//! | [`Prec::solve`] | the value came out of a linear solve or an iterative fit |
//! | [`Prec::zero`] | the expected value is zero, so relative tolerance says nothing |
//!
//! # Why `native` and `literal` are separate
//!
//! They are floored by different things, and conflating them is the mistake this table exists to
//! prevent.
//!
//! `native` is floored by the working scalar's own accuracy. `literal` is floored by the
//! expectation's *route* into the scalar: a decimal literal crosses through
//! `deep_causality_num::lift`, which takes an `f64`, so at `Float106` the comparison can never be
//! tighter than `f64`'s epsilon — about `2.2e-16` — however accurate the arithmetic is. At `f32`
//! the two rows coincide; at `Float106` they differ by ten orders of magnitude.
//!
//! Using `native` where `literal` belongs makes an assertion unsatisfiable at `Float106`. Using
//! `literal` where `native` belongs makes it toothless there.
//!
//! # Why the epsilon multiples are not uniform
//!
//! `f32` has about seven decimal digits, so there is no room to spend: `native` is roughly eight
//! epsilon. `Float106` has about thirty-two, so the same row sits at millions of epsilon and is
//! still far tighter than `f64` could ever be. The multiple is not the invariant; the number of
//! digits an operation actually delivers is.
//!
//! # `BFloat16` is not simply a smaller `f32`
//!
//! It is a *truncated* one: the same eight exponent bits, so the same reach, with the mantissa cut
//! from 23 bits to 7. That makes `epsilon` `7.8e-3` — under one percent relative, about two
//! decimal digits — while `max_finite` and `min_positive` are `f32`'s.
//!
//! Two consequences shape what a suite can ask of it. Integers are exact only to 256, so a
//! cancellation fixture cannot use `f32`'s offset of ten thousand. And a thousand-term reduction
//! is not merely imprecise but meaningless: once the running sum reaches one, an addend of a
//! thousandth is below `epsilon · sum` and vanishes entirely, so the row is set to the point of
//! saying so rather than to a tolerance any implementation could meet.

/// What changes between the three precisions.
///
/// Every field is `f64` because a tolerance is compared after lowering, and because the reach
/// fields describe the type from outside it.
#[derive(Clone, Copy, Debug)]
pub struct Prec {
    /// Machine epsilon: the gap between 1 and the next representable value.
    pub epsilon: f64,
    /// Relative tolerance where both sides were computed in `T` over a handful of operations.
    pub native: f64,
    /// Relative tolerance against an expectation that crossed in as an `f64` decimal literal.
    ///
    /// Floored at `f64`'s epsilon at every precision. See the module header.
    pub literal: f64,
    /// Relative tolerance for a reduction over roughly a thousand terms.
    ///
    /// Accumulated rounding grows with the term count, so a long sum cannot hold [`Self::native`].
    pub reduction: f64,
    /// Relative tolerance for a value that came out of a linear solve or an iterative fit.
    ///
    /// Normal equations square the condition number, so a solve delivers about half the digits a
    /// direct reduction does. At `f32` that is the difference between four and six.
    pub solve: f64,
    /// Absolute floor below which a computed value counts as zero.
    ///
    /// Relative tolerance is meaningless against an expected zero, so a quantity that should
    /// vanish is compared against this instead.
    pub zero: f64,
    /// The largest finite value of the type.
    pub max_finite: f64,
    /// The smallest positive normal value of the type.
    pub min_positive: f64,
    /// A magnitude that is finite in `T` but whose square is not.
    ///
    /// The input separating an implementation that squares before scaling from one that does not.
    pub huge: f64,
    /// An offset large enough that a mean carries most of the sample's significant digits, so a
    /// small dispersion around it has to survive cancellation.
    ///
    /// Capped by the `f64` crossing in `lift`: beyond `2^53` the fixture's `offset + 1` is no
    /// longer exact as an `f64` literal, so the cap is the fixture's, not the type's.
    pub cancel_offset: f64,
    /// A scale at which `Σxᵢ²` overflows the type but `Σ(xᵢ − x̄)²` and the answer do not.
    pub overflow_scale: f64,
    /// A step that sits strictly inside an interval at the fixtures' magnitude (about 10) while
    /// staying well above the type's own spacing there.
    ///
    /// For probing either side of a boundary: large enough that the type can tell the two apart,
    /// small enough not to cross into the next interval. The spacing at 10 is about `1e-6` for
    /// `f32`, `2e-15` for `f64` and `1e-31` for `Float106`, so each nudge is several orders above
    /// its type's step and several orders below a bin width of 2.
    pub nudge: f64,
}

/// `BFloat16`: epsilon `7.8e-3`, about two decimal digits.
///
/// `native` `5e-2` is about six epsilon, the same headroom `f32`'s row takes, which is all two
/// digits allow. `literal` matches it: an `f64` literal is far more accurate than `BFloat16` can
/// hold, so the `BFloat16` side floors the comparison.
/// `solve` `2e-1`: squaring the condition number costs about half of two digits, leaving one.
/// `reduction` `1.0` is not a tolerance to meet but a statement that a thousand-term sum has no
/// accuracy left at this width — a suite should not run one here rather than assert against it.
/// `zero` `1e-1`: an ulp at magnitude ten is `7.8e-2`, so anything tighter sits inside the noise.
/// `cancel_offset` `64`: the binding constraint is the SUM, not the values. Integers are exact
/// only to 256, and a three-element fixture sums to about three times the offset, so an offset of
/// 100 gives 303 — past 256, where the spacing is already 2, and the mean is wrong before the
/// variance is reached. At 64 the sum is 195 and every intermediate stays exact. `f32`'s ten
/// thousand is not representable here at all.
/// `huge` `1e20` squares past `MAX = 3.39e38`; `1e19` does not, since `sqrt(MAX) = 1.84e19`.
/// `nudge` `0.25`: the spacing at ten is `7.8e-2`, so a smaller step would not move the value.
/// The reach fields are `f32`'s, because the exponent field is.
pub const BF16: Prec = Prec {
    epsilon: 7.812_5e-3,
    native: 5e-2,
    literal: 5e-2,
    reduction: 1.0,
    solve: 2e-1,
    zero: 1e-1,
    max_finite: 3.389_531_389_251_535_5e38,
    min_positive: 1.175_494_350_822_287_5e-38,
    huge: 1e20,
    cancel_offset: 64.0,
    overflow_scale: 5e18,
    nudge: 0.25,
};

/// `f32`: epsilon `1.19e-7`, about seven decimal digits.
///
/// `native` `1e-6` is roughly eight epsilon — all the headroom seven digits allow over a
/// handful of operations. `literal` matches it, because an `f64` literal is far more accurate
/// than `f32` can hold, so the `f32` side is what floors the comparison.
/// `solve` `1e-4` is two digits looser: squaring the condition number costs about half of seven.
/// `zero` `1e-5` rather than `1e-6`: a residual between operands of magnitude ten is a few ulps,
/// and an ulp at ten in `f32` is already `1e-6`, so the tighter floor sits inside the noise it is
/// meant to be above. Measured: an exactly-determined two-by-two solve leaves `1.19e-6`.
/// `huge` `1e20` squares to `1e40`, past `f32::MAX = 3.4e38`, which is the point: the value
/// itself is finite and its square is not.
/// `overflow_scale` `5e18`: `55·k² = 1.4e39` exceeds the maximum, while `10·k² = 2.5e38` and the
/// answer `2.5·k² = 6.3e37` do not.
/// `cancel_offset` `1e4`: 10000, 10001 and 10002 are all below `2^24` and so exact in `f32`.
pub const F32: Prec = Prec {
    epsilon: 1.1920929e-7,
    native: 1e-6,
    literal: 1e-6,
    reduction: 1e-2,
    solve: 1e-4,
    zero: 1e-5,
    max_finite: f32::MAX as f64,
    min_positive: f32::MIN_POSITIVE as f64,
    huge: 1e20,
    cancel_offset: 1e4,
    overflow_scale: 5e18,
    nudge: 1e-3,
};

/// `f64`: epsilon `2.22e-16`, about sixteen decimal digits.
///
/// `native` `1e-12` is roughly `4.5e3` epsilon, leaving four digits of headroom.
/// `literal` `1e-13` is tighter than `native` because the crossing is exact here: an `f64`
/// literal is the value, so only the arithmetic contributes.
/// `solve` `1e-12` matches `native`: at sixteen digits, halving still leaves eight, so the solve
/// is not what floors these comparisons.
/// `huge` `2e154` squares to `4e308`, past `f64::MAX = 1.8e308`. `1e154` would not do: its
/// square `1e308` is still finite, so it would not exercise the overflow path at all.
/// `overflow_scale` `2e153`: `55·k² = 2.2e308` exceeds the maximum, while `10·k² = 4.0e307` does
/// not.
/// `cancel_offset` `1e8`: `Σxᵢ²` is then about `3e16`, where an ulp is 4 and the quantity being
/// recovered is 2.
pub const F64: Prec = Prec {
    epsilon: 2.220446049250313e-16,
    native: 1e-12,
    literal: 1e-13,
    reduction: 1e-10,
    solve: 1e-12,
    zero: 1e-10,
    max_finite: f64::MAX,
    min_positive: f64::MIN_POSITIVE,
    huge: 2e154,
    cancel_offset: 1e8,
    overflow_scale: 2e153,
    nudge: 1e-10,
};

/// `Float106`: epsilon `4.93e-32`, about thirty-two decimal digits.
///
/// `native` `1e-25` is roughly `2e6` epsilon, and still ten orders tighter than the `f64` row —
/// the point of the table.
/// `literal` `1e-15` stays at `f64` reach for the reason in the module header: the expectation
/// crossed through an `f64`, so the comparison cannot see past `f64`'s epsilon.
/// `zero` `1e-10` likewise: a quantity that should vanish, built from lifted `f64` fixtures,
/// carries `f64` residue rather than `Float106` residue.
///
/// The exponent range is `f64`'s, because the head of a double-double is an `f64`. So
/// `max_finite`, `min_positive`, `huge` and `overflow_scale` are shared with the `f64` row: the
/// extra precision is in the mantissa, not the reach. `cancel_offset` is capped at `1e15` by the
/// `f64` fixture crossing rather than by the type.
pub const F106: Prec = Prec {
    epsilon: 4.930380657631324e-32,
    native: 1e-25,
    literal: 1e-15,
    reduction: 1e-22,
    solve: 1e-25,
    zero: 1e-10,
    max_finite: f64::MAX,
    min_positive: f64::MIN_POSITIVE,
    huge: 2e154,
    cancel_offset: 1e15,
    overflow_scale: 2e153,
    nudge: 1e-20,
};
