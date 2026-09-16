/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the Hopf fibration walk.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::FIBER_STEPS;
use deep_causality_num::lower;
use deep_causality_tensor::CausalTensor;

pub fn print_header() {
    println!("=== The Hopf fibration: a qubit, its shadow, and the phase between them ===\n");
    println!("Precision:   {}", core::any::type_name::<FloatType>());
    println!("State space: S^3, the unit rotors of Cl(3)");
    println!("Shadow:      S^2, the Bloch sphere\n");
}

/// Where the six cardinal states land, against the convention they are defined by.
pub fn print_cardinal(states: &[(&'static str, [FloatType; 3]); 6]) {
    println!("The six cardinal states and their Bloch vectors");
    println!("  state       x        y        z      lands on");

    for (name, [x, y, z]) in states {
        println!(
            "  {:<6}  {:>+6.3}   {:>+6.3}   {:>+6.3}    {}",
            name,
            lower(*x),
            lower(*y),
            lower(*z),
            semi_axis(*x, *y, *z)
        );
    }

    println!();
    println!("  |0> and |1> hold the poles, |+> and |-> the x axis, |+i> and |-i> the y axis, so");
    println!("  all three axes are pinned and no two of them can be quietly exchanged.\n");
}

/// One row per step around the fiber, with both displacements side by side.
pub fn print_fiber_walk(
    angles: &CausalTensor<FloatType>,
    displaced: &CausalTensor<(FloatType, FloatType)>,
) {
    println!("Walking the fiber from |+>, one complete circuit in {FIBER_STEPS} steps");
    println!("  theta      state moved on S^3   shadow moved on S^2");

    for (&angle, &(on_s3, on_s2)) in angles.as_slice().iter().zip(displaced.as_slice()) {
        println!(
            "  {:>6.3} rad       {:>10.6}          {:>10.6}",
            lower(angle),
            lower(on_s3),
            lower(on_s2)
        );
    }
    println!();
}

/// What the full turn came to.
pub fn print_verdict(shadow: [FloatType; 3], state_moved: FloatType, shadow_moved: FloatType) {
    let [x, y, z] = shadow;

    println!("Over the whole fiber");
    println!("  the state moved by up to    {:>12.6}", lower(state_moved));
    println!(
        "  its shadow moved by up to   {:>12.6}",
        lower(shadow_moved)
    );
    println!();
    println!(
        "  The shadow held at ({:+.3}, {:+.3}, {:+.3}) through every phase, so a measurement that",
        lower(x),
        lower(y),
        lower(z)
    );
    println!("  reads the Bloch vector reads the same answer at each of them. That is what it");
    println!("  means for the global phase to carry no physics: the fibration forgets it, and");
    println!("  the fiber is precisely what it forgets.");
    println!();
    println!("  The state column carries the second half of the story. It peaks at 4 halfway");
    println!("  round, where the rotor has reached -R, and returns to 0 only at the end of the");
    println!("  circuit. A spinor takes two turns to come back, and the shadow takes one.");
}

/// The semi-axis a Bloch vector points along, when it points along one.
///
/// The label goes to the dominant signed component, and only when that component carries the
/// bulk of a unit vector. The cutoff sits far above any scalar's rounding: a cardinal state lands
/// within an ulp of `±1` at every precision, and an off-axis state has no component near it.
fn semi_axis(x: FloatType, y: FloatType, z: FloatType) -> &'static str {
    const DOMINANT: f64 = 0.9;

    let named = [
        ("+x", lower(x)),
        ("-x", -lower(x)),
        ("+y", lower(y)),
        ("-y", -lower(y)),
        ("+z", lower(z)),
        ("-z", -lower(z)),
    ];

    named
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .filter(|(_, component)| *component > DOMINANT)
        .map(|(axis, _)| *axis)
        .unwrap_or("between axes")
}
