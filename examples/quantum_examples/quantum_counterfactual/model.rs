/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the repetition code: the encoding, the bit-flip gate, the parity measurements
//! that make up a syndrome, the decoder, and the fidelity the run is judged by.
//!
//! Every constant is declared at the working type through `const_scalar_from_int!`, so the
//! compiler resolves them against the alias in `main`.

use crate::FloatType;
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::const_scalar_from_int;
use deep_causality_num_complex::Complex;

// =============================================================================
// The small numbers the model is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const THREE: FloatType = const_scalar_from_int!(FloatType, 3);
const FOUR: FloatType = const_scalar_from_int!(FloatType, 4);
const FIVE: FloatType = const_scalar_from_int!(FloatType, 5);

// =============================================================================
// The code
// =============================================================================

/// Physical qubits in the code block.
pub const N_QUBITS: usize = 3;

/// Basis states of three qubits, which is how many complex amplitudes a state carries.
pub const N_AMPLITUDES: usize = 1 << N_QUBITS;

/// The register the noise flips. Nothing depends on which one it is; the decoder is told only the
/// syndrome, and finding this number again is its whole job.
pub const FLIPPED_QUBIT: usize = 1;

/// The metric for a three-qubit register.
pub fn register_metric() -> Metric {
    Metric::Euclidean(N_QUBITS)
}

/// The state to protect, `|ψ⟩ = α|0⟩ + β|1⟩` with `α = 3/5` and `β = 4/5`.
///
/// The two amplitudes differ so that a state which merely *looks* normalised is not mistaken for
/// the right one, and `9/25 + 16/25 = 1` exactly, so the norm is not an artefact of rounding.
pub fn amplitudes() -> (Complex<FloatType>, Complex<FloatType>) {
    (
        Complex::new(THREE / FIVE, ZERO),
        Complex::new(FOUR / FIVE, ZERO),
    )
}

/// Encodes one logical qubit into three physical ones: `α|0⟩ + β|1⟩ → α|000⟩ + β|111⟩`.
///
/// The encoding is what makes error correction possible at all. A lone qubit cannot be checked,
/// because every measurement that would reveal an error also reveals `α` and `β` and destroys the
/// superposition. Spreading the logical state across three qubits leaves questions that can be
/// asked about the *relationship* between them instead.
pub fn encode(
    alpha: Complex<FloatType>,
    beta: Complex<FloatType>,
) -> Result<HilbertState<FloatType>, CodeError> {
    let mut data = vec![Complex::new(ZERO, ZERO); N_AMPLITUDES];
    data[0b000] = alpha;
    data[0b111] = beta;

    HilbertState::new(data, register_metric()).map_err(|_| CodeError::Construction)
}

/// Applies an `X` gate to one qubit: a genuine bit flip, as a permutation of the amplitudes.
///
/// `X` on qubit `k` exchanges every basis state with the one whose `k`-th bit differs, so the
/// amplitude at index `i` moves to index `i XOR (1 << k)`. Nothing is substituted and no amplitude
/// is invented; the same eight numbers come out in a different order.
pub fn bit_flip(
    state: &HilbertState<FloatType>,
    qubit: usize,
) -> Result<HilbertState<FloatType>, CodeError> {
    if qubit >= N_QUBITS {
        return Err(CodeError::NoSuchQubit(qubit));
    }

    let source = state.as_inner().data();
    let mask = 1usize << qubit;

    let data: Vec<Complex<FloatType>> = (0..N_AMPLITUDES).map(|i| source[i ^ mask]).collect();

    HilbertState::new(data, register_metric()).map_err(|_| CodeError::Construction)
}

// =============================================================================
// The syndrome
// =============================================================================

/// The expectation value of `Z_j Z_k`, which is the parity of two qubits.
///
/// ```text
/// ⟨Z_j Z_k⟩ = Σ_i |c_i|² · (−1)^(bit_j(i) ⊕ bit_k(i))
/// ```
///
/// This is the measurement that makes the code work, and the reason is that the answer does not
/// depend on `α` or `β`. Every basis state in the support of a code word, corrupted or not, agrees
/// on the parity of any two qubits, so the weights sum to one and the result is exactly `+1` or
/// `−1`. The measurement learns that two qubits disagree without learning what either of them is.
pub fn parity(state: &HilbertState<FloatType>, qubit_j: usize, qubit_k: usize) -> FloatType {
    state
        .as_inner()
        .data()
        .iter()
        .enumerate()
        .fold(ZERO, |sum, (i, amplitude)| {
            let weight = amplitude.re * amplitude.re + amplitude.im * amplitude.im;
            let differ = ((i >> qubit_j) & 1) ^ ((i >> qubit_k) & 1);

            if differ == 0 {
                sum + weight
            } else {
                sum - weight
            }
        })
}

