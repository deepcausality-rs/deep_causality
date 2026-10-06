/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CPU_TEMP_ID, CPU_TEMP_THRESHOLD_ID, FAN_SPEED_ID, FAN_SPEED_THRESHOLD_ID, POWER_DRAW_ID,
    POWER_DRAW_THRESHOLD_ID, SERVER_HIGH_LOAD_STATE_ID,
};
use deep_causality::CausalEffect;
use deep_causality::{
    CSM, CausalAction, CausalState, CausalityError, CausalityErrorEnum, Causaloid, Identifiable,
    IdentificationValue, NumericalValue, PropagatingEffect, PropagatingProcess,
};
use deep_causality_context::{
    BaseContext, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
};
use std::error::Error;
use std::sync::{Arc, RwLock};

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
    let mut context = BaseContext::with_capacity(1, "Server Context", 10);

    let fan_datoid = Contextoid::new(
        FAN_SPEED_ID,
        ContextoidType::Datoid(Data::new(FAN_SPEED_ID, 0.0)), // Placeholder
    );
    let temp_datoid = Contextoid::new(
        CPU_TEMP_ID,
        ContextoidType::Datoid(Data::new(CPU_TEMP_ID, 0.0)), // Placeholder
    );
    let power_datoid = Contextoid::new(
        POWER_DRAW_ID,
        ContextoidType::Datoid(Data::new(POWER_DRAW_ID, 0.0)), // Placeholder
    );

    context.add_node(fan_datoid)?;
    context.add_node(temp_datoid)?;
    context.add_node(power_datoid)?;

    // The "high" thresholds the fusion logic compares each reading with.
    for (id, threshold) in [
        (FAN_SPEED_THRESHOLD_ID, 80.0),
        (CPU_TEMP_THRESHOLD_ID, 85.0),
        (POWER_DRAW_THRESHOLD_ID, 250.0),
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, threshold)),
        ))?;
    }

    Ok(context)
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

    // Write each reading into its sensor's Datoid. A sensor without a Datoid is an error, so a
    // reading is never left stale.
    for (id, reading) in [
        (FAN_SPEED_ID, fan_speed),
        (CPU_TEMP_ID, cpu_temp),
        (POWER_DRAW_ID, power_draw),
    ] {
        let updated = {
            let node = context
                .get_node_index_by_id(id)
                .and_then(|index| context.get_node(index))
                .ok_or_else(|| format!("Sensor with ID {id} not found in context"))?;
            let ContextoidType::Datoid(datoid) = node.vertex_type() else {
                return Err(format!("Sensor with ID {id} is not a Datoid").into());
            };
            Contextoid::new(
                node.id(),
                ContextoidType::Datoid(Data::new(datoid.id(), reading)),
            )
        };
        context.update_node(id, updated)?;
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
        let ctx_arc = match context {
            Some(c) => c,
            None => {
                return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                    "Context is missing".into(),
                )));
            }
        };

        let ctx = match ctx_arc.read() {
            Ok(guard) => guard,
            Err(_) => {
                return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                    "Server context lock is poisoned".into(),
                )));
            }
        };

        // Helper to get a Datoid value (a reading or a threshold) from the context
        let get_value = |id: IdentificationValue| -> Result<NumericalValue, CausalityError> {
            ctx.get_node_index_by_id(id)
                .and_then(|index| ctx.get_node(index))
                .ok_or_else(|| {
                    CausalityError(CausalityErrorEnum::Custom(format!(
                        "Datoid with ID {} not found in context",
                        id
                    )))
                })
                .and_then(|node| {
                    if let ContextoidType::Datoid(datoid) = node.vertex_type() {
                        Ok(datoid.get_data())
                    } else {
                        Err(CausalityError(CausalityErrorEnum::Custom(format!(
                            "Contextoid for ID {} is not a Datoid",
                            id
                        ))))
                    }
                })
        };

        // Read the thresholds from the context
        let fan_threshold = match get_value(FAN_SPEED_THRESHOLD_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let temp_threshold = match get_value(CPU_TEMP_THRESHOLD_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let power_threshold = match get_value(POWER_DRAW_THRESHOLD_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };

        // Read all sensor values from the context
        let fan_speed = match get_value(FAN_SPEED_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let fan_high = fan_speed > fan_threshold;

        let cpu_temp = match get_value(CPU_TEMP_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let cpu_temp_high = cpu_temp > temp_threshold;

        let power_draw = match get_value(POWER_DRAW_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let power_draw_high = power_draw > power_threshold;

        // The fusion logic: all must be high
        let all_high = fan_high && cpu_temp_high && power_draw_high;

        PropagatingProcess::pure(all_high)
    }

    Causaloid::new_with_context(id, context_causal_fn, context, description)
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
