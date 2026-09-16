/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Bernoulli Flow Network
//!
//! Water falls through a pipe network that changes both diameter and elevation. Each segment is
//! a named stage and the network composes into one `CausalFlow` pipeline that threads the fluid
//! state through the value channel.
//!
//! Two conservation laws run the whole thing, and the run checks the second of them:
//!
//! ```text
//! continuity   A v = Q          the volumetric flow rate is the same in every segment
//! Bernoulli    P + rho v^2 / 2 + rho g h = const   along a streamline
//! ```
//!
//! The Venturi and the vertical drop are the two interesting cases. Narrowing the pipe raises
//! the velocity and so must drop the pressure; losing elevation converts potential energy into
//! pressure. Because the total head is conserved either way, the closing check is that every
//! segment carries the same total head as the reservoir.
//!
//! ## APIs Demonstrated
//! - `CausalFlow::value` and `.bind` once per pipe segment
//! - `bernoulli_pressure` with typed `Pressure`, `Speed`, `Length` and `Density` quantities

use deep_causality_algebra::Real;
use deep_causality_core::{CausalEffect, CausalFlow, PropagatingEffect, PropagatingProcess};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_physics::{
    Density, EARTH_GRAVITY_ACCELERATION, Length, PhysicsError, Pressure, Speed, bernoulli_pressure,
};

/// Small whole numbers, declared once at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// Volumetric flow rate held constant across the network, in m^3/s.
const FLOW_RATE: FloatType = const_scalar_from_float!(FloatType, 0.1);
/// Density of water, in kg/m^3.
const WATER_DENSITY: FloatType = const_scalar_from_int!(FloatType, 1000);
/// Reservoir gauge pressure, in Pa.
const RESERVOIR_PRESSURE: FloatType = const_scalar_from_int!(FloatType, 200_000);
/// Reservoir elevation, in m.
const RESERVOIR_HEIGHT: FloatType = const_scalar_from_int!(FloatType, 10);
/// Main-run pipe diameter, in m.
const MAIN_DIAMETER: FloatType = const_scalar_from_float!(FloatType, 0.2);
/// Venturi throat diameter, in m.
const THROAT_DIAMETER: FloatType = const_scalar_from_float!(FloatType, 0.1);
/// Outlet elevation, in m.
const OUTLET_HEIGHT: FloatType = const_scalar_from_int!(FloatType, 0);
/// Gravitational acceleration, at the working type.
const GRAVITY: FloatType = const_scalar_from_float!(FloatType, EARTH_GRAVITY_ACCELERATION);

/// Relative slack on the head-conservation check. Bernoulli is exact for this idealized flow,
/// so the residual is pure rounding and this only has to clear machine epsilon.
const HEAD_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// `f64` is the right precision here: four closed-form segment transitions, so the head residual
/// is a handful of machine epsilons either way. `Float106` tightens the residual and leaves
/// every reported pressure unchanged.
pub type FloatType = f64;

fn main() -> Result<(), PhysicsError> {
    print_header();

    let reservoir = FluidState {
        pressure: Pressure::new(RESERVOIR_PRESSURE)?,
        velocity: Speed::new(ZERO)?,
        height: Length::new(RESERVOIR_HEIGHT)?,
        label: "Reservoir",
    };
    let trace = vec![reservoir];

    // The network as one pipeline: each segment binds the next fluid state onto the trace.
    let process = CausalFlow::value(trace)
        .bind(segment_main_pipe)
        .bind(segment_venturi)
        .bind(segment_vertical_drop)
        .into_process();

    let trace = match process.value() {
        Some(t) => t,
        None => {
            print_failure(process.error().map(|e| format!("{e:?}")));
            return Ok(());
        }
    };

    print_trace(trace);
    print_head_check(&head_check(trace));

    Ok(())
}

