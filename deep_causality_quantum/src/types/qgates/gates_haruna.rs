/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Haruna's gauge-field formalism for logical quantum gates, as circuits.
//!
//! Junichi Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum
//! Error Correction*, arXiv:2511.15224 (2025). The paper is in this crate's
//! `papers/` directory. Table 1 gives the physical-gate decomposition of each
//! logical gate; §3 derives them.
//!
//! # The carrier is a bitset, not a multivector
//!
//! A logical operator here is supported on a set of physical qubits, and every
//! gate in Table 1 is a product of physical gates ranging over that support, its
//! pairs and its triples. So the input is a [`Gf2Chain`], whose `support`,
//! `support_pairs` and `support_triples` are exactly the three index families
//! the table needs, and the output is a list of [`GateOp`].
//!
//! Nothing here evaluates an operator. `a(γ)` is diagonal with integer
//! eigenvalues, so the gauge-field expressions in Table 1's third column are the
//! compact forms that make the Appendix B invariance proofs tractable; they are
//! not the computational path. This module implements the second column.
//!
//! # Degree
//!
//! Every chain a gate takes is a 1-chain in the paper's construction: `γ ∈ H₁`
//! for the electric side and `γ̃ ∈ H¹` for the magnetic one. The degree is
//! carried but not checked here, because a caller composing gates over a code
//! whose grading differs is doing something this module cannot adjudicate.

use crate::QuantumError;
use crate::types::qpu::circuit::GateOp;
use alloc::vec::Vec;
use core::f64::consts::PI;
use deep_causality_algebra::RealField;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::{FromPrimitive, NaturalNumber};
use deep_causality_num_complex::Complex;

/// The logical Z gate, Table 1 row 1: `Z̄(γ) = ∏_k Z_{i_k}` over `supp(γ)`.
///
/// Transversal Z on the support and nothing else.
pub fn logical_z<W: NaturalNumber>(gamma: &Gf2Chain<W>) -> Vec<GateOp> {
    gamma.support().map(GateOp::Z).collect()
}

/// The logical X gate, Table 1 row 2: `X̄(γ̃) = ∏_k X_{ĩ_k}` over `supp(γ̃)`.
///
/// Transversal X on the support and nothing else.
pub fn logical_x<W: NaturalNumber>(gamma_tilde: &Gf2Chain<W>) -> Vec<GateOp> {
    gamma_tilde.support().map(GateOp::X).collect()
}

/// The logical S gate, Table 1 row 3 and Eq. (3.17):
///
/// `S̄(γ) = ∏_{k} S_{j_k} · ∏_{k₁<k₂} CZ_{j_{k₁} j_{k₂}}`
///
/// Transversal S on the support, then CZ between every pair within it. The
/// pairs are unordered and drawn from a single support, so no index repeats and
/// the paper's `CZ_{i,i} = Z_i` reduction never fires here.
pub fn logical_s<W: NaturalNumber>(gamma: &Gf2Chain<W>) -> Vec<GateOp> {
    let mut ops: Vec<GateOp> = gamma.support().map(GateOp::S).collect();
    ops.extend(gamma.support_pairs().map(|(a, b)| GateOp::Cz {
        control: a,
        target: b,
    }));
    ops
}

/// The logical T gate, Table 1 row 7 and Eq. (3.59):
///
/// `T̄(γ) = ∏_k T_{i_k} · ∏_{k₁<k₂} CS†_{k₁k₂} · ∏_{k₁<k₂<k₃} CCZ_{k₁k₂k₃}`
///
/// Transversal T on the support, controlled-S† between every pair within it,
/// and CCZ among every triple. As with [`logical_s`], every index family is
/// drawn from one support in strictly increasing order, so no index repeats.
pub fn logical_t<W: NaturalNumber>(gamma: &Gf2Chain<W>) -> Vec<GateOp> {
    let mut ops: Vec<GateOp> = gamma.support().map(GateOp::T).collect();
    ops.extend(gamma.support_pairs().map(|(a, b)| GateOp::Csdg {
        control: a,
        target: b,
    }));
    ops.extend(gamma.support_triples().map(|(a, b, c)| GateOp::Ccz {
        q0: a,
        q1: b,
        q2: c,
    }));
    ops
}

