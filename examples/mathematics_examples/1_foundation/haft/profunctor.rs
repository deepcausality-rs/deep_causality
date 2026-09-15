/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT2Unbound, Profunctor};
use deep_causality_num::lift;

/// The working scalar. A product's price carries it.
pub type FloatType = f64;

// ============================================================================
// Domain: Search Filters
// ============================================================================

fn main() {
    print_header();

    // ------------------------------------------------------------------------
    // Profunctor: The Adapter Pattern
    //
    // ENGINEERING VALUE:
    // You have a reusable component (e.g., a String Filter) that works on simple types.
    // You want to apply it to a complex Domain Object (e.g., Product) without rewriting it.
    //
    // Profunctor (`dimap` or `lmap`) allows you to "adapt" the input type.
    // You provide a function `Product -> String`, and the Profunctor gives you back
    // a `Product Filter`.
    // ------------------------------------------------------------------------
    // 1. The Core Component: A generic String Filter
    // Checks if a string contains "Pro"
    let string_filter = Function(Box::new(|s: String| s.contains("Pro")));

    // Test the core component.
    print_core_filter((string_filter.0)("Professional".to_string()));

    // 2. The Requirement: Filter Products by Name
    // We have a Product, but our filter works on Strings.
    let products = vec![
        Product {
            id: 1,
            name: "Pro Laptop".to_string(),
            category: "Electronics".to_string(),
            price: lift(1200.0),
        },
        Product {
            id: 2,
            name: "Basic Mouse".to_string(),
            category: "Electronics".to_string(),
            price: lift(20.0),
        },
    ];

    // 3. The Adapter (lmap / dimap)
    // We want a function: Product -> bool
    // We have: String -> bool
    // We need: Product -> String (The Adapter)

    let product_to_name = |p: Product| p.name;

    // Use Profunctor to create the new filter
    // dimap(adapter, identity)
    // Input: Product -> String
    // Output: bool -> bool (Identity, we don't change the result)
    let product_filter = FunctionWitness::dimap(
        string_filter,
        product_to_name,
        |b| b, // Identity on output
    );

    // 4. Usage
    print_products_header();
    for p in products {
        let is_match = (product_filter.0)(p.clone());
        print_match(&p.name, is_match);
    }

    // ------------------------------------------------------------------------
    // Scenario 2: Price Filter (Adapting f64 -> bool)
    // ------------------------------------------------------------------------
    // Core: Checks if value > 100.0
    let expensive_filter = Function(Box::new(|price: FloatType| {
        price > lift::<FloatType>(100.0)
    }));

    // Adapter: Product -> FloatType
    let product_to_price = |p: Product| p.price;

    // Create Product -> bool
    let expensive_product_filter = FunctionWitness::dimap(
        expensive_filter,
        product_to_price,
        |b| !b, // Let's invert the output! (Covariant map). Now it finds "Not Expensive" (Cheap)
    );

    print_price_header();
    let cheap_product = Product {
        id: 3,
        name: "Cheap Cable".to_string(),
        category: "Accessories".to_string(),
        price: lift(15.0),
    };

    let is_cheap = (expensive_product_filter.0)(cheap_product.clone());
    print_cheap(&cheap_product.name, is_cheap);
    assert!(is_cheap);
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Profunctor Pattern ===\n");
    println!("--- Search Filter Adapter ---");
}

fn print_core_filter(matched: bool) {
    println!("Filter 'Pro': {matched}");
}

fn print_products_header() {
    println!("\nFiltering Products:");
}

fn print_match(name: &str, is_match: bool) {
    println!(
        "- {}: {}",
        name,
        if is_match { "MATCH" } else { "NO MATCH" }
    );
}

fn print_price_header() {
    println!("\n--- Price Filter Adapter ---");
    println!("Finding Cheap Products (< 100.0):");
}

fn print_cheap(name: &str, is_cheap: bool) {
    println!("- {}: {}", name, if is_cheap { "YES" } else { "NO" });
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Product {
    id: u32,
    name: String,
    category: String,
    price: FloatType,
}

// ============================================================================
// Profunctor Implementation for Function
// ============================================================================

// A generic function wrapper: Input -> Output
struct Function<I, O>(Box<dyn Fn(I) -> O>);

struct FunctionWitness;

impl HKT2Unbound for FunctionWitness {
    type Type<A, B> = Function<A, B>;
}

impl Profunctor<FunctionWitness> for FunctionWitness {
    fn dimap<A, B, C, D, F1, F2>(pab: Function<A, B>, f_pre: F1, f_post: F2) -> Function<C, D>
    where
        F1: FnMut(C) -> A + 'static,
        F2: FnMut(B) -> D + 'static,
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static,
    {
        let inner = pab.0;
        // Note: In a real implementation, we'd handle FnMut/FnOnce carefully.
        // For this example, we use RefCell to allow FnMut inside the returned Fn.
        let f_pre = std::cell::RefCell::new(f_pre);
        let f_post = std::cell::RefCell::new(f_post);

        Function(Box::new(move |c| {
            let a = (f_pre.borrow_mut())(c);
            let b = inner(a);
            (f_post.borrow_mut())(b)
        }))
    }
}
