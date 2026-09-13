/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The exact semantics functor: a program as layers a Pauli can be pushed through without a matrix.
//!
//! A Clifford layer acts on a Pauli by the symplectic tableau rule of `clifford_conjugate`. A
//! diagonal layer is a [`GaugeFieldGate`], a function of the logical parities of its blocks
//! (Haruna, arXiv:2511.15224, Eq. 3.63), and conjugating a Pauli through it leaves the Pauli in
//! place and multiplies a remainder that is again a `GaugeFieldGate` on the same blocks. Neither
//! rule forms an operator, so neither has a register-width limit.
//!
//! The one program shape with no normal form of polynomial size known here is a non-diagonal
//! Clifford layer applied after a non-constant remainder: `H · exp(iπ/4 Z̄) · H` is not diagonal and
//! not a Pauli. That is refused by name rather than expanded.

use crate::QuantumError;
use crate::types::qcode::clifford_action::clifford_conjugate;
use crate::types::qcode::gauge_field_gate::GaugeFieldGate;
use crate::types::qcode::logical_pauli::LogicalPauli;
use crate::types::qpu::circuit::GateOp;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_num::NaturalNumber;

/// One layer of an exact program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExactLayer<W> {
    /// A Clifford gate program on the register, in application order.
    Clifford(Vec<GateOp>),
    /// A diagonal gate in the algebra of its blocks' logical `Z̄`s.
    Diagonal(GaugeFieldGate<W>),
}

/// A program the exact semantics can evaluate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExactProgram<W> {
    num_qubits: usize,
    layers: Vec<ExactLayer<W>>,
}

/// A Pauli pushed through an exact program: the Pauli it became and the diagonal remainder the
/// non-Clifford layers left, `U P U† = pauli · remainder` up to the remainder's own phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Propagated<W> {
    /// The Pauli part.
    pub pauli: LogicalPauli<W>,
    /// The diagonal remainder; constant exactly when nothing non-Clifford was crossed with a
    /// flipped parity.
    pub remainder: GaugeFieldGate<W>,
}

/// Whether a gate is Clifford.
pub fn is_clifford_gate(op: &GateOp) -> bool {
    match op {
        GateOp::H(_)
        | GateOp::X(_)
        | GateOp::Y(_)
        | GateOp::Z(_)
        | GateOp::S(_)
        | GateOp::Sdg(_)
        | GateOp::Cnot { .. }
        | GateOp::Cz { .. } => true,
        GateOp::Cmz { qubits } => qubits.len() <= 2,
        GateOp::T(_) | GateOp::Tdg(_) | GateOp::Csdg { .. } | GateOp::Ccz { .. } => false,
    }
}

/// Whether a gate is diagonal in the computational basis.
pub fn is_diagonal_gate(op: &GateOp) -> bool {
    matches!(
        op,
        GateOp::Z(_)
            | GateOp::S(_)
            | GateOp::Sdg(_)
            | GateOp::T(_)
            | GateOp::Tdg(_)
            | GateOp::Cz { .. }
            | GateOp::Csdg { .. }
            | GateOp::Ccz { .. }
            | GateOp::Cmz { .. }
    )
}

impl<W: NaturalNumber> ExactProgram<W> {
    /// A program from its layers over `num_qubits` qubits.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonCliffordGate`] if a Clifford layer holds a non-Clifford gate;
    /// [`QuantumError::DimensionMismatch`] if a gate names a qubit beyond the register, names no
    /// qubit or a qubit more than once, or a diagonal layer's blocks are over another register.
    pub fn new(num_qubits: usize, layers: Vec<ExactLayer<W>>) -> Result<Self, QuantumError> {
        for (i, layer) in layers.iter().enumerate() {
            match layer {
                ExactLayer::Clifford(ops) => {
                    for op in ops {
                        if !is_clifford_gate(op) {
                            return Err(QuantumError::NonCliffordGate(format!(
                                "layer {i}: {op:?} is not Clifford; carry it as a diagonal layer"
                            )));
                        }
                        let mut qubits = op.qubits();
                        if qubits.is_empty() {
                            return Err(QuantumError::DimensionMismatch(format!(
                                "layer {i}: {op:?} names no qubit"
                            )));
                        }
                        if let Some(q) = qubits.iter().copied().find(|&q| q >= num_qubits) {
                            return Err(QuantumError::DimensionMismatch(format!(
                                "layer {i}: {op:?} names qubit {q} on a {num_qubits}-qubit register"
                            )));
                        }
                        let count = qubits.len();
                        qubits.sort_unstable();
                        qubits.dedup();
                        if qubits.len() != count {
                            return Err(QuantumError::DimensionMismatch(format!(
                                "layer {i}: {op:?} names a qubit more than once"
                            )));
                        }
                    }
                }
                ExactLayer::Diagonal(g) => {
                    if g.len() != num_qubits {
                        return Err(QuantumError::DimensionMismatch(format!(
                            "layer {i}: a diagonal gate over {} qubits on a {num_qubits}-qubit register",
                            g.len()
                        )));
                    }
                }
            }
        }
        Ok(Self { num_qubits, layers })
    }

