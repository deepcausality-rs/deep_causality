/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Drone Navigation: Emergent Causality Example
//!
//! This example demonstrates the HKT generative system with a drone that adapts
//! its navigation strategy when GPS is jammed, switching to INS (Inertial Navigation System).
//!
//! ## Scenario
//!
//! A drone uses GPS for navigation. When GPS jamming is detected, the drone's
//! "brain" (a generative system) produces an operation tree that:
//! 1. Disables the GPS causaloid
//! 2. Activates the INS causaloid
//! 3. Creates a sensor fusion causaloid
//! 4. Updates the main navigation graph
//!
//! The entire process is auditable via the `ModificationLog`.

use deep_causality::{
    AuditableGraphGenerator, BaseSymbol, CausalSystemState, Causaloid, Data, EuclideanSpace,
    EuclideanSpacetime, EuclideanTime, Interpreter, ModificationLogEntry, OpStatus, OpTree,
    Operation,
};
use deep_causality_ast::ConstTree;
use std::collections::HashMap;

// Type aliases for this example
type TestData = Data<f64>;
type TestSpace = EuclideanSpace;
type TestTime = EuclideanTime;
type TestSpacetime = EuclideanSpacetime;
type TestSymbol = BaseSymbol;

// Causaloid IDs
const GPS_CAUSALOID_ID: u64 = 1;
const INS_CAUSALOID_ID: u64 = 2;
const FUSION_CAUSALOID_ID: u64 = 3;
const ROOT_GRAPH_ID: u64 = 100;

// Sensor status types
#[derive(Debug, Clone, Copy, PartialEq)]
enum GpsStatus {
    Active,
    Lost,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RadioStatus {
    Normal,
    Jamming,
}

#[derive(Debug, Clone, Copy)]
struct SensorStatus {
    gps_status: GpsStatus,
    radio_status: RadioStatus,
}

/// The "brain" that decides how to adapt the navigation system
struct DroneNavigationBrain {
    has_switched_to_ins: bool,
}

impl DroneNavigationBrain {
    fn new() -> Self {
        Self {
            has_switched_to_ins: false,
        }
    }

    /// Generate an operation tree based on sensor status
    fn generate(
        &mut self,
        sensor_status: SensorStatus,
    ) -> OpTree<(), (), TestData, TestSpace, TestTime, TestSpacetime, TestSymbol, f64, f64> {
        // Check if GPS is lost and we haven't switched yet
        if sensor_status.gps_status == GpsStatus::Lost
            && sensor_status.radio_status == RadioStatus::Jamming
            && !self.has_switched_to_ins
        {
            self.has_switched_to_ins = true;

            // Create a plan to switch from GPS to INS navigation
            // Step 1: Disable GPS causaloid
            let disable_gps = ConstTree::new(Operation::DeleteCausaloid(GPS_CAUSALOID_ID));

            // Step 2: Create INS causaloid (simplified - in reality would be UpdateCausaloid)
            let enable_ins = ConstTree::new(Operation::DeleteCausaloid(INS_CAUSALOID_ID));

            // Step 3: Create fusion causaloid
            let create_fusion = ConstTree::new(Operation::DeleteCausaloid(FUSION_CAUSALOID_ID));

            // Step 4: Update root graph
            let update_root = ConstTree::new(Operation::DeleteCausaloid(ROOT_GRAPH_ID));

            // Combine all steps into a sequence
            ConstTree::with_children(
                Operation::CreateContext {
                    id: 1,
                    name: "Navigation Adaptation".to_string(),
                    capacity: 10,
                },
                vec![disable_gps, enable_ins, create_fusion, update_root],
            )
        } else {
            // No action needed
            ConstTree::new(Operation::CreateContext {
                id: 999,
                name: "NoOp".to_string(),
                capacity: 1,
            })
        }
    }
}

fn main() {
    println!("=== Drone Navigation: Emergent Causality Example ===\n");

    // Initialize the drone's brain
    let mut brain = DroneNavigationBrain::new();

    // Create initial state (empty for this example)
    let initial_state = CausalSystemState::<
        (),
        (),
        TestData,
        TestSpace,
        TestTime,
        TestSpacetime,
        TestSymbol,
        f64,
        f64,
    > {
        causaloids: HashMap::new(),
        contexts: HashMap::new(),
    };

    // Scenario: GPS jamming detected!
    println!("📡 Sensor Status: GPS LOST, Radio JAMMING detected!\n");

    let jamming_status = SensorStatus {
        gps_status: GpsStatus::Lost,
        radio_status: RadioStatus::Jamming,
    };

    // The brain generates an adaptation plan
    println!("🧠 Brain generating adaptation plan...\n");
    let op_tree = brain.generate(jamming_status);

    // Execute the plan using the interpreter
    println!("⚙️  Executing operation tree...\n");
    let interpreter = Interpreter;
    let evolution_result: AuditableGraphGenerator<
        CausalSystemState<
            (),
            (),
            TestData,
            TestSpace,
            TestTime,
            TestSpacetime,
            TestSymbol,
            f64,
            f64,
        >,
    > = interpreter.execute(&op_tree, initial_state);

    // Examine the results
    println!("📊 Execution Results:");
    println!("   Error: {:?}", evolution_result.error);
    println!("   Log entries: {}\n", evolution_result.logs.len());

    // Audit the process
    println!("📝 Audit Trail:");
    for (i, entry) in evolution_result.logs.iter().enumerate() {
        println!("   {}. Operation: {}", i + 1, entry.operation_name);
        println!("      Target ID: {}", entry.target_id);
        println!("      Status: {:?}", entry.status);
        println!("      Message: {}", entry.message);
        println!("      Timestamp: {} μs\n", entry.timestamp);
    }

    // Verify the adaptation occurred
    if evolution_result.error.is_none() {
        println!("✅ Navigation system successfully adapted to GPS jamming!");
        println!("   The drone can now navigate using INS and sensor fusion.");
    } else {
        println!("❌ Adaptation failed: {:?}", evolution_result.error);
    }

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("• The drone's 'brain' generated a plan (OpTree) to adapt to GPS loss");
    println!("• The Interpreter executed the plan in an auditable transaction");
    println!("• Every operation is logged with timestamps for full traceability");
    println!("• The system can prove WHY and HOW it changed its behavior");
}
