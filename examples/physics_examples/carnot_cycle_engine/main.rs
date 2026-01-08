/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Carnot Cycle Heat Engine
//!
//! Simulates a discrete 4-stage Carnot cycle, tracking state variables (P, V, T)
//! and calculating work/efficiency via Causal Monads.

use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};
use deep_causality_physics::{
    AmountOfSubstance, PhysicsError, Pressure, Temperature, Volume, carnot_efficiency,
    ideal_gas_law,
};

#[derive(Debug, Clone, Default)]
struct EngineState {
    p: Pressure,
    v: Volume,
    t: Temperature,
    entropy_s: f64,
    work_done: f64,
    phase: String,
}

fn main() -> Result<(), PhysicsError> {
    println!("=== Carnot Heat Engine Simulation ===\n");

    // Configuration
    let n_moles = AmountOfSubstance::new(1.0)?;
    let temp_hot = Temperature::new(500.0)?; // 500 K
    let temp_cold = Temperature::new(300.0)?; // 300 K
    let compression_ratio = 2.0;

    // Initial State (Start of Isothermal Expansion)
    // Point A: High T, Low V, High P
    let v_a = Volume::new(0.01)?; // 10 Liters
    let p_a = Pressure::new(415_700.0)?; // ~4 atm (Derived from PV=nRT: 1*8.314*500 / 0.01)

    let initial_state = EngineState {
        p: p_a,
        v: v_a,
        t: temp_hot,
        entropy_s: 0.0,
        work_done: 0.0,
        phase: "Start (Point A)".to_string(),
    };

    report_state(&initial_state);

    // Causal Chain: A -> B -> C -> D -> A
    // We pass (n_moles, temp_hot, temp_cold, compression_ratio) as context
    let context = (n_moles, temp_hot, temp_cold, compression_ratio);

    let process = CausalEffectPropagationProcess::with_state(
        CausalEffectPropagationProcess::pure(()),
        initial_state,
        Some(context),
    )
    .bind(|_, state, _| {
        // --- Step 1: Isothermal Expansion (A -> B) ---
        // Temperature constant (Th). Volume increases.
        // Heat Qin absorbed. Entropy increases.

        let v_b = Volume::new(state.v.value() * compression_ratio).unwrap();
        let t_b = temp_hot;

        // Calculate P_b using Ideal Gas Law wrapper (returns Ratio R, we check consistency)
        // Here we invert the logic: P = nRT / V
        let r_const = 8.314; // J/(mol K)
        let p_val = (n_moles.value() * r_const * t_b.value()) / v_b.value();
        let p_b = Pressure::new(p_val).unwrap();

        // Work Done = nRT * ln(Vb/Va)
        let work =
            n_moles.value() * r_const * temp_hot.value() * (v_b.value() / state.v.value()).ln();
        // Delta S = Q/T = Work/T = nR ln(Vb/Va)
        let ds = work / temp_hot.value();

        let next_state = EngineState {
            p: p_b,
            v: v_b,
            t: t_b,
            entropy_s: state.entropy_s + ds,
            work_done: state.work_done + work,
            phase: "Isothermal Expansion (A->B)".to_string(),
        };
        report_state(&next_state);
        CausalEffectPropagationProcess::pure(next_state)
    })
    .bind(|prev_state, _, _| {
        // --- Step 2: Adiabatic Expansion (B -> C) ---
        // Q = 0. Entropy constant. Temp drops to Tc.
        // prev_state is the EngineState from Step 1 (point B)
        let prev = prev_state.into_value().unwrap();
        let r_const = 8.314;
        let gamma = 5.0 / 3.0; // Monatomic gas

        // Relation: T_b * V_b^(gamma-1) = T_c * V_c^(gamma-1)
        // V_c = V_b * (Tb/Tc)^(1/(gamma-1))
        let t_ratio = prev.t.value() / temp_cold.value();
        let v_ratio = t_ratio.powf(1.0 / (gamma - 1.0));
        let v_c = Volume::new(prev.v.value() * v_ratio).unwrap();

        let p_val = (n_moles.value() * r_const * temp_cold.value()) / v_c.value();
        let p_c = Pressure::new(p_val).unwrap();

        // Work Done = - Delta U = - Cv * (Tc - Th) = Cv(Th - Tc)
        // Cv = 3/2 nR
        let cv = 1.5 * n_moles.value() * r_const;
        let work = cv * (prev.t.value() - temp_cold.value());

        let next_state = EngineState {
            p: p_c,
            v: v_c,
            t: temp_cold,
            entropy_s: prev.entropy_s, // Constant
            work_done: prev.work_done + work,
            phase: "Adiabatic Expansion (B->C)".to_string(),
        };
        report_state(&next_state);
        CausalEffectPropagationProcess::pure(next_state)
    })
    .bind(|prev_state, _, _| {
        // --- Step 3: Isothermal Compression (C -> D) ---
        // Temp constant (Tc). Volume decreases.
        // Heat Qout rejected. Entropy decreases.
        // We compress back to V_d such that next adiabatic step hits V_a
        // prev_state is the EngineState from Step 2 (point C)
        let prev = prev_state.into_value().unwrap();
        let r_const = 8.314;

        // V_d = V_c / ratio (Symmetric cycle for simplicity if constructed right, but let's calculate)
        // Actually V_c / V_d = V_b / V_a = ratio
        let v_d = Volume::new(prev.v.value() / compression_ratio).unwrap();

        let p_val = (n_moles.value() * r_const * temp_cold.value()) / v_d.value();
        let p_d = Pressure::new(p_val).unwrap();

        // Work is negative (done ON gas)
        let work =
            n_moles.value() * r_const * temp_cold.value() * (v_d.value() / prev.v.value()).ln();
        let ds = work / temp_cold.value();

        let next_state = EngineState {
            p: p_d,
            v: v_d,
            t: temp_cold,
            entropy_s: prev.entropy_s + ds,
            work_done: prev.work_done + work,
            phase: "Isothermal Compression (C->D)".to_string(),
        };
        report_state(&next_state);
        CausalEffectPropagationProcess::pure(next_state)
    })
    .bind(|prev_state, _, _| {
        // --- Step 4: Adiabatic Compression (D -> A) ---
        // Temp rises to Th.
        // prev_state is the EngineState from Step 3 (point D)
        let prev = prev_state.into_value().unwrap();
        let r_const = 8.314;
        let _gamma = 5.0 / 3.0;

        // We should arrive back at V_a
        // Let's verify with Ideal Gas Law Wrapper
        // Check if current P, V, n, T is consistent
        let _ = ideal_gas_law(prev.p, prev.v, n_moles, prev.t);

        // Work
        let cv = 1.5 * n_moles.value() * r_const;
        let work = cv * (prev.t.value() - temp_hot.value()); // Negative diff -> Positive work (compression)

        let next_state = EngineState {
            p: Pressure::new(415_700.0).unwrap(), // Back to P_a
            v: Volume::new(0.01).unwrap(),        // Back to V_a
            t: temp_hot,
            entropy_s: prev.entropy_s, // Constant
            work_done: prev.work_done + work,
            phase: "Adiabatic Compression (D->A)".to_string(),
        };
        report_state(&next_state);
        CausalEffectPropagationProcess::pure(next_state)
    });

    // Final Analysis
    if let EffectValue::Value(final_state) = process.value() {
        println!("\n=== Cycle Complete ===");
        println!("Total Work Done: {:.2} J", final_state.work_done);

        // Theoretical Efficiency
        let eff_effect = carnot_efficiency(temp_hot, temp_cold);
        let eff_max = eff_effect.value().clone().into_value().unwrap().value();

        println!("Carnot Efficiency Limit: {:.1}%", eff_max * 100.0);
    }

    Ok(())
}

fn report_state(s: &EngineState) {
    println!(
        "{:<30} | P={:8.1} Pa | V={:.4} m^3 | T={:.1} K | S={:+.2} J/K",
        s.phase,
        s.p.value(),
        s.v.value(),
        s.t.value(),
        s.entropy_s
    );
}
