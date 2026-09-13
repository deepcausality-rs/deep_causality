/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::carriers::Channel;
use crate::types::circuit_model::wire::WireId;
use crate::types::qpu::circuit::GateOp;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// One box of a [`CircuitModel`](crate::CircuitModel): a morphism of QC (Lorenz & Tull,
/// arXiv:2602.16612, Figure 2) on the wires it names.
///
/// Quantum wires are register lines and every box acts on them in place, so a box's quantum wires
/// are both its inputs and its outputs. A classical wire is written by the one box that names it
/// as `outcome` and read by the encoders that name it as `input`.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBox<R: RealField> {
    /// Classical input to quantum: on reading value `x` from `input`, prepares `states[x]` on
    /// `outputs`, which must be fresh lines in `|0…0⟩`. Each state is a ket of dimension the
    /// product of the output dimensions.
    Encoder {
        /// The classical wire read.
        input: WireId,
        /// The quantum wires prepared, first most significant.
        outputs: Vec<WireId>,
        /// One ket per classical value.
        states: Vec<CausalTensor<Complex<R>>>,
    },
    /// A gate program on qubit wires; gate indices are local to `wires`.
    Unitary {
        /// The qubit wires, in the order the program's local indices refer to them.
        wires: Vec<WireId>,
        /// The program, in application order.
        program: Vec<GateOp>,
    },
    /// A CPTP map on quantum wires, the noise box.
    Channel {
        /// The quantum wires, first most significant.
        wires: Vec<WireId>,
        /// The channel, validated CPTP once at its construction.
        channel: Channel<R>,
    },
    /// A CPTP map on quantum wires given by its Kraus operators, each `2^|wires| × 2^|wires|`.
    /// The numeric semantics is Kraus-level, so a wide box, an encoder unitary on every qubit of a
    /// code for one, is held here without the Choi operator a [`Channel`] carries.
    Kraus {
        /// The quantum wires, first most significant.
        wires: Vec<WireId>,
        /// The Kraus operators.
        kraus: Vec<CausalTensor<Complex<R>>>,
    },
    /// A quantum instrument: for each classical outcome `y`, a Kraus family `kraus[y]` on the
    /// wires; jointly trace-preserving over `y`.
    Instrument {
        /// The quantum wires, first most significant.
        wires: Vec<WireId>,
        /// The classical wire written with the outcome.
        outcome: WireId,
        /// One Kraus family per outcome.
        kraus: Vec<Vec<CausalTensor<Complex<R>>>>,
    },
    /// A computational-basis measurement of the wires, writing the outcome index (first wire most
    /// significant) and leaving the wires in `|0…0⟩`.
    Measurement {
        /// The quantum wires measured.
        wires: Vec<WireId>,
        /// The classical wire written.
        outcome: WireId,
    },
}

impl<R: RealField> CircuitBox<R> {
    /// The quantum wires the box acts on, in the box's own order.
    pub fn quantum_wires(&self) -> &[WireId] {
        match self {
            Self::Encoder { outputs, .. } => outputs,
            Self::Unitary { wires, .. }
            | Self::Channel { wires, .. }
            | Self::Kraus { wires, .. }
            | Self::Instrument { wires, .. }
            | Self::Measurement { wires, .. } => wires,
        }
    }

    /// The classical wire the box reads, if any.
    pub fn classical_read(&self) -> Option<WireId> {
        match self {
            Self::Encoder { input, .. } => Some(*input),
            _ => None,
        }
    }

    /// The classical wire the box writes, if any.
    pub fn classical_write(&self) -> Option<WireId> {
        match self {
            Self::Instrument { outcome, .. } | Self::Measurement { outcome, .. } => Some(*outcome),
            _ => None,
        }
    }

    /// The box kind, for messages.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Encoder { .. } => "encoder",
            Self::Unitary { .. } => "unitary",
            Self::Channel { .. } => "channel",
            Self::Kraus { .. } => "kraus",
            Self::Instrument { .. } => "instrument",
            Self::Measurement { .. } => "measurement",
        }
    }
}
