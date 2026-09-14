/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Precision as a parameter: one body, every scalar, and the entropy each is owed.

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::{BFloat16, Float106, FromPrimitive, ToPrimitive};
use deep_causality_rand::{Distribution, Xoshiro256};
use deep_causality_stats::{Open01, OpenClosed01, RandWidth, StandardNormal, StandardUniform};

const SEED: u64 = 0x5EED_2026;

fn lower<T: ToPrimitive>(v: T) -> f64 {
    v.to_f64().expect("every supported scalar lowers to f64")
}

/// Mean and `E[x^2]` of `n` uniform draws, computed at the caller's scalar and lowered once.
fn uniform_moments<T>(n: u64, seed: u64) -> (f64, f64)
where
    T: RealField + FromPrimitive + RandWidth,
    StandardUniform: Distribution<T>,
{
    let mut g = Xoshiro256::from_seed(seed);
    let (mut sum, mut sq) = (T::zero(), T::zero());
    for _ in 0..n {
        let x: T = StandardUniform.sample(&mut g);
        sum += x;
        sq += x * x;
    }
    let nn = T::from_u64(n).unwrap();
    (lower(sum / nn), lower(sq / nn))
}

// ---------------------------------------------------------------------------------------------
// The statistical contract, at every scalar
// ---------------------------------------------------------------------------------------------

#[test]
fn uniform_moments_hold_at_every_scalar() {
    let n = 50_000;
    for (name, (mean, e_x2)) in [
        ("f32", uniform_moments::<f32>(n, SEED)),
        ("f64", uniform_moments::<f64>(n, SEED)),
        ("Float106", uniform_moments::<Float106>(n, SEED)),
    ] {
        assert!((mean - 0.5).abs() < 0.02, "{name} mean {mean} vs 0.5");
        assert!(
            (e_x2 - 1.0 / 3.0).abs() < 0.02,
            "{name} E[x^2] {e_x2} vs 1/3"
        );
    }
}

#[test]
fn normal_variance_holds_at_every_scalar() {
    fn variance<T>(n: u64) -> f64
    where
        T: RealField + FromPrimitive + RandWidth,
        StandardNormal: Distribution<T>,
    {
        let mut g = Xoshiro256::from_seed(SEED);
        let mut sq = T::zero();
        for _ in 0..n {
            let z: T = StandardNormal.sample(&mut g);
            sq += z * z;
        }
        lower(sq / T::from_u64(n).unwrap())
    }
    for (name, v) in [
        ("f32", variance::<f32>(50_000)),
        ("f64", variance::<f64>(50_000)),
        ("Float106", variance::<Float106>(50_000)),
    ] {
        assert!((v - 1.0).abs() < 0.05, "{name} variance {v} vs 1.0");
    }
}

// ---------------------------------------------------------------------------------------------
// The load-bearing test: a wide scalar receives its full entropy
// ---------------------------------------------------------------------------------------------

#[test]
fn a_double_double_draw_carries_its_low_limb() {
    // A single 53-bit draw returned as a `Float106` is an `f64` wearing a wider type. It passes
    // every bounds check and every moment test above, and silently defeats the precision claim.
    // The observable is the round trip: a genuine 106-bit draw usually differs from its own `f64`
    // truncation, while a widened `f64` draw never does.
    let mut g = Xoshiro256::from_seed(SEED);
    let mut carried = 0;
    for _ in 0..200 {
        let v: Float106 = StandardUniform.sample(&mut g);
        if Float106::from(lower(v)) != v {
            carried += 1;
        }
    }
    assert!(
        carried > 100,
        "only {carried}/200 draws carried a low limb; the double-double is an f64 in disguise"
    );
}

#[test]
fn the_width_constant_is_declared_per_scalar() {
    assert_eq!(<f32 as RandWidth>::WORDS, 1);
    assert_eq!(<f64 as RandWidth>::WORDS, 1);
    assert_eq!(<BFloat16 as RandWidth>::WORDS, 1);
    assert_eq!(
        <Float106 as RandWidth>::WORDS,
        2,
        "a double-double needs two"
    );
}

// ---------------------------------------------------------------------------------------------
// The half-open interval
// ---------------------------------------------------------------------------------------------

#[test]
fn every_uniform_draw_stays_in_the_half_open_interval() {
    fn check<T>(name: &str, n: usize)
    where
        T: RealField + FromPrimitive + RandWidth + ToPrimitive,
        StandardUniform: Distribution<T>,
    {
        let mut g = Xoshiro256::from_seed(SEED);
        for i in 0..n {
            let v: T = StandardUniform.sample(&mut g);
            let f = lower(v);
            assert!(
                (0.0..1.0).contains(&f),
                "{name} draw {i} was {f}, outside [0, 1)"
            );
        }
    }
    check::<f32>("f32", 2000);
    check::<f64>("f64", 2000);
    check::<Float106>("Float106", 2000);
    // The narrow significand is the one that can round onto the upper bound.
    check::<BFloat16>("BFloat16", 5000);
}

