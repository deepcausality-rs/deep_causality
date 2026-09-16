/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Tissue classification by topology
//!
//! A tumour that outgrows its blood supply dies from the inside out, leaving a **necrotic core**:
//! a shell of living cells around a dead centre. On an MRI slice that reads as a ring, and telling
//! it apart from a solid mass matters, because the core is what marks the tumour as aggressive.
//!
//! Measuring the ring by its shape is fragile, since tumours come in every shape. Measuring it by
//! its **topology** is robust, because topology counts holes and ignores shape entirely. Stretch a
//! ring, bend it, make it lopsided, and it still has exactly one hole.
//!
//! The classifier takes voxel centres, joins every pair closer than a fixed radius into a
//! simplicial complex, and reads two numbers off it:
//!
//! ```text
//! euler_characteristic   V − E + F      1 for a filled mass, 0 or less once a void is enclosed
//! extend  → fold         density range  how much the neighbour count varies across the sample
//! ```
//!
//! The Euler characteristic says whether a void is enclosed. The density range corroborates it
//! from a different direction: a solid mass has an interior and a rim, so its neighbour counts
//! span a range, while a ring is rim everywhere and its counts sit at one value. `extend` focuses
//! the point cloud on one voxel at a time, so its closure reads that voxel's neighbourhood from
//! the cloud, and `fold` reduces the whole map to its range.

mod model;
mod utils_print;

use deep_causality_num::Float106;
use deep_causality_topology::TopologyError;
use model::{Sample, density_range, local_density, necrotic_tissue, read_topology, solid_tissue};
use utils_print::{print_header, print_sample, print_verdict};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the
/// coordinates, the distances and the density map all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), TopologyError> {
    print_header();

    let solid = classify("Sample A, solid mass", &solid_tissue()?)?;
    let necrotic = classify("Sample B, ring", &necrotic_tissue()?)?;

    print_verdict(solid, necrotic);
    Ok(())
}

/// Reads both topological quantities off one sample and reports them. Returns the Euler
/// characteristic, which is what the two samples are compared on.
fn classify(label: &str, sample: &Sample) -> Result<isize, TopologyError> {
    let reading = read_topology(sample)?;

    // extend: every voxel counts the neighbours within the Rips radius.
    // fold: the range those counts span, which says whether the sample has an interior.
    let spread = density_range(local_density(sample));

    print_sample(label, &reading, spread);
    Ok(reading.euler_characteristic)
}
