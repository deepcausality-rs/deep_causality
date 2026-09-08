/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `EntropyConfig`, `LogBase`, `ZeroPolicy` and `Normalisation`.
//!
//! # What this file can assert, and what it cannot
//!
//! These four types carry no arithmetic. They are the three parameters an information measure
//! needs, packaged so they travel together; the meaning of each parameter — what a threshold
//! *does* to a sum, what `BySum` divides by — is behaviour of `entropy`, and is asserted in the
//! entropy suite. Here the claims are structural: which variant a named constructor picks, that a
//! builder replaces exactly one field, that a payload is stored verbatim, and which trait
//! obligations each type meets.
//!
//! Because nothing is computed, no expectation in this file can be a formula. Every expected value
//! is either an enum variant read off the type's own doc comment, or the literal that went in
//! (a round trip — an algebraic invariant, not a recomputation).
//!
//! # Corner-case rows
//!
//! * **A (empty input)**, **B (single element)** and **D (a degenerate index expression)** do not
//!   apply: no function here takes a slice, and none indexes anything. The empty-input contract
//!   belongs to the algorithms that read a distribution.
//! * **E (each documented threshold, both sides)** does not apply *behaviourally*: `SkipBelow`'s
//!   threshold and `BySum`'s floor are carried here and consulted in `entropy`. What is asserted
//!   here is that either side of such a threshold survives storage unchanged, which is the part
//!   this type is responsible for.
//! * **C, F, G, H, I, J, K** are covered below and named at their tests. Row F takes in both
//!   zeros, `0.0` and `-0.0`, because IEEE equality identifies them and the type inherits that;
//!   row J takes in a subnormal as well as the reach fixtures, because a subnormal is the
//!   magnitude a conversion through a narrower scalar would flush away.
//!
//! # Precision (row K)
//!
//! Every numeric test runs at `f32`, `f64` and `Float106` through one generic helper. No tolerance
//! is used, at any precision, and that is deliberate rather than an omission: a config field is
//! assigned and read back with no operation in between, so the correct comparison is exact
//! equality at all three precisions. A tolerance here would weaken the claim. Where the *magnitude*
//! of a fixture has to differ per precision — the reach-of-the-type test — the value is supplied by
//! the caller, one literal per precision, rather than one `f64` literal reused.

use core::fmt::Debug;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::{EntropyConfig, LogBase, Normalisation, ZeroPolicy};

/// The bound every generic helper runs under.
///
/// `RealField + FromPrimitive` is the crate's own scalar bound; `Default` is what
/// `EntropyConfig::bits` and `::nats` additionally require, and `Debug` is what `assert_eq!`
/// requires. `Copy` and `PartialEq` arrive through `Real`/`PartialOrd`.
trait TestScalar: RealField + FromPrimitive + Default + Debug {}
impl<T: RealField + FromPrimitive + Default + Debug> TestScalar for T {}

/// Exercises `Clone` without going through `Copy`.
///
/// Inside a generic function `T` is not known to be `Copy`, so this really is a clone. It also
/// keeps `clippy::clone_on_copy` from firing on the concrete types.
fn clone_of<T: Clone>(t: &T) -> T {
    t.clone()
}

/// Exercises `Copy`: a value used twice, by value, without a move error.
fn copy_twice<T: Copy>(t: T) -> (T, T) {
    (t, t)
}

/// Compiles only if `T` meets the obligations the doc comments claim for every one of these types.
fn requires_copy_clone_partial_eq_debug<T: Copy + Clone + PartialEq + Debug>() {}

/// Compiles only if `T: Eq + Hash`. `LogBase` claims both; the payload-carrying types do not.
fn requires_eq_hash<T: Eq + Hash>() {}

/// Compares one value with itself.
///
/// Written as a direct call to `PartialEq::eq` rather than as `t == t`: comparing a value with
/// itself is the property under test — `Eq` promises reflexivity and these types cannot — and the
/// operator form is what `clippy::eq_op` exists to reject.
fn is_reflexive<T: PartialEq>(t: &T) -> bool {
    PartialEq::eq(t, t)
}

fn hash_of<T: Hash>(t: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    t.hash(&mut hasher);
    hasher.finish()
}

// ---------------------------------------------------------------------------------------------
// Named constructors and Default
// ---------------------------------------------------------------------------------------------

/// `bits()` is documented as "Bits, skipping exactly zero, taking the input as a distribution".
fn check_bits_fields<T: TestScalar>() {
    let config = EntropyConfig::<T>::bits();

    // Expected values are the three variants named in the doc comment of `EntropyConfig::bits`,
    // not computed values.
    assert_eq!(config.base, LogBase::Bits);
    assert_eq!(config.zero_policy, ZeroPolicy::SkipZero);
    assert_eq!(config.normalisation, Normalisation::None);
}

