/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Transcendental function tests for `Float106`.
//!
//! Expected values are external: computed with mpmath at 60 decimal places and split into the
//! exact `(hi, lo)` `f64` pair the type stores, so the whole 106-bit result is checked rather
//! than its leading word. Each argument is fed to mpmath as the `f64` the test constructs, not
//! as a decimal literal — `Float106::from(0.05)` holds the nearest `f64` to 0.05, and comparing
//! it against the decimal 0.05 measures that conversion rather than the function.
//!
//! `TOL` is the relative error the type actually delivers, measured across these tables. The
//! type carries roughly 106 bits, so a tolerance in the `1e-14` range would accept an answer
//! with no correct low word at all.

use deep_causality_num::{Float, Float106};

/// Relative error between a computed value and its reference, at full `Float106` precision.
fn rel_err(got: Float106, want: Float106) -> f64 {
    if want == Float106::from(0.0) {
        return f64::from(<Float106 as Float>::abs(got));
    }
    f64::from(<Float106 as Float>::abs(got - want) / <Float106 as Float>::abs(want))
}

/// Reference value from its exact two-word decomposition.
fn r(hi: f64, lo: f64) -> Float106 {
    Float106::from_raw(hi, lo)
}

fn d(x: f64) -> Float106 {
    Float106::from(x)
}

/// Measured worst case across every table below is about 5e-31; the bound leaves headroom
/// without admitting an `f64`-accurate answer.
const TOL: f64 = 1e-29;

fn check(name: &str, f: impl Fn(Float106) -> Float106, table: &[(f64, f64, f64)]) {
    for &(x, hi, lo) in table {
        let got = f(d(x));
        let want = r(hi, lo);
        let e = rel_err(got, want);
        assert!(
            e <= TOL,
            "{name}({x}): relative error {e:e} exceeds {TOL:e}"
        );
    }
}

// =============================================================================
// Constants — both words, against the published decimal expansion
// =============================================================================

#[test]
fn constants_carry_both_words_of_the_published_value() {
    // π = 3.14159265358979323846264338327950288..., split at f64.
    assert_eq!(Float106::PI.hi(), core::f64::consts::PI);
    assert_eq!(Float106::PI.lo(), 1.2246467991473532e-16);
    // e = 2.71828182845904523536028747135266249...
    assert_eq!(Float106::E.hi(), core::f64::consts::E);
    assert_eq!(Float106::E.lo(), 1.4456468917292502e-16);
    // ln 2 = 0.69314718055994530941723212145817656...
    assert_eq!(Float106::LN_2.hi(), core::f64::consts::LN_2);
    assert_eq!(Float106::LN_2.lo(), 2.3190468138462996e-17);
    // ln 10 = 2.30258509299404568401799145468436420...
    assert_eq!(Float106::LN_10.hi(), core::f64::consts::LN_10);
    assert_eq!(Float106::LN_10.lo(), -2.1707562233822494e-16);
}

#[test]
fn the_pi_family_are_exact_binary_scalings_of_each_other() {
    // Halving and doubling are exact in binary floating point, so these must agree bit for bit.
    assert_eq!(Float106::TWO_PI.hi(), Float106::PI.hi() * 2.0);
    assert_eq!(Float106::TWO_PI.lo(), Float106::PI.lo() * 2.0);
    assert_eq!(Float106::FRAC_PI_2.hi(), Float106::PI.hi() / 2.0);
    assert_eq!(Float106::FRAC_PI_2.lo(), Float106::PI.lo() / 2.0);
    assert_eq!(Float106::FRAC_PI_4.hi(), Float106::PI.hi() / 4.0);
    assert_eq!(Float106::FRAC_PI_4.lo(), Float106::PI.lo() / 4.0);
}

#[test]
fn epsilon_is_the_double_double_resolution_not_the_f64_one() {
    // ~2^-106. An f64 epsilon here (2.2e-16) would mean the extra word buys nothing.
    assert!(f64::from(Float106::EPSILON) < 1e-31);
    assert!(f64::from(Float106::EPSILON) > 0.0);
    // 1 + eps is distinguishable from 1; 1 + eps/4 is not.
    assert!(d(1.0) + Float106::EPSILON != d(1.0));
}

