/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Carnot Cycle Heat Engine
//!
//! A discrete four-stroke Carnot cycle between a hot and a cold reservoir. Each stroke is a
//! named stage and the four compose into one `CausalFlow` pipeline that threads the engine
//! state through the value channel. The engine's world, the two reservoir temperatures, the
//! expansion ratio, the starting volume and the amount and kind of working gas, is the
//! pipeline's context, which every stroke reads.
//!
//! Every state variable is computed from the previous one. The closing stroke `D -> A` is the
//! one that makes the cycle a *cycle*, so it derives its end point from the adiabat
//! `T V^(gamma-1) = const` rather than restating the starting values, and the run then checks
//! two things the cycle must satisfy if that derivation is right:
//!
//! ```text
//! the adiabat returns to the starting volume     |V_a' - V_a| / V_a  ~ 0
//! the end state recovers the gas constant        P V / (n T)         = R
//! net work equals the Carnot efficiency          W / Q_hot           = 1 - Tc/Th
//! ```
//!
//! ## APIs Demonstrated
//! - `CausalFlow::value`, `.context` and `.bind` across four stages
//! - A `Context` of `Data` contextoids as the reservoirs and gas the strokes read
//! - `ideal_gas_law` as a closure check: the R it recovers is compared against CODATA
//! - `carnot_efficiency` against the cycle's own measured efficiency

use deep_causality_algebra::Real;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalEffect, CausalFlow, PropagatingEffect, PropagatingProcess};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_physics::{
    AmountOfSubstance, MOLAR_GAS_CONSTANT, PhysicsError, Pressure, Temperature, Volume,
    carnot_efficiency, ideal_gas_law,
};

/// Small whole numbers, declared once at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// The relative error the two closure checks must come in under.
const CLOSURE_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);
/// The molar gas constant, at the working type.
const GAS_CONSTANT: FloatType = const_scalar_from_float!(FloatType, MOLAR_GAS_CONSTANT);

/// The engine's world as a context: one `Data` contextoid per world fact. The cycle has no
/// modelled position or clock, so the spatial, temporal and spacetime slots are empty.
type EngineContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the engine's world facts; `carnot_engine` adds them in this order.
/// Hot reservoir temperature (K).
const TEMP_HOT: usize = 0;
/// Cold reservoir temperature (K).
const TEMP_COLD: usize = 1;
/// Volume expansion ratio of the isothermal stroke, `V_b / V_a`.
const EXPANSION_RATIO: usize = 2;
/// Working gas, in moles.
const MOLES: usize = 3;
/// Degrees of freedom `f` of a molecule of the working gas, 3 for a monatomic ideal gas. They
/// fix both `Cv = (f/2) n R` and the adiabatic index `gamma = (f + 2) / f`.
const DEGREES_OF_FREEDOM: usize = 4;
/// Starting volume at point A (m^3).
const VOLUME_A: usize = 5;
/// How many world facts the engine holds.
const ENGINE_FACTS: usize = 6;

/// `f64` is the right precision here: the cycle is four closed-form strokes, not an iterated
/// solve, so the closure residual sits at a handful of machine epsilons either way. `Float106`
/// would tighten the residual without changing a single reported state variable.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = carnot_engine()?;
    print_header(&engine)?;

    // Point A is derived from the gas law, not asserted: P_a = n R T_h / V_a.
    let volume_a = read(&engine, VOLUME_A)?;
    let v_a = Volume::<FloatType>::new(volume_a)?;
    let p_a =
        Pressure::<FloatType>::new(gas_pressure(&engine, volume_a, temp_hot(&engine)?.value())?)?;

    let initial = EngineState {
        p: p_a,
        v: v_a,
        t: temp_hot(&engine)?,
        entropy: ZERO,
        work: ZERO,
        heat_absorbed: ZERO,
        phase: "Start (Point A)".to_string(),
    };
    let trace = vec![initial.clone()];

    // A -> B -> C -> D -> A as one pipeline. The engine rides in the context channel; each stroke
    // reads it and binds the next state onto the trace.
    let process = CausalFlow::value(trace)
        .context(engine.clone())
        .bind(stroke_isothermal_expansion)
        .bind(stroke_adiabatic_expansion)
        .bind(stroke_isothermal_compression)
        .bind(stroke_adiabatic_compression)
        .into_process();

    let trace = match process.value() {
        Some(t) => t,
        None => {
            print_failure(process.error().map(|e| format!("{e:?}")));
            return Err("the Carnot cycle failed".into());
        }
    };
    print_trace(trace);

    let final_state = trace
        .last()
        .ok_or_else(|| PhysicsError::CalculationError("the trace is empty".into()))?;
    print_closure(&closure_report(&engine, final_state, v_a)?);

    let eff_effect = carnot_efficiency(temp_hot(&engine)?, temp_cold(&engine)?);
    let eff_limit = match eff_effect.value_cloned() {
        Some(e) => e.value(),
        None => {
            return Err(PhysicsError::NumericalInstability("carnot_efficiency".into()).into());
        }
    };
    print_efficiency(
        final_state.work / final_state.heat_absorbed,
        eff_limit,
        Real::abs(final_state.work / final_state.heat_absorbed - eff_limit),
    );

    Ok(())
}

