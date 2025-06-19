// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::thread;
use std::time::Duration;

use deep_causality::prelude::{CausalState, CSM};

use crate::utils_actions::*;
use crate::utils_data::{get_explosion_sensor_data, get_fire_sensor_data, get_smoke_sensor_data};
use crate::utils_states::*;

const SMOKE_SENSOR: usize = 1;
const FIRE_SENSOR: usize = 2;
const EXPLOSION_SENSOR: usize = 3;

pub fn run() {
    let data = 0.0f64;
    let smoke_causloid = get_smoke_sensor_causaloid();
    let smoke_cs = CausalState::new(SMOKE_SENSOR, 1, data, &smoke_causloid);
    let smoke_ca = get_smoke_alert_action();

    let fire_causaloid = get_fire_sensor_causaloid();
    let fire_cs = CausalState::new(FIRE_SENSOR, 1, data, &fire_causaloid);
    let fire_ca = get_fire_alert_action();

    let explosion_causaloid = get_explosion_sensor_causaloid();
    let explosion_cs = CausalState::new(EXPLOSION_SENSOR, 1, data, &explosion_causaloid);
    let explosion_ca = get_explosion_alert_action();

    println!("Create Causal State Machine");
    let state_actions = &[(&smoke_cs, &smoke_ca), (&fire_cs, &fire_ca)];
    let csm = CSM::new(state_actions);

    println!("Add a new sensor");
    csm.add_single_state(EXPLOSION_SENSOR, (&explosion_cs, &explosion_ca))
        .expect("Failed to add Explosion sensor");

    println!("Start data feed and monitor senors");
    let smoke_data = get_smoke_sensor_data();
    let fire_data = get_fire_sensor_data();
    let exp_data = get_explosion_sensor_data();

    for i in 0..12 {
        wait();
        csm.eval_single_state(SMOKE_SENSOR, smoke_data[i])
            .expect("Panic: Smoke sensor failed");
        csm.eval_single_state(FIRE_SENSOR, fire_data[i])
            .expect("Panic: Fire sensor failed");
        csm.eval_single_state(EXPLOSION_SENSOR, exp_data[i])
            .expect("Panic: Explosion sensor failed");
    }
}

fn wait() {
    println!("Reading Sensor...");
    thread::sleep(Duration::from_millis(250));
}
