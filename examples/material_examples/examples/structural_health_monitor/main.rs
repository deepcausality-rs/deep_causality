/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{EffectValue, Intervenable, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Graph;

// ----------------------------------------------------------------
// Structural Health Monitoring System
// ----------------------------------------------------------------
// A decentralized monitoring system for high-stakes environments (Space/Underwater).
// Each node represents a hull plate. Edges represent structural bonds.
// When stress exceeds threshold, the system uses Causal Interventions to
// autonomously seal/reinforce the affected section.

// Material constants
const YIELD_STRENGTH: f64 = 250.0; // MPa (typical steel)
const CRITICAL_YIELD: f64 = 0.8; // 80% of yield = warning threshold
const SAFE_STRESS_LIMIT: f64 = 100.0; // Target stress after intervention
const DEFAULT_MODULUS: f64 = 200.0; // GPa (steel)
const ENHANCED_MODULUS: f64 = 400.0; // GPa (with active reinforcement)

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================");
    println!("   High-Stakes Structural Health Monitoring System");
    println!("   (Space Station / Deep-Sea Habitat Simulation)");
    println!("================================================================\n");

    // ----------------------------------------------------------------
    // 1. Initialize Hull Topology (Graph)
    // ----------------------------------------------------------------
    println!("[1] Initializing Hull Topology...");

    let num_plates = 6;
    // Each node stores stress value as f64
    let initial_stresses: Vec<f64> = vec![0.0; num_plates];

    let tensor = CausalTensor::new(initial_stresses, vec![num_plates])?;
    let mut hull = Graph::new(num_plates, tensor, 0)?;

    // Connect plates in a ring topology (each plate connected to neighbors)
    // 0 - 1 - 2
    // |       |
    // 5 - 4 - 3
    hull.add_edge(0, 1)?;
    hull.add_edge(1, 2)?;
    hull.add_edge(2, 3)?;
    hull.add_edge(3, 4)?;
    hull.add_edge(4, 5)?;
    hull.add_edge(5, 0)?;
    // Add cross-bracing
    hull.add_edge(0, 4)?;
    hull.add_edge(1, 5)?;

    println!(
        "    Created {} plates with {} structural bonds.",
        num_plates,
        hull.num_edges()
    );
    println!("    Topology: Hexagonal ring with cross-bracing.\n");

    // ----------------------------------------------------------------
    // 2. Simulate Impact Event
    // ----------------------------------------------------------------
    println!("[2] Simulating Micrometeoroid Impact on Plate 2...");

    // Impact introduces sudden stress
    let impact_stress = YIELD_STRENGTH * 0.9; // 90% of yield - critical!

    println!("    Impact Stress: {:.1} MPa", impact_stress);
    println!("    Yield Strength: {:.1} MPa", YIELD_STRENGTH);
    println!(
        "    Warning Threshold: {:.1} MPa ({}%)\n",
        YIELD_STRENGTH * CRITICAL_YIELD,
        (CRITICAL_YIELD * 100.0) as i32
    );

    // ----------------------------------------------------------------
    // 3. Decentralized Monitoring & Intervention
    // ----------------------------------------------------------------
    println!("[3] Running Decentralized Monitoring Loop...\n");

    // Simulate stress on plate 2
    let current_stress = impact_stress;

    // Wrap current state in Causal Effect (monadic container)
    let stress_effect: PropagatingEffect<Option<f64>> =
        PropagatingEffect::pure(Some(current_stress));

    println!("    Plate 2 Sensor Reading: {:.1} MPa", current_stress);

    // Check if intervention is needed
    let warning_threshold = YIELD_STRENGTH * CRITICAL_YIELD;
    let needs_intervention = current_stress > warning_threshold;

    if needs_intervention {
        println!("\n    [\x1b[33mWARNING\x1b[0m] Stress exceeds safety threshold!");
        println!("    [\x1b[33mWARNING\x1b[0m] Central server unreachable (latency: 2.4s).");
        println!("    [\x1b[32mACTION\x1b[0m]  Initiating LOCAL autonomous intervention...\n");

        // ============================================================
        // CAUSAL INTERVENTION (Layer 2 - Pearl's Hierarchy)
        // ============================================================
        // This creates a counterfactual branch: "What if we override the stress?"
        // The intervention is recorded in the effect's causal log.

        let healed_effect = stress_effect.intervene(Some(SAFE_STRESS_LIMIT));

        println!("    > [BLACKBOX AUDIT]: Autonomous Intervention Recorded.");
        println!("    > Intervention Type: ACTIVE_DAMPENING");
        println!("    > Original Stress:  {:.1} MPa", current_stress);
        println!("    > Intervened Stress: {:.1} MPa", SAFE_STRESS_LIMIT);
        println!(
            "    > Modulus Change:   {:.0} GPa -> {:.0} GPa (Smart Material Activated)",
            DEFAULT_MODULUS, ENHANCED_MODULUS
        );

        // Log the intervention result
        match healed_effect.value() {
            EffectValue::Value(Some(v)) => {
                println!(
                    "\n    [\x1b[32mSUCCESS\x1b[0m] Stress reduced to {:.1} MPa.",
                    v
                );
                println!(
                    "    [\x1b[32mSUCCESS\x1b[0m] Plate 2 STABLE. Catastrophic failure AVERTED."
                );
            }
            _ => {
                println!("    [ERROR] Intervention failed!");
            }
        }
    } else {
        println!("    Stress within normal limits. No intervention required.");
    }

    // ----------------------------------------------------------------
    // 4. Comparison: What if we had NO intervention?
    // ----------------------------------------------------------------
    println!("\n[4] Counterfactual Analysis: No Intervention Scenario...\n");

    // Without intervention, stress propagates to neighbors
    let propagation_factor = 1.15; // Each neighbor gets 15% more load
    let neighbor_stress = current_stress * propagation_factor;

    println!("    If Plate 2 had failed (no intervention):");
    println!("    -> Load redistributes to neighbors (Plates 1, 3)");
    println!(
        "    -> Neighbor stress: {:.1} MPa (>{:.1} MPa limit)",
        neighbor_stress, YIELD_STRENGTH
    );

    if neighbor_stress > YIELD_STRENGTH {
        println!("    -> Plates 1 and 3 FAIL!");
        println!("    -> Cascade continues to Plates 0, 4, 5...");
        println!("\n    [\x1b[31mCATASTROPHIC FAILURE\x1b[0m] Complete hull breach.");
        println!("    -> Oxygen depletion in 47 seconds.");
    }

    println!("\n================================================================");
    println!("   Simulation Complete");
    println!("================================================================");
    println!("   The Intervenable trait enabled AUTONOMOUS, LOCAL decision-making.");
    println!("   Latency-critical interventions saved the structure.");

    Ok(())
}