/// Stroke 1, `A -> B`: isothermal expansion at `T_h`. The gas absorbs `Q_h = n R T_h ln(r)`,
/// all of which leaves as work, and the entropy rises by `Q_h / T_h`.
fn stroke_isothermal_expansion(
    value: CausalEffect<Vec<EngineState>>,
    _state: (),
    engine: Option<EngineContext>,
) -> PropagatingProcess<Vec<EngineState>, (), EngineContext> {
    extend(value, engine, |prev, engine| {
        let t_hot = temp_hot(engine)?;
        let ratio = read(engine, EXPANSION_RATIO)?;
        let v_b = prev.v.value() * ratio;
        let work = read(engine, MOLES)? * GAS_CONSTANT * t_hot.value() * Real::ln(ratio);

        Ok(EngineState {
            p: Pressure::new(gas_pressure(engine, v_b, t_hot.value())?)?,
            v: Volume::new(v_b)?,
            t: t_hot,
            entropy: prev.entropy + work / t_hot.value(),
            work: prev.work + work,
            // Isothermal: no internal-energy change, so every joule of work came from the
            // hot reservoir. This is the cycle's Q_hot.
            heat_absorbed: prev.heat_absorbed + work,
            phase: "Isothermal Expansion (A->B)".to_string(),
        })
    })
}

/// Stroke 2, `B -> C`: adiabatic expansion. `Q = 0`, so entropy holds and the work comes out of
/// internal energy: `W = Cv (T_h - T_c)`. The end volume follows `T V^(gamma-1) = const`.
fn stroke_adiabatic_expansion(
    value: CausalEffect<Vec<EngineState>>,
    _state: (),
    engine: Option<EngineContext>,
) -> PropagatingProcess<Vec<EngineState>, (), EngineContext> {
    extend(value, engine, |prev, engine| {
        let t_cold = temp_cold(engine)?;
        let v_c = adiabatic_volume(engine, prev.v.value(), prev.t.value(), t_cold.value())?;
        let work = heat_capacity_v(engine)? * (prev.t.value() - t_cold.value());

        Ok(EngineState {
            p: Pressure::new(gas_pressure(engine, v_c, t_cold.value())?)?,
            v: Volume::new(v_c)?,
            t: t_cold,
            entropy: prev.entropy,
            work: prev.work + work,
            heat_absorbed: prev.heat_absorbed,
            phase: "Adiabatic Expansion (B->C)".to_string(),
        })
    })
}

/// Stroke 3, `C -> D`: isothermal compression at `T_c`. The compression ratio must match the
/// expansion ratio of stroke 1, which is what puts `D` on the adiabat back to `A`.
fn stroke_isothermal_compression(
    value: CausalEffect<Vec<EngineState>>,
    _state: (),
    engine: Option<EngineContext>,
) -> PropagatingProcess<Vec<EngineState>, (), EngineContext> {
    extend(value, engine, |prev, engine| {
        let t_cold = temp_cold(engine)?;
        let ratio = read(engine, EXPANSION_RATIO)?;
        let v_d = prev.v.value() / ratio;
        // Work is negative here: the surroundings do work on the gas and heat is rejected.
        let work = read(engine, MOLES)? * GAS_CONSTANT * t_cold.value() * Real::ln(ONE / ratio);

        Ok(EngineState {
            p: Pressure::new(gas_pressure(engine, v_d, t_cold.value())?)?,
            v: Volume::new(v_d)?,
            t: t_cold,
            entropy: prev.entropy + work / t_cold.value(),
            work: prev.work + work,
            heat_absorbed: prev.heat_absorbed,
            phase: "Isothermal Compression (C->D)".to_string(),
        })
    })
}