// =============================================================================
// Reference tables (mpmath, 60 dps)
// =============================================================================

#[test]
fn exp_matches_the_reference() {
    check(
        "exp",
        <Float106 as Float>::exp,
        &[
            (0.0, 1.0, 0.0),
            (0.5, 1.6487212707001282, -4.731568479435833e-17),
            (1.0, core::f64::consts::E, 1.4456468917292502e-16), // exp(1) = e
            (-1.0, 0.36787944117144233, -1.2428753672788363e-17),
            (2.5, 12.182493960703473, 2.0334002173348147e-16),
            (-3.75, 0.023517745856009107, 1.2666758876675962e-18),
            (10.0, 22026.465794806718, -1.3780134700517372e-12),
            (0.125, 1.1331484530668263, -5.370737708558031e-18),
            (-0.0625, 0.9394130628134758, -2.152447043447057e-17),
            (5.0, 148.4131591025766, 3.4863514900464198e-15),
        ],
    );
}

#[test]
fn ln_matches_the_reference() {
    check(
        "ln",
        <Float106 as Float>::ln,
        &[
            (1.0, 0.0, 0.0),
            (2.0, core::f64::consts::LN_2, 2.3190468138462996e-17),
            (0.5, -core::f64::consts::LN_2, -2.3190468138462996e-17),
            (10.0, core::f64::consts::LN_10, -2.1707562233822494e-16),
            (123.456, 4.815884817283264, 1.2224723590869397e-16),
            (1e-05, -11.512925464970229, 2.790027459050308e-16),
            (0.125, -2.0794415416798357, -1.8059370687790465e-16),
            (7.0, 1.9459101490553132, 7.323586207904907e-17),
            (1000.0, 6.907755278982137, 2.369515526854504e-16),
        ],
    );
}

#[test]
fn sin_matches_the_reference() {
    check(
        "sin",
        <Float106 as Float>::sin,
        &[
            (0.0, 0.0, 0.0),
            (0.5, 0.479425538604203, -5.103969860556013e-18),
            (1.0, 0.8414709848078965, 1.776845092935536e-18),
            (-1.0, -0.8414709848078965, -1.776845092935536e-18),
            (2.0, 0.9092974268256817, -1.4020906557816256e-17),
            (3.0, 0.1411200080598672, 8.577269787017502e-18),
            (0.125, 0.12467473338522769, -2.925947496057858e-18),
            (-2.5, -0.5984721441039565, 5.521403334082375e-17),
            (10.0, -0.5440211108893698, -3.8949898668223557e-17),
        ],
    );
}

#[test]
fn cos_matches_the_reference() {
    check(
        "cos",
        <Float106 as Float>::cos,
        &[
            (0.0, 1.0, 0.0),
            (0.5, 0.8775825618903728, -4.2623149864279997e-17),
            (1.0, 0.5403023058681398, -4.760954612604417e-17),
            (-1.0, 0.5403023058681398, -4.760954612604417e-17),
            (2.0, -0.4161468365471424, 1.990596398957495e-17),
            (3.0, -0.9899924966004454, -4.2060261566099734e-17),
            (0.125, 0.992197667229329, 4.754870575189364e-17),
            (-2.5, -0.8011436155469337, -1.8674742705085553e-17),
            (10.0, -0.8390715290764524, -1.4147119988953418e-17),
        ],
    );
}

#[test]
fn tan_matches_the_reference() {
    check(
        "tan",
        <Float106 as Float>::tan,
        &[
            (0.0, 0.0, 0.0),
            (0.5, 0.5463024898437905, 2.9096576216837176e-17),
            (1.0, 1.5574077246549023, -6.186464176037592e-17),
            (-1.0, -1.5574077246549023, 6.186464176037592e-17),
            (0.25, 0.25534192122103627, -5.589358343564783e-18),
            (1.2, 2.5721516221263188, -1.6240475489929127e-16),
            (-0.75, -0.9315964599440725, 1.3547381396593036e-17),
        ],
    );
}