/// The logical CZ gate, Table 1 row 5 and Eq. (3.42):
///
/// `CZ̄(γ₁, γ₂) = ∏_{k,l} CZ_{i_k, j_l}` over `supp(γ₁) × supp(γ₂)`
///
/// The full Cartesian product, not the ordered pairs: the two supports are
/// independent and every combination contributes.
///
/// # The supports may overlap, and then the paper's reduction fires
///
/// When `i_k == j_l` the physical gate is `CZ_{i,i}`, which the paper defines as
/// `exp(iπ z_i z_i) = exp(iπ z_i) = Z_i`, a single-qubit Z. This function emits
/// that `Z_i`. Emitting a `CZ` with both indices equal instead would be rejected
/// by [`QuantumCircuit::new`](crate::QuantumCircuit::new), which refuses a gate
/// naming one qubit twice, so the reduction is not an optimisation but the only
/// correct output.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the two chains have different lengths,
/// since then they index different registers and the product is not defined.
pub fn logical_cz<W: NaturalNumber>(
    gamma1: &Gf2Chain<W>,
    gamma2: &Gf2Chain<W>,
) -> Result<Vec<GateOp>, QuantumError> {
    if gamma1.len() != gamma2.len() {
        return Err(QuantumError::DimensionMismatch(alloc::format!(
            "logical_cz needs two chains over one register: lengths {} and {}",
            gamma1.len(),
            gamma2.len()
        )));
    }
    let right: Vec<usize> = gamma2.support().collect();
    let mut ops = Vec::new();
    for i in gamma1.support() {
        for &j in &right {
            if i == j {
                ops.push(GateOp::Z(i));
            } else {
                ops.push(GateOp::Cz {
                    control: i,
                    target: j,
                });
            }
        }
    }
    Ok(ops)
}

/// The logical multi-controlled Z, Table 1 row 6:
///
/// `C^{m-1}Z̄(γ₁, …, γ_m) = ∏_{k₁,…,k_m} C^{m-1}Z_{i₁,…,i_m}`
///
/// over the Cartesian product of the `m` supports.
///
/// # The reduction rule is load-bearing here
///
/// The paper defines `C^{m-1}Z` so that repeated indices reduce: coincident
/// controls drop to a lower-control gate, and a control coinciding with the
/// target drops to a single-qubit Z. Its examples are `C³Z_{i,i,j,k} = C²Z_{i,j,k}`
/// and `C²Z_{i,i,i} = CZ_{i,i} = Z_i`. Because the supports may overlap, a raw
/// tuple can carry repeats, so this function deduplicates each tuple before
/// emitting it. That is the paper's rule and also what
/// [`QuantumCircuit::new`](crate::QuantumCircuit::new) requires, since it
/// rejects a gate naming one qubit twice.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if `chains` is empty, or if the chains do
/// not all have the same length.
pub fn logical_multi_cz<W: NaturalNumber>(
    chains: &[&Gf2Chain<W>],
) -> Result<Vec<GateOp>, QuantumError> {
    let first = chains.first().ok_or_else(|| {
        QuantumError::DimensionMismatch("logical_multi_cz needs at least one chain".into())
    })?;
    if let Some(bad) = chains.iter().find(|c| c.len() != first.len()) {
        return Err(QuantumError::DimensionMismatch(alloc::format!(
            "logical_multi_cz needs chains over one register: lengths {} and {}",
            first.len(),
            bad.len()
        )));
    }

    let supports: Vec<Vec<usize>> = chains.iter().map(|c| c.support().collect()).collect();
    if supports.iter().any(|s| s.is_empty()) {
        return Ok(Vec::new());
    }

    let mut ops = Vec::new();
    let mut odometer = alloc::vec![0usize; supports.len()];
    loop {
        // One tuple of the Cartesian product, deduplicated per the paper's
        // reduction rule. `dedup` after sorting would lose the tuple's order,
        // which does not matter: C^{m-1}Z is symmetric in its indices.
        let mut tuple: Vec<usize> = Vec::with_capacity(odometer.len());
        for (axis, &pos) in odometer.iter().enumerate() {
            let q = supports[axis][pos];
            if !tuple.contains(&q) {
                tuple.push(q);
            }
        }
        ops.push(reduced_multi_cz(tuple));

        // Advance the odometer over the supports, least significant axis first.
        let mut axis = odometer.len();
        loop {
            if axis == 0 {
                return Ok(ops);
            }
            axis -= 1;
            odometer[axis] += 1;
            if odometer[axis] < supports[axis].len() {
                break;
            }
            odometer[axis] = 0;
        }
    }
}