#[test]
fn the_open_variants_exclude_their_endpoints() {
    let mut g = Xoshiro256::from_seed(SEED);
    for _ in 0..5000 {
        let o: f64 = Open01.sample(&mut g);
        assert!(o > 0.0 && o < 1.0, "Open01 gave {o}");
        let oc: f64 = OpenClosed01.sample(&mut g);
        assert!(oc > 0.0 && oc <= 1.0, "OpenClosed01 gave {oc}");
    }
}

#[test]
fn a_log_transform_never_sees_a_zero_argument() {
    // Several distributions take `ln(u)`. An `Open01` draw of exactly zero would give an infinity
    // that no bounds check on the result would catch.
    let mut g = Xoshiro256::from_seed(SEED);
    for _ in 0..10_000 {
        let u: f64 = Open01.sample(&mut g);
        assert!(Real::ln(u).is_finite(), "ln({u}) was not finite");
    }
}

// ---------------------------------------------------------------------------------------------
// BFloat16: samplable for the first time
// ---------------------------------------------------------------------------------------------

#[test]
fn bfloat16_can_be_sampled_at_all() {
    // No `Distribution<BFloat16>` existed before this change, at any effort.
    let mut g = Xoshiro256::from_seed(SEED);
    let u: BFloat16 = StandardUniform.sample(&mut g);
    let z: BFloat16 = StandardNormal.sample(&mut g);
    assert!((0.0..1.0).contains(&lower(u)), "bf16 uniform {}", lower(u));
    assert!(lower(z).is_finite(), "bf16 normal {}", lower(z));
}

#[test]
fn bfloat16_draws_are_correct_even_where_its_sums_are_not() {
    // Averaged over a count small enough to avoid saturation. A longer run stalls at 256 because
    // an 8-bit significand stops absorbing terms — the scalar's property, not the sampler's, and
    // the same one the project README records as "BFloat16 stops adding at k = 23".
    let mut g = Xoshiro256::from_seed(SEED);
    let n = 64;
    let mut sum = 0.0f64;
    for _ in 0..n {
        let u: BFloat16 = StandardUniform.sample(&mut g);
        sum += lower(u);
    }
    let mean = sum / n as f64;
    assert!((mean - 0.5).abs() < 0.1, "bf16 mean {mean} vs 0.5");
}

// ---------------------------------------------------------------------------------------------
// One bound
// ---------------------------------------------------------------------------------------------

#[test]
fn a_generic_caller_states_one_scalar_bound() {
    // The shape the retrofit exists for. Compare the README's Monte Carlo example, which needs
    // `where StandardUniform: Distribution<S>` — a clause no other unified-math crate asks for.
    fn monte_carlo<S: RealField + FromPrimitive + RandWidth + ToPrimitive>(n: u64) -> f64 {
        let mut g = Xoshiro256::from_seed(SEED);
        let mut sum = S::zero();
        for _ in 0..n {
            let x: S = StandardUniform.sample(&mut g);
            sum += x * x;
        }
        lower(sum / S::from_u64(n).unwrap())
    }
    for (name, v) in [
        ("f32", monte_carlo::<f32>(20_000)),
        ("f64", monte_carlo::<f64>(20_000)),
        ("Float106", monte_carlo::<Float106>(20_000)),
    ] {
        assert!((v - 1.0 / 3.0).abs() < 0.02, "{name} integral {v} vs 1/3");
    }
}

// ---------------------------------------------------------------------------------------------
// The rejected mass goes nowhere
// ---------------------------------------------------------------------------------------------

#[test]
fn a_rounded_up_draw_is_redrawn_rather_than_clamped() {
    // `BFloat16` keeps 8 significand bits, so an `[0, 1)` value near the top rounds onto exactly
    // `1.0` about `2^-9` of the time and must be redrawn. Clamping instead — returning the largest
    // value below 1 — would hand that mass to a single point.
    //
    // No other test in this file detects that. Bounds still hold, the mean still holds, the
    // moments still hold; only the shape at one value changes. Measured over 200 000 draws: the
    // most frequent value takes 0.00429 of them when the draw is redrawn and 0.00584 when it is
    // clamped, a 36% excess on one point. The threshold sits between, closer to the correct value
    // than to the defective one.
    use std::collections::HashMap;

    let mut g = Xoshiro256::from_seed(SEED);
    let n = 200_000;
    let mut counts: HashMap<u64, u64> = HashMap::new();
    for _ in 0..n {
        let v: BFloat16 = StandardUniform.sample(&mut g);
        *counts.entry(f64::from(v).to_bits()).or_insert(0) += 1;
    }

    let top = counts.values().max().copied().expect("draws were taken");
    let fraction = top as f64 / n as f64;
    assert!(
        fraction < 0.005,
        "the most frequent value took {fraction:.5} of draws; clamping rather than redrawing \
         piles the rejected mass onto one point"
    );
}

