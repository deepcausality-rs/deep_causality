/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Bernoulli Flow Network
//!
//! Water falls through a pipe network that changes both diameter and elevation. Each segment is
//! a named stage and the network composes into one `CausalFlow` pipeline that threads the fluid
//! state through the value channel. The network itself, its reservoir, flow rate, water
//! density, pipe diameters and outlet height, is the pipeline's context: the reservoir state the
//! trace starts from is read from it, and every segment reads its geometry from there.
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
//! - `CausalFlow::value`, `.context` and `.bind` once per pipe segment
//! - A `Context` of `Data` contextoids as the network the segments read
//! - `bernoulli_pressure` with typed `Pressure`, `Speed`, `Length` and `Density` quantities

use deep_causality_algebra::Real;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalEffect, CausalFlow, PropagatingEffect, PropagatingProcess};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lower};
use deep_causality_physics::{
    Density, EARTH_GRAVITY_ACCELERATION, Length, PhysicsError, Pressure, Speed, bernoulli_pressure,
};

/// Small whole numbers, declared once at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// Gravitational acceleration, at the working type. `bernoulli_pressure` applies the same
/// constant internally, so the head check uses it too.
const GRAVITY: FloatType = const_scalar_from_float!(FloatType, EARTH_GRAVITY_ACCELERATION);

/// The pipe network as a context: one `Data` contextoid per world fact. The network has no
/// modelled position or clock, so the spatial, temporal and spacetime slots are empty.
type NetworkContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node indices of the network's world facts; `pipe_network` adds them in this order.
/// Reservoir gauge pressure, in Pa.
const RESERVOIR_PRESSURE: usize = 0;
/// Reservoir elevation, in m.
const RESERVOIR_HEIGHT: usize = 1;
/// Volumetric flow rate held constant across the network, in m^3/s.
const FLOW_RATE: usize = 2;
/// Density of water, in kg/m^3.
const WATER_DENSITY: usize = 3;
/// Main-run pipe diameter, in m.
const MAIN_DIAMETER: usize = 4;
/// Venturi throat diameter, in m.
const THROAT_DIAMETER: usize = 5;
/// Outlet elevation, in m.
const OUTLET_HEIGHT: usize = 6;
/// How many world facts the network holds.
const NETWORK_FACTS: usize = 7;

/// Relative slack on the head-conservation check. Bernoulli is exact for this idealized flow,
/// so the residual is pure rounding and this only has to clear machine epsilon.
const HEAD_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// `f64` is the right precision here: four closed-form segment transitions, so the head residual
/// is a handful of machine epsilons either way. `Float106` tightens the residual and leaves
/// every reported pressure unchanged.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = pipe_network()?;
    print_header(&network)?;

    let reservoir = FluidState {
        pressure: Pressure::new(read(&network, RESERVOIR_PRESSURE)?)?,
        velocity: Speed::new(ZERO)?,
        height: Length::new(read(&network, RESERVOIR_HEIGHT)?)?,
        label: "Reservoir",
    };
    let trace = vec![reservoir];

    // The network as one pipeline: the network rides in the context channel, and each segment
    // reads its geometry from it and binds the next fluid state onto the trace.
    let process = CausalFlow::value(trace)
        .context(network.clone())
        .bind(segment_main_pipe)
        .bind(segment_venturi)
        .bind(segment_vertical_drop)
        .into_process();

    let trace = match process.value() {
        Some(t) => t,
        None => {
            print_failure(process.error().map(|e| format!("{e:?}")));
            return Err("the flow network failed".into());
        }
    };

    print_trace(trace, &network)?;
    print_head_check(&head_check(trace, &network)?);

    Ok(())
}

/// Segment 1: the main pipe, same elevation as the reservoir.
fn segment_main_pipe(
    value: CausalEffect<Vec<FluidState>>,
    _state: (),
    network: Option<NetworkContext>,
) -> PropagatingProcess<Vec<FluidState>, (), NetworkContext> {
    extend(value, network, |prev, network| {
        let diameter = read(network, MAIN_DIAMETER)?;
        flow_segment(prev, network, diameter, prev.height, "Main Pipe")
    })
}

/// Segment 2: the Venturi throat. Half the diameter is a quarter of the area, so the velocity
/// quadruples and the pressure must fall to pay for it.
fn segment_venturi(
    value: CausalEffect<Vec<FluidState>>,
    _state: (),
    network: Option<NetworkContext>,
) -> PropagatingProcess<Vec<FluidState>, (), NetworkContext> {
    extend(value, network, |prev, network| {
        let diameter = read(network, THROAT_DIAMETER)?;
        flow_segment(prev, network, diameter, prev.height, "Venturi Throat")
    })
}

/// Segment 3: the vertical drop to the outlet. The diameter returns to the main run, so the
/// velocity returns with it and the lost elevation shows up entirely as pressure.
fn segment_vertical_drop(
    value: CausalEffect<Vec<FluidState>>,
    _state: (),
    network: Option<NetworkContext>,
) -> PropagatingProcess<Vec<FluidState>, (), NetworkContext> {
    extend(value, network, |prev, network| {
        let outlet = Length::new(read(network, OUTLET_HEIGHT)?)?;
        let diameter = read(network, MAIN_DIAMETER)?;
        flow_segment(prev, network, diameter, outlet, "Ground Outlet")
    })
}

