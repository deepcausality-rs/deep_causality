/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{Intervenable, PropagatingEffectWitness, PropagatingProcess};
use deep_causality_haft::{LogAddEntry, Pure};

#[derive(Debug, Clone, Default)]
struct SystemState {
    energy: i32,
    operations_count: usize,
}

fn main() {
    println!("--- Counterfactual Observation (Stateful) ---");

    // --------------------------------------------------------------------------------------------
    // ENGINEERING VALUE: State-Aware Logic Testing
    //
    // Complex logic often fails because of state accumulation (e.g., a battery draining).
    // This example demonstrates how to simulate a "Counterfactual World" where we intervene
    // on a sensor reading to see if it saves the mission.
    //
    // By keeping the State (`SystemState`) separate from the computation steps, we can:
    // 1. Run the Factual path (Battery dies, Mission fails).
    // 2. Fork the process.
    // 3. Intervene on the input (Battery reading = 100).
    // 4. Verify that logic correctly identifies "Mission Approved" in the new timeline.
    // --------------------------------------------------------------------------------------------

    let initial_energy = 20;
    println!("\n1. Factual World (Low Battery: {})", initial_energy);

    // Initial Process
    let initial_effect = PropagatingEffectWitness::pure(0);
    let mut process = PropagatingProcess::with_state(
        initial_effect,
        SystemState {
            energy: initial_energy,
            operations_count: 0,
        },
        None::<()>,
    );
    process
        .logs
        .add_entry(&format!("System initialized. Energy: {}", initial_energy));

    // Operation 1: Heavy Lift
    let final_process = process.bind(|_, mut state: SystemState, _ctx| {
        state.operations_count += 1;
        state.energy -= 15;

        let (success, msg) = if state.energy > 0 {
            (
                true,
                format!("Operation Successful. Remaining Energy: {}", state.energy),
            )
        } else {
            (false, "Operation Failed. Battery Depleted.".to_string())
        };

        let mut p = PropagatingProcess::pure(success);
        p.state = state;
        p.logs
            .add_entry(&format!("Operation 1 Executed. Result: {}", msg));
        p
    });

    println!(
        "  Factual Outcome: {:?}",
        final_process.value.into_value().unwrap()
    );
    println!("  Factual Logs:\n{}", final_process.logs);

    // Counterfactual World
    println!("\n2. Counterfactual World (Intervention: Sensor Battery Reading = 100)");

    // Sensor reading process
    let mut sensor_reading = PropagatingProcess::with_state(
        PropagatingEffectWitness::pure(20), // Reading 20
        SystemState::default(),
        None::<()>,
    );
    sensor_reading
        .logs
        .add_entry("Sensor initialized. Reading: 20");

    // Intervention! Force reading to 100
    // .intervene() adds the log entry automatically
    let intervened_process = sensor_reading.intervene(100);

    let outcome = intervened_process.bind(|battery_level, mut state: SystemState, _ctx| {
        let battery_val = battery_level.into_value().unwrap();

        let (status, msg) = if battery_val > 50 {
            state.operations_count += 1;
            (
                "Approved",
                format!("Mission Approved. Battery Level {} > 50", battery_val),
            )
        } else {
            (
                "Aborted",
                format!("Mission Aborted. Battery Level {} <= 50", battery_val),
            )
        };

        let mut p = PropagatingProcess::pure(status);
        p.state = state;
        p.logs.add_entry(&msg);
        p
    });

    println!(
        "  Counterfactual Outcome: {:?}",
        outcome.value.into_value().unwrap()
    );
    println!("\n3. Full Audit Trail (Counterfactual):");
    println!("{}", outcome.logs);
}
