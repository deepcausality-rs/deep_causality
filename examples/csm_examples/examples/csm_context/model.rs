/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CPU_TEMP_ID, FAN_SPEED_ID, POWER_DRAW_ID, SERVER_HIGH_LOAD_STATE_ID};
use deep_causality::EffectValue;
use deep_causality::{
    BaseContext, CSM, CausalAction, CausalState, CausalityError, CausalityErrorEnum, Causaloid,
    Contextoid, ContextoidType, ContextuableGraph, Data, Datable, Identifiable,
    IdentificationValue, NumericalValue, PropagatingEffect, PropagatingProcess,
};
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

/// Creates the initial context for the server, populating it with Datoid nodes for each sensor reading.
pub(crate) fn get_server_context_initial() -> BaseContext {
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

    context
        .add_node(fan_datoid)
        .expect("Failed to add fan datoid");
    context
        .add_node(temp_datoid)
        .expect("Failed to add temp datoid");
    context
        .add_node(power_datoid)
        .expect("Failed to add power datoid");

    context
}

/// Updates the sensor data within the provided context.
pub(crate) fn update_context_dataoids(
    context_arc: &Arc<RwLock<BaseContext>>,
    fan_speed: NumericalValue,
    cpu_temp: NumericalValue,
    power_draw: NumericalValue,
) {
    let mut context = context_arc.write().unwrap();

    // Update fan speed
    if let Some(node_index) = (*context).get_node_index_by_id(FAN_SPEED_ID)
        && let Some(node) = (*context).get_node(node_index)
        && let ContextoidType::Datoid(datoid) = node.vertex_type()
    {
        let new_datoid = Data::new(datoid.id(), fan_speed);
        let new_contextoid = Contextoid::new(node.id(), ContextoidType::Datoid(new_datoid));
        (*context)
            .update_node(FAN_SPEED_ID, new_contextoid)
            .expect("Failed to update fan speed node");
    }

    // Update CPU temp
    if let Some(node_index) = (*context).get_node_index_by_id(CPU_TEMP_ID)
        && let Some(node) = (*context).get_node(node_index)
        && let ContextoidType::Datoid(datoid) = node.vertex_type()
    {
        let new_datoid = Data::new(datoid.id(), cpu_temp);
        let new_contextoid = Contextoid::new(node.id(), ContextoidType::Datoid(new_datoid));
        (*context)
            .update_node(CPU_TEMP_ID, new_contextoid)
            .expect("Failed to update CPU temp node");
    }

    // Update power draw
    if let Some(node_index) = (*context).get_node_index_by_id(POWER_DRAW_ID)
        && let Some(node) = (*context).get_node(node_index)
        && let ContextoidType::Datoid(datoid) = node.vertex_type()
    {
        let new_datoid = Data::new(datoid.id(), power_draw);
        let new_contextoid = Contextoid::new(node.id(), ContextoidType::Datoid(new_datoid));
        (*context)
            .update_node(POWER_DRAW_ID, new_contextoid)
            .expect("Failed to update power draw node");
    }
}

/// Builds the causal model for the server. A single causaloid with a contextual function
/// that performs the fusion logic.
pub(crate) fn get_server_causaloid(context: Arc<RwLock<BaseContext>>) -> CsmCausaloid {
    let id: IdentificationValue = 1;
    let description = "Fused Server Sensors Logic";

    // New API: fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
    fn context_causal_fn(
        _effect: EffectValue<f64>,
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

        // Thresholds
        let fan_threshold = 80.0;
        let temp_threshold = 85.0;
        let power_threshold = 250.0;

        let ctx = ctx_arc.read().unwrap();

        // Helper to get a sensor value from the context
        let get_sensor_value = |id: IdentificationValue| -> Result<NumericalValue, CausalityError> {
            ctx.get_node_index_by_id(id)
                .and_then(|index| ctx.get_node(index))
                .ok_or_else(|| {
                    CausalityError(CausalityErrorEnum::Custom(format!(
                        "Sensor with ID {} not found in context",
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

        // Read all sensor values from the context
        let fan_speed = match get_sensor_value(FAN_SPEED_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let fan_high = fan_speed > fan_threshold;

        let cpu_temp = match get_sensor_value(CPU_TEMP_ID) {
            Ok(v) => v,
            Err(e) => return PropagatingProcess::from_error(e),
        };
        let cpu_temp_high = cpu_temp > temp_threshold;

        let power_draw = match get_sensor_value(POWER_DRAW_ID) {
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
