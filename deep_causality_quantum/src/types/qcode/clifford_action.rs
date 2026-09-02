/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Clifford conjugation of a Pauli, as a symplectic update over 𝔽₂.
//!
//! A Clifford gate maps Paulis to Paulis under conjugation, so pushing a Pauli through a Clifford
//! program is a sequence of updates on its `(x, z)` bit vectors: the stabilizer-tableau rule
//! (Aaronson & Gottesman, arXiv:quant-ph/0406196, §III) restricted to one operator. No state
//! vector is formed, there is no register-width limit, and the arithmetic is exact.
//!
//! This is what decides `H̄`. Haruna's Eq. (3.32) builds the logical Hadamard from `S`, `CZ` and
//! `H`, all Clifford, so `H̄` is a Clifford circuit, and the paper's own criterion for it is that
//! it "behaves as a Hadamard gate on the code space" under conjugation of the logical `Z` and `X`
//! (§3.2). That is a statement about images of Paulis, which is what a tableau computes. The
//! diagonal check cannot reach `H̄`, being neither a Pauli nor diagonal, and this check cannot
//! reach `T̄`, `CS̄†` or `CC̄Z`, which are non-Clifford; between the two, every Table 1 gate is
//! decided by exactly one exact predicate.
//!
//! # Phase
//!
//! The symplectic form is phase-blind: `S X S† = Y` and `S† X S = −Y` have the same `(x, z)`.
//! So the images here are Paulis up to phase, equivalence is decided up to phase through
//! [`LogicalBasis`], and the global phase a construction carries travels on its
//! [`LogicalProgram`](crate::LogicalProgram) rather than here.

use crate::QuantumError;
use crate::types::qcode::logical_equivalence::LogicalBasis;
use crate::types::qcode::logical_pauli::LogicalPauli;
use crate::types::qpu::circuit::GateOp;
use alloc::vec::Vec;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::{Gf2, NaturalNumber};

/// The image `U P U†` of a Pauli under a Clifford program, up to phase.
///
/// `program` is in application order, so the first gate acts first on a state, and conjugation
/// runs in the same order: `U = gₙ ⋯ g₁` gives `U P U† = gₙ(⋯(g₁ P g₁†)⋯)gₙ†`.
///
/// The rules, one per Clifford gate on the register, all read off `HXH = Z`, `SXS† = Y`,
/// `CNOT: X_c ↦ X_c X_t, Z_t ↦ Z_c Z_t` and `CZ: X_a ↦ X_a Z_b`:
///
/// ```text
/// H(q)          x_q ↔ z_q
/// S(q), S†(q)   z_q ^= x_q
/// X, Y, Z       no change
/// CNOT(c, t)    x_t ^= x_c ;  z_c ^= z_t
/// CZ(a, b)      z_b ^= x_a ;  z_a ^= x_b
/// ```
///
/// A `Cmz` on two qubits is a `CZ`; on one it is a `Z`.
///
/// # Errors
///
/// [`QuantumError::NonCliffordGate`] naming the first `T`, `T†`, `CS†`, `CCZ` or `Cmz` on three
/// or more qubits, since their conjugation action is not a Pauli.
/// [`QuantumError::DimensionMismatch`] if a gate names a qubit beyond the Pauli's register.
pub fn clifford_conjugate<W: NaturalNumber>(
    pauli: &LogicalPauli<W>,
    program: &[GateOp],
) -> Result<LogicalPauli<W>, QuantumError> {
    let n = pauli.len();
    let degree = pauli.x().degree();
    let mut x = alloc::vec![false; n];
    let mut z = alloc::vec![false; n];
    for q in pauli.x().support() {
        x[q] = true;
    }
    for q in pauli.z().support() {
        z[q] = true;
    }

    for (position, gate) in program.iter().enumerate() {
        if let Some(bad) = gate.qubits().into_iter().find(|&q| q >= n) {
            return Err(QuantumError::DimensionMismatch(alloc::format!(
                "gate {gate:?} at position {position} names qubit {bad}, beyond a {n}-qubit register"
            )));
        }
        match gate {
            GateOp::H(q) => core::mem::swap(&mut x[*q], &mut z[*q]),
            GateOp::S(q) | GateOp::Sdg(q) => z[*q] ^= x[*q],
            GateOp::X(_) | GateOp::Y(_) | GateOp::Z(_) => {}
            GateOp::Cnot { control, target } => {
                x[*target] ^= x[*control];
                z[*control] ^= z[*target];
            }
            GateOp::Cz { control, target } => {
                let (xa, xb) = (x[*control], x[*target]);
                z[*target] ^= xa;
                z[*control] ^= xb;
            }
            GateOp::Cmz { qubits } => match qubits.as_slice() {
                [] | [_] => {}
                [a, b] => {
                    let (xa, xb) = (x[*a], x[*b]);
                    z[*b] ^= xa;
                    z[*a] ^= xb;
                }
                _ => {
                    return Err(QuantumError::NonCliffordGate(alloc::format!(
                        "{gate:?} at position {position}: C^{}Z is not Clifford",
                        qubits.len() - 1
                    )));
                }
            },
            GateOp::T(_) | GateOp::Tdg(_) | GateOp::Csdg { .. } | GateOp::Ccz { .. } => {
                return Err(QuantumError::NonCliffordGate(alloc::format!(
                    "{gate:?} at position {position} is not Clifford"
                )));
            }
        }
    }

    let support = |bits: &[bool]| -> Vec<usize> {
        bits.iter()
            .enumerate()
            .filter_map(|(i, &b)| b.then_some(i))
            .collect()
    };
    let to_chain = |bits: &[bool]| {
        Gf2Chain::from_support(n, degree, &support(bits))
            .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))
    };
    LogicalPauli::new(to_chain(&x)?, to_chain(&z)?)
}

