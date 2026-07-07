/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unit tests for [`CausalEffect`] — the free-monad success channel
//! (`Free<CausalCommandWitness, Option<V>>`): value / none / command.

use deep_causality_core::CausalEffect;

// ---- Constructors + discriminators ---------------------------------------------------------------

#[test]
fn value_is_a_value_effect() {
    let e = CausalEffect::value(42_i32);
    assert!(e.is_value());
    assert!(!e.is_none());
    assert!(!e.is_command());
}

#[test]
fn none_is_the_absence_effect() {
    let e: CausalEffect<i32> = CausalEffect::none();
    assert!(e.is_none());
    assert!(!e.is_value());
    assert!(!e.is_command());
}

#[test]
fn relay_to_is_a_command_effect() {
    let e = CausalEffect::relay_to(7, CausalEffect::value(1_i32));
    assert!(e.is_command());
    assert!(!e.is_value());
    assert!(!e.is_none());
}

#[test]
fn from_option_round_trips_value_and_none() {
    assert!(CausalEffect::from_option(Some(5_i32)).is_value());
    assert!(CausalEffect::from_option(None::<i32>).is_none());
}

#[test]
fn default_is_none() {
    let e: CausalEffect<i32> = CausalEffect::default();
    assert!(e.is_none());
}

// ---- Value access ---------------------------------------------------------------------------------

#[test]
fn as_value_lends_only_for_a_value() {
    assert_eq!(CausalEffect::value(9_i32).as_value(), Some(&9));
    assert_eq!(CausalEffect::<i32>::none().as_value(), None);
    assert_eq!(
        CausalEffect::relay_to(1, CausalEffect::value(3_i32)).as_value(),
        None
    );
}

#[test]
fn into_value_projects_the_maybe() {
    assert_eq!(CausalEffect::value(9_i32).into_value(), Some(9));
    assert_eq!(CausalEffect::<i32>::none().into_value(), None);
    // A command has no scalar value.
    assert_eq!(
        CausalEffect::relay_to(1, CausalEffect::value(3_i32)).into_value(),
        None
    );
}

// ---- Command access -------------------------------------------------------------------------------

#[test]
fn command_target_reads_the_jump_target() {
    assert_eq!(
        CausalEffect::relay_to(5, CausalEffect::value(1_i32)).command_target(),
        Some(5)
    );
    assert_eq!(CausalEffect::value(1_i32).command_target(), None);
    assert_eq!(CausalEffect::<i32>::none().command_target(), None);
}

#[test]
fn into_command_yields_target_and_sub_program() {
    let cmd = CausalEffect::relay_to(3, CausalEffect::value(11_i32));
    let (target, sub) = cmd.into_command().expect("is a command");
    assert_eq!(target, 3);
    assert_eq!(sub.into_value(), Some(11));

    assert!(CausalEffect::value(1_i32).into_command().is_none());
}

// ---- Functor: `map` is total ---------------------------------------------------------------------

#[test]
fn map_transforms_a_value() {
    assert_eq!(
        CausalEffect::value(4_i32).map(|x| x + 1).into_value(),
        Some(5)
    );
}

#[test]
fn map_passes_none_through() {
    let mapped = CausalEffect::<i32>::none().map(|x| x + 1);
    assert!(mapped.is_none());
}

#[test]
fn map_threads_through_a_command_totally() {
    // The total functor maps the command's sub-program leaves — no panic, no error.
    let cmd = CausalEffect::relay_to(2, CausalEffect::value(10_i32));
    let mapped = cmd.map(|x| x * 3);
    assert_eq!(mapped.command_target(), Some(2));
    let (_t, sub) = mapped.into_command().expect("still a command");
    assert_eq!(sub.into_value(), Some(30));
}

// ---- Handler: `fold` (catamorphism) --------------------------------------------------------------

#[test]
fn fold_interprets_a_value_leaf() {
    let out = CausalEffect::value(7_i32).fold(&|opt: Option<i32>| opt.unwrap_or(0), &|_t, x| x);
    assert_eq!(out, 7);
}

#[test]
fn fold_interprets_a_command_via_the_algebra() {
    // Program: RelayTo(9, RelayTo(4, value(1))). Algebra sums the targets onto the leaf.
    let prog = CausalEffect::relay_to(9, CausalEffect::relay_to(4, CausalEffect::value(1_i32)));
    let out = prog.fold(&|opt: Option<i32>| opt.unwrap_or(0), &|t, x| t as i32 + x);
    assert_eq!(out, 9 + 4 + 1);
}

// ---- Congruent equality + clone + debug ----------------------------------------------------------

#[test]
fn eq_is_a_congruence_on_values() {
    assert_eq!(CausalEffect::value(3_i32), CausalEffect::value(3));
    assert_ne!(CausalEffect::value(3_i32), CausalEffect::value(4));
    assert_eq!(CausalEffect::<i32>::none(), CausalEffect::none());
    assert_ne!(CausalEffect::<i32>::none(), CausalEffect::value(0));
}

#[test]
fn eq_compares_the_command_payload_recursively() {
    let a = CausalEffect::relay_to(2, CausalEffect::value(1_i32));
    let b = CausalEffect::relay_to(2, CausalEffect::value(1_i32));
    let diff_target = CausalEffect::relay_to(3, CausalEffect::value(1_i32));
    let diff_payload = CausalEffect::relay_to(2, CausalEffect::value(9_i32));
    assert_eq!(a, b);
    assert_ne!(a, diff_target);
    // The former `Map` PER ignored the payload; the new congruence compares it.
    assert_ne!(a, diff_payload);
}

#[test]
fn clone_preserves_structure_including_commands() {
    let cmd = CausalEffect::relay_to(4, CausalEffect::value(8_i32));
    assert_eq!(cmd.clone(), cmd);
    let val = CausalEffect::value(8_i32);
    assert_eq!(val.clone(), val);
}

#[test]
fn debug_renders_each_shape() {
    assert_eq!(format!("{:?}", CausalEffect::value(5_i32)), "Value(5)");
    assert_eq!(format!("{:?}", CausalEffect::<i32>::none()), "None");
    assert_eq!(
        format!(
            "{:?}",
            CausalEffect::relay_to(2, CausalEffect::value(1_i32))
        ),
        "RelayTo(2, Value(1))"
    );
}
