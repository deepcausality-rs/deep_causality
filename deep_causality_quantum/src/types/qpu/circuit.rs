/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A reified quantum circuit: pure, storable data carrying no `HilbertState`
//! and no amplitudes (R2). The emergent-modality seam takes a `QuantumCircuit`
//! as inert input so both an in-process simulator and a future cloud adapter
//! satisfy the same `QpuSampler` trait.

use crate::QuantumError;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

/// A single reified gate over the migrated gate alphabet. Plain data — no
/// function pointers, no amplitudes.
///
/// # The diagonal family
///
/// `Z`, `S`, `Sdg`, `T`, `Tdg`, `Cz`, `Csdg`, `Ccz` and `Cmz` are all diagonal
/// in the computational basis: each multiplies an amplitude by a fixed phase
/// when every qubit it names is set, and leaves the amplitude alone otherwise.
/// One simulator kernel serves the whole family. `H`, `X`, `Y` and `Cnot` move
/// amplitude between basis states and do not.
///
/// # Arity
///
/// [`Cmz`](GateOp::Cmz) is the general `C^{m-1}Z` over a symmetric qubit list.
/// [`Cz`](GateOp::Cz) and [`Ccz`](GateOp::Ccz) are its two- and three-qubit
/// cases, kept as fixed-arity variants because they carry no allocation and
/// clone without one. Prefer them where the arity is known.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateOp {
    /// Hadamard on a qubit.
    H(usize),
    /// Pauli-X (bit flip).
    X(usize),
    /// Pauli-Y.
    Y(usize),
    /// Pauli-Z (phase flip), `diag(1, -1)`.
    Z(usize),
    /// Phase gate `S = diag(1, i)`.
    S(usize),
    /// `S† = diag(1, -i)`, the adjoint of [`S`](GateOp::S).
    Sdg(usize),
    /// `T = diag(1, e^{iπ/4})`.
    T(usize),
    /// `T† = diag(1, e^{-iπ/4})`, the adjoint of [`T`](GateOp::T).
    Tdg(usize),
    /// Controlled-NOT.
    Cnot { control: usize, target: usize },
    /// Controlled-Z, `diag(1, 1, 1, -1)`. Symmetric in its two qubits.
    Cz { control: usize, target: usize },
    /// Controlled-`S†`, `diag(1, 1, 1, -i)`. Symmetric in its two qubits.
    Csdg { control: usize, target: usize },
    /// Doubly-controlled Z, `diag(1, 1, 1, 1, 1, 1, 1, -1)`. Symmetric in all
    /// three qubits, so the field names carry no control/target distinction.
    Ccz { q0: usize, q1: usize, q2: usize },
    /// `C^{m-1}Z` over `m = qubits.len()` qubits: negates the amplitude of the
    /// single basis state with every named qubit set. Symmetric in the list,
    /// which must be non-empty and free of repeats.
    Cmz { qubits: Vec<usize> },
}

impl GateOp {
    /// The qubit indices this gate touches.
    ///
    /// For the symmetric multi-qubit gates the order is the order the variant
    /// stores; nothing downstream reads position as meaning.
    pub fn qubits(&self) -> Vec<usize> {
        match self {
            GateOp::H(q)
            | GateOp::X(q)
            | GateOp::Y(q)
            | GateOp::Z(q)
            | GateOp::S(q)
            | GateOp::Sdg(q)
            | GateOp::T(q)
            | GateOp::Tdg(q) => vec![*q],
            GateOp::Cnot { control, target }
            | GateOp::Cz { control, target }
            | GateOp::Csdg { control, target } => {
                vec![*control, *target]
            }
            GateOp::Ccz { q0, q1, q2 } => vec![*q0, *q1, *q2],
            GateOp::Cmz { qubits } => qubits.clone(),
        }
    }
}

/// A storable circuit: a `num_qubits` register, an ordered gate program, and a
/// computational-basis measurement over a subset of qubits. `Clone + Debug +
/// PartialEq`; carries no amplitudes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantumCircuit {
    num_qubits: usize,
    ops: Vec<GateOp>,
    measure: Vec<usize>,
}

impl QuantumCircuit {
    /// Builds a circuit, rejecting any out-of-range or repeated qubit index
    /// within a gate with a typed [`QuantumError`]. The `measure` list names the
    /// qubits read out in the computational basis (its order fixes the outcome
    /// bit order, LSB first).
    pub fn new(
        num_qubits: usize,
        ops: Vec<GateOp>,
        measure: Vec<usize>,
    ) -> Result<Self, QuantumError> {
        if num_qubits == 0 {
            return Err(QuantumError::DimensionMismatch(
                "a circuit needs at least one qubit".into(),
            ));
        }
        for op in &ops {
            let qs = op.qubits();
            for &q in &qs {
                if q >= num_qubits {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "gate {:?} references qubit {} ≥ num_qubits {}",
                        op, q, num_qubits
                    )));
                }
            }
            // A gate that names the same qubit twice is rejected at every
            // arity, not just at two. A repeat in `Ccz` or `Cmz` would reduce
            // the gate's control set silently: `Cmz{[0, 0, 1]}` acts as
            // `Cz{0, 1}`, which is a different gate from the one written. The
            // scan is quadratic in a list bounded by the register width.
            for (i, &a) in qs.iter().enumerate() {
                if qs[..i].contains(&a) {
                    return Err(QuantumError::DimensionMismatch(format!(
                        "gate {:?} names qubit {} more than once",
                        op, a
                    )));
                }
            }
            // An empty control list phases the whole register by -1, a global
            // phase with no observable effect and no way to express intent.
            if qs.is_empty() {
                return Err(QuantumError::DimensionMismatch(format!(
                    "gate {:?} names no qubits",
                    op
                )));
            }
        }
        for (idx, &m) in measure.iter().enumerate() {
            if m >= num_qubits {
                return Err(QuantumError::DimensionMismatch(format!(
                    "measurement references qubit {} ≥ num_qubits {}",
                    m, num_qubits
                )));
            }
            // Each measured qubit maps to exactly one outcome bit; a repeat both
            // inflates the outcome table (2^len) beyond the register and makes the
            // bit order ambiguous. Reject duplicates here so the sampler is safe
            // by construction.
            if measure[..idx].contains(&m) {
                return Err(QuantumError::DimensionMismatch(format!(
                    "measurement qubit {} is measured more than once",
                    m
                )));
            }
        }
        Ok(Self {
            num_qubits,
            ops,
            measure,
        })
    }

    /// The register width.
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// The ordered gate program.
    pub fn ops(&self) -> &[GateOp] {
        &self.ops
    }

    /// The measured qubits (outcome bit order, LSB first).
    pub fn measure(&self) -> &[usize] {
        &self.measure
    }
}