#[test]
fn test_bits_sets_documented_fields() {
    check_bits_fields::<f32>();
    check_bits_fields::<f64>();
    check_bits_fields::<Float106>();
}

/// `nats()` is documented as "Nats, skipping exactly zero, taking the input as a distribution".
fn check_nats_fields<T: TestScalar>() {
    let config = EntropyConfig::<T>::nats();

    assert_eq!(config.base, LogBase::Nats);
    assert_eq!(config.zero_policy, ZeroPolicy::SkipZero);
    assert_eq!(config.normalisation, Normalisation::None);
}

#[test]
fn test_nats_sets_documented_fields() {
    check_nats_fields::<f32>();
    check_nats_fields::<f64>();
    check_nats_fields::<Float106>();
}

/// `EntropyConfig`: "The default is bits, skip-at-zero, no normalisation." Each of the three
/// variant defaults is also declared at its own type by `#[default]`, and this asserts all three
/// compose into the config's default rather than asserting the derive in the abstract.
fn check_default_fields<T: TestScalar>() {
    let config = EntropyConfig::<T>::default();

    assert_eq!(config.base, LogBase::Bits);
    assert_eq!(config.zero_policy, ZeroPolicy::SkipZero);
    assert_eq!(config.normalisation, Normalisation::None);

    // The per-type defaults, asserted directly against the `#[default]` attribute on each enum.
    assert_eq!(LogBase::default(), LogBase::Bits);
    assert_eq!(ZeroPolicy::<T>::default(), ZeroPolicy::SkipZero);
    assert_eq!(Normalisation::<T>::default(), Normalisation::None);
}

#[test]
fn test_default_is_bits_skip_zero_none() {
    check_default_fields::<f32>();
    check_default_fields::<f64>();
    check_default_fields::<Float106>();
}

/// Row C — two distinct constructions coincide. `bits()` and `default()` are documented to name
/// the same configuration, so they must be equal as values, not merely field-by-field similar.
fn check_bits_and_default_coincide<T: TestScalar>() {
    assert_eq!(EntropyConfig::<T>::bits(), EntropyConfig::<T>::default());

    // And `nats()` is a different configuration, so the coincidence is not vacuous.
    assert_ne!(EntropyConfig::<T>::nats(), EntropyConfig::<T>::default());
}

#[test]
fn test_bits_and_default_coincide() {
    check_bits_and_default_coincide::<f32>();
    check_bits_and_default_coincide::<f64>();
    check_bits_and_default_coincide::<Float106>();
}

/// Invariant: the two named constructors differ in the base and in nothing else, so rebasing one
/// reproduces the other exactly. This is the "factor of `ln 2` apart, same everything else"
/// statement from the crate docs, expressed at the config level.
fn check_nats_is_rebased_bits<T: TestScalar>() {
    let bits = EntropyConfig::<T>::bits();
    let nats = EntropyConfig::<T>::nats();

    assert_eq!(bits.with_base(LogBase::Nats), nats);
    assert_eq!(nats.with_base(LogBase::Bits), bits);
}

