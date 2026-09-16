/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    Bifunctor, HKT2Unbound, MonoidalMerge, Profunctor, ResultUnboundWitness, Tuple3Witness,
};
use deep_causality_num::{const_scalar_from_int, lift, lift_i32};
use std::fmt::Debug;

/// The working scalar. Calibrated readings and fused states carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const THREE: FloatType = const_scalar_from_int!(FloatType, 3);
const TWENTY: FloatType = const_scalar_from_int!(FloatType, 20);

fn main() {
    print_header();

    // ------------------------------------------------------------------------
    // Step 1: Dual-Track Processing (Bifunctor)
    //
    // ENGINEERING VALUE:
    // In complex systems, operations can fail. Standard error handling often
    // forces you to stop processing or handle errors separately.
    //
    // The Bifunctor pattern allows you to build a single pipeline that processes
    // BOTH the "Happy Path" (Data) and the "Failure Path" (Error) simultaneously.
    // This ensures that even if a sensor fails, the error is normalized and
    // formatted correctly without breaking the flow or requiring `if/else` spaghetti.
    // ------------------------------------------------------------------------
    let raw_success: Result<RawSensorData, &str> = Ok(RawSensorData {
        id: 1,
        value: 100,
        noise_level: 5,
    });
    let raw_failure: Result<RawSensorData, &str> = Err("Sensor Timeout");

    // Transformation Logic
    let calibrate = |raw: RawSensorData| CalibratedData {
        id: raw.id,
        value: lift_i32::<FloatType>(raw.value) * lift::<FloatType>(0.98), // Calibration factor
    };
    let format_err = |e: &str| SystemError {
        code: 500,
        message: e.to_string(),
    };

    // Apply the Dual-Track Map (Bifunctor)
    // If Ok: Calibrate the data.
    // If Err: Format the error message.
    let processed_ok = ResultUnboundWitness::bimap(raw_success, calibrate, format_err);
    let processed_err = ResultUnboundWitness::bimap(raw_failure, calibrate, format_err);

    print_dual_track(&processed_ok, &processed_err);

    // ------------------------------------------------------------------------
    // Step 2: The Adapter Pattern (Profunctor)
    //
    // ENGINEERING VALUE:
    // You often have highly optimized core algorithms (like signal processing or AI models)
    // that work on primitive types (f64, Tensor). However, your real-world data comes
    // in messy domain structs (RawSensorData).
    //
    // The Profunctor pattern creates a reusable "Adapter Layer". It wraps your
    // core algorithm, automatically unpacking the input data before it hits the core,
    // and repackaging the output result afterwards.
    //
    // This keeps your core logic pure and reusable, while the adapter handles the
    // dirty work of data integration.
    // ------------------------------------------------------------------------
    // The Core Algorithm: A pure signal amplifier (f64 -> f64).
    // It knows nothing about "Sensors" or "IDs".
    let amplifier = DataProcessor(Box::new(|signal: FloatType| signal * TWO));

    // The Adapter:
    // 1. Pre-processing (Input Adapter): Extracts f64 from RawSensorData.
    // 2. Post-processing (Output Adapter): Wraps the resulting f64 into CalibratedData.
    let sensor_pipeline = ProcessorWitness::dimap(
        amplifier,
        |raw: RawSensorData| lift_i32::<FloatType>(raw.value), // Input Adapter
        |amplified: FloatType| CalibratedData {
            // Output Adapter
            id: 0, // Dummy ID for example
            value: amplified,
        },
    );

    let input = RawSensorData {
        id: 42,
        value: 10,
        noise_level: 1,
    };

    // Execute the adapted pipeline
    let output = (sensor_pipeline.0)(input);
    print_pipeline(&output);
    assert_eq!(output.value, TWENTY); // 10.0 * 2.0

    // ------------------------------------------------------------------------
    // Step 3: Multi-Stream Fusion (MonoidalMerge)
    //
    // ENGINEERING VALUE:
    // In avionics and robotics, you rarely rely on a single sensor. You need to
    // "fuse" data from multiple sources (e.g., 3-axis accelerometer) to get the truth.
    //
    // The MonoidalMerge pattern provides a structured way to merge multiple independent
    // data streams. It handles the complexity of combining inputs (like zipping streams)
    // so you can focus purely on the fusion logic (how to combine X, Y, and Z).
    //
    // Below, we link Step 2 and Step 3: We take raw data, run it through our
    // Adapter Pipeline (Step 2) to calibrate it, and then Fuse it (Step 3).
    // ------------------------------------------------------------------------
    // Raw Sensor X data stream (needs calibration)
    let raw_x_stream = (
        RawSensorData {
            id: 1,
            value: 5,
            noise_level: 2,
        },
        RawSensorData {
            id: 1,
            value: 6,
            noise_level: 1,
        },
        RawSensorData {
            id: 1,
            value: 7,
            noise_level: 3,
        },
    );

    // 1. CALIBRATION PHASE
    // Apply the Adapter Pipeline from Step 2 to the entire X-stream.
    let calibrate_fn = &sensor_pipeline.0;
    let calibrated_x_stream = (
        calibrate_fn(raw_x_stream.0),
        calibrate_fn(raw_x_stream.1),
        calibrate_fn(raw_x_stream.2),
    );

    print_calibrated(&calibrated_x_stream);

    // Extract values for fusion (f64)
    let sensor_x = (
        calibrated_x_stream.0.value,
        calibrated_x_stream.1.value,
        calibrated_x_stream.2.value,
    );

    // Time series Y and Z, already calibrated.
    let sensor_y = (TWO, lift::<FloatType>(2.1), lift::<FloatType>(2.2));
    let sensor_z = (THREE, lift::<FloatType>(3.1), lift::<FloatType>(3.2));

    // 2. FUSION PHASE
    // Merge the independent streams (X, Y, Z) into a single coherent state.

    // First, merge X and Y streams.
    let partial_xy = Tuple3Witness::merge(sensor_x, sensor_y, |x, y| (x, y));

    // Then, merge the result with Z to create the final FusedState.
    let fused_states = Tuple3Witness::merge(partial_xy, sensor_z, |(x, y), z| {
        FusedState {
            x,
            y,
            z,
            confidence: lift(0.95), // Calculated confidence score
        }
    });

    print_fused(&fused_states);
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Cybernetic Sensor Fusion ===\n");
    println!("--- 1. Dual-Track Processing: Calibrate Data OR Format Error ---");
}

