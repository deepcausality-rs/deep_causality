/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use alloc::format;

/// A wire's index in a [`CircuitModel`](crate::CircuitModel).
pub type WireId = usize;
/// A box's index in a [`CircuitModel`](crate::CircuitModel).
pub type BoxId = usize;
/// A node's index in a [`CircuitModel`](crate::CircuitModel)'s grouping.
pub type NodeId = usize;

/// The type of a wire in the category QC (Lorenz & Tull, arXiv:2602.16612, Example 57): a
/// finite-dimensional Hilbert space, drawn thick, or a finite set of classical outcomes, drawn thin.
///
/// A quantum wire is a line of the register that boxes act on in place; a classical wire is a
/// value written by one box and read by later ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireType {
    /// A quantum system of the given Hilbert dimension.
    Quantum {
        /// The Hilbert dimension, at least one.
        dim: usize,
    },
    /// A classical variable with the given number of outcomes.
    Classical {
        /// The outcome count, at least one.
        outcomes: usize,
    },
}

impl WireType {
    /// A qubit.
    pub fn qubit() -> Self {
        Self::Quantum { dim: 2 }
    }

    /// A classical bit.
    pub fn bit() -> Self {
        Self::Classical { outcomes: 2 }
    }

    /// Whether the wire is quantum.
    pub fn is_quantum(&self) -> bool {
        matches!(self, Self::Quantum { .. })
    }

    /// The Hilbert dimension of a quantum wire or the outcome count of a classical one.
    pub fn cardinality(&self) -> usize {
        match self {
            Self::Quantum { dim } => *dim,
            Self::Classical { outcomes } => *outcomes,
        }
    }

    /// A wire of cardinality zero carries nothing and is refused.
    pub(crate) fn validate(&self, id: WireId) -> Result<(), QuantumError> {
        if self.cardinality() == 0 {
            return Err(QuantumError::DimensionMismatch(format!(
                "wire {id} has cardinality zero"
            )));
        }
        Ok(())
    }
}