#[test]
fn test_nats_is_bits_with_only_the_base_replaced() {
    check_nats_is_rebased_bits::<f32>();
    check_nats_is_rebased_bits::<f64>();
    check_nats_is_rebased_bits::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// Builders: exactly one field each
// ---------------------------------------------------------------------------------------------

/// `with_base` "Replaces the base." The other two fields are the ones the starting config had.
fn check_with_base_replaces_only_the_base<T: TestScalar>() {
    // 0.25 is an arbitrary carried payload; nothing computes with it. It exists so that a builder
    // that reset its sibling fields to their defaults would be caught.
    let threshold = lift::<T>(0.25);
    let floor = lift::<T>(0.5);

    let start = EntropyConfig::<T>::bits()
        .with_zero_policy(ZeroPolicy::SkipBelow(threshold))
        .with_normalisation(Normalisation::BySum { floor });

    let moved = start.with_base(LogBase::Nats);

    assert_eq!(moved.base, LogBase::Nats);
    // Unchanged: the same payloads that went in, byte for byte.
    assert_eq!(moved.zero_policy, ZeroPolicy::SkipBelow(threshold));
    assert_eq!(moved.normalisation, Normalisation::BySum { floor });

    // Applying it with the value already in place is the identity (row C: the two coincide).
    assert_eq!(start.with_base(start.base), start);
}

#[test]
fn test_with_base_replaces_only_the_base() {
    check_with_base_replaces_only_the_base::<f32>();
    check_with_base_replaces_only_the_base::<f64>();
    check_with_base_replaces_only_the_base::<Float106>();
}

/// `with_zero_policy` "Replaces the zero policy."
fn check_with_zero_policy_replaces_only_that<T: TestScalar>() {
    let floor = lift::<T>(0.5);
    let threshold = lift::<T>(0.25);

    let start = EntropyConfig::<T>::nats().with_normalisation(Normalisation::BySum { floor });

    let moved = start.with_zero_policy(ZeroPolicy::SkipBelow(threshold));

    assert_eq!(moved.zero_policy, ZeroPolicy::SkipBelow(threshold));
    assert_eq!(moved.base, LogBase::Nats);
    assert_eq!(moved.normalisation, Normalisation::BySum { floor });

    assert_eq!(start.with_zero_policy(start.zero_policy), start);
}

#[test]
fn test_with_zero_policy_replaces_only_the_zero_policy() {
    check_with_zero_policy_replaces_only_that::<f32>();
    check_with_zero_policy_replaces_only_that::<f64>();
    check_with_zero_policy_replaces_only_that::<Float106>();
}

/// `with_normalisation` "Replaces the normalisation."
fn check_with_normalisation_replaces_only_that<T: TestScalar>() {
    let threshold = lift::<T>(0.25);
    let floor = lift::<T>(0.5);

    let start = EntropyConfig::<T>::nats().with_zero_policy(ZeroPolicy::SkipBelow(threshold));

    let moved = start.with_normalisation(Normalisation::BySum { floor });

    assert_eq!(moved.normalisation, Normalisation::BySum { floor });
    assert_eq!(moved.base, LogBase::Nats);
    assert_eq!(moved.zero_policy, ZeroPolicy::SkipBelow(threshold));

    assert_eq!(start.with_normalisation(start.normalisation), start);
}

#[test]
fn test_with_normalisation_replaces_only_the_normalisation() {
    check_with_normalisation_replaces_only_that::<f32>();
    check_with_normalisation_replaces_only_that::<f64>();
    check_with_normalisation_replaces_only_that::<Float106>();
}

/// Invariant: because each builder owns a different field, the three commute, and a repeated
/// builder keeps the last value written. Both follow from "replaces exactly one field" and neither
/// recomputes anything.
fn check_builders_commute_and_overwrite<T: TestScalar>() {
    let threshold = lift::<T>(0.25);
    let floor = lift::<T>(0.5);
    let start = EntropyConfig::<T>::bits();

    let one_order = start
        .with_base(LogBase::Nats)
        .with_zero_policy(ZeroPolicy::SkipBelow(threshold))
        .with_normalisation(Normalisation::BySum { floor });
    let other_order = start
        .with_normalisation(Normalisation::BySum { floor })
        .with_zero_policy(ZeroPolicy::SkipBelow(threshold))
        .with_base(LogBase::Nats);

    assert_eq!(one_order, other_order);

    // Last write wins, on every field.
    assert_eq!(
        start.with_base(LogBase::Nats).with_base(LogBase::Bits),
        start.with_base(LogBase::Bits)
    );
    assert_eq!(
        start
            .with_zero_policy(ZeroPolicy::SkipBelow(threshold))
            .with_zero_policy(ZeroPolicy::SkipZero),
        start.with_zero_policy(ZeroPolicy::SkipZero)
    );
    assert_eq!(
        start
            .with_normalisation(Normalisation::BySum { floor })
            .with_normalisation(Normalisation::None),
        start.with_normalisation(Normalisation::None)
    );
}

#[test]
fn test_builders_commute_and_last_write_wins() {
    check_builders_commute_and_overwrite::<f32>();
    check_builders_commute_and_overwrite::<f64>();
    check_builders_commute_and_overwrite::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// Payloads: what the config stores, at the numeric corners
// ---------------------------------------------------------------------------------------------

/// Reads the threshold out of a `SkipBelow`, failing on any other variant.
fn threshold_of<T: TestScalar>(policy: ZeroPolicy<T>) -> T {
    match policy {
        ZeroPolicy::SkipBelow(t) => t,
        ZeroPolicy::SkipZero => panic!("expected SkipBelow, got SkipZero"),
    }
}

/// Reads the floor out of a `BySum`, failing on any other variant.
fn floor_of<T: TestScalar>(normalisation: Normalisation<T>) -> T {
    match normalisation {
        Normalisation::BySum { floor } => floor,
        Normalisation::None => panic!("expected BySum, got None"),
    }
}

/// Rows F, G and H — zero, a negative value, and the exact ends of the probability interval.
///
/// The payload is stored, not interpreted: `ZeroPolicy` documents that a *negative entry* in the
/// data is rejected by every function in the crate, and says nothing about the threshold itself
/// being constrained, so the reading asserted here is that the config stores whatever it is given
/// and validation happens where the data is read. Each expectation is the literal that went in.
fn check_edge_payloads_stored_verbatim<T: TestScalar>() {
    // 0.0 (row F), -1.0 (row G), and the two ends of the probability domain, 0.0 and 1.0 (row H).
    let cases = [0.0_f64, -1.0_f64, 1.0_f64];

    for &value in cases.iter() {
        let lifted = lift::<T>(value);

        let config = EntropyConfig::<T>::bits()
            .with_zero_policy(ZeroPolicy::SkipBelow(lifted))
            .with_normalisation(Normalisation::BySum { floor: lifted });

        assert_eq!(threshold_of(config.zero_policy), lifted);
        assert_eq!(floor_of(config.normalisation), lifted);
    }

    // Zero specifically: the stored threshold is the additive identity of the scalar, so a config
    // built from the literal 0.0 and one built from `T::zero()` are the same value.
    assert_eq!(
        ZeroPolicy::SkipBelow(lift::<T>(0.0)),
        ZeroPolicy::SkipBelow(T::zero())
    );
    // And one (row H, upper end) is the multiplicative identity.
    assert_eq!(
        Normalisation::BySum {
            floor: lift::<T>(1.0)
        },
        Normalisation::BySum { floor: T::one() }
    );
    // A negative threshold is a different value from its magnitude — the sign is kept (row G).
    assert_ne!(
        ZeroPolicy::SkipBelow(lift::<T>(-1.0)),
        ZeroPolicy::SkipBelow(lift::<T>(1.0))
    );
    // The other side of zero (row F). `-0.0` and `0.0` are distinct bit patterns and IEEE
    // equality identifies them, so — unlike the `-1.0` case just above — the sign of a zero does
    // *not* separate two configs. Asserted rather than assumed: equality on these types is the
    // scalar's, and the scalar's says these two are one value.
    assert_eq!(
        ZeroPolicy::SkipBelow(lift::<T>(-0.0)),
        ZeroPolicy::SkipBelow(T::zero())
    );
    assert_eq!(
        Normalisation::BySum {
            floor: lift::<T>(-0.0)
        },
        Normalisation::BySum { floor: T::zero() }
    );
    assert_eq!(
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(lift::<T>(-0.0))),
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(T::zero()))
    );
}

