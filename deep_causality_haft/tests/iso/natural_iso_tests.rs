/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier 3 `NaturalIso<F, G>` round-trip and naturality tests.

use deep_causality_haft::iso::test_support::{
    assert_natural_iso_naturality, assert_natural_iso_round_trip,
};
use deep_causality_haft::{Functor, HKT, NaturalIso, NoConstraint, OptionWitness, Satisfies};

// =============================================================================
// Fixtures (inlined per Bazel `rust_test_suite` per-file model)
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum MyOption<T> {
    MySome(T),
    MyNone,
}

struct MyOptionWitness;

impl HKT for MyOptionWitness {
    type Constraint = NoConstraint;
    type Type<T> = MyOption<T>;
}

impl Functor<MyOptionWitness> for MyOptionWitness {
    fn fmap<A, B, Func>(m_a: MyOption<A>, mut f: Func) -> MyOption<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        match m_a {
            MyOption::MySome(a) => MyOption::MySome(f(a)),
            MyOption::MyNone => MyOption::MyNone,
        }
    }
}

/// Canonical natural iso between `Option` and `MyOption`.
struct OptionMyOptionIso;

impl NaturalIso<OptionWitness, MyOptionWitness> for OptionMyOptionIso {
    fn to_target<T>(fa: Option<T>) -> MyOption<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        match fa {
            Some(t) => MyOption::MySome(t),
            None => MyOption::MyNone,
        }
    }

    fn to_source<T>(ga: MyOption<T>) -> Option<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        match ga {
            MyOption::MySome(t) => Some(t),
            MyOption::MyNone => None,
        }
    }
}

/// `to_target` is correct; `to_source` always returns `None`. Used to
/// exercise the `F -> G -> F` panic branch.
struct BrokenReverseIso;

impl NaturalIso<OptionWitness, MyOptionWitness> for BrokenReverseIso {
    fn to_target<T>(fa: Option<T>) -> MyOption<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        match fa {
            Some(t) => MyOption::MySome(t),
            None => MyOption::MyNone,
        }
    }

    fn to_source<T>(_ga: MyOption<T>) -> Option<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        None
    }
}

/// `to_target` collapses any `Option` to `MyNone`; `to_source` is correct.
/// Used to exercise the independent `G -> F -> G` panic branch (the case
/// would slip through if `ga` were derived from `fa`).
struct LossyForwardIso;

impl NaturalIso<OptionWitness, MyOptionWitness> for LossyForwardIso {
    fn to_target<T>(_fa: Option<T>) -> MyOption<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        MyOption::MyNone
    }

    fn to_source<T>(ga: MyOption<T>) -> Option<T>
    where
        T: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        match ga {
            MyOption::MySome(t) => Some(t),
            MyOption::MyNone => None,
        }
    }
}

// =============================================================================
// to_target / to_source direct calls
// =============================================================================

#[test]
fn to_target_maps_some_to_my_some() {
    let out: MyOption<i32> =
        <OptionMyOptionIso as NaturalIso<OptionWitness, MyOptionWitness>>::to_target(Some(7));
    assert_eq!(out, MyOption::MySome(7));
}

#[test]
fn to_target_maps_none_to_my_none() {
    let out: MyOption<i32> =
        <OptionMyOptionIso as NaturalIso<OptionWitness, MyOptionWitness>>::to_target(None);
    assert_eq!(out, MyOption::MyNone);
}

#[test]
fn to_source_maps_my_some_to_some() {
    let out: Option<i32> =
        <OptionMyOptionIso as NaturalIso<OptionWitness, MyOptionWitness>>::to_source(
            MyOption::MySome(11),
        );
    assert_eq!(out, Some(11));
}

#[test]
fn to_source_maps_my_none_to_none() {
    let out: Option<i32> =
        <OptionMyOptionIso as NaturalIso<OptionWitness, MyOptionWitness>>::to_source(
            MyOption::MyNone,
        );
    assert_eq!(out, None);
}

// =============================================================================
// Round-trip law
// =============================================================================

#[test]
fn round_trip_holds_for_canonical_iso() {
    assert_natural_iso_round_trip::<OptionMyOptionIso, OptionWitness, MyOptionWitness, i32>(
        Some(42),
        MyOption::MySome(42),
    );
    assert_natural_iso_round_trip::<OptionMyOptionIso, OptionWitness, MyOptionWitness, i32>(
        None,
        MyOption::MyNone,
    );
    assert_natural_iso_round_trip::<OptionMyOptionIso, OptionWitness, MyOptionWitness, &'static str>(
        Some("hi"),
        MyOption::MySome("hi"),
    );
}

#[test]
#[should_panic(expected = "NaturalIso round-trip F -> G -> F failed")]
fn round_trip_panics_on_broken_reverse() {
    assert_natural_iso_round_trip::<BrokenReverseIso, OptionWitness, MyOptionWitness, i32>(
        Some(5),
        MyOption::MySome(5),
    );
}

#[test]
#[should_panic(expected = "NaturalIso round-trip G -> F -> G failed")]
fn round_trip_panics_independently_on_g_to_f_to_g() {
    assert_natural_iso_round_trip::<LossyForwardIso, OptionWitness, MyOptionWitness, i32>(
        None,
        MyOption::MySome(9),
    );
}

// =============================================================================
// Naturality law
// =============================================================================

#[test]
fn naturality_holds_for_canonical_iso_with_doubling() {
    assert_natural_iso_naturality::<OptionMyOptionIso, OptionWitness, MyOptionWitness, i32, i32, _>(
        Some(3),
        |x: i32| x * 2,
    );
    assert_natural_iso_naturality::<OptionMyOptionIso, OptionWitness, MyOptionWitness, i32, i32, _>(
        None,
        |x: i32| x * 2,
    );
}

#[test]
fn naturality_holds_for_canonical_iso_with_type_change() {
    assert_natural_iso_naturality::<OptionMyOptionIso, OptionWitness, MyOptionWitness, i32, bool, _>(
        Some(7),
        |x: i32| x % 2 == 0,
    );
    assert_natural_iso_naturality::<OptionMyOptionIso, OptionWitness, MyOptionWitness, i32, bool, _>(
        None,
        |x: i32| x % 2 == 0,
    );
}
