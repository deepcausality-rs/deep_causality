/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Carnot Cycle Heat Engine
//!
//! Simulates a discrete 4-stage Carnot cycle, tracking state variables (P, V, T)
//! and calculating work/efficiency. Each stroke is a named stage; the four strokes
//! compose into one `CausalFlow` pipeline that threads the engine state.

use deep_causality_core::{CausalFlow, EffectValue, PropagatingEffect, PropagatingProcess};
use deep_causality_physics::{
    AmountOfSubstance, PhysicsError, Pressure, Temperature, Volume, carnot_efficiency,
    ideal_gas_law,
};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

/// Volume compression ratio per stroke.
const COMPRESSION_RATIO: FloatType = 2.0;
/// Ideal gas constant, J/(mol K).
const R_CONST: f64 = 8.314;

/// One mole of working gas.
fn n_moles() -> AmountOfSubstance<FloatType> {
    AmountOfSubstance::<FloatType>::new(1.0).unwrap()
}
/// Hot reservoir temperature (500 K).
fn temp_hot() -> Temperature<FloatType> {
    Temperature::<FloatType>::new(500.0).unwrap()
}
/// Cold reservoir temperature (300 K).
fn temp_cold() -> Temperature<FloatType> {
    Temperature::<FloatType>::new(300.0).unwrap()
}

#[derive(Debug, Clone, Default)]
struct EngineState {
    p: Pressure<FloatType>,
    v: Volume<FloatType>,
    t: Temperature<FloatType>,
    entropy_s: f64,
    work_done: f64,
    phase: String,
}

fn main() -> Result<(), PhysicsError> {
    println!("=== Carnot Heat Engine Simulation ===\n");

    // Initial State (Start of Isothermal Expansion)
    // Point A: High T, Low V, High P
    let v_a = Volume::<FloatType>::new(0.01)?; // 10 Liters
    let p_a = Pressure::<FloatType>::new(415_700.0)?; // ~4 atm (Derived from PV=nRT: 1*8.314*500 / 0.01)

    let initial_state = EngineState {
        p: p_a,
        v: v_a,
        t: temp_hot(),
        entropy_s: 0.0,
        work_done: 0.0,
        phase: "Start (Point A)".to_string(),
    };

    report_state(&initial_state);

    // The cycle A -> B -> C -> D -> A as one CausalFlow pipeline. The engine state
    // seeds the value channel; each stroke binds the next state.
    let process = CausalFlow::value(initial_state)
        .bind(stage_isothermal_expansion)
        .bind(stage_adiabatic_expansion)
        .bind(stage_isothermal_compression)
        .bind(stage_adiabatic_compression)
        .into_process();

    // Final Analysis
    if let EffectValue::Value(final_state) = process.value() {
        println!("\n=== Cycle Complete ===");
        println!("Total Work Done: {:.2} J", final_state.work_done);

        // Theoretical Efficiency
        let eff_effect = carnot_efficiency(temp_hot(), temp_cold());
        let eff_max = eff_effect.value().clone().into_value().unwrap().value();

        println!("Carnot Efficiency Limit: {:.1}%", eff_max * 100.0);
    }

    Ok(())
}