#[test]
fn test_zero_negative_and_boundary_payloads_stored_verbatim() {
    check_edge_payloads_stored_verbatim::<f32>();
    check_edge_payloads_stored_verbatim::<f64>();
    check_edge_payloads_stored_verbatim::<Float106>();
}

/// Row I — a non-finite payload. The infinities are ordinary values under `PartialEq`, so they
/// round-trip and compare equal to themselves; the config performs no arithmetic that could turn
/// one into a `NaN`.
fn check_non_finite_payloads<T: TestScalar>() {
    let pos_inf = lift::<T>(f64::INFINITY);
    let neg_inf = lift::<T>(f64::NEG_INFINITY);

    let config = EntropyConfig::<T>::bits()
        .with_zero_policy(ZeroPolicy::SkipBelow(pos_inf))
        .with_normalisation(Normalisation::BySum { floor: neg_inf });

    // The stored values are still the infinities that went in.
    assert!(threshold_of(config.zero_policy).is_infinite());
    assert!(floor_of(config.normalisation).is_infinite());
    assert_eq!(threshold_of(config.zero_policy), pos_inf);
    assert_eq!(floor_of(config.normalisation), neg_inf);
    // The two infinities are distinct, so the sign survived storage.
    assert_ne!(
        ZeroPolicy::SkipBelow(pos_inf),
        ZeroPolicy::SkipBelow(neg_inf)
    );

    // A NaN threshold is stored as a NaN, and so is a NaN floor: the two payload fields are
    // asserted separately because they are separate fields, and only one of them is on the
    // `ZeroPolicy` side.
    let nan = lift::<T>(f64::NAN);
    let nan_config = EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(nan));
    assert!(threshold_of(nan_config.zero_policy).is_nan());
    let nan_floor_config =
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: nan });
    assert!(floor_of(nan_floor_config.normalisation).is_nan());
    // The NaN did not leak into the field next to it: the zero policy is the one `bits()` set.
    assert_eq!(nan_floor_config.zero_policy, ZeroPolicy::SkipZero);
}

#[test]
fn test_non_finite_payloads_stored_verbatim() {
    check_non_finite_payloads::<f32>();
    check_non_finite_payloads::<f64>();
    check_non_finite_payloads::<Float106>();
}

