/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_homology::Gf2Chain;
use deep_causality_num::NaturalNumber;
use deep_causality_num_rational::Rational;

/// The phase a diagonal logical gate applies, as an exact rational multiple of `2π`.
///
/// `Rational<i64>` rather than a float, because `Q(n)/M` is a rational by construction and the
/// question asked of it is *integrality*, not proximity. A gate is logically trivial when a phase
/// difference vanishes, and "vanishes" here means the rational is a whole number of turns. There is
/// no tolerance in this check and there is nothing for one to do: comparing `3.1e-16` against a
/// threshold would be answering a question the arithmetic never asks.
pub type Turns = Rational<i64>;

/// A diagonal logical gate: `exp(2πi · Q(a(γ)) / M)` with `M` a power of two.
///
/// Junichi Haruna, arXiv:2511.15224. Every gate in Table 1 except the Hadamard is diagonal in the
/// computational basis, and `a(γ)` is diagonal with integer eigenvalues: on a basis state `|x⟩` it
/// reads `|supp(γ) ∩ x|`. So the whole gate is one integer polynomial `Q` evaluated at an overlap
/// count, and a modulus.
///
/// # The polynomials, and where they come from
///
/// Table 1's gauge-field column, normalised so the phase reads `exp(2πi·Q(n)/M)`:
///
/// | Gate | Table 1 | `Q(n)` | `M` |
/// |---|---|---|---|
/// | `Z̄(γ)` | `exp(iπ a)` | `n` | 2 |
/// | `S̄(γ)` | `exp(i(π/2) a²)` | `n²` | 4 |
/// | `T̄(γ)` | `exp(i(π/4)(2a³ − 3a² + 2a))` | `2n³ − 3n² + 2n` | 8 |
///
/// Each reproduces its single-qubit gate at `n = 1`: `Z̄` gives `−1`, `S̄` gives `i`, `T̄` gives
/// `e^{iπ/4}`, matching `diag(1, −1)`, `diag(1, i)` and `diag(1, e^{iπ/4})`.
///
/// # Why this is not a `LogicalPauli`
///
/// [`LogicalPauli`](crate::LogicalPauli) is a symplectic pair, which fixes a Pauli. `S̄`, `T̄` and
/// the controlled gates have no symplectic representation, so deciding their logical action needs a
/// different carrier. This is that carrier, and
/// [`LogicalBasis::is_diagonal_trivial`](crate::LogicalBasis::is_diagonal_trivial) is the predicate
/// over it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagonalPhase<W> {
    chain: Gf2Chain<W>,
    /// `Q(n) = Σ_j coeffs[j] · n^j`, ascending.
    coeffs: Vec<i64>,
    /// `M = 2^log2_modulus`.
    log2_modulus: u32,
}

impl<W: NaturalNumber> DiagonalPhase<W> {
    /// A diagonal gate from its phase polynomial and its power-of-two modulus.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if `coeffs` is empty, or if `M = 2^log2_modulus` would
    /// not fit an `i64`.
    pub fn new(
        chain: Gf2Chain<W>,
        coeffs: Vec<i64>,
        log2_modulus: u32,
    ) -> Result<Self, QuantumError> {
        if coeffs.is_empty() {
            return Err(QuantumError::DimensionMismatch(
                "a phase polynomial needs at least one coefficient".into(),
            ));
        }
        if log2_modulus >= 62 {
            return Err(QuantumError::DimensionMismatch(format!(
                "modulus 2^{log2_modulus} does not fit the phase arithmetic"
            )));
        }
        Ok(Self {
            chain,
            coeffs,
            log2_modulus,
        })
    }

    /// `Z̄(γ) = exp(iπ a(γ))`, Table 1 row 1.
    pub fn z(chain: Gf2Chain<W>) -> Self {
        Self {
            chain,
            coeffs: alloc::vec![0, 1],
            log2_modulus: 1,
        }
    }

    /// `S̄(γ) = exp(i(π/2) a(γ)²)`, Table 1 row 3.
    pub fn s(chain: Gf2Chain<W>) -> Self {
        Self {
            chain,
            coeffs: alloc::vec![0, 0, 1],
            log2_modulus: 2,
        }
    }

    /// `T̄(γ) = exp(i(π/4)(2a(γ)³ − 3a(γ)² + 2a(γ)))`, Table 1 row 7 and Eq. (3.61).
    pub fn t(chain: Gf2Chain<W>) -> Self {
        Self {
            chain,
            coeffs: alloc::vec![0, 2, -3, 2],
            log2_modulus: 3,
        }
    }

    /// The chain the gate is supported on.
    pub fn chain(&self) -> &Gf2Chain<W> {
        &self.chain
    }

    /// `M`, the phase denominator.
    pub fn modulus(&self) -> i64 {
        1i64 << self.log2_modulus
    }

    /// The phase this gate applies to a basis state whose overlap with the chain is `n`, in turns.
    ///
    /// Exact: `Q(n) / M` as a reduced rational, never a float.
    pub fn phase_at(&self, n: u64) -> Turns {
        let mut acc: i64 = 0;
        let mut power: i64 = 1;
        let n = n as i64;
        for c in &self.coeffs {
            acc = acc.wrapping_add(c.wrapping_mul(power));
            power = power.wrapping_mul(n);
        }
        Rational::new(acc, self.modulus())
    }

    /// The gate obtained by moving to another representative, `γ ↦ γ ⊕ b`.
    ///
    /// The polynomial and modulus are unchanged; only the chain moves. This is the operation
    /// class invariance quantifies over.
    ///
    /// # Errors
    ///
    /// [`QuantumError::DimensionMismatch`] if `b` is over a register of a different width.
    pub fn shifted_by(&self, b: &Gf2Chain<W>) -> Result<Self, QuantumError> {
        let chain = self
            .chain
            .add(b)
            .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))?;
        Ok(Self {
            chain,
            coeffs: self.coeffs.clone(),
            log2_modulus: self.log2_modulus,
        })
    }
}