/// Stroke 4, `D -> A`: adiabatic compression back to the hot reservoir. The end volume is
/// derived from the same adiabat as stroke 2, so whether the cycle closes is an outcome of the
/// physics rather than a restatement of the starting values.
fn stroke_adiabatic_compression(
    value: CausalEffect<Vec<EngineState>>,
    _state: (),
    engine: Option<EngineContext>,
) -> PropagatingProcess<Vec<EngineState>, (), EngineContext> {
    extend(value, engine, |prev, engine| {
        let t_hot = temp_hot(engine)?;
        let v_a = adiabatic_volume(engine, prev.v.value(), prev.t.value(), t_hot.value())?;
        let work = heat_capacity_v(engine)? * (prev.t.value() - t_hot.value());

        Ok(EngineState {
            p: Pressure::new(gas_pressure(engine, v_a, t_hot.value())?)?,
            v: Volume::new(v_a)?,
            t: t_hot,
            entropy: prev.entropy,
            work: prev.work + work,
            heat_absorbed: prev.heat_absorbed,
            phase: "Adiabatic Compression (D->A)".to_string(),
        })
    })
}

/// Applies one stroke to the last state on the trace and appends the result.
///
/// The stroke body reads the engine from the context and returns `Result`, which is what lets it
/// use `?` on the quantity constructors; this wrapper turns a returned error into the flow's own
/// error channel and hands the engine on to the next stroke.
fn extend(
    value: CausalEffect<Vec<EngineState>>,
    engine: Option<EngineContext>,
    stroke: impl Fn(&EngineState, &EngineContext) -> Result<EngineState, PhysicsError>,
) -> PropagatingProcess<Vec<EngineState>, (), EngineContext> {
    let mut trace = match value.into_value() {
        Some(t) => t,
        None => return fail("the flow carried no trace"),
    };
    let engine = match engine {
        Some(e) => e,
        None => return fail("the flow carried no engine"),
    };
    let prev = match trace.last() {
        Some(p) => p,
        None => return fail("the trace is empty"),
    };
    match stroke(prev, &engine) {
        Ok(next) => {
            trace.push(next);
            PropagatingProcess::with_state(PropagatingEffect::pure(trace), (), Some(engine))
        }
        Err(e) => fail(format!("{e:?}")),
    }
}

fn fail<T: Default + Clone + core::fmt::Debug, C: Clone + core::fmt::Debug>(
    reason: impl Into<String>,
) -> PropagatingProcess<T, (), C> {
    PropagatingProcess::from_error(deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::Custom(reason.into()),
    ))
}

/// Builds the engine's world: each fact as a `Data` contextoid at its node index.
fn carnot_engine() -> Result<EngineContext, ContextIndexError> {
    let mut facts = [ZERO; ENGINE_FACTS];
    facts[TEMP_HOT] = const_scalar_from_int!(FloatType, 500);
    facts[TEMP_COLD] = const_scalar_from_int!(FloatType, 300);
    facts[EXPANSION_RATIO] = const_scalar_from_int!(FloatType, 2);
    facts[MOLES] = const_scalar_from_int!(FloatType, 1);
    facts[DEGREES_OF_FREEDOM] = const_scalar_from_int!(FloatType, 3);
    facts[VOLUME_A] = const_scalar_from_float!(FloatType, 0.01);

    let mut engine = Context::with_capacity(1, "carnot engine", ENGINE_FACTS);
    for (id, value) in (1..).zip(facts) {
        engine.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(engine)
}

/// Reads one world fact out of the engine, or the error naming the node that holds none.
fn read(engine: &EngineContext, index: usize) -> Result<FloatType, PhysicsError> {
    engine
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            PhysicsError::CalculationError(format!("the engine holds no Datoid at node {index}"))
        })
}

// --- The cycle's own laws, over the engine's world and constants at the working type ---

/// Heat capacity at constant volume of the working gas, `Cv = (f/2) n R`.
fn heat_capacity_v(engine: &EngineContext) -> Result<FloatType, PhysicsError> {
    Ok(read(engine, DEGREES_OF_FREEDOM)? / TWO * read(engine, MOLES)? * GAS_CONSTANT)
}
fn temp_hot(engine: &EngineContext) -> Result<Temperature<FloatType>, PhysicsError> {
    Ok(Temperature::new_unchecked(read(engine, TEMP_HOT)?))
}
fn temp_cold(engine: &EngineContext) -> Result<Temperature<FloatType>, PhysicsError> {
    Ok(Temperature::new_unchecked(read(engine, TEMP_COLD)?))
}

/// `P = n R T / V`, the ideal gas law solved for pressure.
fn gas_pressure(
    engine: &EngineContext,
    volume: FloatType,
    temp: FloatType,
) -> Result<FloatType, PhysicsError> {
    Ok(read(engine, MOLES)? * GAS_CONSTANT * temp / volume)
}

