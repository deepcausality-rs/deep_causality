/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{BoxWitness, CoMonad, HKT};
use deep_causality_num::{lift, lower};

/// The working scalar. Temperature and pressure carry it through every evolution step.
pub type FloatType = f64;

// ============================================================================
// Domain Logic: System Evolution
//
// Concept: Comonad (the "viewer" pattern)
//
// Where a Monad is about sequencing (A -> M<B>), a Comonad is about contextual
// computation (W<A> -> B). You have a value in a context -- a system snapshot --
// and you want a new value computed from that *entire* context. `extend` chains
// those context-aware computations.
//
// Scenario: a physical system cooling down, each step depending on the current state.
// ============================================================================

fn main() {
    // Initial state: hot and high pressure.
    let initial_state = Box::new(SystemState {
        temperature: lift(100.0),
        pressure: lift(50.0),
        step: 0,
    });

    // EXTRACT reads the value out of the context, for when you just need the sensor.
    let current_temp = BoxWitness::extract(&initial_state).temperature;
    print_initial(&initial_state, current_temp);

    // EXTEND evolves the system. The rule takes the *entire* previous context.
    let evolve_system = |w: &<BoxWitness as HKT>::Type<SystemState>| {
        let prev = &**w;
        SystemState {
            temperature: prev.temperature * lift::<FloatType>(0.90), // cools by 10%
            pressure: prev.pressure * lift::<FloatType>(0.95),       // pressure drops by 5%
            step: prev.step + 1,
        }
    };

    let state_t1 = BoxWitness::extend(&initial_state, evolve_system);
    let state_t2 = BoxWitness::extend(&state_t1, evolve_system);
    print_evolution(&state_t1, &state_t2);

    // EXTEND also computes *derived* metrics that depend on the context.
    let analyze_alert = |w: &<BoxWitness as HKT>::Type<SystemState>| {
        let state = &**w;
        if state.temperature > lift::<FloatType>(85.0) {
            "CRITICAL"
        } else if state.temperature > lift::<FloatType>(50.0) {
            "WARNING"
        } else {
            "NORMAL"
        }
    };

    // This turns Box<SystemState> into Box<&str>, the alert context.
    let alert_t0 = BoxWitness::extend(&initial_state, analyze_alert);
    let alert_t1 = BoxWitness::extend(&state_t1, analyze_alert);
    let alert_t2 = BoxWitness::extend(&state_t2, analyze_alert);
    print_alerts(&alert_t0, &alert_t1, &alert_t2);

    assert_eq!(*alert_t0, "CRITICAL"); // 100.0
    assert_eq!(*alert_t1, "CRITICAL"); // 90.0
    assert_eq!(*alert_t2, "WARNING"); // 81.0
}

#[derive(Debug, Clone, PartialEq)]
struct SystemState {
    temperature: FloatType,
    pressure: FloatType,
    step: u32,
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_initial(state: &SystemState, current_temp: FloatType) {
    println!("=== DeepCausality HKT: Contextual Computation (Comonad) ===\n");
    println!("T0: Initial State: {state:?}");
    // The display boundary: `f64` appears here and nowhere else.
    println!("    Current Temp: {:.2}", lower(current_temp));
}

fn print_evolution(t1: &SystemState, t2: &SystemState) {
    println!("T1: Evolved State: {t1:?}");
    println!("T2: Evolved State: {t2:?}");
}

fn print_alerts(t0: &str, t1: &str, t2: &str) {
    println!("\n--- System Alert Log ---");
    println!("T0 Alert: {t0:?}");
    println!("T1 Alert: {t1:?}");
    println!("T2 Alert: {t2:?}");
}
