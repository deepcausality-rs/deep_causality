/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Virtual epilepsy surgery planning
//!
//! Around a third of epilepsy patients keep having seizures on medication, and for some of them
//! surgery is the remaining option: remove the tissue that starts the seizure. The hard part is
//! deciding which tissue. A seizure is the brain's regions falling into pathological lockstep, and
//! the region driving that lockstep is often a **hub**, one that connects to many others.
//!
//! This example builds a digital twin of a patient's connectome, confirms it seizes, then resects
//! each region in turn and re-simulates. A resection that leaves the network scattered is curative.
//!
//! Two categorical operations carry the whole simulation:
//!
//! ```text
//! extend  connectome → its next state   each region reads its neighbours through the cursor
//! fold    phasors    → synchronisation  a reduction to the Kuramoto order parameter
//! ```
//!
//! Regions are modelled as Kuramoto oscillators: each runs at its own natural frequency and is
//! pulled toward the phase of every region it connects to. `extend` focuses the graph on one
//! region at a time and hands the closure a view that carries the payload and the adjacency
//! together, so the coupling sum reads its neighbours from the graph itself. A resection is then a
//! change to the wiring alone, and the dynamics follow.

mod model;
mod utils_print;

use deep_causality_num::Float106;
use model::{build_connectome, is_seizing, simulate};
use utils_print::{print_baseline, print_header, print_resection_row, print_verdict};

/// Brain regions in the connectome. Region 0 is the seizure focus.
const N_REGIONS: usize = 10;

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the phases,
/// the coupling, the integration and the order parameter all re-run at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header(N_REGIONS);

    // The untreated patient: does this connectome seize?
    let baseline = simulate(&build_connectome(N_REGIONS, None)?);
    print_baseline(baseline);

    if !is_seizing(baseline) {
        print_verdict(&[]);
        return Ok(());
    }

    // Virtual resection: disconnect one region, re-simulate, and see whether the seizure survives.
    let mut curative = Vec::new();
    for region in 0..N_REGIONS {
        let post_op = simulate(&build_connectome(N_REGIONS, Some(region))?);
        let seizing = is_seizing(post_op);
        print_resection_row(region, post_op, seizing);
        if !seizing {
            curative.push((region, post_op));
        }
    }

    print_verdict(&curative);
    Ok(())
}