/// The volume an adiabat carries `(v, from)` to at temperature `to`, from
/// `T V^(gamma-1) = const`, so `V' = V (T/T')^(1/(gamma-1))`.
///
/// The adiabatic index is `gamma = (f + 2) / f` for the working gas's `f` degrees of freedom,
/// computed as the fraction it is: for `f = 3` that is `5/3`, which has no exact decimal literal,
/// so dividing two exact whole numbers at the working type keeps it as accurate as that type
/// allows.
fn adiabatic_volume(
    engine: &EngineContext,
    volume: FloatType,
    from: FloatType,
    to: FloatType,
) -> Result<FloatType, PhysicsError> {
    let f = read(engine, DEGREES_OF_FREEDOM)?;
    let gamma = (f + TWO) / f;
    let exponent = ONE / (gamma - ONE);
    Ok(volume * Real::powf(from / to, exponent))
}

/// What the closing stroke has to satisfy for the cycle to be a cycle.
struct ClosureReport {
    volume_error: FloatType,
    recovered_r: FloatType,
    entropy: FloatType,
    closed: bool,
}

/// Measures the final state against point A and against the gas law.
fn closure_report(
    engine: &EngineContext,
    final_state: &EngineState,
    v_a: Volume<FloatType>,
) -> Result<ClosureReport, PhysicsError> {
    let volume_error = Real::abs(final_state.v.value() - v_a.value()) / v_a.value();

    // `ideal_gas_law` recovers the gas constant, P V / (n T). The closing stroke derived its
    // end state from the adiabat alone and never used R, so recovering R back out of that
    // state is an independent check on it.
    let moles = AmountOfSubstance::<FloatType>::new(read(engine, MOLES)?)?;
    let r_effect = ideal_gas_law(final_state.p, final_state.v, moles, final_state.t);
    let recovered_r = match r_effect.value_cloned() {
        Some(r) => r.value(),
        None => return Err(PhysicsError::NumericalInstability("ideal_gas_law".into())),
    };

    Ok(ClosureReport {
        volume_error,
        recovered_r,
        entropy: final_state.entropy,
        closed: volume_error < CLOSURE_TOLERANCE
            && Real::abs(recovered_r - GAS_CONSTANT) / GAS_CONSTANT < CLOSURE_TOLERANCE
            && Real::abs(final_state.entropy) < CLOSURE_TOLERANCE,
    })
}

/// One thermodynamic state of the working gas, plus the path quantities accumulated to reach it.
#[derive(Debug, Clone)]
struct EngineState {
    p: Pressure<FloatType>,
    v: Volume<FloatType>,
    t: Temperature<FloatType>,
    entropy: FloatType,
    work: FloatType,
    heat_absorbed: FloatType,
    phase: String,
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header(engine: &EngineContext) -> Result<(), PhysicsError> {
    println!("=== Carnot Heat Engine ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!(
        "Reservoirs: T_hot = {:.0} K, T_cold = {:.0} K, expansion ratio {:.0}\n",
        lower(read(engine, TEMP_HOT)?),
        lower(read(engine, TEMP_COLD)?),
        lower(read(engine, EXPANSION_RATIO)?)
    );
    Ok(())
}

fn print_trace(trace: &[EngineState]) {
    println!(
        "{:<30} | {:>10} | {:>9} | {:>7} | {:>8} | {:>9}",
        "Stroke", "P (Pa)", "V (m^3)", "T (K)", "S (J/K)", "W (J)"
    );
    for s in trace {
        print_state(s);
    }
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_state(s: &EngineState) {
    println!(
        "{:<30} | {:>10.1} | {:>9.5} | {:>7.1} | {:>+8.3} | {:>+9.2}",
        s.phase,
        lower(s.p.value()),
        lower(s.v.value()),
        lower(s.t.value()),
        lower(s.entropy),
        lower(s.work)
    );
}

fn print_closure(r: &ClosureReport) {
    println!("\n--- Does the cycle close? ---");
    println!(
        "  |V_a' - V_a| / V_a      = {:.2e}   (the closing adiabat returns to the start volume)",
        lower(r.volume_error)
    );
    println!(
        "  P V / (n T)             = {:.9}  (recovers R; CODATA is {MOLAR_GAS_CONSTANT})",
        lower(r.recovered_r)
    );
    println!(
        "  net entropy change      = {:.2e}   (reversible cycle, so zero)",
        lower(r.entropy)
    );
    println!(
        "  => {}",
        if r.closed {
            "the cycle closes to within the tolerance"
        } else {
            "the cycle does NOT close"
        }
    );
}

fn print_efficiency(measured: FloatType, limit: FloatType, error: FloatType) {
    println!("\n--- Efficiency ---");
    println!("  measured W / Q_hot      = {:.6}", lower(measured));
    println!("  Carnot limit 1 - Tc/Th  = {:.6}", lower(limit));
    println!("  difference              = {:.2e}", lower(error));
}

fn print_failure(error: Option<String>) {
    match error {
        Some(e) => println!("\nThe cycle failed: {e}"),
        None => println!("\nThe cycle produced no final state."),
    }
}
