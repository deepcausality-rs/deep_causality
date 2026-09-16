/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Aneurysm wall degeneration from the wall shear stress profile
//!
//! An intracranial aneurysm is a balloon-like bulge on an artery. Whether it grows and ruptures
//! tracks the mechanical load the blood flow puts on the wall, and the load that matters is
//! **wall shear stress**: the tangential drag of blood on the endothelium, in pascals.
//!
//! Healthy cerebral arteries run at roughly 1 to 7 Pa. Inside an aneurysm dome the vessel widens,
//! the flow slows, and the shear collapses. Sustained shear below about 0.4 Pa leaves the
//! endothelium unable to maintain the wall, which is the degeneration pathway this example
//! models. The second recognised factor is the **spatial gradient** of that stress, which peaks at
//! the dome neck where the profile falls off a cliff.
//!
//! The vessel centreline is a simplicial manifold, and three categorical operations carry the
//! whole analysis:
//!
//! ```text
//! fmap    radius   → wall shear stress     a local closure, one value per node
//! extend  stress   → its spatial gradient  a neighbour stencil, so it asks for the cursor
//! fold    stress   → the dome minimum      a reduction over the payload
//! ```
//!
//! `fmap` applies a law to one value at a time, and `fold` reduces the payload to a single number.
//! The gradient reads the nodes on either side, so it asks for a cursor, and `extend` supplies
//! one: the closure receives the manifold focused at each node in turn.
//!
//! Degeneration then accumulates over cardiac cycles wherever the shear sits below the threshold,
//! and the run reports the epoch at which the index crosses the rupture-risk line.

mod model;
mod utils_print;

use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Foldable, Functor};
use deep_causality_num::{lift, lift_count};
use deep_causality_topology::ManifoldWitness;
use model::{build_vessel_manifold, degeneration_history, wall_shear_stress};
use utils_print::{
    print_geometry, print_header, print_history, print_profile, print_risk_factors, print_verdict,
};

/// Centreline nodes along the segment.
const N_NODES: usize = 41;

/// The working scalar. Switch it to `f32`, `deep_causality_num::BFloat16` or
/// `deep_causality_num::Float106`; the geometry, the stress profile, its gradient and the
/// degeneration accumulation all re-run at that precision.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // The centreline as a manifold whose payload is the lumen radius at each node.
    let vessel = build_vessel_manifold(N_NODES)?;
    print_geometry(N_NODES);

    // fmap: the Poiseuille wall-shear closure τ = 4μQ / (πR³). The law reads one radius and
    // returns one stress, so the functor carries it across every node.
    let shear = ManifoldWitness::fmap(vessel, wall_shear_stress);

    // extend: the spatial gradient |dτ/dx| reads the nodes on either side, and the cursor the
    // comonad hands the closure is what reaches them.
    let node_spacing =
        lift::<FloatType>(model::SEGMENT_LENGTH_M) / lift_count::<FloatType>(N_NODES as u64 - 1);
    let gradient = ManifoldWitness::extend(&shear, |view| {
        let i = view.cursor();
        let tau = view.data().as_slice();
        if i >= N_NODES {
            return lift::<FloatType>(0.0);
        }
        let left = if i > 0 { tau[i - 1] } else { tau[i] };
        let right = if i + 1 < N_NODES { tau[i + 1] } else { tau[i] };
        Real::abs(right - left) / (lift::<FloatType>(2.0) * node_spacing)
    });
    print_profile(&shear, &gradient, N_NODES);

    // fold: the lowest shear anywhere on the segment, which is the dome floor. Node 0 sits in the
    // healthy calibre, so it seeds the reduction with a measured value.
    let proximal = shear.data().as_slice()[0];
    let dome_shear =
        ManifoldWitness::fold(
            shear,
            proximal,
            |lowest, tau| {
                if tau < lowest { tau } else { lowest }
            },
        );

    // The peak gradient sits at the neck, where the profile drops into the dome.
    let peak_gradient = ManifoldWitness::fold(gradient, lift::<FloatType>(0.0), |highest, g| {
        if g > highest { g } else { highest }
    });

    print_risk_factors(dome_shear, peak_gradient);

    // Degeneration accrues where the shear sits under the threshold, in proportion to how far
    // under it sits.
    let history = degeneration_history(dome_shear);
    print_history(&history);
    print_verdict(&history);

    Ok(())
}
