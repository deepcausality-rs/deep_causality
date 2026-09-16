/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Decentralised structural health monitoring
//!
//! A micrometeoroid strikes one plate of a pressurised hull. The struck plate is now carrying more
//! than it can hold, and when it yields it does not simply fail: it sheds its load onto the plates
//! it is bonded to, which may then exceed their own limit. That is a **failure cascade**, and
//! whether it stops depends on the topology of the bonds.
//!
//! The plate cannot ask a ground station what to do. The cascade completes in a fraction of a
//! second and the round trip is seconds, so the decision is local or it is too late.
//!
//! # The two abstractions
//!
//! **The graph comonad carries the cascade.** The hull is a [`Hull`], a graph whose payload is the
//! stress on each plate. One redistribution step is a single `extend`: the closure receives the
//! hull focused on one plate, reads that plate's stress and asks the graph for its bonds, and
//! returns the plate's next stress. The topology is never copied into a side structure, so the
//! cascade is computed on the graph that describes the hull.
//!
//! **The causal monad carries the intervention.** `alternate_value_if` is Pearl's do-operator: it
//! substitutes the value the sensor reported with the value the intervention forces, and records
//! the substitution. Running the same cascade from the observed reading and from the intervened
//! one is the counterfactual, computed twice rather than asserted once.
//!
//! ```text
//! extend               hull → its next stress state     one redistribution step
//! alternate_value_if   observed stress → forced stress  do(stress := safe limit)
//! ```

mod model;
mod utils_print;

use deep_causality_core::CausalFlow;
use deep_causality_num::Float106;
use deep_causality_topology::TopologyError;
use model::{
    DEFAULT_MODULUS_GPA, ENHANCED_MODULUS_GPA, Hull, IMPACT_LOAD_MPA, IMPACT_PLATE,
    NOMINAL_LOAD_MPA, SAFE_LIMIT_MPA, WARNING_THRESHOLD_MPA, breached_plates, build_hull,
    load_plate, redistribute, yielded_plates,
};
use utils_print::{print_cascade_step, print_header, print_hull, print_reading, print_verdict};

/// How many redistribution steps to run before declaring the hull settled. The six-plate hull
/// cannot cascade longer than it has plates.
const MAX_STEPS: usize = model::N_PLATES;

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the stresses,
/// the redistribution and the strain all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();
    print_hull(&build_hull()?);

    // The sensor reading on the struck plate: nominal service load plus the impact.
    let observed = NOMINAL_LOAD_MPA + IMPACT_LOAD_MPA;

    // ── The causal monad ────────────────────────────────────────────────────────────────────
    // `alternate_value_if` is the do-operator. The reading passes the warning threshold, so the
    // plate forces its own stress down to the safe limit and the substitution is recorded.
    let intervened: FloatType = CausalFlow::value(observed)
        .alternate_value_if(|stress| *stress > WARNING_THRESHOLD_MPA, |_| SAFE_LIMIT_MPA)
        .finish()?;

    print_reading(observed, intervened);

    // ── The graph comonad ───────────────────────────────────────────────────────────────────
    // The same cascade, run from each reading. That comparison is the counterfactual.
    let without = settle("Without intervention", observed)?;
    let with = settle("With intervention", intervened)?;

    print_verdict(&without, &with);
    Ok(())
}

/// Runs the cascade until the hull stops shedding load, and reports the final hull.
///
/// Each step is one `extend` over the graph. The loop stops as soon as no plate is over yield,
/// which is what "settled" means: nothing further will move.
fn settle(label: &str, struck_plate_stress: FloatType) -> Result<Hull, TopologyError> {
    let mut hull = load_plate(&build_hull()?, IMPACT_PLATE, struck_plate_stress);

    println!("{label}");
    print_cascade_step(0, &hull, &yielded_plates(&hull));

    for step in 1..=MAX_STEPS {
        let yielding = yielded_plates(&hull);
        if yielding.is_empty() {
            break;
        }
        hull = redistribute(&hull);
        print_cascade_step(step, &hull, &yielded_plates(&hull));
    }

    println!(
        "  settled: {} of {} plates breached\n",
        breached_plates(&hull).len(),
        model::N_PLATES
    );
    Ok(hull)
}

/// The two moduli the plate can present, for the strain comparison the printer renders.
pub fn moduli() -> [(&'static str, FloatType); 2] {
    [
        ("as manufactured", DEFAULT_MODULUS_GPA),
        ("reinforcement engaged", ENHANCED_MODULUS_GPA),
    ]
}