    /// A one-layer Clifford program.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new).
    pub fn clifford(num_qubits: usize, ops: Vec<GateOp>) -> Result<Self, QuantumError> {
        Self::new(num_qubits, alloc::vec![ExactLayer::Clifford(ops)])
    }

    /// A one-layer diagonal program.
    pub fn diagonal(gate: GaugeFieldGate<W>) -> Self {
        Self {
            num_qubits: gate.len(),
            layers: alloc::vec![ExactLayer::Diagonal(gate)],
        }
    }

    /// The register width.
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// The layers, in application order.
    pub fn layers(&self) -> &[ExactLayer<W>] {
        &self.layers
    }

    /// Whether every layer is diagonal, so the whole program is one `GaugeFieldGate`.
    pub fn is_diagonal(&self) -> bool {
        self.layers.iter().all(|l| match l {
            ExactLayer::Clifford(ops) => ops.iter().all(is_diagonal_gate),
            ExactLayer::Diagonal(_) => true,
        })
    }

    /// The product of the diagonal layers as one gate, when the program is diagonal.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if a layer is not diagonal, or if the blocks' union
    /// exceeds the gate's block limit.
    pub fn as_gauge_field_gate(&self) -> Result<GaugeFieldGate<W>, QuantumError> {
        let mut acc = GaugeFieldGate::identity(self.num_qubits, 1);
        for (i, layer) in self.layers.iter().enumerate() {
            match layer {
                ExactLayer::Diagonal(g) => acc = acc.product(g)?,
                ExactLayer::Clifford(ops) => {
                    let g = GaugeFieldGate::from_diagonal_clifford(self.num_qubits, ops).map_err(
                        |e| {
                            QuantumError::CalculationError(format!(
                                "layer {i} is not a diagonal program: {e}"
                            ))
                        },
                    )?;
                    acc = acc.product(&g)?;
                }
            }
        }
        Ok(acc)
    }

    /// Pushes a Pauli through the program.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NoPropagationNormalForm`] when a non-diagonal Clifford layer follows a
    /// non-constant remainder; [`QuantumError::DimensionMismatch`] on a register mismatch; and the
    /// errors of `clifford_conjugate`.
    pub fn conjugate(&self, pauli: &LogicalPauli<W>) -> Result<Propagated<W>, QuantumError> {
        if pauli.len() != self.num_qubits {
            return Err(QuantumError::DimensionMismatch(format!(
                "the Pauli is over {} qubits, the program over {}",
                pauli.len(),
                self.num_qubits
            )));
        }
        let mut current = pauli.clone();
        let mut remainder = GaugeFieldGate::identity(self.num_qubits, pauli.x().degree());
        let mut last_non_clifford: Option<usize> = None;
        for (i, layer) in self.layers.iter().enumerate() {
            match layer {
                ExactLayer::Clifford(ops) => {
                    if !remainder.is_constant() && !ops.iter().all(is_diagonal_gate) {
                        return Err(QuantumError::NoPropagationNormalForm(
                            i,
                            last_non_clifford.unwrap_or(i),
                        ));
                    }
                    current = clifford_conjugate(&current, ops)?;
                }
                ExactLayer::Diagonal(g) => {
                    let r = g.conjugated_by_pauli(current.x())?;
                    if !r.is_constant() {
                        last_non_clifford = Some(i);
                    }
                    remainder = remainder.product(&r)?;
                }
            }
        }
        Ok(Propagated {
            pauli: current,
            remainder,
        })
    }
}
