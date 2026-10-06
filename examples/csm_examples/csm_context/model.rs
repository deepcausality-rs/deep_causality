/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::SERVER_HIGH_LOAD_STATE_ID;
use deep_causality::CausalEffect;
use deep_causality::{
    CSM, CausalAction, CausalState, CausalityError, CausalityErrorEnum, Causaloid,
    IdentificationValue, NumericalValue, PropagatingEffect, PropagatingProcess,
};
use deep_causality_context::{
    BaseContext, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph,
    Data,
};
use std::error::Error;
use std::sync::{Arc, RwLock};

/// Contextoid id: the latest fan speed reading.
const FAN_SPEED_ID: ContextoidId = 1;
/// Contextoid id: the latest CPU temperature reading.
const CPU_TEMP_ID: ContextoidId = 2;
/// Contextoid id: the latest power draw reading.
const POWER_DRAW_ID: ContextoidId = 3;
/// Contextoid id: the fan speed above which the fan counts as high.
const FAN_SPEED_THRESHOLD_ID: ContextoidId = 4;
/// Contextoid id: the CPU temperature above which the CPU counts as hot.
const CPU_TEMP_THRESHOLD_ID: ContextoidId = 5;
/// Contextoid id: the power draw above which the draw counts as high.
const POWER_DRAW_THRESHOLD_ID: ContextoidId = 6;

pub type CsmCausaloid = Causaloid<f64, bool, (), Arc<RwLock<BaseContext>>>;

// CSM<I, O, C> where C must match the Causaloid's context type
pub type ServerCSM = CSM<f64, bool, Arc<RwLock<BaseContext>>>;

// Some sensor test data
pub(crate) fn get_all_sensor_data() -> Vec<(NumericalValue, NumericalValue, NumericalValue)> {
    vec![
        (50.0, 70.0, 150.0), // Normal
        (52.0, 72.0, 155.0), // Normal
        (55.0, 75.0, 160.0), // Normal
        (60.0, 80.0, 180.0), // Normal
        (65.0, 82.0, 190.0), // Normal
        (68.0, 85.0, 200.0), // Normal
        (70.0, 86.0, 210.0), // Normal
        (72.0, 88.0, 220.0), // Normal
        (85.0, 90.0, 260.0), // High load - triggers alert
        (75.0, 89.0, 230.0), // Normal
    ]
}

/// Creates the initial context for the server, populating it with Datoid nodes for each sensor
/// reading and for each sensor's "high" threshold.
pub(crate) fn get_server_context_initial() -> Result<BaseContext, ContextIndexError> {
    let facts = [
        // Readings: placeholders until the first monitoring cycle writes them.
        (FAN_SPEED_ID, 0.0),
        (CPU_TEMP_ID, 0.0),
        (POWER_DRAW_ID, 0.0),
        // The "high" thresholds the fusion logic compares each reading with.
        (FAN_SPEED_THRESHOLD_ID, 80.0),
        (CPU_TEMP_THRESHOLD_ID, 85.0),
        (POWER_DRAW_THRESHOLD_ID, 250.0),
    ];
    let mut context = BaseContext::with_capacity(1, "Server Context", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    Ok(context)
}

/// Reads the `Data` contextoid with contextoid id `id`, a reading or a threshold, out of the
/// server context.
fn read(context: &BaseContext, id: ContextoidId) -> Result<NumericalValue, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError(CausalityErrorEnum::Custom(format!(
            "No Datoid with contextoid id {id} in the server context"
        )))
    })
}

/// Updates the sensor data within the provided context.
pub(crate) fn update_context_dataoids(
    context_arc: &Arc<RwLock<BaseContext>>,
    fan_speed: NumericalValue,
    cpu_temp: NumericalValue,
    power_draw: NumericalValue,
) -> Result<(), Box<dyn Error>> {
    let mut context = context_arc
        .write()
        .map_err(|_| "Server context lock is poisoned")?;

    let readings = [
        (FAN_SPEED_ID, fan_speed),
        (CPU_TEMP_ID, cpu_temp),
        (POWER_DRAW_ID, power_draw),
    ];

    // Check every sensor's Datoid before writing any reading. A sensor without a Datoid fails the
    // update with all three readings unchanged, so the context never holds a mix of this cycle's
    // readings and the previous cycle's.
    for (id, _) in readings {
        read(&context, id)?;
    }
    for (id, reading) in readings {
        context.update_node(
            id,
            Contextoid::new(id, ContextoidType::Datoid(Data::new(id, reading))),
        )?;
    }

    Ok(())
}

/// Builds the causal model for the server. A single causaloid with a contextual function
/// that performs the fusion logic.
pub(crate) fn get_server_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 1;
    let description = "Fused Server Sensors Logic";

    // New API: fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
    fn context_causal_fn(
        _effect: CausalEffect<f64>,
        _state: (),
        context: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        match all_sensors_high(context) {
            Ok(all_high) => PropagatingProcess::pure(all_high),
            Err(e) => PropagatingProcess::from_error(e),
        }
    }

    Causaloid::new_with_context(id, context_causal_fn, context, description)
}

/// The fusion logic: whether fan speed, CPU temperature and power draw all exceed their
/// thresholds. Reads the three thresholds, then the three readings, from the context.
fn all_sensors_high(context: Option<Arc<RwLock<BaseContext>>>) -> Result<bool, CausalityError> {
    let ctx_arc = context
        .ok_or_else(|| CausalityError(CausalityErrorEnum::Custom("Context is missing".into())))?;
    let ctx = ctx_arc.read().map_err(|_| {
        CausalityError(CausalityErrorEnum::Custom(
            "Server context lock is poisoned".into(),
        ))
    })?;

    let fan_threshold = read(&ctx, FAN_SPEED_THRESHOLD_ID)?;
    let temp_threshold = read(&ctx, CPU_TEMP_THRESHOLD_ID)?;
    let power_threshold = read(&ctx, POWER_DRAW_THRESHOLD_ID)?;

    let fan_speed = read(&ctx, FAN_SPEED_ID)?;
    let cpu_temp = read(&ctx, CPU_TEMP_ID)?;
    let power_draw = read(&ctx, POWER_DRAW_ID)?;

    Ok(fan_speed > fan_threshold && cpu_temp > temp_threshold && power_draw > power_threshold)
}

/// Creates a Causal State Machine (CSM) that links the server's causal model
/// to a specific action.
pub(crate) fn get_server_csm(server_model: CsmCausaloid) -> ServerCSM {
    let default_data: PropagatingEffect<f64> = PropagatingEffect::pure(0.0);

    let high_load_state = CausalState::new(
        SERVER_HIGH_LOAD_STATE_ID as usize,
        1,            // version
        default_data, // Data is in the context
        server_model,
        None,
    );

    // Could also trigger a process to add more servers to decrease load for each
    let high_load_action = CausalAction::new(
        || {
            println!();
            println!(
                "\n>>> (!)ALERT(!): Server is under high load! Risk of failure. (!)ALERT(!)<<<",
            );
            println!();
            Ok(())
        },
        "High Load Alert",
        1, // version
    );

    CSM::new(&[(&high_load_state, &high_load_action)])
}
