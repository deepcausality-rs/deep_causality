/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![no_std]
extern crate alloc;

use alloc::collections::VecDeque;
use deep_causality_core::{ControlFlowBuilder, ControlFlowProtocol, FromProtocol, ToProtocol};

// Define the logic functions using the new explicit types.
//
// ------------------------------------------------------------------------------------------------
// ENGINEERING VALUE: Zero-Overhead Safety for Embedded Systems
//
// In safety-critical or embedded contexts (RTOS, aerospace), you cannot afford runtime overhead
// like heavy boxing, dynamic dispatch, or complex heap allocations. You also need strict guarantees
// that semantic types don't mix (e.g., don't interpret "Altitude" as "Speed").
//
// This example demonstrates:
// 1. **Zero-Sized Types (ZSTs)**: State transitions and protocols will be removed by the compiler.
// 2. **Static Dispatch**: All logic is known at compile time.
// 3. **No_Std**: Compatible with bare-metal environments.
// 4. **NewType Pattern**: `AttitudeReading` vs `ControlSurfaceUpdate` prevents logic errors.
// ------------------------------------------------------------------------------------------------

// Reads from a sensor.
fn read_attitude_sensor(request: bool) -> AttitudeReading {
    if request {
        AttitudeReading([0.1, -0.2, 0.05]) // Dummy roll, pitch, yaw
    } else {
        AttitudeReading([0.0, 0.0, 0.0])
    }
}

// Analyzes sensor data to determine required control adjustments.
fn attitude_correction_logic(reading: AttitudeReading) -> ControlSurfaceUpdate {
    // Simplified logic: invert the roll and pitch to correct.
    ControlSurfaceUpdate([-reading.0[0], -reading.0[1], 0.0])
}

// Final check to ensure control values are within safety limits.
fn check_control_limits(controls: ControlSurfaceUpdate) -> VerifiedSurfaceUpdate {
    // Clamp values to a safe range of [-1.0, 1.0]
    VerifiedSurfaceUpdate([
        controls.0[0].clamp(-1.0, 1.0),
        controls.0[1].clamp(-1.0, 1.0),
        controls.0[2].clamp(-1.0, 1.0),
    ])
}

// Main function to build and execute the graph.
pub fn main() {
    // This example will only work when compiled with `strict-zst` feature.
    // run:
    // cargo run --example control_flow_strict_zst --features strict-zst

    let mut builder = ControlFlowBuilder::<DroneControlProtocol>::new();

    // Add nodes using static function items.
    let n_sensor = builder.add_node(read_attitude_sensor);
    let n_logic = builder.add_node(attitude_correction_logic);
    let n_limits = builder.add_node(check_control_limits);

    // Connect nodes. The compiler verifies that the types flow correctly.
    builder.connect(n_sensor, n_logic);
    builder.connect(n_logic, n_limits);

    // Incorrect node connections result in compiler errors. Uncomment to trigger:
    // builder.connect(n_sensor, n_limits);
    // mismatched types: expected `NodeType<AttitudeReading, _>`, found `NodeType<ControlSurfaceUpdate, ...>`
    // builder.connect(n_sensor, n_limits);
    // mismatched types: expected `NodeType<AttitudeReading, _>`, found `NodeType<ControlSurfaceUpdate, ...>`
    // builder.connect(n_sensor, n_sensor);
    // mismatched types: expected `NodeType<AttitudeReading, _>`, found `NodeType<bool, AttitudeReading>`

    let graph = builder.build();

    // Execute with a pre-allocated queue to prevent allocation in the loop.
    let mut queue = VecDeque::with_capacity(10);
    let input = true.to_protocol(); // Start by requesting a sensor read.

    let result = graph.execute(input, n_sensor.id(), 10, &mut queue);

    // The final result should be the clamped control surface update.
    assert_eq!(
        result.unwrap(),
        DroneControlProtocol::VerifiedSurfaceUpdate(VerifiedSurfaceUpdate([-0.1, 0.2, 0.0]))
    );
}

// Define a static, fixed-size error enum for the flight system.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlightSystemError {
    SensorFailure,
    ValueOutOfRange,
    ProtocolMismatch,
}

// Define distinct wrapper types for semantically different data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttitudeReading(pub [f32; 3]);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlSurfaceUpdate(pub [f32; 3]);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerifiedSurfaceUpdate(pub [f32; 3]);

// Define a fixed-size data protocol using the new wrapper types.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DroneControlProtocol {
    AttitudeRequest(bool),
    Attitude(AttitudeReading),
    ControlUpdate(ControlSurfaceUpdate),
    VerifiedSurfaceUpdate(VerifiedSurfaceUpdate),
    Error(FlightSystemError),
}

// Implement the main CausalProtocol trait for our flight protocol.
impl ControlFlowProtocol for DroneControlProtocol {
    fn error<E: core::fmt::Debug>(_e: E) -> Self {
        // For a safety-critical system, return a pre-defined, static error code.
        Self::Error(FlightSystemError::ProtocolMismatch)
    }
}

// Implement protocol conversions for all data types used in the graph.
impl ToProtocol<DroneControlProtocol> for bool {
    fn to_protocol(self) -> DroneControlProtocol {
        DroneControlProtocol::AttitudeRequest(self)
    }
}
impl FromProtocol<DroneControlProtocol> for bool {
    type Error = FlightSystemError;
    fn from_protocol(p: DroneControlProtocol) -> Result<Self, Self::Error> {
        match p {
            DroneControlProtocol::AttitudeRequest(v) => Ok(v),
            _ => Err(FlightSystemError::ProtocolMismatch),
        }
    }
}

impl ToProtocol<DroneControlProtocol> for AttitudeReading {
    fn to_protocol(self) -> DroneControlProtocol {
        DroneControlProtocol::Attitude(self)
    }
}
impl FromProtocol<DroneControlProtocol> for AttitudeReading {
    type Error = FlightSystemError;
    fn from_protocol(p: DroneControlProtocol) -> Result<Self, Self::Error> {
        match p {
            DroneControlProtocol::Attitude(v) => Ok(v),
            _ => Err(FlightSystemError::ProtocolMismatch),
        }
    }
}

impl ToProtocol<DroneControlProtocol> for ControlSurfaceUpdate {
    fn to_protocol(self) -> DroneControlProtocol {
        DroneControlProtocol::ControlUpdate(self)
    }
}
impl FromProtocol<DroneControlProtocol> for ControlSurfaceUpdate {
    type Error = FlightSystemError;
    fn from_protocol(p: DroneControlProtocol) -> Result<Self, Self::Error> {
        match p {
            DroneControlProtocol::ControlUpdate(v) => Ok(v),
            _ => Err(FlightSystemError::ProtocolMismatch),
        }
    }
}

impl ToProtocol<DroneControlProtocol> for VerifiedSurfaceUpdate {
    fn to_protocol(self) -> DroneControlProtocol {
        DroneControlProtocol::VerifiedSurfaceUpdate(self)
    }
}

impl FromProtocol<DroneControlProtocol> for VerifiedSurfaceUpdate {
    type Error = FlightSystemError;
    fn from_protocol(p: DroneControlProtocol) -> Result<Self, Self::Error> {
        match p {
            DroneControlProtocol::VerifiedSurfaceUpdate(v) => Ok(v),
            _ => Err(FlightSystemError::ProtocolMismatch),
        }
    }
}