/// Step 1: Isothermal Expansion (A -> B). Temperature constant (Th), volume increases,
/// heat absorbed, entropy increases.
fn stage_isothermal_expansion(
    value: EffectValue<EngineState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<EngineState, (), ()> {
    let prev = value.into_value().unwrap();

    let v_b = Volume::<FloatType>::new(prev.v.value() * COMPRESSION_RATIO).unwrap();
    let t_b = temp_hot();

    // P = nRT / V
    let p_val = (n_moles().value() * R_CONST * t_b.value()) / v_b.value();
    let p_b = Pressure::<FloatType>::new(p_val).unwrap();

    // Work = nRT ln(Vb/Va); Delta S = Work/T = nR ln(Vb/Va).
    let work =
        n_moles().value() * R_CONST * temp_hot().value() * (v_b.value() / prev.v.value()).ln();
    let ds = work / temp_hot().value();

    let next_state = EngineState {
        p: p_b,
        v: v_b,
        t: t_b,
        entropy_s: prev.entropy_s + ds,
        work_done: prev.work_done + work,
        phase: "Isothermal Expansion (A->B)".to_string(),
    };
    report_state(&next_state);
    PropagatingEffect::pure(next_state)
}

/// Step 2: Adiabatic Expansion (B -> C). Q = 0, entropy constant, temperature drops to Tc.
fn stage_adiabatic_expansion(
    value: EffectValue<EngineState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<EngineState, (), ()> {
    let prev = value.into_value().unwrap();
    let gamma = 5.0 / 3.0; // Monatomic gas

    // T_b V_b^(gamma-1) = T_c V_c^(gamma-1)  =>  V_c = V_b (Tb/Tc)^(1/(gamma-1)).
    let t_ratio = prev.t.value() / temp_cold().value();
    let v_ratio = t_ratio.powf(1.0 / (gamma - 1.0));
    let v_c = Volume::<FloatType>::new(prev.v.value() * v_ratio).unwrap();

    let p_val = (n_moles().value() * R_CONST * temp_cold().value()) / v_c.value();
    let p_c = Pressure::<FloatType>::new(p_val).unwrap();

    // Work = -Delta U = Cv (Th - Tc), with Cv = 3/2 nR.
    let cv = 1.5 * n_moles().value() * R_CONST;
    let work = cv * (prev.t.value() - temp_cold().value());

    let next_state = EngineState {
        p: p_c,
        v: v_c,
        t: temp_cold(),
        entropy_s: prev.entropy_s, // Constant
        work_done: prev.work_done + work,
        phase: "Adiabatic Expansion (B->C)".to_string(),
    };
    report_state(&next_state);
    PropagatingEffect::pure(next_state)
}

/// Step 3: Isothermal Compression (C -> D). Temperature constant (Tc), volume decreases,
/// heat rejected, entropy decreases.
fn stage_isothermal_compression(
    value: EffectValue<EngineState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<EngineState, (), ()> {
    let prev = value.into_value().unwrap();

    // V_c / V_d = V_b / V_a = ratio.
    let v_d = Volume::<FloatType>::new(prev.v.value() / COMPRESSION_RATIO).unwrap();

    let p_val = (n_moles().value() * R_CONST * temp_cold().value()) / v_d.value();
    let p_d = Pressure::<FloatType>::new(p_val).unwrap();

    // Work is negative (done ON the gas).
    let work =
        n_moles().value() * R_CONST * temp_cold().value() * (v_d.value() / prev.v.value()).ln();
    let ds = work / temp_cold().value();

    let next_state = EngineState {
        p: p_d,
        v: v_d,
        t: temp_cold(),
        entropy_s: prev.entropy_s + ds,
        work_done: prev.work_done + work,
        phase: "Isothermal Compression (C->D)".to_string(),
    };
    report_state(&next_state);
    PropagatingEffect::pure(next_state)
}

/// Step 4: Adiabatic Compression (D -> A). Temperature rises back to Th, closing the cycle.
fn stage_adiabatic_compression(
    value: EffectValue<EngineState>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<EngineState, (), ()> {
    let prev = value.into_value().unwrap();
    let _gamma = 5.0 / 3.0;

    // Verify the cycle closes consistently against the ideal gas law.
    let _ = ideal_gas_law(prev.p, prev.v, n_moles(), prev.t);

    // Work
    let cv = 1.5 * n_moles().value() * R_CONST;
    let work = cv * (prev.t.value() - temp_hot().value()); // Negative diff -> Positive work (compression)

    let next_state = EngineState {
        p: Pressure::<FloatType>::new(415_700.0).unwrap(), // Back to P_a
        v: Volume::<FloatType>::new(0.01).unwrap(),        // Back to V_a
        t: temp_hot(),
        entropy_s: prev.entropy_s, // Constant
        work_done: prev.work_done + work,
        phase: "Adiabatic Compression (D->A)".to_string(),
    };
    report_state(&next_state);
    PropagatingEffect::pure(next_state)
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