/// Row I, and the reason `ZeroPolicy`, `Normalisation` and `EntropyConfig` derive `PartialEq`
/// rather than `Eq`: they carry a float, `NaN != NaN`, and reflexivity — which `Eq` promises —
/// fails. This is the positive demonstration of that claim. Rust cannot express "`T` does not
/// implement `Eq`" as an assertion, so the absence of the impl is not asserted here; what is
/// asserted is that adding it would be wrong.
fn check_nan_payload_breaks_reflexivity<T: TestScalar>() {
    let nan = lift::<T>(f64::NAN);

    let policy = ZeroPolicy::SkipBelow(nan);
    assert!(!is_reflexive(&policy));

    let normalisation = Normalisation::BySum { floor: nan };
    assert!(!is_reflexive(&normalisation));

    let config = EntropyConfig::<T>::bits().with_zero_policy(policy);
    assert!(!is_reflexive(&config));

    // A clone of a NaN-carrying config is likewise not equal to its source, for the same reason.
    assert_ne!(clone_of(&config), config);

    // The payload-free variants are reflexive, so the failure above is the payload's doing.
    assert!(is_reflexive(&ZeroPolicy::<T>::SkipZero));
    assert!(is_reflexive(&Normalisation::<T>::None));
    assert!(is_reflexive(&EntropyConfig::<T>::bits()));
    // And a clone of one of those does compare equal, which is the round trip the payload denies.
    let skip_zero = ZeroPolicy::<T>::SkipZero;
    assert_eq!(clone_of(&skip_zero), skip_zero);
}

#[test]
fn test_nan_payload_breaks_reflexivity() {
    check_nan_payload_breaks_reflexivity::<f32>();
    check_nan_payload_breaks_reflexivity::<f64>();
    check_nan_payload_breaks_reflexivity::<Float106>();
}

/// Row J — the reach of the scalar itself. `huge`, `tiny` and `subnormal` are supplied per
/// precision rather than as one `f64` literal, because `f32` overflows five orders of magnitude
/// short of where `f64` and `Float106` do, and a fixture near `f64`'s ceiling would silently
/// become an infinity at `f32`. The claim is that storage neither overflows nor flushes: what goes
/// in comes out, and stays finite and non-zero.
///
/// `subnormal` sits below the scalar's smallest normal, which is the magnitude a conversion that
/// went through a narrower type would flush to zero.
fn check_extreme_magnitudes_round_trip<T: TestScalar>(huge: T, tiny: T, subnormal: T) {
    assert!(huge.is_finite(), "fixture must be finite in this scalar");
    assert!(tiny > T::zero(), "fixture must not underflow to zero");
    assert!(
        subnormal > T::zero(),
        "fixture must not underflow to zero in this scalar"
    );
    assert!(
        subnormal < tiny,
        "fixture must be smaller than the normal-range one"
    );

    let config = EntropyConfig::<T>::bits()
        .with_zero_policy(ZeroPolicy::SkipBelow(tiny))
        .with_normalisation(Normalisation::BySum { floor: huge });

    assert_eq!(threshold_of(config.zero_policy), tiny);
    assert_eq!(floor_of(config.normalisation), huge);
    assert!(threshold_of(config.zero_policy).is_finite());
    assert!(floor_of(config.normalisation).is_finite());

    // The type's own smallest meaningful step also survives, and is distinct from zero.
    // `epsilon` and the finiteness predicates belong to `Real`, the analytic axis of the tower;
    // `RealField` is `Real + Field` and resolves them too, so the qualification just names where
    // they are defined.
    let eps_config =
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(<T as Real>::epsilon()));
    assert_eq!(threshold_of(eps_config.zero_policy), <T as Real>::epsilon());
    assert_ne!(
        ZeroPolicy::SkipBelow(<T as Real>::epsilon()),
        ZeroPolicy::SkipBelow(T::zero())
    );

    // And a payload below the smallest normal survives too: the config stores it rather than
    // flushing it to zero or rounding it up.
    let subnormal_config = EntropyConfig::<T>::bits()
        .with_zero_policy(ZeroPolicy::SkipBelow(subnormal))
        .with_normalisation(Normalisation::BySum { floor: subnormal });
    assert_eq!(threshold_of(subnormal_config.zero_policy), subnormal);
    assert_eq!(floor_of(subnormal_config.normalisation), subnormal);
    assert_ne!(
        ZeroPolicy::SkipBelow(subnormal),
        ZeroPolicy::SkipBelow(T::zero())
    );
}

#[test]
fn test_extreme_magnitude_payloads_round_trip() {
    // f32 tops out near 3.4028235e38 and its smallest normal is near 1.1754944e-38; 3.0e38 and
    // 1.0e-37 sit inside both, so neither fixture is an infinity or a subnormal at this precision.
    // 1.0e-42 is below that smallest normal and above the smallest subnormal, near 1.4012985e-45.
    check_extreme_magnitudes_round_trip::<f32>(3.0e38_f32, 1.0e-37_f32, 1.0e-42_f32);
    // f64 tops out near 1.7976931e308 with smallest normal near 2.2250738e-308; 1.0e-320 is below
    // that and above the smallest subnormal, near 4.9406565e-324.
    check_extreme_magnitudes_round_trip::<f64>(1.0e308_f64, 1.0e-300_f64, 1.0e-320_f64);
    // Float106 is a double-double: its exponent range is the f64 range of its high limb, so the
    // f64 fixtures are the right magnitudes here too.
    check_extreme_magnitudes_round_trip::<Float106>(lift(1.0e308), lift(1.0e-300), lift(1.0e-320));
}