/// What a Clifford-action check examined and concluded.
///
/// No margin, for the reason the class-invariance report gives none: the question is logical
/// equivalence over 𝔽₂, which is exact, and a failure is an obstruction rather than a distance.
/// The two images are carried so that a failure can be read: they are the Paulis the program
/// actually produced, and they name what the gate did instead of what was claimed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliffordAction<W> {
    /// Whether `Z̄(γ)` maps to `X̄(γ̃)` and `X̄(γ̃)` to `Z̄(γ)`, both up to phase and stabilizers.
    pub holds: bool,
    /// How many gates were pushed through.
    pub gates_applied: usize,
    /// The image of `Z̄(γ)` under the program.
    pub z_image: LogicalPauli<W>,
    /// The image of `X̄(γ̃)` under the program.
    pub x_image: LogicalPauli<W>,
    /// Whether the image of `Z̄(γ)` is logically equivalent to `X̄(γ̃)`.
    pub z_to_x: bool,
    /// Whether the image of `X̄(γ̃)` is logically equivalent to `Z̄(γ)`.
    pub x_to_z: bool,
}

impl<W: NaturalNumber> LogicalBasis<W> {
    /// Whether a Clifford program acts as the logical Hadamard on the qubit `(γ, γ̃)`:
    /// `Z̄(γ) ↦ X̄(γ̃)` and `X̄(γ̃) ↦ Z̄(γ)`, up to phase and up to stabilizers.
    ///
    /// This is the check Haruna names for `H̄` in §3.2, "checking its commutation relations with
    /// the logical X and Z operators", and it needs `γ̃` dual to `γ`: `⟨γ, γ̃⟩ = 1`. Pushing
    /// `Z̄(γ)` through Eq. (3.32) by hand shows why. At the middle `S̄(γ̃)` every `z_j`, `j ∈ γ̃`,
    /// flips by `|γ ∩ γ̃| mod 2`, and at the last `S̄(γ)` every `z_j`, `j ∈ γ`, flips by the same
    /// count; with the pairing one the result is exactly `(γ̃, 0) = X̄(γ̃)`, and with the pairing
    /// zero it is not. A `γ̃` pairing to zero with every other homology generator additionally
    /// leaves the other logical qubits fixed, which is what makes `H̄` a gate on one qubit.
    ///
    /// The images are decided by [`are_logically_equivalent`](Self::are_logically_equivalent), so
    /// a program that lands on `X̄(γ̃)` times a stabilizer passes, and a program that leaves the
    /// code space fails with [`QuantumError::NotInNormalizer`] rather than passing vacuously.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] on a register-width mismatch;
    /// [`QuantumError::CalculationError`] if `⟨γ, γ̃⟩ = 0`, since then the question is not
    /// well-posed; [`QuantumError::NonCliffordGate`] if the program is not Clifford; and
    /// [`QuantumError::NotInNormalizer`] if an image leaves the code space.
    pub fn check_clifford_action(
        &self,
        program: &[GateOp],
        gamma: &Gf2Chain<W>,
        gamma_tilde: &Gf2Chain<W>,
    ) -> Result<CliffordAction<W>, QuantumError> {
        let n = self.len();
        if gamma.len() != n || gamma_tilde.len() != n {
            return Err(QuantumError::DimensionMismatch(alloc::format!(
                "the chains are over {} and {} qubits, the code over {}",
                gamma.len(),
                gamma_tilde.len(),
                n
            )));
        }
        let pairing = gamma
            .inner(gamma_tilde)
            .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))?;
        if pairing != Gf2::ONE {
            return Err(QuantumError::CalculationError(
                "check_clifford_action needs ⟨γ, γ̃⟩ = 1: the cochain is not dual to the chain"
                    .into(),
            ));
        }

        let zero = Gf2Chain::zeros(n, gamma.degree());
        let z_bar = LogicalPauli::new(zero.clone(), gamma.clone())?;
        let x_bar = LogicalPauli::new(gamma_tilde.clone(), zero)?;

        let z_image = clifford_conjugate(&z_bar, program)?;
        let x_image = clifford_conjugate(&x_bar, program)?;
        let z_to_x = self.are_logically_equivalent(&z_image, &x_bar)?;
        let x_to_z = self.are_logically_equivalent(&x_image, &z_bar)?;

        Ok(CliffordAction {
            holds: z_to_x && x_to_z,
            gates_applied: program.len(),
            z_image,
            x_image,
            z_to_x,
            x_to_z,
        })
    }
}
