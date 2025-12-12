/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Quantum Counterfactual: The Dead Qubit
//!
//! Demonstrates quantum error correction via history-aware state management.
//!
//! ## Key Concepts
//! - **Error Detection**: Syndrome measurement identifies bit-flip errors
//! - **State Rewind**: Pop corrupted states from history to "time travel"
//! - **Error Correction**: Apply corrective gate after rewind
//!
//! ## APIs Demonstrated
//! - `CausalEffectPropagationProcess::with_state()` - Stateful monadic composition
//! - `HilbertState` - Quantum state vectors with complex amplitudes
//! - `.bind()` - Chain quantum operations monadically

use deep_causality_core::CausalEffectPropagationProcess;
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::{Complex, DivisionAlgebra};

/// Holds the history of quantum states for counterfactual debugging.
/// Each state represents a snapshot at a different point in time.
#[derive(Debug, Clone, Default)]
struct QuantumHistory {
    states: Vec<HilbertState>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== The Dead Qubit: Time-Travel Debugging ===\n");

    // 1. Initial State: |0> + |1> (Superposition)
    let metric = Metric::Euclidean(1); // 1 Qubit approx
    let psi_0 = vec![Complex::new(0.707, 0.0), Complex::new(0.707, 0.0)];
    let initial_state = HilbertState::new(psi_0, metric).expect("Failed to create state");

    let history = QuantumHistory {
        states: vec![initial_state],
    };

    // Monadic Chain: Simulating Evolution with Error
    let initial_effect = CausalEffectPropagationProcess::pure(false); // "Is Error Detected?" flag

    // Wrap with state
    let result = CausalEffectPropagationProcess::with_state(initial_effect, history, None::<()>)
        .bind(|_, mut hist, ctx| {
            // Step 1: Apply Gate (Simulate Bit Flip Error)
            println!("[t=1] Applying Quantum Gate...");

            // Here we simulate a "drift" to an error state.
            // Error State: |1> (flipped from desired |0>)
            let bad_psi = vec![Complex::new(0.01, 0.0), Complex::new(0.99, 0.0)];
            let bad_state = HilbertState::new(bad_psi, Metric::Euclidean(1)).unwrap();

            hist.states.push(bad_state);

            // Return "No Error Detected Yet". Wrap in pure and restore state.
            let next = CausalEffectPropagationProcess::pure(false);
            CausalEffectPropagationProcess::with_state(next, hist, ctx)
        })
        .bind(|prev_val_effect, hist, ctx| {
            // Step 2: Measure / Check Syndrome
            println!("[t=2] Measuring Syndrome...");
            let prev_val = prev_val_effect.into_value().unwrap_or(false);

            let current_state = hist.states.last().unwrap();
            // Check probability of |1> using as_inner() to access the underlying MultiVector
            let prob_1 = current_state.as_inner().data()[1].norm_sqr();

            if prob_1 > 0.9 {
                println!("[ALARM] Bit Flip Error Detected! P(|1>) = {:.4}", prob_1);
                let next = CausalEffectPropagationProcess::pure(true); // Error Detected = true
                return CausalEffectPropagationProcess::with_state(next, hist, ctx);
            }

            let next = CausalEffectPropagationProcess::pure(prev_val);
            CausalEffectPropagationProcess::with_state(next, hist, ctx)
        })
        .bind(|error_detected_effect, mut hist, ctx| {
            // Step 3: Counterfactual Correction (Time Travel)
            let error_detected = error_detected_effect.into_value().unwrap_or(false);

            if error_detected {
                println!("[t=3] Initiating Post-Selection / Rewind...");

                // "Rewind" to t=0 (pop the bad state)
                hist.states.pop(); // Removes t=1 (Bad State)

                println!("[t=3] History Rewound. State restored to t=0.");

                // Apply Correction: X Gate (In this metaphor, we re-apply correct evolution or fix)
                // Let's say we force it back to |0>
                let corrected_psi = vec![Complex::new(0.99, 0.0), Complex::new(0.01, 0.0)];
                let corrected_state =
                    HilbertState::new(corrected_psi, Metric::Euclidean(1)).unwrap();

                hist.states.push(corrected_state);
                println!("[t=4] Applied Correction (X Gate).");

                let next = CausalEffectPropagationProcess::pure(false); // Error cleared
                return CausalEffectPropagationProcess::with_state(next, hist, ctx);
            }

            let next = CausalEffectPropagationProcess::pure(error_detected);
            CausalEffectPropagationProcess::with_state(next, hist, ctx)
        });

    // Verification
    // Access final state from the process struct
    let final_state_struct = &result.state;
    let final_quantum_state = final_state_struct.states.last().unwrap();
    let prob_0 = final_quantum_state.as_inner().data()[0].norm_sqr();

    println!("\nFinal System State:");
    println!("  History Length: {}", final_state_struct.states.len());
    println!("  P(|0>) = {:.4}", prob_0);

    if prob_0 > 0.9 {
        println!("[SUCCESS] Qubit is alive and corrected.");
    } else {
        println!("[FAILURE] Qubit is dead.");
    }

    Ok(())
}