/// The fluid state at the exit of a segment of the given diameter and outlet elevation, for the
/// flow rate and fluid the network carries.
fn flow_segment(
    prev: &FluidState,
    network: &NetworkContext,
    diameter: FloatType,
    height: Length<FloatType>,
    label: &'static str,
) -> Result<FluidState, PhysicsError> {
    // Continuity: A v = Q, with A the circular cross-section.
    let radius = diameter / TWO;
    let area = FloatType::pi() * radius * radius;
    let velocity = Speed::new(read(network, FLOW_RATE)? / area)?;

    let pressure = bernoulli_pressure(
        &prev.pressure,
        &prev.velocity,
        &prev.height,
        &velocity,
        &height,
        &Density::new(read(network, WATER_DENSITY)?)?,
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
/// The segment body reads the network from the context and returns `Result`, so it can use `?`
/// on the quantity constructors; this wrapper turns a returned error into the flow's own error
/// channel and hands the network on to the next segment.
fn extend(
    value: CausalEffect<Vec<FluidState>>,
    network: Option<NetworkContext>,
    segment: impl Fn(&FluidState, &NetworkContext) -> Result<FluidState, PhysicsError>,
) -> PropagatingProcess<Vec<FluidState>, (), NetworkContext> {
    let mut trace = match value.into_value() {
        Some(t) => t,
        None => return fail("the flow carried no trace"),
    };
    let network = match network {
        Some(n) => n,
        None => return fail("the flow carried no network"),
    };
    let prev = match trace.last() {
        Some(p) => p,
        None => return fail("the trace is empty"),
    };
    match segment(prev, &network) {
        Ok(next) => {
            trace.push(next);
            PropagatingProcess::with_state(PropagatingEffect::pure(trace), (), Some(network))
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

/// Builds the pipe network: each world fact as a `Data` contextoid at its node index.
fn pipe_network() -> Result<NetworkContext, ContextIndexError> {
    let mut facts = [ZERO; NETWORK_FACTS];
    facts[RESERVOIR_PRESSURE] = const_scalar_from_int!(FloatType, 200_000);
    facts[RESERVOIR_HEIGHT] = const_scalar_from_int!(FloatType, 10);
    facts[FLOW_RATE] = const_scalar_from_float!(FloatType, 0.1);
    facts[WATER_DENSITY] = const_scalar_from_int!(FloatType, 1000);
    facts[MAIN_DIAMETER] = const_scalar_from_float!(FloatType, 0.2);
    facts[THROAT_DIAMETER] = const_scalar_from_float!(FloatType, 0.1);
    facts[OUTLET_HEIGHT] = const_scalar_from_int!(FloatType, 0);

    let mut network = Context::with_capacity(1, "pipe network", NETWORK_FACTS);
    for (id, value) in (1..).zip(facts) {
        network.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(network)
}

/// Reads one world fact out of the network, or the error naming the node that holds none.
fn read(network: &NetworkContext, index: usize) -> Result<FloatType, PhysicsError> {
    network
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            PhysicsError::CalculationError(format!(
                "the pipe network holds no Datoid at node {index}"
            ))
        })
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
    /// Total head `P + rho v^2 / 2 + rho g h`, the quantity Bernoulli conserves, for the fluid
    /// the network carries.
    fn total_head(&self, network: &NetworkContext) -> Result<FloatType, PhysicsError> {
        let density = read(network, WATER_DENSITY)?;
        let v = self.velocity.value();
        Ok(self.pressure.value() + density * v * v / TWO + density * GRAVITY * self.height.value())
    }
}

/// Whether every segment carries the head the reservoir started with.
struct HeadCheck {
    reference: FloatType,
    worst_deviation: FloatType,
    conserved: bool,
}

fn head_check(trace: &[FluidState], network: &NetworkContext) -> Result<HeadCheck, PhysicsError> {
    let reference = match trace.first() {
        Some(reservoir) => reservoir.total_head(network)?,
        None => return Err(PhysicsError::CalculationError("the trace is empty".into())),
    };

    let worst_deviation = trace.iter().try_fold(ZERO, |m, s| {
        let v = Real::abs(s.total_head(network)? - reference) / reference;
        Ok::<_, PhysicsError>(if v > m { v } else { m })
    })?;

    Ok(HeadCheck {
        reference,
        worst_deviation,
        conserved: worst_deviation < HEAD_TOLERANCE,
    })
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header(network: &NetworkContext) -> Result<(), PhysicsError> {
    println!("=== Bernoulli Flow Network ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!(
        "Fluid: water at {:.0} kg/m^3, Q = {:.2} m^3/s\n",
        lower(read(network, WATER_DENSITY)?),
        lower(read(network, FLOW_RATE)?)
    );
    Ok(())
}

fn print_trace(trace: &[FluidState], network: &NetworkContext) -> Result<(), PhysicsError> {
    println!(
        "  {:<16} {:>12} {:>10} {:>8} {:>14}",
        "segment", "P (Pa)", "v (m/s)", "h (m)", "head (J/m^3)"
    );
    for s in trace {
        print_state(s, network)?;
    }
    println!("\n  The throat trades pressure for velocity; the drop trades elevation back into");
    println!("  pressure. The head column is what stays the same through both.");
    Ok(())
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_state(s: &FluidState, network: &NetworkContext) -> Result<(), PhysicsError> {
    println!(
        "  {:<16} {:>12.1} {:>10.3} {:>8.1} {:>14.1}",
        s.label,
        lower(s.pressure.value()),
        lower(s.velocity.value()),
        lower(s.height.value()),
        lower(s.total_head(network)?)
    );
    Ok(())
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