// ---------------------------------------------------------------------------------------------
// The zero a random generator will not produce
// ---------------------------------------------------------------------------------------------

/// A generator whose words are all zero.
///
/// `Open01` must exclude its lower endpoint, and the draw that tests it has probability `2^-53`
/// from a real generator — one in nine quadrillion. No sampling test reaches it, which is why
/// mutation testing found `>` against `>=` here and nothing objected. A rigged generator reaches
/// it on the first draw.
struct ZeroRng;

impl deep_causality_rand::RngCore for ZeroRng {
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

impl deep_causality_rand::Rng for ZeroRng {}

#[test]
fn open01_excludes_zero_even_when_the_generator_only_produces_it() {
    // A zero word gives a zero unit value, which `StandardUniform` may return and `Open01` may
    // not. With `>=` in place of `>` this loops forever rather than returning zero, so the test
    // is guarded by a bounded number of attempts on a separate thread.
    let handle = std::thread::spawn(|| {
        let mut rng = ZeroRng;
        // Correct behaviour: this never terminates, because every redraw is also zero. The test
        // asserts the guard is *reached*, not that a value comes back.
        let _: f64 = Open01.sample(&mut rng);
    });
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(
        !handle.is_finished(),
        "Open01 returned a value from an all-zero generator, so it admits its lower endpoint"
    );
}

#[test]
fn standard_uniform_admits_zero_which_open01_must_not() {
    // The complement of the test above, and the reason the two types differ at all.
    let mut rng = ZeroRng;
    let v: f64 = StandardUniform.sample(&mut rng);
    assert_eq!(v, 0.0, "the half-open interval includes its lower endpoint");
}

// ---------------------------------------------------------------------------------------------
// How the limbs compose
// ---------------------------------------------------------------------------------------------

/// A generator that hands back a fixed sequence of words, then zeros.
///
/// The limbs of a double-double are a **sum**: the second word is placed `2^-53` below the first
/// and added. Subtracting it instead shifts the value by about `1e-16`, which is below every
/// tolerance in this file and leaves the low limb populated, so no moment, bounds or entropy test
/// can see it. Mutation testing found exactly that gap. Scripted words pin the arithmetic itself.
struct ScriptedRng {
    words: Vec<u64>,
    at: usize,
}

impl ScriptedRng {
    fn new(words: Vec<u64>) -> Self {
        Self { words, at: 0 }
    }
}

impl deep_causality_rand::RngCore for ScriptedRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        let w = self.words.get(self.at).copied().unwrap_or(0);
        self.at += 1;
        w
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill(0);
    }
}

impl deep_causality_rand::Rng for ScriptedRng {}

#[test]
fn the_second_limb_is_added_not_subtracted() {
    // First word zero, second word non-zero. A sum leaves a small positive value; a difference
    // leaves a small negative one, which is outside the support.
    let second = 1_u64 << 40;
    let mut rng = ScriptedRng::new(vec![0, second]);
    let v: Float106 = StandardUniform.sample(&mut rng);
    let as_f64 = lower(v);
    assert!(
        as_f64 >= 0.0,
        "a zero high limb with a positive low limb gave {as_f64}; the limbs are being subtracted"
    );
    assert!(as_f64 > 0.0, "the low limb was dropped entirely");
}

#[test]
fn the_limbs_compose_to_the_exact_expected_value() {
    // The whole rule, pinned: `hi + lo * 2^-53`, each word taken as its top 53 bits over 2^53.
    let (w1, w2) = (3_u64 << 30, 7_u64 << 20);
    let mut rng = ScriptedRng::new(vec![w1, w2]);
    let v: Float106 = StandardUniform.sample(&mut rng);

    let scale = 1.0f64 / ((1_u64 << 53) as f64);
    let hi = (w1 >> 11) as f64 * scale;
    let lo = (w2 >> 11) as f64 * scale;
    let expected = Float106::from(hi) + Float106::from(lo) * Float106::from(scale);

    assert_eq!(
        v, expected,
        "the two limbs did not compose as hi + lo * 2^-53"
    );
}
