/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::NaturalNumber;

/// A Pauli operator on the physical qubits, in symplectic form.
///
/// `x` carries the qubits acted on by `X` and `z` those acted on by `Z`; a qubit in both carries a
/// `Y` up to phase. Both are chains over the same register, which is what lets the commutation
/// tests be inner products.
///
/// # Phase is not carried
///
/// `Y = iXZ`, so a symplectic pair fixes a Pauli only up to a phase. Appendix A.1 concludes
/// equivalence up to phase and B.1's matrix-element definition is phase-sensitive, so the
/// difference matters to anyone comparing operators rather than their action on the code space.
/// [`LogicalBasis`](crate::LogicalBasis) decides the latter, where phase drops out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalPauli<W> {
    x: Gf2Chain<W>,
    z: Gf2Chain<W>,
}

impl<W: NaturalNumber> LogicalPauli<W> {
    /// A Pauli from its `X` and `Z` supports.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if the two chains are over registers of different
    /// widths, since then they do not describe one operator.
    pub fn new(x: Gf2Chain<W>, z: Gf2Chain<W>) -> Result<Self, QuantumError> {
        if x.len() != z.len() {
            return Err(QuantumError::DimensionMismatch(alloc::format!(
                "a Pauli needs its X and Z parts over one register: lengths {} and {}",
                x.len(),
                z.len()
            )));
        }
        Ok(Self { x, z })
    }

    /// The `X` support.
    pub fn x(&self) -> &Gf2Chain<W> {
        &self.x
    }

    /// The `Z` support.
    pub fn z(&self) -> &Gf2Chain<W> {
        &self.z
    }

    /// The register width.
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Whether the register is empty.
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Whether this is the identity, acting nowhere.
    pub fn is_identity(&self) -> bool {
        self.x.is_zero() && self.z.is_zero()
    }

    /// The product `self · other`, up to phase.
    ///
    /// Pauli multiplication adds the symplectic vectors over 𝔽₂. The phase that a faithful product
    /// would accumulate is dropped, which is sound for every use in this module: `O₁ ~ O₂` is
    /// decided through `O₁O₂⁻¹`, and a Pauli is its own inverse up to phase.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if the two are over different registers.
    pub fn compose(&self, other: &Self) -> Result<Self, QuantumError> {
        let x = self
            .x
            .add(&other.x)
            .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))?;
        let z = self
            .z
            .add(&other.z)
            .map_err(|e| QuantumError::DimensionMismatch(alloc::format!("{e}")))?;
        Ok(Self { x, z })
    }
}
