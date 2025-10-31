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
    let all_sensor_data = get_all_sensor_data();

    // Initialize context, model, and CSM outside the loop
    let server_context = Arc::new(RwLock::new(get_server_context_initial()));
    let server_model = get_server_causal_model(Arc::clone(&server_context));
    let server_csm = get_server_csm(server_model);

    println!("\n--- Starting Server Monitoring Loop (10 cycles) ---");
    for (i, (fan_speed, cpu_temp, power_draw)) in all_sensor_data.iter().enumerate() {
        println!("\n--- Cycle {} ---", i + 1);

        println!(
            "Sensor readings: Fan Speed: {}, CPU Temp: {}, Power Draw: {}",
            fan_speed, cpu_temp, power_draw
        );

        // Update the context dataoids with the current sensor data
        update_context_dataoids(&server_context, *fan_speed, *cpu_temp, *power_draw);

        // Evaluate the CSM state. The data is now in the context, so we pass a None effect.
        server_csm
            .eval_single_state(SERVER_HIGH_LOAD_STATE_ID as usize, &PropagatingEffect::None)
            .expect("Evaluation failed");

        // Pause for a moment to simulate a real-time loop.
        thread::sleep(Duration::from_millis(250));
    }

    println!("\n--- Monitoring Loop Finished ---");
}

// Some sensor test data
fn get_all_sensor_data() -> Vec<(NumericalValue, NumericalValue, NumericalValue)> {
    let all_sensor_data = vec![
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
    ];
    all_sensor_data
}

/// Creates the initial context for the server, populating it with Datoid nodes for each sensor reading.
fn get_server_context_initial() -> BaseContext {
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
fn update_context_dataoids(
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

/// Builds the causal model for the server. Instead of a collection, it's now a single
/// causaloid with a contextual function that performs the fusion logic.
fn get_server_causal_model(context: Arc<RwLock<BaseContext>>) -> BaseCausaloid {
    let fusion_logic_fn = |_: &PropagatingEffect,
                           context: &Arc<RwLock<BaseContext>>|
     -> Result<PropagatingEffect, CausalityError> {
        // Thresholds
        let fan_threshold = 80.0;
        let temp_threshold = 85.0;
        let power_threshold = 250.0;

        let ctx = context.read().unwrap();

        // Helper to get a sensor value from the context
        let get_sensor_value = |id: IdentificationValue| -> Result<NumericalValue, CausalityError> {
            ctx.get_node_index_by_id(id)
                .and_then(|index| ctx.get_node(index))
                .ok_or_else(|| {
                    CausalityError(format!("Sensor with ID {} not found in context", id))
                })
                .and_then(|node| {
                    if let ContextoidType::Datoid(datoid) = node.vertex_type() {
                        Ok(datoid.get_data())
                    } else {
                        Err(CausalityError(format!(
                            "Contextoid for ID {} is not a Datoid",
                            id
                        )))
                    }
                })
        };

        // Read all sensor values from the context
        let fan_speed = get_sensor_value(FAN_SPEED_ID)?;
        let cpu_temp = get_sensor_value(CPU_TEMP_ID)?;
        let power_draw = get_sensor_value(POWER_DRAW_ID)?;

        // The fusion logic: all must be high
        let all_high =
            fan_speed > fan_threshold && cpu_temp > temp_threshold && power_draw > power_threshold;

        Ok(PropagatingEffect::Deterministic(all_high))
    };

    Causaloid::new_with_context(
        4, // A new ID for the fused model
        fusion_logic_fn,
        context,
        "Fused Server Sensors Logic",
    )
}

/// Creates a Causal State Machine (CSM) that links the server's causal model
/// to a specific action.
fn get_server_csm(
    server_model: BaseCausaloid,
) -> CSM<Data<f64>, EuclideanSpace, EuclideanTime, EuclideanSpacetime, BaseSymbol, f64, f64> {
    let high_load_state = CausalState::new(
        SERVER_HIGH_LOAD_STATE_ID as usize,
        1,                       // version
        PropagatingEffect::None, // Data is in the context
        server_model,
        None,
    );

    // Could also trigger a process to add more servers to decrease load for each
    let high_load_action = CausalAction::new(
        || {
            println!();
            println!(
                "\n>>> (!)ALERT(!): Server is under high load! Risk of failure. (!)ALERT(!)<<<"
            );
            println!();
            Ok(())
        },
        "High Load Alert",
        1, // version
    );

    CSM::new(&[(&high_load_state, &high_load_action)], None)
}