/// Segment 1: the main pipe, same elevation as the reservoir.
fn segment_main_pipe(
    value: CausalEffect<Vec<FluidState>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<Vec<FluidState>, (), ()> {
    extend(value, |prev| {
        flow_segment(prev, MAIN_DIAMETER, prev.height, "Main Pipe")
    })
}

/// Segment 2: the Venturi throat. Half the diameter is a quarter of the area, so the velocity
/// quadruples and the pressure must fall to pay for it.
fn segment_venturi(
    value: CausalEffect<Vec<FluidState>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<Vec<FluidState>, (), ()> {
    extend(value, |prev| {
        flow_segment(prev, THROAT_DIAMETER, prev.height, "Venturi Throat")
    })
}

/// Segment 3: the vertical drop to the outlet. The diameter returns to the main run, so the
/// velocity returns with it and the lost elevation shows up entirely as pressure.
fn segment_vertical_drop(
    value: CausalEffect<Vec<FluidState>>,
    _state: (),
    _ctx: Option<()>,
) -> PropagatingProcess<Vec<FluidState>, (), ()> {
    extend(value, |prev| {
        let outlet = Length::new(OUTLET_HEIGHT)?;
        flow_segment(prev, MAIN_DIAMETER, outlet, "Ground Outlet")
    })
}

/// The fluid state at the exit of a segment of the given diameter and outlet elevation.
fn flow_segment(
    prev: &FluidState,
    diameter: FloatType,
    height: Length<FloatType>,
    label: &'static str,
) -> Result<FluidState, PhysicsError> {
    // Continuity: A v = Q, with A the circular cross-section.
    let radius = diameter / TWO;
    let area = FloatType::pi() * radius * radius;
    let velocity = Speed::new(FLOW_RATE / area)?;

    let pressure = bernoulli_pressure(
        &prev.pressure,
        &prev.velocity,
        &prev.height,
        &velocity,
        &height,
        &water_density()?,
    )
    .value_cloned()
    .ok_or_else(|| PhysicsError::NumericalInstability("bernoulli_pressure".into()))?;

    Ok(FluidState {
        pressure,
        velocity,
        height,
        label,
    })
}

/// Applies one segment to the last state on the trace and appends the result.
///
/// The segment body returns `Result` so it can use `?` on the quantity constructors; this
/// wrapper turns a returned error into the flow's own error channel.
fn extend(
    value: CausalEffect<Vec<FluidState>>,
    segment: impl Fn(&FluidState) -> Result<FluidState, PhysicsError>,
) -> PropagatingProcess<Vec<FluidState>, (), ()> {
    let mut trace = match value.into_value() {
        Some(t) => t,
        None => return fail("the flow carried no trace"),
    };
    let prev = match trace.last() {
        Some(p) => p,
        None => return fail("the trace is empty"),
    };
    match segment(prev) {
        Ok(next) => {
            trace.push(next);
            PropagatingEffect::pure(trace)
        }
        Err(e) => fail(format!("{e:?}")),
    }
}

fn fail<T: Default + Clone + core::fmt::Debug>(
    reason: impl Into<String>,
) -> PropagatingProcess<T, (), ()> {
    PropagatingEffect::from_error(deep_causality_core::CausalityError::new(
        deep_causality_core::CausalityErrorEnum::Custom(reason.into()),
    ))
}

fn water_density() -> Result<Density<FloatType>, PhysicsError> {
    Density::new(WATER_DENSITY)
}

/// The fluid at one point in the network.
#[derive(Debug, Clone, Default)]
struct FluidState {
    pressure: Pressure<FloatType>,
    velocity: Speed<FloatType>,
    height: Length<FloatType>,
    label: &'static str,
}

impl FluidState {
    /// Total head `P + rho v^2 / 2 + rho g h`, the quantity Bernoulli conserves.
    fn total_head(&self) -> FloatType {
        let v = self.velocity.value();
        self.pressure.value()
            + WATER_DENSITY * v * v / TWO
            + WATER_DENSITY * GRAVITY * self.height.value()
    }
}

/// Whether every segment carries the head the reservoir started with.
struct HeadCheck {
    reference: FloatType,
    worst_deviation: FloatType,
    conserved: bool,
}

fn head_check(trace: &[FluidState]) -> HeadCheck {
    let reference = trace.first().map(FluidState::total_head).unwrap_or(ZERO);

    let worst_deviation = trace
        .iter()
        .map(|s| Real::abs(s.total_head() - reference) / reference)
        .fold(ZERO, |m, v| if v > m { v } else { m });

    HeadCheck {
        reference,
        worst_deviation,
        conserved: worst_deviation < HEAD_TOLERANCE,
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Bernoulli Flow Network ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!(
        "Fluid: water at {:.0} kg/m^3, Q = {:.2} m^3/s\n",
        lower(WATER_DENSITY),
        lower(FLOW_RATE)
    );
}

fn print_trace(trace: &[FluidState]) {
    println!(
        "  {:<16} {:>12} {:>10} {:>8} {:>14}",
        "segment", "P (Pa)", "v (m/s)", "h (m)", "head (J/m^3)"
    );
    for s in trace {
        print_state(s);
    }
    println!("\n  The throat trades pressure for velocity; the drop trades elevation back into");
    println!("  pressure. The head column is what stays the same through both.");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_state(s: &FluidState) {
    println!(
        "  {:<16} {:>12.1} {:>10.3} {:>8.1} {:>14.1}",
        s.label,
        lower(s.pressure.value()),
        lower(s.velocity.value()),
        lower(s.height.value()),
        lower(s.total_head())
    );
}

fn print_head_check(c: &HeadCheck) {
    println!("\n--- Is the total head conserved? ---");
    println!("  reservoir head       = {:.4} J/m^3", lower(c.reference));
    println!("  worst relative drift = {:.2e}", lower(c.worst_deviation));
    println!(
        "  => {}",
        if c.conserved {
            "every segment carries the head the reservoir started with"
        } else {
            "HEAD IS NOT CONSERVED"
        }
    );
}

fn print_failure(error: Option<String>) {
    match error {
        Some(e) => println!("\nThe network failed: {e}"),
        None => println!("\nThe network produced no final state."),
    }
}
