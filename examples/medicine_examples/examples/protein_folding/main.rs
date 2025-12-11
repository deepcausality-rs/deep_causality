/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Protein Folding Simulation (Generalized Master Equation)
//!
//! Simulates protein folding dynamics using the Generalized Master Equation (GME)
//! with memory kernels for non-Markovian behavior.
//!
//! ## Key Concepts
//! - **Markov Operator**: Transition matrix for instantaneous state changes
//! - **Memory Kernels**: History-dependent corrections (proteins "remember" past states)
//! - **Conformational States**: Unfolded → Intermediate → Native folding pathway
//!
//! ## APIs Demonstrated
//! - `generalized_master_equation()` - Non-Markovian dynamics with memory
//! - `Probability` - Type-safe probability values in [0,1]
//! - `CausalTensor` - Transition and memory kernel matrices

use deep_causality_core::EffectValue;
use deep_causality_physics::{Probability, generalized_master_equation};
use deep_causality_tensor::CausalTensor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Protein Folding: Generalized Master Equation ===\n");

    // Define 4 conformational states:
    // 0: Unfolded
    // 1: Partially Folded (Intermediate 1)
    // 2: Partially Folded (Intermediate 2)
    // 3: Fully Folded (Native State)
    let num_states = 4;

    // Initial state: 100% in Unfolded state
    let mut state: Vec<Probability> = vec![
        Probability::new(1.0).unwrap(),
        Probability::new(0.0).unwrap(),
        Probability::new(0.0).unwrap(),
        Probability::new(0.0).unwrap(),
    ];

    println!("Initial Conformational Distribution:");
    print_state(&state);

    // History: past states (for non-Markovian memory effects)
    // History length must match memory kernel length
    let mut history: Vec<Vec<Probability>> = vec![state.clone(), state.clone(), state.clone()];

    // Markov Transition Matrix (instantaneous folding)
    // For matmul T * P where P is [n,1], T should be [n,n]
    // T[i,j] = probability of transitioning TO state i FROM state j
    // Result: new_P[i] = sum_j(T[i,j] * P[j])
    // In row-major order: row i contains weights for output state i
    #[rustfmt::skip]
    let markov_data = vec![
        // Row 0 (TO Unfolded): from U=0.7, from I1=0.1, from I2=0, from N=0
        0.70, 0.10, 0.00, 0.00,
        // Row 1 (TO I1): from U=0.3, from I1=0.7, from I2=0.1, from N=0
        0.30, 0.70, 0.10, 0.00,
        // Row 2 (TO I2): from U=0, from I1=0.2, from I2=0.4, from N=0
        0.00, 0.20, 0.40, 0.00,
        // Row 3 (TO Native): from U=0, from I1=0, from I2=0.5, from N=1.0
        0.00, 0.00, 0.50, 1.00,
    ];
    let markov_operator = CausalTensor::new(markov_data, vec![num_states, num_states])
        .expect("Failed to create Markov operator");

    // Memory Kernels: small corrections based on history
    // These represent memory effects (e.g., protein "remembers" recent conformations)
    let memory_kernels: Vec<CausalTensor<f64>> = (0..3)
        .map(|lag| {
            // Decay factor for this lag
            let decay = (-0.5 * (lag as f64 + 1.0)).exp() * 0.02;

            // Small memory corrections
            let mut data = vec![0.0; num_states * num_states];

            // Memory favors forward progression
            data[1] = decay; // Unfolded -> I1
            data[num_states + 2] = decay; // I1 -> I2
            data[2 * num_states + 3] = decay; // I2 -> Native

            CausalTensor::new(data, vec![num_states, num_states])
                .expect("Failed to create memory kernel")
        })
        .collect();

    println!("\nSimulating folding dynamics with Markov + Memory...\n");

    // Simulation loop
    let time_steps = 15;
    for t in 1..=time_steps {
        // Apply Generalized Master Equation with Markov operator
        let effect =
            generalized_master_equation(&state, &history, Some(&markov_operator), &memory_kernels);

        match effect.value() {
            EffectValue::Value(new_state) => {
                // Normalize to ensure probabilities sum to 1
                let total: f64 = new_state.iter().map(|p| p.value()).sum();

                // Handle edge case: if total is zero or NaN, keep current state
                if total <= 0.0 || total.is_nan() {
                    println!("[t={:>2}] Normalization issue, keeping previous state", t);
                    continue;
                }

                state = new_state
                    .iter()
                    .map(|p| {
                        let normalized = (p.value() / total).clamp(0.0, 1.0);
                        Probability::new(if normalized.is_nan() { 0.0 } else { normalized })
                            .unwrap_or(Probability::new(0.0).unwrap())
                    })
                    .collect();

                // Update history (sliding window)
                history.remove(0);
                history.push(state.clone());

                println!("[t={:>2}] Distribution:", t);
                print_state(&state);
            }
            _ => {
                eprintln!("[t={}] GME computation failed: {:?}", t, effect.error);
            }
        }
    }

    println!("\n--- Folding Summary ---");
    let native_prob = state[3].value();
    println!("Final Native State Probability: {:.4}", native_prob);

    if native_prob > 0.5 {
        println!("[SUCCESS] Protein has reached the native state!");
    } else if native_prob > 0.2 {
        println!("[PROGRESS] Protein is folding...");
    } else {
        println!("[SLOW] Folding is still in early stages.");
    }

    println!("\n[COMPLETE] Protein Folding Simulation Finished.");

    Ok(())
}

fn print_state(state: &[Probability]) {
    let labels = ["Unfolded", "Intermed1", "Intermed2", "Native  "];
    for (i, p) in state.iter().enumerate() {
        let bar_len = (p.value() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len);
        println!("  {}: {:>6.2}% {}", labels[i], p.value() * 100.0, bar);
    }
}
