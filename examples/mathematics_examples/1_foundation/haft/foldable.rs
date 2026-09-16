/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Foldable, VecWitness};
use deep_causality_num::{const_scalar_from_int, lift, lift_u32, lower};

// ============================================================================
// Domain: E-Commerce Order Processing
//
// Foldable abstracts the "loop and accumulate" pattern: you have a collection
// (Vec, List, Tree) and you need one value out of it -- a sum, a max, an average,
// a concatenation. `fold` takes the seed and the step, and the witness supplies
// the traversal.
// ============================================================================

/// The working scalar. Money is the quantity this example reduces, so it carries the alias.
pub type FloatType = f64;

/// Small numbers and tolerances, at the working type.
const FIVE: FloatType = const_scalar_from_int!(FloatType, 5);
const HUNDRED: FloatType = const_scalar_from_int!(FloatType, 100);
const TEN: FloatType = const_scalar_from_int!(FloatType, 10);

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

fn main() {
    print_header();

    let orders = vec![
        OrderItem {
            id: "A".to_string(),
            price: TEN,
            quantity: 2,
        },
        OrderItem {
            id: "B".to_string(),
            price: FIVE,
            quantity: 10,
        },
        OrderItem {
            id: "C".to_string(),
            price: HUNDRED,
            quantity: 1,
        },
    ];

    // Total revenue: the accumulator is the working scalar, seeded through `lift`.
    let total_revenue = VecWitness::fold(orders.clone(), ZERO, |acc, item| {
        acc + (item.price * lift_u32::<FloatType>(item.quantity))
    });
    assert_eq!(total_revenue, lift::<FloatType>(170.0));

    // Total items: the same fold over a plain integer accumulator.
    let total_items = VecWitness::fold(orders, 0, |acc, item| acc + item.quantity);
    assert_eq!(total_items, 13);

    print_totals(total_revenue, total_items);
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
    println!("=== DeepCausality HKT: Foldable Pattern ===\n");
    println!("--- Order Aggregation ---");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_totals(revenue: FloatType, items: u32) {
    println!("Total Revenue: ${:.2}", lower(revenue));
    println!("Total Items:   {items}");
}
