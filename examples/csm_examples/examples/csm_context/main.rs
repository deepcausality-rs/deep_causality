/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod model;

use deep_causality::*;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

// Define IDs for our sensors. These will be used to identify data in the context.
const FAN_SPEED_ID: IdentificationValue = 1;
const CPU_TEMP_ID: IdentificationValue = 2;
const POWER_DRAW_ID: IdentificationValue = 3;

// Define an ID for the server state in the CSM
const SERVER_HIGH_LOAD_STATE_ID: IdentificationValue = 100;

fn main() {
    println!("--- Server Sensor Fusion Example with Context ---");

    // Sample data for 10 monitoring cycles.
    let all_sensor_data = model::get_all_sensor_data();

    // Initialize context, model, and CSM outside the loop
    let server_context = Arc::new(RwLock::new(model::get_server_context_initial()));
    let server_model = model::get_server_causaloid(Arc::clone(&server_context));
    let server_csm = model::get_server_csm(server_model);

    println!("\n--- Starting Server Monitoring Loop (10 cycles) ---");
    for (i, (fan_speed, cpu_temp, power_draw)) in all_sensor_data.iter().enumerate() {
        println!("\n--- Cycle {} ---", i + 1);

        println!(
            "Sensor readings: Fan Speed: {}, CPU Temp: {}, Power Draw: {}",
            fan_speed, cpu_temp, power_draw
        );

        // Update the context dataoids with the current sensor data
        model::update_context_dataoids(&server_context, *fan_speed, *cpu_temp, *power_draw);

        // Evaluate the CSM state. Data is in the context, so we pass a default effect.
        let input_effect: PropagatingEffect<f64> = PropagatingEffect::pure(0.0);
        server_csm
            .eval_single_state(SERVER_HIGH_LOAD_STATE_ID as usize, &input_effect)
            .expect("Evaluation failed");

        // Pause for a moment to simulate a real-time loop.
        thread::sleep(Duration::from_millis(150));
    }

    println!("\n--- Monitoring Loop Finished ---");
}
