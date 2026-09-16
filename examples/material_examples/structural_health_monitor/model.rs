/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the hull monitor: the material constants, the hull topology, and the load
//! redistribution that runs over it.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_haft::CoMonad;
use deep_causality_num::const_scalar_from_int;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Graph, GraphWitness, TopologyError};

// =============================================================================
// The small numbers the model is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);

// =============================================================================
// Material and loading
// =============================================================================

/// Yield strength of the hull plate, in MPa. Typical structural steel.
///
/// A plate carrying more than this has yielded, and this model treats that as failure: the plate
/// stops carrying load and sheds what it held to the plates it is bonded to.
pub const YIELD_STRENGTH_MPA: FloatType = const_scalar_from_int!(FloatType, 250);

/// The share of yield at which the monitor raises a warning, in MPa. Eighty percent of yield.
pub const WARNING_THRESHOLD_MPA: FloatType = const_scalar_from_int!(FloatType, 200);

/// The stress every plate carries in normal service, in MPa.
pub const NOMINAL_LOAD_MPA: FloatType = const_scalar_from_int!(FloatType, 120);

/// The stress a micrometeoroid strike adds to the plate it hits, in MPa.
pub const IMPACT_LOAD_MPA: FloatType = const_scalar_from_int!(FloatType, 200);

/// The stress the active intervention holds the struck plate at, in MPa.
pub const SAFE_LIMIT_MPA: FloatType = const_scalar_from_int!(FloatType, 100);

/// Young's modulus of the plate, in GPa: as manufactured, and with active reinforcement engaged.
pub const DEFAULT_MODULUS_GPA: FloatType = const_scalar_from_int!(FloatType, 200);
pub const ENHANCED_MODULUS_GPA: FloatType = const_scalar_from_int!(FloatType, 400);

/// MPa per GPa, for the unit crossing inside Hooke's law.
const MPA_PER_GPA: FloatType = const_scalar_from_int!(FloatType, 1000);

// =============================================================================
// Hull topology
// =============================================================================

/// Plates in the hull section.
pub const N_PLATES: usize = 6;

/// The plate the micrometeoroid strikes.
pub const IMPACT_PLATE: usize = 2;

/// The structural bonds: a hexagonal ring, then two cross-braces.
///
/// The bracing is what makes the cascade interesting. Without it the ring sheds load one way
/// around; with it a failure reaches plates that are not its ring neighbours.
pub const BONDS: [(usize, usize); 8] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 4),
    (4, 5),
    (5, 0),
    (0, 4),
    (1, 5),
];

/// The hull: plates as vertices, structural bonds as edges, the stress each plate carries as the
/// payload.
pub type Hull = Graph<FloatType>;

/// Builds the hull with every plate at its nominal service load.
pub fn build_hull() -> Result<Hull, TopologyError> {
    let payload = CausalTensor::new(vec![NOMINAL_LOAD_MPA; N_PLATES], vec![N_PLATES])?;
    let mut hull = Graph::new(N_PLATES, payload, 0)?;

    for (a, b) in BONDS {
        hull.add_edge(a, b)?;
    }
    Ok(hull)
}

/// Sets the stress on one plate, leaving the rest of the hull as it stands.
pub fn load_plate(hull: &Hull, plate: usize, stress_mpa: FloatType) -> Hull {
    GraphWitness::extend(hull, |view| {
        let i = view.cursor();
        let stress = view.data().as_slice();
        if i == plate { stress_mpa } else { stress[i] }
    })
}

// =============================================================================
// Load redistribution
// =============================================================================

/// One redistribution step over the whole hull.
///
/// `extend` focuses the hull on each plate in turn and hands the closure that focused view, so the
/// closure reads its own stress, asks the graph for its bonds, and returns the plate's next
/// stress. The topology is never copied out into a side structure: the cascade is computed on the
/// graph that describes the hull.
///
/// The rule is local. A plate over yield has failed, so it carries nothing afterwards and its load
/// leaves it. Every surviving plate takes an equal share of each failed neighbour's load, split
/// among that neighbour's surviving bonds. A plate already at zero has failed in an earlier step
/// and neither sheds again nor receives.
pub fn redistribute(hull: &Hull) -> Hull {
    GraphWitness::extend(hull, |view| {
        let plate = view.cursor();
        let stress = view.data().as_slice();

        // A plate over yield sheds everything it holds this step.
        if has_yielded(stress[plate]) {
            return ZERO;
        }

        // A plate that shed in an earlier step carries nothing and takes no further load, so the
        // shares of its failing neighbours pass it by.
        if stress[plate] <= ZERO {
            return ZERO;
        }

        let bonds = match view.neighbors(plate) {
            Ok(bonds) => bonds,
            Err(_) => return stress[plate],
        };

        let inherited = bonds
            .iter()
            .filter(|&&bonded| has_yielded(stress[bonded]))
            .fold(ZERO, |sum, &failing| {
                match surviving_bonds(view, failing, stress) {
                    0 => sum,
                    survivors => sum + stress[failing] / count(survivors),
                }
            });

        stress[plate] + inherited
    })
}

/// Whether a stress has passed the yield strength.
pub fn has_yielded(stress_mpa: FloatType) -> bool {
    stress_mpa > YIELD_STRENGTH_MPA
}

/// Whether a plate is still carrying load: below yield, and not already shed to zero.
fn is_carrying(stress_mpa: FloatType) -> bool {
    stress_mpa > ZERO && !has_yielded(stress_mpa)
}

/// How many of `plate`'s bonds lead to a plate still carrying load.
fn surviving_bonds(view: &Hull, plate: usize, stress: &[FloatType]) -> usize {
    view.neighbors(plate)
        .map(|bonds| {
            bonds
                .iter()
                .filter(|&&bonded| is_carrying(stress[bonded]))
                .count()
        })
        .unwrap_or(0)
}

/// The plates carrying more than the yield strength right now.
pub fn yielded_plates(hull: &Hull) -> Vec<usize> {
    hull.data()
        .as_slice()
        .iter()
        .enumerate()
        .filter(|(_, stress)| has_yielded(**stress))
        .map(|(plate, _)| plate)
        .collect()
}

/// The plates that have shed their load and carry nothing.
pub fn breached_plates(hull: &Hull) -> Vec<usize> {
    hull.data()
        .as_slice()
        .iter()
        .enumerate()
        .filter(|(_, stress)| **stress <= ZERO)
        .map(|(plate, _)| plate)
        .collect()
}

/// A count lifted onto the real axis.
fn count(n: usize) -> FloatType {
    deep_causality_num::lift_usize::<FloatType>(n)
}

// =============================================================================
// Hooke's law
// =============================================================================

/// The strain a stress produces in a plate of the given modulus, dimensionless.
///
/// Hooke's law is `σ = E·ε`, so `ε = σ/E`. Stress is in MPa and modulus in GPa, so the ratio is
/// scaled by the thousand between them. Engaging the active reinforcement raises `E`, and the same
/// stress then deforms the plate less, which is what the reinforcement is for.
pub fn strain(stress_mpa: FloatType, modulus_gpa: FloatType) -> FloatType {
    stress_mpa / (modulus_gpa * MPA_PER_GPA)
}