#[test]
fn atan_matches_the_reference_including_large_arguments() {
    // The large arguments are the point: a single argument reduction leaves the series
    // argument near 1, where it converges too slowly to reach this tolerance in a bounded
    // number of terms.
    check(
        "atan",
        <Float106 as Float>::atan,
        &[
            (0.05, 0.049958395721942765, -9.988278773268116e-19),
            (0.3, 0.2914567944778671, -1.6448555435075034e-17),
            (0.7, 0.6107259643892086, 2.2418914462967458e-17),
            (1.5, 0.982793723247329, 1.3903311031230998e-17),
            (2.0, 1.1071487177940904, 9.40447137356638e-17),
            (3.0, 1.2490457723982544, -2.196203799612311e-18),
            (10.0, 1.4711276743037347, -1.0849222762061424e-16),
            (100.0, 1.5607966601082315, -1.0067563200998669e-16),
            (0.125, 0.12435499454676144, -3.1253241424539383e-18),
            (0.25, 0.24497866312686414, 1.0698755618734451e-17),
        ],
    );
}

#[test]
fn asin_matches_the_reference_including_near_one() {
    // 0.99875 forces the internal ratio x/sqrt(1-x²) up to about 20, which is where a
    // large-argument weakness in `atan` would surface.
    check(
        "asin",
        <Float106 as Float>::asin,
        &[
            (0.05, 0.050020856805770016, 1.8011559081271158e-18),
            (0.3, 0.3046926540153975, -2.7469740051157017e-17),
            (0.5, core::f64::consts::FRAC_PI_6, -5.360408832255455e-17), // asin(1/2) = pi/6
            (0.7, 0.775397496610753, 3.395027444633425e-17),
            (0.9, 1.1197695149986342, 4.092642558112641e-17),
            (0.99875, 1.520791116996175, -9.390169921393806e-17),
            (0.125, 0.1253278311680654, 1.2906010488810617e-18),
        ],
    );
}

#[test]
fn acos_matches_the_reference_including_near_one() {
    check(
        "acos",
        <Float106 as Float>::acos,
        &[
            (0.05, 1.5207754699891265, 1.0800344137659114e-16),
            (0.3, 1.2661036727794992, -7.78313736852488e-17),
            (0.5, core::f64::consts::FRAC_PI_3, -1.072081766451091e-16), // acos(1/2) = pi/3
            (0.7, 0.7953988301841436, 2.7282065511033407e-17),
            (0.9, 0.45102681179626236, 2.0305914376241248e-17),
            (0.99875, 0.050005209798721736, 2.478373285346697e-18),
            (0.125, 1.4454684956268313, -1.0659171478528689e-16),
        ],
    );
}

#[test]
fn sinh_matches_the_reference() {
    check(
        "sinh",
        <Float106 as Float>::sinh,
        &[
            (0.0, 0.0, 0.0),
            (0.5, 0.5210953054937474, -2.3328183476404597e-17),
            (1.0, 1.1752011936438014, 7.849672142285669e-17),
            (-1.0, -1.1752011936438014, -7.849672142285669e-17),
            (2.0, 3.6268604078470186, 1.9291196578353674e-16),
            (-3.0, -10.017874927409903, 6.97789774734877e-16),
            (5.0, 74.20321057778875, 6.687957117824193e-15),
        ],
    );
}

#[test]
fn cosh_matches_the_reference() {
    check(
        "cosh",
        <Float106 as Float>::cosh,
        &[
            (0.0, 1.0, 0.0),
            (0.5, 1.1276259652063807, 8.703480114456192e-17),
            (1.0, 1.5430806348152437, 6.606796775006833e-17),
            (-1.0, 1.5430806348152437, 6.606796775006833e-17),
            (2.0, 3.7621956910836314, 7.146584908813439e-17),
            (-3.0, 10.067661995777765, 5.150335194797485e-16),
            (5.0, 74.20994852478785, -3.2016056277777725e-15),
        ],
    );
}

