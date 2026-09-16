/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The Hopf fibration: why a qubit's global phase is unobservable
//!
//! A qubit state `|ψ⟩ = α|0⟩ + β|1⟩` with `|α|² + |β|² = 1` is a point on the 3-sphere `S³`. What
//! an experiment can read off it is the Bloch vector, a point on the ordinary 2-sphere `S²`. The
//! map between them is the **Hopf fibration** `h: S³ → S²`, and every point of `S²` is the image
//! of a whole circle of points in `S³`. That circle is the **fiber**, and moving along it is
//! exactly multiplying the state by a global phase `e^{iθ}`.
//!
//! So the textbook statement that global phase carries no physics is a geometric one: the fiber is
//! what the projection forgets.
//!
//! # The fibration, in geometric algebra
//!
//! [`HopfState`] holds the state as a rotor in `Cl(3)`, the even subalgebra whose elements are the
//! unit quaternions. The projection is the sandwich product `v = R σ₃ R̃`, which is the rotation
//! the rotor names applied to the north pole, and the fiber is right-multiplication by a rotor in
//! the `e₁₂` plane. Both are one line of algebra rather than a coordinate formula.
//!
//! ```text
//! from_spinor   (α, β) ∈ ℂ²  →  R ∈ S³      the state as a rotor
//! project       R            →  R σ₃ R̃      the Bloch vector
//! fiber_shift   (R, θ)       →  R e^{−θe₁₂/2}   a step along the fiber
//! ```
//!
//! # What the run does
//!
//! Three categorical operations carry the whole argument:
//!
//! ```text
//! fmap       angle → state       one fiber step at a time
//! zip_with   (state, shadow) → (how far the state moved, how far its shadow moved)
//! fold       the pairs → the largest of each over the whole fiber
//! ```
//!
//! `fmap` walks the fiber, `zip_with` measures each state against its own projection, and `fold`
//! reduces a full turn to two numbers. The claim lands as a statement about the whole circle
//! rather than about one convenient angle.
//!
//! The run first prints where the six cardinal states land, which is the check that the projection
//! is the Bloch vector of the standard convention and not a relabelled copy of it.

mod model;
mod utils_print;

use deep_causality_haft::{Foldable, Functor, Semigroupal};
use deep_causality_num::Float106;
use deep_causality_tensor::{
    CausalTensor, CausalTensorError, CausalTensorWitness, ZipTensorWitness,
};
use model::{
    ZERO, bloch_axes, bloch_vector, cardinal_states, equatorial_state, fiber_angles, larger,
    squared_distance,
};
use utils_print::{print_cardinal, print_fiber_walk, print_header, print_verdict};

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the rotors,
/// the projection and both displacements all recompute at that precision.
///
/// It sits at [`Float106`] by default on purpose. A hard-coded `f64` anywhere in the program is
/// invisible while the alias *is* `f64`, and shows up here as a compile error the moment the two
/// types differ.
pub type FloatType = Float106;

fn main() -> Result<(), CausalTensorError> {
    print_header();

    // Where the six cardinal states land on the sphere. |0⟩ and |1⟩ on ±z, |±⟩ on ±x, |±i⟩ on ±y.
    let cardinal = cardinal_states();
    print_cardinal(&cardinal.map(|(name, alpha, beta)| (name, bloch_vector(alpha, beta))));

    // The state the walk starts from, and the shadow it casts.
    let state = equatorial_state();
    let shadow = state.project();

    // fmap: each angle names one step along the fiber, so the functor carries the shift across a
    // full turn without the turn appearing in the shift.
    let angles = CausalTensor::new(fiber_angles(), vec![model::FIBER_STEPS])?;
    let walked = CausalTensorWitness::fmap(angles.clone(), |angle| state.fiber_shift(angle));

    // fmap again: every state on the fiber projects to its own point on S².
    let shadows = CausalTensorWitness::fmap(walked.clone(), |shifted| shifted.project());

    // zip_with: pair each state with its own shadow and measure both against where they started.
    // Pairing them is what makes the two numbers comparable; measured apart they are two sweeps
    // that happen to have the same length.
    let displaced = ZipTensorWitness::zip_with(walked, shadows, |shifted, cast| {
        (
            squared_distance(state.as_inner(), shifted.as_inner()),
            squared_distance(&shadow, &cast),
        )
    });

    print_fiber_walk(&angles, &displaced);

    // fold: a full turn reduced to the furthest the state ever got, and the furthest its shadow
    // ever got. The first is the diameter of the fiber; the second is what the projection forgot.
    let (state_moved, shadow_moved) = CausalTensorWitness::fold(
        displaced,
        (ZERO, ZERO),
        |(state_max, shadow_max), (on_s3, on_s2)| {
            (larger(state_max, on_s3), larger(shadow_max, on_s2))
        },
    );

    print_verdict(bloch_axes(&shadow), state_moved, shadow_moved);
    Ok(())
}
