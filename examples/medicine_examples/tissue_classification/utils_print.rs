/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the tissue classifier.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{DIMENSIONS, RIPS_RADIUS, TopologyReading};
use deep_causality_num::lower;

/// The Euler characteristic of a filled, connected sample.
const CONTRACTIBLE: isize = 1;

pub fn print_header() {
    println!("=== Tissue classification by topology ===\n");
    println!(
        "Precision:            {}",
        core::any::type_name::<FloatType>()
    );
    println!("Vietoris-Rips radius: {RIPS_RADIUS}");
    println!("Voxel coordinates:    {DIMENSIONS}\n");
}

pub fn print_sample(label: &str, reading: &TopologyReading, spread: (FloatType, FloatType)) {
    let TopologyReading {
        vertices,
        edges,
        triangles,
        euler_characteristic: chi,
    } = *reading;
    println!("{label}");
    println!("  complex             {vertices} vertices, {edges} edges, {triangles} triangles");
    println!("  Euler characteristic  {vertices} - {edges} + {triangles} = {chi}");
    let (low, high) = spread;
    println!(
        "  neighbour count     {:.0} to {:.0}   ({})",
        lower(low),
        lower(high),
        if lower(low) == lower(high) {
            "uniform, the sample is all rim"
        } else {
            "varies, the sample has an interior"
        }
    );
    println!("  reading             {}\n", verdict_for(chi));
}

/// What an Euler characteristic says about the sample.
fn verdict_for(chi: isize) -> &'static str {
    if chi == CONTRACTIBLE {
        "solid, the complex fills in with no void"
    } else if chi < CONTRACTIBLE {
        "a void is enclosed, consistent with a necrotic core"
    } else {
        "several disconnected pieces, the sample is too sparse to read"
    }
}

pub fn print_verdict(solid: isize, necrotic: isize) {
    println!("Both samples were measured the same way, and they separate on χ alone.");
    println!("  solid mass   χ = {solid}");
    println!("  ring         χ = {necrotic}");
    println!();
    println!("χ counts holes, so it survives any deformation that leaves the connectivity alone.");
    println!("A lopsided, stretched or rotated tumour gives the same reading as this one.");
}