/// The two parity checks of the three-qubit code, as a pair of signs.
///
/// The default is the clean syndrome, both pairs agreeing, which is the reading that asks for no
/// correction. A stage that loses its value therefore leaves the register alone rather than
/// flipping an arbitrary qubit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Syndrome {
    /// Whether qubits 0 and 1 disagree.
    pub first_pair_differs: bool,
    /// Whether qubits 1 and 2 disagree.
    pub second_pair_differs: bool,
}

impl Syndrome {
    /// The syndrome a clean code word gives: both pairs agree.
    pub const CLEAN: Self = Self {
        first_pair_differs: false,
        second_pair_differs: false,
    };

    /// The syndrome that names qubit 0, which is what the counterfactual forces.
    pub const NAMES_QUBIT_0: Self = Self {
        first_pair_differs: true,
        second_pair_differs: false,
    };

    /// Whether the syndrome reports anything wrong.
    pub fn is_error(&self) -> bool {
        self.first_pair_differs || self.second_pair_differs
    }
}

/// Measures both parity checks.
pub fn measure_syndrome(state: &HilbertState<FloatType>) -> Syndrome {
    Syndrome {
        first_pair_differs: parity(state, 0, 1) < ZERO,
        second_pair_differs: parity(state, 1, 2) < ZERO,
    }
}

/// Which qubit a syndrome names, if it names one.
///
/// Three single-qubit flips give three distinct syndromes, and the fourth pattern is the clean
/// one, so the table is exhaustive and the decode is a lookup rather than a search:
///
/// ```text
/// (agree, agree)        no error
/// (differ, agree)       qubit 0
/// (differ, differ)      qubit 1
/// (agree, differ)       qubit 2
/// ```
pub fn decode(syndrome: Syndrome) -> Option<usize> {
    match (syndrome.first_pair_differs, syndrome.second_pair_differs) {
        (false, false) => None,
        (true, false) => Some(0),
        (true, true) => Some(1),
        (false, true) => Some(2),
    }
}

/// Applies whatever correction the syndrome calls for, and leaves the state alone when it calls
/// for none.
pub fn recover(
    state: &HilbertState<FloatType>,
    syndrome: Syndrome,
) -> Result<HilbertState<FloatType>, CodeError> {
    match decode(syndrome) {
        Some(qubit) => bit_flip(state, qubit),
        None => Ok(state.clone()),
    }
}

// =============================================================================
// Judging the result
// =============================================================================

/// The fidelity `|⟨a|b⟩|²` between two states: one when they are the same state, zero when they
/// are orthogonal.
///
/// This is the only honest verdict on a recovery. Checking that some amplitude came out large
/// would pass a state that is large in the wrong place, and the miscorrected run below is exactly
/// such a state.
pub fn fidelity(a: &HilbertState<FloatType>, b: &HilbertState<FloatType>) -> FloatType {
    let overlap = a
        .as_inner()
        .data()
        .iter()
        .zip(b.as_inner().data())
        .fold(Complex::new(ZERO, ZERO), |sum, (x, y)| {
            sum + Complex::new(x.re, -x.im) * *y
        });

    overlap.re * overlap.re + overlap.im * overlap.im
}

/// The total probability in a state, which stays at one through every operation here.
pub fn norm_squared(state: &HilbertState<FloatType>) -> FloatType {
    state
        .as_inner()
        .data()
        .iter()
        .fold(ZERO, |sum, c| sum + c.re * c.re + c.im * c.im)
}

// =============================================================================
// Errors
// =============================================================================

/// What can go wrong building a state in this example.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeError {
    /// A register index outside the code block.
    NoSuchQubit(usize),
    /// The register could not be built from the amplitudes given.
    Construction,
}

impl core::fmt::Display for CodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CodeError::NoSuchQubit(q) => {
                write!(f, "qubit {q} is outside a {N_QUBITS}-qubit code block")
            }
            CodeError::Construction => write!(f, "the register could not be built"),
        }
    }
}

impl core::error::Error for CodeError {}