// ---------------------------------------------------------------------------------------------
// Trait obligations
// ---------------------------------------------------------------------------------------------

/// All four types are `Copy`, `Clone`, `PartialEq` and `Debug`, and a clone round-trips.
fn check_trait_surface_and_clone_round_trip<T: TestScalar>() {
    requires_copy_clone_partial_eq_debug::<LogBase>();
    requires_copy_clone_partial_eq_debug::<ZeroPolicy<T>>();
    requires_copy_clone_partial_eq_debug::<Normalisation<T>>();
    requires_copy_clone_partial_eq_debug::<EntropyConfig<T>>();

    let threshold = lift::<T>(0.25);
    let floor = lift::<T>(0.5);

    let bases = [LogBase::Bits, LogBase::Nats];
    let policies = [ZeroPolicy::SkipZero, ZeroPolicy::SkipBelow(threshold)];
    let normalisations = [Normalisation::None, Normalisation::BySum { floor }];

    for &base in bases.iter() {
        assert_eq!(clone_of(&base), base);
        let (a, b) = copy_twice(base);
        assert_eq!(a, b);
    }
    for &policy in policies.iter() {
        assert_eq!(clone_of(&policy), policy);
        let (a, b) = copy_twice(policy);
        assert_eq!(a, b);
    }
    for &normalisation in normalisations.iter() {
        assert_eq!(clone_of(&normalisation), normalisation);
        let (a, b) = copy_twice(normalisation);
        assert_eq!(a, b);
    }

    let config = EntropyConfig::<T>::nats()
        .with_zero_policy(ZeroPolicy::SkipBelow(threshold))
        .with_normalisation(Normalisation::BySum { floor });
    assert_eq!(clone_of(&config), config);
    let (a, b) = copy_twice(config);
    assert_eq!(a, b);
    // The original is still usable after being copied twice, which is what `Copy` buys.
    assert_eq!(config.base, LogBase::Nats);

    // A derived `Debug` on a fieldless variant prints exactly the variant's name; on a variant
    // with a payload it prints the name followed by the payload, whose own rendering differs by
    // scalar, so only the name is asserted there.
    assert_eq!(format!("{:?}", LogBase::Bits), "Bits");
    assert_eq!(format!("{:?}", LogBase::Nats), "Nats");
    assert_eq!(format!("{:?}", ZeroPolicy::<T>::SkipZero), "SkipZero");
    assert_eq!(format!("{:?}", Normalisation::<T>::None), "None");
    assert!(format!("{:?}", ZeroPolicy::SkipBelow(threshold)).starts_with("SkipBelow"));
    assert!(format!("{:?}", Normalisation::BySum { floor }).starts_with("BySum"));
    assert!(format!("{config:?}").starts_with("EntropyConfig"));
}

#[test]
fn test_types_are_copy_clone_partial_eq_debug_and_round_trip() {
    check_trait_surface_and_clone_round_trip::<f32>();
    check_trait_surface_and_clone_round_trip::<f64>();
    check_trait_surface_and_clone_round_trip::<Float106>();
}

/// `LogBase` has no float payload, so it is `Eq` and `Hash` where the other three are not. This
/// test is not generic: `LogBase` has no scalar parameter, so there is nothing to run three times.
#[test]
fn test_log_base_is_eq_and_hash() {
    requires_eq_hash::<LogBase>();

    // `Eq` is reflexivity, which the payload-carrying types cannot promise (see the NaN test).
    assert_eq!(LogBase::Bits, LogBase::Bits);
    assert_eq!(LogBase::Nats, LogBase::Nats);
    assert_ne!(LogBase::Bits, LogBase::Nats);

    // The `Hash`/`Eq` contract: equal values hash equal. The converse is not asserted, because
    // unequal values are permitted to collide.
    assert_eq!(hash_of(&LogBase::Bits), hash_of(&LogBase::Bits));
    assert_eq!(hash_of(&LogBase::Nats), hash_of(&LogBase::Nats));

    // Usable as a key: two distinct variants occupy two distinct slots, and a repeat of one does
    // not add a third.
    let mut set = HashSet::new();
    set.insert(LogBase::Bits);
    set.insert(LogBase::Nats);
    set.insert(LogBase::Bits);
    assert_eq!(set.len(), 2);

    let mut units = HashMap::new();
    units.insert(LogBase::Bits, "shannon");
    units.insert(LogBase::Nats, "nat");
    assert_eq!(units.get(&LogBase::Bits), Some(&"shannon"));
    assert_eq!(units.get(&LogBase::Nats), Some(&"nat"));
}