#[test]
fn tanh_matches_the_reference() {
    check(
        "tanh",
        <Float106 as Float>::tanh,
        &[
            (0.0, 0.0, 0.0),
            (0.5, 0.46211715726000974, 2.1916603238260928e-17),
            (1.0, 0.7615941559557649, 3.7090214482164924e-17),
            (-1.0, -0.7615941559557649, -3.7090214482164924e-17),
            (2.0, 0.9640275800758169, -1.9413550547557176e-17),
            (-3.0, -0.9950547536867305, 1.2991892863562624e-17),
            (5.0, 0.9999092042625951, 2.298523750966507e-17),
        ],
    );
}

#[test]
fn sqrt_matches_the_reference() {
    check(
        "sqrt",
        <Float106 as Float>::sqrt,
        &[
            (0.0, 0.0, 0.0),
            (1.0, 1.0, 0.0),
            (2.0, core::f64::consts::SQRT_2, -9.667293313452913e-17),
            (4.0, 2.0, 0.0),
            (0.25, 0.5, 0.0),
            (123.456, 11.111075555498667, -4.773446557130318e-16),
            (1e-08, 0.0001, -3.746045560879506e-21),
            (100000000.0, 10000.0, 0.0),
        ],
    );
}

#[test]
fn cbrt_matches_the_reference() {
    check(
        "cbrt",
        <Float106 as Float>::cbrt,
        &[
            (1.0, 1.0, 0.0),
            (2.0, 1.2599210498948732, -2.589933375300507e-17),
            (8.0, 2.0, 0.0),
            (27.0, 3.0, 0.0),
            (0.125, 0.5, 0.0),
            (123.456, 4.979327984674048, 4.351928218889529e-16),
            (1000.0, 10.0, 0.0),
        ],
    );
}

// =============================================================================
// Exact results — a perfect cube or square must come back with a zero low word
// =============================================================================

#[test]
fn exactly_representable_roots_are_exact() {
    for (x, want) in [(4.0, 2.0), (0.25, 0.5), (1.0, 1.0), (1e8, 1e4)] {
        let got = <Float106 as Float>::sqrt(d(x));
        assert_eq!(got.hi(), want, "sqrt({x}) high word");
        assert_eq!(got.lo(), 0.0, "sqrt({x}) low word must be exactly zero");
    }
    for (x, want) in [
        (8.0, 2.0),
        (27.0, 3.0),
        (0.125, 0.5),
        (1000.0, 10.0),
        (-27.0, -3.0),
    ] {
        let got = <Float106 as Float>::cbrt(d(x));
        assert_eq!(got.hi(), want, "cbrt({x}) high word");
        assert_eq!(got.lo(), 0.0, "cbrt({x}) low word must be exactly zero");
    }
}

// =============================================================================
// Identities — independent of any reference table
// =============================================================================

#[test]
fn exponential_and_logarithm_invert_each_other() {
    for x in [0.125_f64, 0.5, 1.0, 2.0, 7.0, 123.456, 1e-5] {
        let round_trip = <Float106 as Float>::exp(<Float106 as Float>::ln(d(x)));
        assert!(rel_err(round_trip, d(x)) <= TOL, "exp(ln({x}))");
    }
    for x in [-3.0_f64, -0.5, 0.0, 0.5, 3.0, 10.0] {
        let round_trip = <Float106 as Float>::ln(<Float106 as Float>::exp(d(x)));
        assert!(rel_err(round_trip, d(x)) <= TOL, "ln(exp({x}))");
    }
}

#[test]
fn the_pythagorean_identity_holds() {
    for x in [0.0_f64, 0.125, 0.5, 1.0, 2.0, 3.0, -2.5, 10.0] {
        let s = <Float106 as Float>::sin(d(x));
        let c = <Float106 as Float>::cos(d(x));
        assert!(rel_err(s * s + c * c, d(1.0)) <= TOL, "sin²+cos² at {x}");
    }
}

#[test]
fn the_hyperbolic_identity_holds() {
    for x in [0.1_f64, 0.25, 0.5, 1.0] {
        let s = <Float106 as Float>::sinh(d(x));
        let c = <Float106 as Float>::cosh(d(x));
        assert!(rel_err(c * c - s * s, d(1.0)) <= TOL, "cosh²−sinh² at {x}");
    }
    // Away from zero the difference of squares cancels catastrophically, so use the
    // exponential form there instead.
    for x in [2.0_f64, 3.0, 5.0] {
        let sum = <Float106 as Float>::cosh(d(x)) + <Float106 as Float>::sinh(d(x));
        assert!(
            rel_err(sum, <Float106 as Float>::exp(d(x))) <= TOL,
            "cosh+sinh at {x}"
        );
    }
}

