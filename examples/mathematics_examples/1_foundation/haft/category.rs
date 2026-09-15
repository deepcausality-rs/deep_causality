/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Category Example
//!
//! A `Category` is an identity morphism and an associative way to put two morphisms end to end.
//!
//! Two of them ship with `deep_causality_haft`:
//!
//!   `Fun`          the category of plain functions. `Hom<B> = B`, `id` is `|a| a`,
//!                  `compose(f, g)` is `g` after `f`.
//!   `Kleisli<M>`   the category of *effectful* functions `A -> M<B>` for any monad `M`.
//!                  `Hom<B> = M::Type<B>`, `id` is `pure`, `compose` is `bind`.
//!
//! `Kleisli` is the useful one: it comes free with every monad, and it is what lets a chain
//! of fallible steps be built once and applied later, rather than nested at the call site.

use deep_causality_haft::{Category, Fun, Kleisli, OptionWitness};

/// The Kleisli category of `Option`: morphisms are `A -> Option<B>`.
type MaybeStep = Kleisli<OptionWitness>;

fn main() {
    // ---------------------------------------------------------------------
    // 1. `Fun`: composition of ordinary functions.
    // ---------------------------------------------------------------------
    let add_one = |x: i32| x + 1;
    let double = |x: i32| x * 2;

    let add_then_double = Fun::compose(add_one, double);
    let identity = Fun::id::<i32>();

    print_fun(add_then_double(5), identity(5));
    assert_eq!(add_then_double(5), 12); // (5 + 1) * 2

    // The category laws, checked on a value.
    let left = Fun::compose(Fun::id::<i32>(), double);
    let right = Fun::compose(double, Fun::id::<i32>());
    let assoc_l = Fun::compose(Fun::compose(add_one, double), add_one);
    let assoc_r = Fun::compose(add_one, Fun::compose(double, add_one));
    print_fun_laws(
        left(7) == double(7),
        right(7) == double(7),
        assoc_l(7) == assoc_r(7),
    );

    // ---------------------------------------------------------------------
    // 2. `Kleisli<OptionWitness>`: composition of fallible steps.
    // ---------------------------------------------------------------------
    // Each step can fail. Composing them once gives a single pipeline, and a failure
    // anywhere short-circuits the rest.
    let pipeline = MaybeStep::compose(parse_port, MaybeStep::compose(check_range, to_label));

    print_kleisli(
        &pipeline("8080".to_string()),
        &pipeline("22".to_string()),
        &pipeline("not-a-port".to_string()),
    );
    assert_eq!(pipeline("8080".to_string()), Some("port:8080".to_string()));
    assert_eq!(pipeline("22".to_string()), None); // below 1024
    assert_eq!(pipeline("not-a-port".to_string()), None); // unparseable

    // Kleisli identity is `pure`, so composing with it changes nothing.
    let with_id = MaybeStep::compose(MaybeStep::id::<String>(), parse_port);
    print_kleisli_identity(
        &with_id("8080".to_string()),
        &parse_port("8080".to_string()),
    );
    assert_eq!(with_id("8080".to_string()), parse_port("8080".to_string()));
}

/// Step 1: a string becomes a port number, or nothing.
fn parse_port(s: String) -> Option<u16> {
    s.parse::<u16>().ok()
}

/// Step 2: reject anything a user may not bind.
fn check_range(port: u16) -> Option<u16> {
    if port >= 1024 { Some(port) } else { None }
}

/// Step 3: render it.
fn to_label(port: u16) -> Option<String> {
    Some(format!("port:{port}"))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_fun(composed: i32, identity: i32) {
    println!("=== DeepCausality HKT: Category (Fun and Kleisli) ===\n");
    println!("--- 1. Fun: the category of functions ---");
    println!("compose(add_one, double)(5) = {composed}");
    println!("id(5)                       = {identity}");
}

fn print_fun_laws(left: bool, right: bool, assoc: bool) {
    println!("  left identity  compose(id, g) == g : {left}");
    println!("  right identity compose(f, id) == f : {right}");
    println!("  associativity                      : {assoc}");
}

fn print_kleisli(ok: &Option<String>, low: &Option<String>, bad: &Option<String>) {
    println!("\n--- 2. Kleisli<Option>: the category of fallible steps ---");
    println!("  parse >=> check_range >=> label");
    println!("  \"8080\"       -> {ok:?}");
    println!("  \"22\"         -> {low:?}   (below 1024, the check fails)");
    println!("  \"not-a-port\" -> {bad:?}   (the parse fails, later steps never run)");
}

fn print_kleisli_identity(with_id: &Option<u16>, plain: &Option<u16>) {
    println!("\n--- 3. Kleisli identity is `pure` ---");
    println!("  compose(id, parse_port)(\"8080\") = {with_id:?}");
    println!("  parse_port(\"8080\")              = {plain:?}");
}
