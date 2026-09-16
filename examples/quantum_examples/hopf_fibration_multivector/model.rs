/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the Hopf fibration: the cardinal qubit states, the fiber the global phase
//! traverses, and the two displacements the run measures.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_multivector::{CausalMultiVector, HopfState, MultiVector};
use deep_causality_num::{const_scalar_from_int, lift_usize};
use deep_causality_num_complex::Complex;

// =============================================================================
// The small numbers the geometry is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

// =============================================================================
// The algebra's index layout
// =============================================================================

/// Where the three vector generators sit among the eight coefficients of `Cl(3)`.
///
/// The basis is ordered by the binary pattern of its generators, so `e1` is `001`, `e2` is `010`
/// and `e3` is `100`. The even-numbered gaps hold the scalar and the bivectors.
pub const E1: usize = 1;
pub const E2: usize = 2;
pub const E3: usize = 4;

// =============================================================================
// The fiber
// =============================================================================

/// How many steps the run takes around the fiber.
///
/// The fiber is a circle, so a complete circuit returns the state to where it started. Walking it
/// in steps is what turns a claim about one phase into a statement about every phase.
pub const FIBER_STEPS: usize = 8;

/// A complete circuit of the fiber, in radians of `fiber_shift`.
///
/// It is `4π` and not `2π`. `fiber_shift(θ)` multiplies the rotor by `e^{−θe₁₂/2}`, so the phase
/// it applies is `θ/2`, and the rotor only comes back to itself after `θ` has run through twice
/// the circle. That factor of two is the spinor double cover: `SU(2)` wraps `SO(3)` twice, and at
/// `θ = 2π` the walk is at `−R`, which is the antipode of the state in `S³` and the very same
/// point on `S²`.
pub fn fiber_circuit() -> FloatType {
    TWO * TWO * FloatType::pi()
}

/// The angles the run visits, one complete circuit in [`FIBER_STEPS`] equal steps.
pub fn fiber_angles() -> Vec<FloatType> {
    let steps = lift_usize::<FloatType>(FIBER_STEPS);

    (1..=FIBER_STEPS)
        .map(|step| fiber_circuit() * lift_usize::<FloatType>(step) / steps)
        .collect()
}

// =============================================================================
// States
// =============================================================================

/// `1/√2`, computed at the working precision rather than rounded through a wider one.
pub fn frac_1_sqrt_2() -> FloatType {
    ONE / Real::sqrt(TWO)
}

/// The six states that sit on the semi-axes of the Bloch sphere.
///
/// `|0⟩` and `|1⟩` are the poles, `|±⟩` are the equal superpositions, and `|±i⟩` are the same two
/// amplitudes a quarter-turn apart in phase. Together they pin all three axes, which is what makes
/// them worth printing: a projection that got two axes confused still looks plausible on any one
/// of them.
pub fn cardinal_states() -> [(&'static str, Complex<FloatType>, Complex<FloatType>); 6] {
    let s = frac_1_sqrt_2();

    [
        ("|0⟩", Complex::new(ONE, ZERO), Complex::new(ZERO, ZERO)),
        ("|1⟩", Complex::new(ZERO, ZERO), Complex::new(ONE, ZERO)),
        ("|+⟩", Complex::new(s, ZERO), Complex::new(s, ZERO)),
        ("|−⟩", Complex::new(s, ZERO), Complex::new(-s, ZERO)),
        ("|+i⟩", Complex::new(s, ZERO), Complex::new(ZERO, s)),
        ("|−i⟩", Complex::new(s, ZERO), Complex::new(ZERO, -s)),
    ]
}

/// The state the fiber walk starts from: `|+⟩`, an equal superposition on the equator.
pub fn equatorial_state() -> HopfState<FloatType> {
    let s = frac_1_sqrt_2();
    HopfState::from_spinor(Complex::new(s, ZERO), Complex::new(s, ZERO))
}

// =============================================================================
// Reading the geometry
// =============================================================================

/// The three components of a vector in `Cl(3)`, pulled out of the eight coefficients.
pub fn bloch_axes(vector: &CausalMultiVector<FloatType>) -> [FloatType; 3] {
    let data = vector.data();
    [data[E1], data[E2], data[E3]]
}

/// The Bloch vector of a spinor: build the rotor, project it, read the three axes.
pub fn bloch_vector(alpha: Complex<FloatType>, beta: Complex<FloatType>) -> [FloatType; 3] {
    bloch_axes(&HopfState::from_spinor(alpha, beta).project())
}

/// How far apart two multivectors are, as a squared length.
///
/// The same measure serves on `S³` and on `S²`, which is the point: the run compares one against
/// the other, and a comparison between two different measures would say nothing.
pub fn squared_distance(
    a: &CausalMultiVector<FloatType>,
    b: &CausalMultiVector<FloatType>,
) -> FloatType {
    (a - b).squared_magnitude()
}

/// The larger of two displacements.
pub fn larger(a: FloatType, b: FloatType) -> FloatType {
    if a > b { a } else { b }
}