#[test]
fn the_inverse_trigonometric_functions_invert_their_partners() {
    // Valid only on the principal branch, |y| < π/2 for asin and atan, 0 < y < π for acos.
    for y in [0.05_f64, 0.3, 0.7, 1.0, 1.4] {
        let a = <Float106 as Float>::asin(<Float106 as Float>::sin(d(y)));
        assert!(rel_err(a, d(y)) <= TOL, "asin(sin({y}))");
        let t = <Float106 as Float>::atan(<Float106 as Float>::tan(d(y)));
        assert!(rel_err(t, d(y)) <= TOL, "atan(tan({y}))");
        let c = <Float106 as Float>::acos(<Float106 as Float>::cos(d(y)));
        assert!(rel_err(c, d(y)) <= TOL, "acos(cos({y}))");
    }
}

#[test]
fn asin_and_acos_are_complementary() {
    for x in [-0.9_f64, -0.5, 0.0, 0.125, 0.5, 0.9, 0.99875] {
        let sum = <Float106 as Float>::asin(d(x)) + <Float106 as Float>::acos(d(x));
        assert!(rel_err(sum, Float106::FRAC_PI_2) <= TOL, "asin+acos at {x}");
    }
}

#[test]
fn logarithms_of_other_bases_agree_with_the_natural_one() {
    for x in [0.5_f64, 1.0, 2.0, 8.0, 1000.0, 123.456] {
        let l2 = <Float106 as Float>::log2(d(x));
        assert!(
            rel_err(l2, <Float106 as Float>::ln(d(x)) / Float106::LN_2) <= TOL,
            "log2({x})"
        );
        let l10 = <Float106 as Float>::log10(d(x));
        assert!(
            rel_err(l10, <Float106 as Float>::ln(d(x)) / Float106::LN_10) <= TOL,
            "log10({x})"
        );
    }
    // Exact powers must land on the integer exactly.
    assert_eq!(<Float106 as Float>::log2(d(1024.0)).hi(), 10.0);
    assert_eq!(<Float106 as Float>::log10(d(1000.0)).hi(), 3.0);
}

#[test]
fn the_exponential_turns_addition_into_multiplication() {
    for (a, b) in [(0.5_f64, 0.75_f64), (1.0, 2.0), (-1.5, 0.25), (3.0, -3.0)] {
        let lhs = <Float106 as Float>::exp(d(a) + d(b));
        let rhs = <Float106 as Float>::exp(d(a)) * <Float106 as Float>::exp(d(b));
        assert!(rel_err(lhs, rhs) <= TOL, "exp({a}+{b})");
    }
}

#[test]
fn powf_agrees_with_repeated_multiplication_on_integer_exponents() {
    for x in [0.5_f64, 1.5, 2.0, 7.0] {
        let v = d(x);
        assert!(
            rel_err(<Float106 as Float>::powf(v, d(2.0)), v * v) <= TOL,
            "{x}²"
        );
        assert!(
            rel_err(<Float106 as Float>::powf(v, d(3.0)), v * v * v) <= TOL,
            "{x}³"
        );
    }
}

// =============================================================================
// Corner cases and domain boundaries
// =============================================================================

#[test]
fn trigonometric_values_at_the_axes_are_exact_or_negligible() {
    assert!(
        f64::from(<Float106 as Float>::abs(<Float106 as Float>::sin(
            Float106::PI
        ))) < 1e-31
    );
    assert!(
        f64::from(<Float106 as Float>::abs(<Float106 as Float>::cos(
            Float106::FRAC_PI_2
        ))) < 1e-31
    );
    assert!(rel_err(<Float106 as Float>::cos(Float106::PI), d(-1.0)) <= TOL);
    assert!(rel_err(<Float106 as Float>::sin(Float106::FRAC_PI_2), d(1.0)) <= TOL);
    assert!(rel_err(<Float106 as Float>::tan(Float106::FRAC_PI_4), d(1.0)) <= TOL);
    // π recovered from its own arctangent.
    assert!(rel_err(d(4.0) * <Float106 as Float>::atan(d(1.0)), Float106::PI) <= TOL);
}