fn print_dual_track<A: Debug, B: Debug>(ok: &A, err: &B) {
    println!("Processed OK: {ok:?}");
    println!("Processed Err: {err:?}");
    println!("\n--- 2. Adapter Pattern: Reusing Core Algorithms ---");
}

fn print_pipeline<T: Debug>(output: &T) {
    println!("Pipeline Output: {output:?}");
    println!("\n--- 3. Sensor Fusion: Calibrate & Merge Streams ---");
}

fn print_calibrated<T: Debug>(stream: &T) {
    println!("Calibrated Sensor X: {stream:?}");
}

fn print_fused<T: Debug>(fused: &T) {
    println!("Fused States: {fused:#?}");
}

// ============================================================================
// Domain Types: Cybernetic Sensor System
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
struct RawSensorData {
    id: u32,
    value: i32,
    noise_level: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct CalibratedData {
    id: u32,
    value: FloatType,
}

#[derive(Debug, Clone, PartialEq)]
struct SystemError {
    code: u16,
    message: String,
}

#[derive(Debug, Clone, PartialEq)]
struct FusedState {
    x: FloatType,
    y: FloatType,
    z: FloatType,
    confidence: FloatType,
}

// ============================================================================
// Profunctor: Data Adapter
// ============================================================================

// A generic data processor: Input -> Output
struct DataProcessor<I, O>(Box<dyn Fn(I) -> O>);

struct ProcessorWitness;
impl HKT2Unbound for ProcessorWitness {
    type Type<A, B> = DataProcessor<A, B>;
}

impl Profunctor<ProcessorWitness> for ProcessorWitness {
    fn dimap<A, B, C, D, F1, F2>(
        pab: DataProcessor<A, B>,
        f_pre: F1,
        f_post: F2,
    ) -> DataProcessor<C, D>
    where
        A: 'static,
        B: 'static,
        C: 'static,
        D: 'static,
        F1: FnMut(C) -> A + 'static,
        F2: FnMut(B) -> D + 'static,
    {
        let inner = pab.0;
        let f_pre = std::cell::RefCell::new(f_pre);
        let f_post = std::cell::RefCell::new(f_post);

        DataProcessor(Box::new(move |c| {
            let a = (f_pre.borrow_mut())(c);
            let b = inner(a);
            (f_post.borrow_mut())(b)
        }))
    }
}
