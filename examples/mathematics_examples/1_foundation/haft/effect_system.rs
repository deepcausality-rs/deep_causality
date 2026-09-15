/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::utils_tests::*;
use deep_causality_haft::{Effect5, HKT5, MonadEffect5};

// ============================================================================
// Domain Logic: Audited Financial Transactions
// ============================================================================

// We use the Type-Encoded Effect System to track 5 dimensions of a transaction simultaneously:
// 1. Value (T): The current account balance.
// 2. Fixed1 (Option<String>): Failure Reason (if any).
// 3. Fixed2 (Vec<String>): Audit Log (Business-level events).
// 4. Fixed3 (Vec<i32>): Operation Cost (in micro-cents).
// 5. Fixed4 (Vec<String>): System Trace (Debug-level events).

type TransactionEffect<T> = <<MyEffect5 as Effect5>::HktWitness as HKT5<
    <MyEffect5 as Effect5>::Fixed1,
    <MyEffect5 as Effect5>::Fixed2,
    <MyEffect5 as Effect5>::Fixed3,
    <MyEffect5 as Effect5>::Fixed4,
>>::Type<T>;

fn main() {
    // ------------------------------------------------------------------------
    // Concept: Type-Encoded Effect System
    //
    // ENGINEERING VALUE:
    // In complex domains (Finance, Healthcare), a "function" is never just Input -> Output.
    // It has side effects: Logging, Auditing, Cost Calculation, Error Propagation, Tracing.
    //
    // Managing these side effects manually (passing context objects everywhere) is error-prone.
    // The Effect System embeds these effects into the TYPE itself.
    // You write pure functions that return "Effects", and the Monad composes them,
    // automatically aggregating logs, costs, and errors.
    // ------------------------------------------------------------------------

    // Initial State: Account Balance $1000
    let initial_balance: TransactionEffect<i32> = MyMonadEffect5::pure(1000);
    print_opening(initial_balance.value.unwrap_or_default());

    // Define Transaction Steps
    // Each step returns a TransactionEffect that modifies the balance AND emits effects.

    let debit_step = |amount: i32| {
        Box::new(move |balance: i32| -> TransactionEffect<i32> {
            if balance < amount {
                // Failure Case: a well-formed errored carrier holds no value.
                MyCustomEffectType5 {
                    value: None,
                    f1: Some("Insufficient Funds".to_string()),
                    f2: vec!["Failed Debit".to_string()],
                    f3: vec![10], // Cost of check
                    f4: vec!["WARN: Balance check failed".to_string()],
                }
            } else {
                // Success Case
                MyCustomEffectType5 {
                    value: Some(balance - amount),
                    f1: None,
                    f2: vec![format!("Debited ${}", amount)],
                    f3: vec![50], // Cost of transaction
                    f4: vec!["INFO: Debit applied".to_string()],
                }
            }
        })
    };

    let apply_tax = |rate_percent: i32| {
        Box::new(move |balance: i32| -> TransactionEffect<i32> {
            let tax = balance * rate_percent / 100;
            MyCustomEffectType5 {
                value: Some(balance - tax),
                f1: None,
                f2: vec![format!("Applied Tax ${} ({}%)", tax, rate_percent)],
                f3: vec![5], // Cost of calculation
                f4: vec!["INFO: Tax calculated".to_string()],
            }
        })
    };

    let credit_bonus = |bonus: i32| {
        Box::new(move |balance: i32| -> TransactionEffect<i32> {
            MyCustomEffectType5 {
                value: Some(balance + bonus),
                f1: None,
                f2: vec![format!("Credited Bonus ${}", bonus)],
                f3: vec![20],
                f4: vec!["INFO: Bonus applied".to_string()],
            }
        })
    };

    print_success_header();

    // Pipeline: Debit $200 -> Apply 10% Tax -> Credit $50 Bonus
    let steps: Vec<Box<dyn Fn(i32) -> TransactionEffect<i32>>> =
        vec![debit_step(200), apply_tax(10), credit_bonus(50)];

    let mut current_tx = initial_balance;

    for (i, step) in steps.into_iter().enumerate() {
        // Bind automatically chains the value AND aggregates the effects (logs, costs, etc.)
        current_tx = MyMonadEffect5::bind(current_tx, step);
        print_step(i + 1, current_tx.value.unwrap_or_default());
    }

    print_report(
        current_tx.value.unwrap_or_default(),
        &current_tx.f1.clone().unwrap_or("Success".to_string()),
        &current_tx.f2,
        current_tx.f3.iter().sum::<u64>(),
        &current_tx.f4,
    );

    assert_eq!(current_tx.value, Some(770)); // (1000 - 200) * 0.9 + 50 = 720 + 50 = 770
    assert_eq!(current_tx.f3.iter().sum::<u64>(), 75); // 50 + 5 + 20

    print_failure_header();

    // Re-initialize for the second run since the previous one was moved
    let initial_balance_2: TransactionEffect<i32> = MyMonadEffect5::pure(1000);

    // Pipeline: Debit $2000 (Overdraft) -> Apply Tax.
    // Bind short-circuits on error (raise is a left zero): the tax step never runs, the
    // error and the audit trail accumulated so far are preserved, and no value remains.

    let steps_fail: Vec<Box<dyn Fn(i32) -> TransactionEffect<i32>>> =
        vec![debit_step(2000), apply_tax(10)];

    let mut fail_tx = initial_balance_2;

    for step in steps_fail {
        fail_tx = MyMonadEffect5::bind(fail_tx, step);
    }

    print_failure_report(&fail_tx.value, &fail_tx.f1, &fail_tx.f2);

    // Verify error is captured, no value survives, and the tax step did not run.
    assert_eq!(fail_tx.value, None);
    assert_eq!(fail_tx.f2, vec!["Failed Debit".to_string()]); // no "Applied Tax" entry
    assert_eq!(fail_tx.f1.as_deref(), Some("Insufficient Funds"));
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_opening(balance: i32) {
    println!("=== DeepCausality HKT: Audited Financial Transaction (Effect System) ===\n");
    println!("Initial Balance: ${balance}");
}

fn print_success_header() {
    println!("\n--- 1. Successful Transaction Pipeline ---");
}

fn print_step(step: usize, balance: i32) {
    println!("Step {step} complete. Current Balance: ${balance}");
}

fn print_report(balance: i32, status: &str, audit: &[String], cost: u64, trace: &[String]) {
    println!("\n--- Final Transaction Report ---");
    println!("Final Balance: ${balance}");
    println!("Status:        {status:?}");
    println!("Audit Log:     {audit:?}");
    println!("Total Cost:    {cost} micro-cents");
    println!("System Trace:  {trace:?}");
}

fn print_failure_header() {
    println!("\n--- 2. Failed Transaction Pipeline ---");
}

fn print_failure_report(balance: &Option<i32>, status: &Option<String>, audit: &[String]) {
    println!("Final Balance: {balance:?}");
    println!("Status:        {status:?}");
    println!("Audit Log:     {audit:?}");
}