#[test]
fn the_atan_boundary_at_one_is_not_a_flat_region() {
    // A shortcut returning π/4 for every argument within a tolerance of 1 would make these
    // three values identical. They are distinct.
    let below = d(1.0 - f64::EPSILON);
    let at = d(1.0);
    let above = d(1.0 + f64::EPSILON);
    let (a, b, c) = (
        <Float106 as Float>::atan(below),
        <Float106 as Float>::atan(at),
        <Float106 as Float>::atan(above),
    );
    assert!(a < b, "atan is strictly increasing below 1");
    assert!(b < c, "atan is strictly increasing above 1");
    assert_eq!(b.hi(), Float106::FRAC_PI_4.hi());
}

#[test]
fn asin_and_acos_reject_arguments_outside_their_domain() {
    for x in [1.5_f64, -1.5, 2.0, -100.0] {
        assert!(<Float106 as Float>::asin(d(x)).is_nan(), "asin({x})");
        assert!(<Float106 as Float>::acos(d(x)).is_nan(), "acos({x})");
    }
    // The endpoints are in the domain.
    assert!(rel_err(<Float106 as Float>::asin(d(1.0)), Float106::FRAC_PI_2) <= TOL);
    assert!(rel_err(<Float106 as Float>::asin(d(-1.0)), -Float106::FRAC_PI_2) <= TOL);
    assert_eq!(<Float106 as Float>::acos(d(1.0)).hi(), 0.0);
    assert!(rel_err(<Float106 as Float>::acos(d(-1.0)), Float106::PI) <= TOL);
}

#[test]
fn non_finite_arguments_propagate() {
    let inf = <Float106 as Float>::infinity();
    let nan = <Float106 as Float>::nan();
    assert!(<Float106 as Float>::sin(nan).is_nan());
    assert!(<Float106 as Float>::exp(nan).is_nan());
    assert!(<Float106 as Float>::ln(nan).is_nan());
    assert!(<Float106 as Float>::atan(nan).is_nan());
    assert!(<Float106 as Float>::exp(inf).is_infinite());
    assert!(<Float106 as Float>::ln(inf).is_infinite());
    // atan saturates at ±π/2 rather than diverging.
    assert!(rel_err(<Float106 as Float>::atan(inf), Float106::FRAC_PI_2) <= TOL);
    assert!(rel_err(<Float106 as Float>::atan(-inf), -Float106::FRAC_PI_2) <= TOL);
}

#[test]
fn the_logarithm_reports_its_domain_boundary() {
    assert!(<Float106 as Float>::ln(d(-1.0)).is_nan());
    assert!(<Float106 as Float>::ln(d(0.0)).is_infinite());
    assert!(<Float106 as Float>::ln(d(0.0)) < d(0.0));
}

#[test]
fn odd_and_even_symmetries_hold_exactly() {
    for x in [0.125_f64, 0.5, 1.0, 2.5, 10.0] {
        // Odd: f(-x) = -f(x), bit for bit.
        for f in [
            <Float106 as Float>::sin as fn(Float106) -> Float106,
            <Float106 as Float>::tan,
            <Float106 as Float>::sinh,
            <Float106 as Float>::tanh,
            <Float106 as Float>::atan,
            <Float106 as Float>::cbrt,
        ] {
            let (p, n) = (f(d(x)), f(d(-x)));
            assert_eq!(n.hi(), -p.hi(), "odd symmetry high word at {x}");
            assert_eq!(n.lo(), -p.lo(), "odd symmetry low word at {x}");
        }
        // Even: f(-x) = f(x).
        for f in [
            <Float106 as Float>::cos as fn(Float106) -> Float106,
            <Float106 as Float>::cosh,
        ] {
            assert_eq!(f(d(-x)).hi(), f(d(x)).hi(), "even symmetry at {x}");
            assert_eq!(f(d(-x)).lo(), f(d(x)).lo(), "even symmetry at {x}");
        }
    }
}
