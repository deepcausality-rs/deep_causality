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
//!
//! # One qubit, not one pair
//!
//! The pair check reads two images and nothing else, and Haruna's derivation goes through for any
//! `γ̃` with `⟨γ, γ̃⟩ = 1`. So a `γ̃` that also meets a second homology generator builds a program
//! that passes the pair check while acting on the second logical qubit. On a code with more than
//! one logical qubit, `H̄` on one of them is decided by
//! [`LogicalBasis::check_clifford_action_on_qubit`] against a symplectic dual basis from
//! [`symplectic_dual_basis`], which requires the other logical generators fixed.

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
    /// Whether `Z̄(γ)` maps to `X̄(γ̃)` and `X̄(γ̃)` to `Z̄(γ)`, both up to phase and stabilizers,
    /// and every other logical generator examined came back logically equivalent to itself.
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
    /// How many logical generators of the other qubits were pushed through the program: `Z̄(γ_j)`
    /// and `X̄(γ̃_j)` for each `j` other than the qubit under test, up to the first one that moved.
    /// Zero when the check decided the pair alone, so a pass that examined no other qubit is
    /// visible as one.
    pub others_examined: usize,
    /// Whether every other logical generator examined came back logically equivalent to itself.
    pub others_fixed: bool,
}

/// A symplectic dual basis: one cocycle `γ̃_j` per homology generator, with `⟨γ_i, γ̃_j⟩ = δ_ij`.
///
/// A cohomology basis from an elimination pairs with the homology basis through some invertible
/// matrix `M[i][j] = ⟨γ_i, c_j⟩`, not through the identity. A cocycle dual to `γ_i` that also
/// meets `γ_j` builds a [`logical_hadamard`](crate::logical_hadamard) that acts on qubit `j` as
/// well, and the pair check cannot see that. Inverting `M` over 𝔽₂ and setting
/// `γ̃_j = Σ_i (M⁻¹)[i][j] · c_i` gives `⟨γ_a, γ̃_j⟩ = Σ_i M[a][i] (M⁻¹)[i][j] = δ_aj`, so each dual
/// meets its own generator once and the others not at all. Each `γ̃_j` is an 𝔽₂ sum of cocycles,
/// so it is a cocycle.
///
/// `M` is `k × k` for `k` logical qubits, so the elimination is on a boolean matrix of that size
/// and nothing here grows with the register.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the two bases differ in count, or if a chain is over a
/// different register or degree from the others; [`QuantumError::CalculationError`] if `M` is
/// singular, which is a degenerate pairing. The homology and cohomology of one complex over a
/// field pair perfectly, so on a [`LogicalBasis`] this path is not reached.
pub fn symplectic_dual_basis<W: NaturalNumber>(
    homology: &[Gf2Chain<W>],
    cohomology: &[Gf2Chain<W>],
) -> Result<Vec<Gf2Chain<W>>, QuantumError> {
    let k = homology.len();
    if cohomology.len() != k {
        return Err(QuantumError::DimensionMismatch(alloc::format!(
            "{k} homology generators against {} cohomology generators: the pairing is not square",
            cohomology.len()
        )));
    }
    // `[M | I]`, taken to `[I | M⁻¹]` by Gauss-Jordan over 𝔽₂, where a row operation is an XOR.
    let mut rows: Vec<Vec<bool>> = Vec::with_capacity(k);
    for (i, gamma) in homology.iter().enumerate() {
        let mut row = alloc::vec![false; 2 * k];
        for (j, c) in cohomology.iter().enumerate() {
            row[j] = gamma
                .inner(c)
                .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))?
                .bit();
        }
        row[k + i] = true;
        rows.push(row);
    }
    for col in 0..k {
        let Some(pivot) = (col..k).find(|&r| rows[r][col]) else {
            return Err(QuantumError::CalculationError(alloc::format!(
                "the pairing matrix ⟨γ_i, c_j⟩ over the {k} logical qubits is singular, of rank \
                 {col}: no cocycle is dual to one generator alone, so no symplectic dual basis \
                 exists"
            )));
        };
        rows.swap(col, pivot);
        let pivot_row = rows[col].clone();
        for (r, row) in rows.iter_mut().enumerate() {
            if r != col && row[col] {
                for (a, b) in row.iter_mut().zip(&pivot_row) {
                    *a ^= *b;
                }
            }
        }
    }
    // `γ̃_j = Σ_i (M⁻¹)[i][j] · c_i`, reading the inverse down its columns.
    (0..k)
        .map(|j| {
            let zero = Gf2Chain::zeros(cohomology[j].len(), cohomology[j].degree());
            rows.iter()
                .enumerate()
                .filter(|(_, row)| row[k + j])
                .try_fold(zero, |acc, (i, _)| acc.add(&cohomology[i]))
                .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))
        })
        .collect()
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
    /// # This decides the pair alone
    ///
    /// Nothing about the other logical qubits is examined: `others_examined` is zero and
    /// `others_fixed` is true by default. The derivation above holds for any `γ̃` with
    /// `⟨γ, γ̃⟩ = 1`, so a `γ̃` that also meets another homology generator builds a program that
    /// passes here while moving that generator. On a code with more than one logical qubit,
    /// [`check_clifford_action_on_qubit`](Self::check_clifford_action_on_qubit) is the check that
    /// decides a gate on one qubit.
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
            others_examined: 0,
            others_fixed: true,
        })
    }

    /// Whether a Clifford program acts as the logical Hadamard on logical qubit `index` and on
    /// that qubit alone, against a symplectic dual basis `duals` with `⟨γ_i, duals[j]⟩ = δ_ij`.
    ///
    /// The pair check decides two images and nothing else, and a dual meeting a second generator
    /// `γ_j` builds a program that passes it while moving `Z̄(γ_j)`. Pushing `Z̄(γ_j)` through
    /// Eq. (3.32) by hand: the middle `S̄(γ̃)` attaches `γ̃` because `⟨γ_j, γ̃⟩ = 1`, and the last
    /// `S̄(γ)` attaches `γ` because `⟨γ̃, γ⟩ = 1`, so the image is `X̄(γ̃) Z̄(γ) Z̄(γ_j)` up to
    /// phase, which is a gate on two qubits reported as a gate on one. This form closes that gap in
    /// two steps. It refuses a `duals` that does not pair with the homology basis as the identity,
    /// which makes the emitted `H̄` a gate on one qubit. It then pushes `Z̄(γ_j)` and `X̄(γ̃_j)` for
    /// every `j ≠ index` through the program and requires each to come back logically equivalent to
    /// itself, which decides the same thing for an arbitrary program. The loop stops at the first
    /// generator that moved, so `others_examined` counts what was actually visited.
    ///
    /// [`symplectic_dual_basis`] builds a `duals` satisfying the precondition from
    /// [`homology`](Self::homology) and [`cohomology`](Self::cohomology).
    /// The public method validates the complete `k × k` pairing matrix on every call, then
    /// checks at most `2(k - 1)` other logical Paulis. Pipeline callers should therefore call it
    /// once per logical qubit only when they need each single-qubit action certified.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if `index` names no logical qubit or `duals` has a count
    /// other than the number of logical qubits; [`QuantumError::CalculationError`] naming the first
    /// pairing `⟨γ_i, γ̃_j⟩` that is not `δ_ij`; and otherwise as
    /// [`check_clifford_action`](Self::check_clifford_action).
    pub fn check_clifford_action_on_qubit(
        &self,
        program: &[GateOp],
        index: usize,
        duals: &[Gf2Chain<W>],
    ) -> Result<CliffordAction<W>, QuantumError> {
        let k = self.num_logical_qubits();
        if duals.len() != k {
            return Err(QuantumError::DimensionMismatch(alloc::format!(
                "{} duals for {k} logical qubits",
                duals.len()
            )));
        }
        let Some(gamma) = self.homology().get(index) else {
            return Err(QuantumError::DimensionMismatch(alloc::format!(
                "logical qubit {index} does not exist: the code encodes {k}"
            )));
        };
        for (i, g) in self.homology().iter().enumerate() {
            for (j, d) in duals.iter().enumerate() {
                let pairing = g
                    .inner(d)
                    .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))?;
                if pairing != Gf2::new(i == j) {
                    return Err(QuantumError::CalculationError(alloc::format!(
                        "⟨γ_{i}, γ̃_{j}⟩ = {pairing} where a symplectic dual basis has {}: a dual \
                         meeting another generator builds a gate on more than one qubit",
                        Gf2::new(i == j)
                    )));
                }
            }
        }

        let pair = self.check_clifford_action(program, gamma, &duals[index])?;

        let zero = Gf2Chain::zeros(self.len(), gamma.degree());
        // Pairings establish the logical coordinate, not preservation of the code space. A
        // caller can supply an arbitrary chain with the right pairings, so validate the X
        // representative itself before equivalence can cancel it against an image.
        let x_target = LogicalPauli::new(duals[index].clone(), zero.clone())?;
        let _ = self.is_logically_trivial(&x_target)?;
        let mut others_examined = 0;
        let mut others_fixed = true;
        'others: for j in (0..k).filter(|&j| j != index) {
            let z_other = LogicalPauli::new(zero.clone(), self.homology()[j].clone())?;
            let x_other = LogicalPauli::new(duals[j].clone(), zero.clone())?;
            for op in [z_other, x_other] {
                let image = clifford_conjugate(&op, program)?;
                others_examined += 1;
                if !self.are_logically_equivalent(&image, &op)? {
                    others_fixed = false;
                    break 'others;
                }
            }
        }

        Ok(CliffordAction {
            holds: pair.z_to_x && pair.x_to_z && others_fixed,
            others_examined,
            others_fixed,
            ..pair
        })
    }
}
