/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Functor, OptionWitness};
use deep_causality_num::lift;

// ============================================================================
// Domain: E-Commerce Order Processing
//
// Applicative: independent validation.
//
// When validating a form you often want ALL the errors, not just the first, or you
// want to combine several independent results such as parallel API calls. Applicative
// (`apply`) combines values inside a context (Result/Option) independently of each other.
// ============================================================================

/// The working scalar. The order's price carries it.
pub type FloatType = f64;

fn main() {
    print_header();

    let validate_id = |id: &str| -> Option<String> {
        if id.len() > 3 {
            Some(id.to_string())
        } else {
            None
        }
    };
    let validate_qty = |qty: u32| -> Option<u32> { if qty > 0 { Some(qty) } else { None } };

    // An OrderItem should exist only if both the id and the quantity validate.
    // Applicative style is pure(constructor).apply(id).apply(qty).
    let valid_id = validate_id("item_123");
    let _valid_qty = validate_qty(5);
    let price = lift::<FloatType>(10.0);

    // Currying is verbose in Rust, so the raw `apply` mechanism is shown step by step.
    // The constructor takes (String, u32) and yields an OrderItem.
    let constructor = |id: String| {
        move |qty: u32| OrderItem {
            id,
            price,
            quantity: qty,
        }
    };

    // Step 1 lifts the constructor into Option, giving Option<Fn(u32) -> OrderItem>.
    let _partial_constructor = OptionWitness::fmap(valid_id, constructor);

    // The same shape on a smaller pair: combine a host and a port into "host:port".
    let config_host = Some("localhost".to_string());
    let config_port = 8080;
    let combine = |host: String| move |port: i32| format!("{host}:{port}");

    // `partial` is Some(Fn(i32) -> String). Coercing that closure type for a generic `apply`
    // needs boxing on stable Rust, so the final application is written out here.
    let partial = OptionWitness::fmap(config_host, combine);
    if let Some(f) = partial {
        print_result(&f(config_port));
    }
}

#[derive(Debug, Clone, PartialEq)]
struct OrderItem {
    id: String,
    price: FloatType,
    quantity: u32,
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Applicative Pattern ===\n");
    println!("--- Independent Validation ---");
}

fn print_result(result: &str) {
    println!("Applicative Result: {result}");
}