/// Row C — where two things could be confused for one, they are not.
///
/// `SkipZero` and `SkipBelow(0)` describe predicates that agree on every non-negative input
/// ("exactly zero" and "at or below zero" select the same entries once negatives are refused), yet
/// they are different values of the type, and a config carrying one is not a config carrying the
/// other. Same for `None` against a `BySum` at any floor.
fn check_variants_are_distinct<T: TestScalar>() {
    let zero = T::zero();

    assert_ne!(ZeroPolicy::SkipZero, ZeroPolicy::SkipBelow(zero));
    assert_ne!(Normalisation::None, Normalisation::BySum { floor: zero });

    assert_ne!(
        EntropyConfig::<T>::bits(),
        EntropyConfig::<T>::bits().with_zero_policy(ZeroPolicy::SkipBelow(zero))
    );
    assert_ne!(
        EntropyConfig::<T>::bits(),
        EntropyConfig::<T>::bits().with_normalisation(Normalisation::BySum { floor: zero })
    );

    // Two `SkipBelow`s differ exactly when their thresholds do.
    assert_eq!(
        ZeroPolicy::SkipBelow(lift::<T>(0.25)),
        ZeroPolicy::SkipBelow(lift::<T>(0.25))
    );
    assert_ne!(
        ZeroPolicy::SkipBelow(lift::<T>(0.25)),
        ZeroPolicy::SkipBelow(lift::<T>(0.5))
    );
}

#[test]
fn test_coinciding_variants_are_still_distinct_values() {
    check_variants_are_distinct::<f32>();
    check_variants_are_distinct::<f64>();
    check_variants_are_distinct::<Float106>();
}

/// `EntropyConfig` documents that "the three axes are independent". Two consequences are asserted
/// here, and neither recomputes anything.
///
/// First, the builders are the field assignments they claim to be: a config named field by field
/// in a struct literal is the same value as one assembled by three builder calls. The struct
/// literal also fails to compile if the type ever grows a fourth field, so the "replaces exactly
/// one field" tests above cannot go stale by omission.
///
/// Second, the eight settings of the three axes — two bases, a payload-free and a payload-carrying
/// choice on each of the other two — are eight distinct configurations. If any pair collapsed, one
/// axis would not be free of the others.
fn check_axes_are_independent<T: TestScalar>() {
    let threshold = lift::<T>(0.25);
    let floor = lift::<T>(0.5);

    let bases = [LogBase::Bits, LogBase::Nats];
    let policies = [ZeroPolicy::SkipZero, ZeroPolicy::SkipBelow(threshold)];
    let normalisations = [Normalisation::None, Normalisation::BySum { floor }];

    let mut settings = Vec::new();
    for &base in bases.iter() {
        for &zero_policy in policies.iter() {
            for &normalisation in normalisations.iter() {
                let named = EntropyConfig {
                    base,
                    zero_policy,
                    normalisation,
                };
                let built = EntropyConfig::<T>::bits()
                    .with_base(base)
                    .with_zero_policy(zero_policy)
                    .with_normalisation(normalisation);
                assert_eq!(built, named);
                settings.push(named);
            }
        }
    }

    assert_eq!(settings.len(), 8);
    for (i, left) in settings.iter().enumerate() {
        for right in settings.iter().skip(i + 1) {
            assert_ne!(left, right);
        }
    }
}

