/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the IKKT matrix model: the configuration, the action, the equation of motion,
//! and one relaxation step along it.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::{const_scalar_from_int, lift_usize};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{Operator, QuantumError, commutator_kernel};

// =============================================================================
// The small numbers the model is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TEN: FloatType = const_scalar_from_int!(FloatType, 10);

// =============================================================================
// The configuration
// =============================================================================

/// How many coordinate matrices the run carries.
///
/// The IKKT model proper has ten, one per dimension of type IIB superstring theory. Four is enough
/// to make the commutators interact, and every loop below is written over the constant.
pub const N_MATRICES: usize = 4;

/// The Clifford algebra the matrices live in: `Cl(2)`, so each carries four complex coefficients.
pub const ALGEBRA_DIMENSION: usize = 2;
pub const MATRIX_SIZE: usize = 1 << ALGEBRA_DIMENSION;

/// How many relaxation steps to take, and how far each one moves.
pub const MAX_STEPS: usize = 200;

/// The step length.
///
/// It is short enough that the action falls on every step, which the run checks rather than
/// assumes, and long enough to reach a commuting configuration inside [`MAX_STEPS`]. A longer step
/// overshoots and the action starts climbing, which the outcome line would then report.
pub fn step_size() -> FloatType {
    const FIVE: FloatType = const_scalar_from_int!(FloatType, 5);

    ONE / FIVE
}

/// The action below which the configuration counts as commuting.
pub fn convergence_threshold() -> FloatType {
    const A_MILLIONTH: FloatType = const_scalar_from_int!(FloatType, 1_000_000);

    ONE / A_MILLIONTH
}

/// The metric the coordinate matrices share.
pub fn algebra_metric() -> Metric {
    Metric::Euclidean(ALGEBRA_DIMENSION)
}

/// A spread-out starting configuration, deliberately non-commuting.
///
/// Coefficient `j` of matrix `i` is `(i + j)/10`. Nothing is special about those numbers beyond
/// their failing to commute, which is the only property the run needs of them.
pub fn initial_configuration() -> Result<Vec<Operator<FloatType>>, QuantumError> {
    (0..N_MATRICES)
        .map(|i| {
            let data: Vec<Complex<FloatType>> = (0..MATRIX_SIZE)
                .map(|j| {
                    let coefficient = lift_usize::<FloatType>(i + j) / TEN;
                    Complex::new(coefficient, ZERO)
                })
                .collect();

            Ok(HilbertState::new(data, algebra_metric())?)
        })
        .collect()
}

// =============================================================================
// The action
// =============================================================================

/// The bosonic IKKT action, `S = Σ_{μ<ν} ‖[X_μ, X_ν]‖²`.
///
/// It is zero exactly when every pair commutes. A configuration of commuting matrices can be
/// simultaneously diagonalised, and the joint eigenvalues are then points — which is the sense in
/// which spacetime is said to emerge from the matrices rather than to contain them.
pub fn action(configuration: &[Operator<FloatType>]) -> Result<FloatType, QuantumError> {
    let mut total = ZERO;

    for mu in 0..configuration.len() {
        for nu in (mu + 1)..configuration.len() {
            let commutator = commutator_kernel(&configuration[mu], &configuration[nu])?;
            total += squared_norm(&commutator);
        }
    }

    Ok(total)
}

/// The squared coefficient norm of an operator.
pub fn squared_norm(operator: &Operator<FloatType>) -> FloatType {
    operator
        .as_inner()
        .data()
        .iter()
        .fold(ZERO, |sum, c| sum + c.re * c.re + c.im * c.im)
}

/// The norm of the whole configuration, which the relaxation holds fixed.
pub fn configuration_norm(configuration: &[Operator<FloatType>]) -> FloatType {
    Real::sqrt(
        configuration
            .iter()
            .fold(ZERO, |sum, x| sum + squared_norm(x)),
    )
}

/// The largest single commutator in the configuration, as a norm.
pub fn largest_commutator(
    configuration: &[Operator<FloatType>],
) -> Result<FloatType, QuantumError> {
    let mut largest = ZERO;

    for mu in 0..configuration.len() {
        for nu in (mu + 1)..configuration.len() {
            let commutator = commutator_kernel(&configuration[mu], &configuration[nu])?;
            let norm = Real::sqrt(squared_norm(&commutator));
            if norm > largest {
                largest = norm;
            }
        }
    }

    Ok(largest)
}

// =============================================================================
// The equation of motion
// =============================================================================

/// The IKKT equation of motion for one coordinate, `Σ_{ν ≠ μ} [X_ν, [X_μ, X_ν]]`.
///
/// Varying the action gives `Σ_ν [X_ν, [X^μ, X^ν]] = 0`, so this expression is zero exactly at a
/// solution. Every configuration of mutually commuting matrices is one, and so are the fuzzy-sphere
/// configurations where the double commutator cancels without the single one vanishing.
///
/// Moving against it is what the relaxation below does, which is why the run reports the action at
/// every step: a step that raised it would mean the step length, not the direction, was wrong.
pub fn equation_of_motion(
    configuration: &[Operator<FloatType>],
    mu: usize,
) -> Result<Vec<Complex<FloatType>>, QuantumError> {
    let mut total = vec![Complex::new(ZERO, ZERO); MATRIX_SIZE];

    for (nu, x_nu) in configuration.iter().enumerate() {
        if nu == mu {
            continue;
        }

        let inner = commutator_kernel(&configuration[mu], x_nu)?;
        let outer = commutator_kernel(x_nu, &inner)?;

        for (slot, term) in total.iter_mut().zip(outer.as_inner().data()) {
            *slot += *term;
        }
    }

    Ok(total)
}

/// One relaxation step: move every coordinate against the equation of motion, then restore the
/// norm the configuration started with.
///
/// The rescaling is what keeps the answer interesting. The action is quartic in the coordinates,
/// so shrinking every matrix toward the origin drives it to zero whatever the configuration is
/// doing, and the limit is an empty vacuum rather than a commuting one. Holding the norm fixed
/// leaves only one way for the action to fall, which is for the matrices to genuinely commute.
pub fn relax(
    configuration: &[Operator<FloatType>],
    step: FloatType,
) -> Result<Vec<Operator<FloatType>>, QuantumError> {
    let before = configuration_norm(configuration);

    let mut moved = Vec::with_capacity(configuration.len());
    for (mu, x_mu) in configuration.iter().enumerate() {
        let gradient = equation_of_motion(configuration, mu)?;

        let data: Vec<Complex<FloatType>> = x_mu
            .as_inner()
            .data()
            .iter()
            .zip(&gradient)
            .map(|(coefficient, force)| *coefficient - *force * Complex::new(step, ZERO))
            .collect();

        moved.push(HilbertState::new(data, algebra_metric())?);
    }

    let after = configuration_norm(&moved);
    if after <= ZERO {
        return Ok(moved);
    }

    rescale(&moved, before / after)
}

/// Multiplies every coordinate by a real factor.
fn rescale(
    configuration: &[Operator<FloatType>],
    factor: FloatType,
) -> Result<Vec<Operator<FloatType>>, QuantumError> {
    configuration
        .iter()
        .map(|x| {
            let data: Vec<Complex<FloatType>> = x
                .as_inner()
                .data()
                .iter()
                .map(|c| *c * Complex::new(factor, ZERO))
                .collect();

            Ok(HilbertState::new(data, algebra_metric())?)
        })
        .collect()
}