/// The gate a deduplicated index tuple denotes, at the arity that survived the
/// paper's reduction. One index is a Z, two a CZ, three a CCZ, and more the
/// general form.
fn reduced_multi_cz(tuple: Vec<usize>) -> GateOp {
    match tuple.len() {
        1 => GateOp::Z(tuple[0]),
        2 => GateOp::Cz {
            control: tuple[0],
            target: tuple[1],
        },
        3 => GateOp::Ccz {
            q0: tuple[0],
            q1: tuple[1],
            q2: tuple[2],
        },
        _ => GateOp::Cmz { qubits: tuple },
    }
}

/// The logical Hadamard, Table 1 row 4 and Eq. (3.27):
///
/// `H̄(γ) = e^{-iπ/4} · S̄(γ) · ∏_k H_{ĩ_k} · S̄(γ̃) · ∏_k H_{ĩ_k} · S̄(γ)`
///
/// The transversal Hadamards run over `supp(γ̃)`, conjugating the middle `S̄(γ̃)`
/// into the X basis; the outer factors are `S̄(γ)` on the electric side.
///
/// # The global phase is returned, not discarded
///
/// `e^{-iπ/4}` is unobservable under a computational-basis measurement, so a
/// circuit type has nowhere to put it. It stops being unobservable the moment
/// this gate is used as a controlled operation, where a global phase becomes a
/// relative one, and the paper's Appendix B invariance arguments carry it. So it
/// comes back alongside the circuit and the caller decides.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if the two chains have different lengths.
/// [`QuantumError::CalculationError`] if the scalar type cannot represent `π/4`.
pub fn logical_hadamard<W, R>(
    gamma: &Gf2Chain<W>,
    gamma_tilde: &Gf2Chain<W>,
) -> Result<(Vec<GateOp>, Complex<R>), QuantumError>
where
    W: NaturalNumber,
    R: RealField + FromPrimitive,
{
    if gamma.len() != gamma_tilde.len() {
        return Err(QuantumError::DimensionMismatch(alloc::format!(
            "logical_hadamard needs two chains over one register: lengths {} and {}",
            gamma.len(),
            gamma_tilde.len()
        )));
    }
    let neg_pi_4 = R::from_f64(-PI / 4.0).ok_or_else(|| {
        QuantumError::CalculationError("scalar type cannot represent -π/4".into())
    })?;
    let phase = Complex::new(neg_pi_4.cos(), neg_pi_4.sin());

    let s_gamma = logical_s(gamma);
    let hadamards: Vec<GateOp> = gamma_tilde.support().map(GateOp::H).collect();

    let mut ops = Vec::new();
    ops.extend(s_gamma.iter().cloned());
    ops.extend(hadamards.iter().cloned());
    ops.extend(logical_s(gamma_tilde));
    ops.extend(hadamards);
    ops.extend(s_gamma);

    Ok((ops, phase))
}
