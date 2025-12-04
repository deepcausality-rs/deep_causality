/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{HopfState, MultiVector};
use deep_causality_num::Complex;
use std::f64::consts;

// -----------------------------------------------------------------------------------------
// ENGINEERING VALUE:
// Visualizing and analyzing high-dimensional data (like 4D Quaternion states or Quantum Spinors)
// is difficult. The Hopf Fibration provides a map from the 3-sphere (S3) to the 2-sphere (S2),
// allowing us to project complex states onto a visible surface (the Bloch Sphere).
//
// This example demonstrates:
// 1. Encoding a Quantum State (Spinor) in a MultiVector.
// 2. Applying a "Fiber Shift" (Global Phase change), which is invisible in the projection.
// 3. Projecting the state to 3D space for analysis.
//
// This is valuable for Topological Data Analysis (TDA) and debugging quantum control systems.
// -----------------------------------------------------------------------------------------

fn main() {
    println!("--- HOPF FIBRATION DEMO ---");

    // 1. Create a Quantum State (Spinor)
    // Alpha=1/sqrt(2), Beta=1/sqrt(2) -> State |+> (X-axis)
    let alpha = Complex::new(consts::FRAC_1_SQRT_2, 0.0);
    let beta = Complex::new(consts::FRAC_1_SQRT_2, 0.0);

    let state = HopfState::from_spinor(alpha, beta);
    println!("Initial  HopfState: {:?}", &state);

    // 2. Project to S2 (The Bloch Sphere)
    let bloch_vector = state.project();
    println!("Bloch Vector (S2): {:?}", bloch_vector.data());
    // Should point along X (e1) or Y depending on mapping convention

    // 3. Apply Fiber Shift (Global Phase)
    // Rotate by 90 degrees (Pi/2) in the fiber
    let twisted_state = state.fiber_shift(std::f64::consts::PI / 2.0);

    // 4. Verify the "Magic": The State changed, but the Projection didn't.
    let new_bloch = twisted_state.project();

    // Check Distance in S3 (State Space)
    let dist_s3 = (state.as_inner() - twisted_state.as_inner()).squared_magnitude();
    println!("Change in State Space (S3): {:.4} (State Moved)", dist_s3);

    // Check Distance in S2 (Shadow Space)
    let dist_s2 = (bloch_vector - new_bloch).squared_magnitude();
    println!("Change in Projection (S2):  {:.4} (Shadow Stayed)", dist_s2);
}
