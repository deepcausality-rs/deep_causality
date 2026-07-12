/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A reified quantum circuit: pure, storable data carrying no `HilbertState`
//! and no amplitudes (R2). The emergent-modality seam takes a `QuantumCircuit`
//! as inert input so both an in-process simulator and a future cloud adapter
//! satisfy the same `QpuSampler` trait.

use crate::QuantumError;

/// A single reified gate over the migrated gate alphabet. Plain data — no
/// function pointers, no amplitudes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateOp {
    /// Hadamard on a qubit.
    H(usize),
    /// Pauli-X (bit flip).
    X(usize),
    /// Pauli-Y.
    Y(usize),
    /// Pauli-Z (phase flip).
    Z(usize),
    /// Phase gate `S = diag(1, i)`.
    S(usize),
    /// `T = diag(1, e^{iπ/4})`.
    T(usize),
    /// Controlled-NOT.
    Cnot { control: usize, target: usize },
    /// Controlled-Z.
    Cz { control: usize, target: usize },
}

impl GateOp {
    /// The qubit indices this gate touches.
    pub fn qubits(&self) -> Vec<usize> {
        match *self {
            GateOp::H(q)
            | GateOp::X(q)
            | GateOp::Y(q)
            | GateOp::Z(q)
            | GateOp::S(q)
            | GateOp::T(q) => vec![q],
            GateOp::Cnot { control, target } | GateOp::Cz { control, target } => {
                vec![control, target]
            }
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
    /// Builds a circuit, rejecting any out-of-range or (for two-qubit gates)
    /// coincident qubit index with a typed [`QuantumError`]. The `measure` list
    /// names the qubits read out in the computational basis (its order fixes the
    /// outcome bit order, LSB first).
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
            if qs.len() == 2 && qs[0] == qs[1] {
                return Err(QuantumError::DimensionMismatch(format!(
                    "two-qubit gate {:?} has coincident control/target",
                    op
                )));
            }
        }
        for &m in &measure {
            if m >= num_qubits {
                return Err(QuantumError::DimensionMismatch(format!(
                    "measurement references qubit {} ≥ num_qubits {}",
                    m, num_qubits
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
