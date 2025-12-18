/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::model::{
    get_explosion_sensor_causaloid, get_explosion_sensor_data, get_fire_sensor_causaloid,
    get_fire_sensor_data, get_smoke_sensor_causaloid, get_smoke_sensor_data,
};
use crate::model_actions::{
    get_explosion_alert_action, get_fire_alert_action, get_smoke_alert_action,
};
use deep_causality::{CSM, CausalState, PropagatingEffect};
use std::thread;
use std::time::Duration;

mod model;
mod model_actions;
mod types;

const SMOKE_SENSOR: usize = 1;
const FIRE_SENSOR: usize = 2;
const EXPLOSION_SENSOR: usize = 3;

fn main() {
    // The initial data in a CausalState is often just a default.
    let default_data: PropagatingEffect<f64> = PropagatingEffect::pure(0.0);

    let smoke_causaloid = get_smoke_sensor_causaloid();
    let smoke_cs = CausalState::new(SMOKE_SENSOR, 1, default_data.clone(), smoke_causaloid, None);
    let smoke_ca = get_smoke_alert_action();

    let fire_causaloid = get_fire_sensor_causaloid();
    let fire_cs = CausalState::new(FIRE_SENSOR, 1, default_data.clone(), fire_causaloid, None);
    let fire_ca = get_fire_alert_action();

    let explosion_causaloid = get_explosion_sensor_causaloid();
    let explosion_cs =
        CausalState::new(EXPLOSION_SENSOR, 1, default_data, explosion_causaloid, None);
    let explosion_ca = get_explosion_alert_action();

    println!("Create Causal State Machine");
    let state_actions = &[(&smoke_cs, &smoke_ca), (&fire_cs, &fire_ca)];
    let csm = CSM::new(state_actions);

    println!("Add a new sensor");
    csm.add_single_state((explosion_cs, explosion_ca))
        .expect("Failed to add Explosion sensor");

    println!("Start data feed and monitor sensors");
    let smoke_data = get_smoke_sensor_data();
    let fire_data = get_fire_sensor_data();
    let exp_data = get_explosion_sensor_data();

    for i in 0..12 {
        wait();

        // Wrap the raw numerical data in PropagatingEffect before passing it to the CSM.
        let smoke_evidence: PropagatingEffect<f64> = PropagatingEffect::pure(smoke_data[i]);
        if let Err(e) = csm.eval_single_state(SMOKE_SENSOR, &smoke_evidence) {
            eprintln!("[CSM Error] Smoke sensor evaluation failed: {e}");
        }

        let fire_evidence: PropagatingEffect<f64> = PropagatingEffect::pure(fire_data[i]);
        if let Err(e) = csm.eval_single_state(FIRE_SENSOR, &fire_evidence) {
            eprintln!("[CSM Error] Fire sensor evaluation failed: {e}");
        }

        let explosion_effect: PropagatingEffect<f64> = PropagatingEffect::pure(exp_data[i]);
        if let Err(e) = csm.eval_single_state(EXPLOSION_SENSOR, &explosion_effect) {
            eprintln!("[CSM Error] Explosion sensor evaluation failed: {e}");
        }
    }
}

fn wait() {
    println!("\nReading Sensors...");
    thread::sleep(Duration::from_millis(100));
}
