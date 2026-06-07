/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
//! The episode is one `CausalFlow` pipeline: the quantum history rides the state channel,
//! each step is a named stage, and the value channel carries the "error detected" flag.

use deep_causality_core::{
    CausalEffectPropagationProcess, CausalFlow, EffectValue, PropagatingProcess,
};
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::{Complex, DivisionAlgebra};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

/// Holds the history of quantum states for counterfactual debugging.
/// Each state represents a snapshot at a different point in time.
#[derive(Debug, Clone, Default)]
struct QuantumHistory {
    states: Vec<HilbertState<FloatType>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== The Dead Qubit: Time-Travel Debugging ===\n");

    // 1. Initial State: |0> + |1> (Superposition)
    let metric = Metric::Euclidean(1); // 1 Qubit approx
    let psi_0 = vec![Complex::new(0.707, 0.0), Complex::new(0.707, 0.0)];
    let initial_state =
        HilbertState::<FloatType>::new(psi_0, metric).expect("Failed to create state");

    let history = QuantumHistory {
        states: vec![initial_state],
    };

    // The error-correction episode as one CausalFlow pipeline. The quantum history is the
    // state channel; each step binds the next "error detected" flag and may rewrite history.
    let result = CausalFlow::process(history)
        .bind(stage_apply_gate)
        .bind(stage_measure_syndrome)
        .bind(stage_correct)
        .into_process();

    // Verification: access final state from the process struct.
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

/// Step 1: apply a gate that drifts the qubit into a bit-flip error state |1>.
fn stage_apply_gate(
    _value: EffectValue<()>,
    mut hist: QuantumHistory,
    ctx: Option<()>,
) -> PropagatingProcess<bool, QuantumHistory, ()> {
    println!("[t=1] Applying Quantum Gate...");

    // Simulate a drift to an error state |1> (flipped from the desired |0>).
    let bad_psi = vec![Complex::new(0.01, 0.0), Complex::new(0.99, 0.0)];
    let bad_state = HilbertState::<FloatType>::new(bad_psi, Metric::Euclidean(1)).unwrap();
    hist.states.push(bad_state);

    // No error detected yet.
    let next = CausalEffectPropagationProcess::pure(false);
    CausalEffectPropagationProcess::with_state(next, hist, ctx)
}

/// Step 2: measure the syndrome. Raise the error flag if P(|1>) is dominant.
fn stage_measure_syndrome(
    prev_val_effect: EffectValue<bool>,
    hist: QuantumHistory,
    ctx: Option<()>,
) -> PropagatingProcess<bool, QuantumHistory, ()> {
    println!("[t=2] Measuring Syndrome...");
    let prev_val = prev_val_effect.into_value().unwrap_or(false);

    let current_state = hist.states.last().unwrap();
    let prob_1 = current_state.as_inner().data()[1].norm_sqr();

    if prob_1 > 0.9 {
        println!("[ALARM] Bit Flip Error Detected! P(|1>) = {:.4}", prob_1);
        let next = CausalEffectPropagationProcess::pure(true);
        return CausalEffectPropagationProcess::with_state(next, hist, ctx);
    }

    let next = CausalEffectPropagationProcess::pure(prev_val);
    CausalEffectPropagationProcess::with_state(next, hist, ctx)
}

/// Step 3: counterfactual correction. On a detected error, rewind history to t=0 and
/// re-apply the correct |0> state (an X gate in this metaphor).
fn stage_correct(
    error_detected_effect: EffectValue<bool>,
    mut hist: QuantumHistory,
    ctx: Option<()>,
) -> PropagatingProcess<bool, QuantumHistory, ()> {
    let error_detected = error_detected_effect.into_value().unwrap_or(false);

    if error_detected {
        println!("[t=3] Initiating Post-Selection / Rewind...");

        // "Rewind" to t=0 (pop the bad state).
        hist.states.pop();
        println!("[t=3] History Rewound. State restored to t=0.");

        // Force the qubit back to |0>.
        let corrected_psi = vec![Complex::new(0.99, 0.0), Complex::new(0.01, 0.0)];
        let corrected_state =
            HilbertState::<FloatType>::new(corrected_psi, Metric::Euclidean(1)).unwrap();
        hist.states.push(corrected_state);
        println!("[t=4] Applied Correction (X Gate).");

        let next = CausalEffectPropagationProcess::pure(false); // Error cleared
        return CausalEffectPropagationProcess::with_state(next, hist, ctx);
    }

    let next = CausalEffectPropagationProcess::pure(error_detected);
    CausalEffectPropagationProcess::with_state(next, hist, ctx)
}