#[test]
fn test_axes_are_independent_and_builders_match_a_struct_literal() {
    check_axes_are_independent::<f32>();
    check_axes_are_independent::<f64>();
    check_axes_are_independent::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// The three combinations the workspace actually needs
//
// `EntropyConfig` documents that "Each combination named here has a caller; none is speculative."
// Each test below names that caller and asserts the fields its shipped code implies.
// ---------------------------------------------------------------------------------------------

/// The causal-discovery path: bits, skip at exactly zero, input taken as a distribution.
///
/// Caller: `entropy_nvars` in
/// `deep_causality_algorithms/src/causal_discovery/surd/surd_utils/mod.rs`, which folds
/// `acc - prob * prob.log2()` over the entries with `prob > zero` and normalises nothing.
/// Base 2 gives `LogBase::Bits`; the `> zero` guard gives `ZeroPolicy::SkipZero`; the absence of
/// any division by a sum gives `Normalisation::None`.
fn check_causal_discovery_config<T: TestScalar>() {
    let config = EntropyConfig::<T>::bits();

    assert_eq!(config.base, LogBase::Bits);
    assert_eq!(config.zero_policy, ZeroPolicy::SkipZero);
    assert_eq!(config.normalisation, Normalisation::None);
}

#[test]
fn test_causal_discovery_config_is_bits_skip_zero_none() {
    check_causal_discovery_config::<f32>();
    check_causal_discovery_config::<f64>();
    check_causal_discovery_config::<Float106>();
}

/// The option-carrying variant: bits, skip below a threshold, normalise by the sum.
///
/// Caller: `entropy_nvars_cdl` in
/// `deep_causality_algorithms/src/causal_discovery/surd/surd_utils/surd_utils_cdl.rs`, which sets
/// `eps = T::epsilon()`, returns zero when `sum_of_marginals.abs() < eps`, divides each present
/// entry by that sum, and admits the result only when `normalized_prob > eps`. So the threshold
/// and the floor are both `T::epsilon()` — one config value per scalar, which is why the payload
/// is generic rather than an `f64`.
fn check_option_carrying_config<T: TestScalar>() {
    let eps = <T as Real>::epsilon();

    let config = EntropyConfig::<T>::bits()
        .with_zero_policy(ZeroPolicy::SkipBelow(eps))
        .with_normalisation(Normalisation::BySum { floor: eps });

    assert_eq!(config.base, LogBase::Bits);
    assert_eq!(config.zero_policy, ZeroPolicy::SkipBelow(eps));
    assert_eq!(config.normalisation, Normalisation::BySum { floor: eps });

    // The two payloads are the same number here, and the type keeps them in separate fields
    // rather than collapsing them.
    assert_eq!(
        threshold_of(config.zero_policy),
        floor_of(config.normalisation)
    );
    // Each is the scalar's own epsilon, so the config is not carrying a value tuned for f64.
    assert_eq!(threshold_of(config.zero_policy), eps);
    assert!(threshold_of(config.zero_policy) > T::zero());
}

#[test]
fn test_option_carrying_config_is_bits_skip_below_by_sum() {
    check_option_carrying_config::<f32>();
    check_option_carrying_config::<f64>();
    check_option_carrying_config::<Float106>();
}

/// The thermodynamics path, before its migration to bits: nats, skip at exactly zero, no
/// normalisation.
///
/// Caller: `shannon_entropy_kernel` in
/// `deep_causality_physics/src/kernels/thermodynamics/stats.rs`, which sums `-p * p.ln()` over the
/// entries passing `p > R::zero()`. The natural logarithm gives `LogBase::Nats`; the guard gives
/// `ZeroPolicy::SkipZero`; nothing is divided by a sum, so `Normalisation::None`.
fn check_thermodynamics_config<T: TestScalar>() {
    let config = EntropyConfig::<T>::nats();

    assert_eq!(config.base, LogBase::Nats);
    assert_eq!(config.zero_policy, ZeroPolicy::SkipZero);
    assert_eq!(config.normalisation, Normalisation::None);
}

#[test]
fn test_thermodynamics_config_is_nats_skip_zero_none() {
    check_thermodynamics_config::<f32>();
    check_thermodynamics_config::<f64>();
    check_thermodynamics_config::<Float106>();
}

/// The three shipped combinations are three different configurations. If any two were equal the
/// crate's reason for existing — that the shipped implementations disagree — would not hold.
fn check_shipped_configs_are_pairwise_distinct<T: TestScalar>() {
    let eps = <T as Real>::epsilon();

    let causal_discovery = EntropyConfig::<T>::bits();
    let option_carrying = EntropyConfig::<T>::bits()
        .with_zero_policy(ZeroPolicy::SkipBelow(eps))
        .with_normalisation(Normalisation::BySum { floor: eps });
    let thermodynamics = EntropyConfig::<T>::nats();

    assert_ne!(causal_discovery, option_carrying);
    assert_ne!(causal_discovery, thermodynamics);
    assert_ne!(option_carrying, thermodynamics);

    // The first and the third differ in the base alone; that is the `ln 2` disagreement.
    assert_eq!(causal_discovery.zero_policy, thermodynamics.zero_policy);
    assert_eq!(causal_discovery.normalisation, thermodynamics.normalisation);
    assert_ne!(causal_discovery.base, thermodynamics.base);

    // The first and the second share a base and differ in the other two axes.
    assert_eq!(causal_discovery.base, option_carrying.base);
    assert_ne!(causal_discovery.zero_policy, option_carrying.zero_policy);
    assert_ne!(
        causal_discovery.normalisation,
        option_carrying.normalisation
    );
}

#[test]
fn test_the_three_shipped_configs_are_pairwise_distinct() {
    check_shipped_configs_are_pairwise_distinct::<f32>();
    check_shipped_configs_are_pairwise_distinct::<f64>();
    check_shipped_configs_are_pairwise_distinct::<Float106>();
}
