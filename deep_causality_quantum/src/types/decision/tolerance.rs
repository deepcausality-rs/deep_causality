/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::density_matrix::DensityMatrix;
#[cfg(feature = "qcm")]
use crate::types::qcm::markov_freeze::CommutatorTolerance;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The tolerance family: the policies the crate's checks derive their thresholds from, named.
///
/// Every member is a function of `R::epsilon()` and nothing else, so widening the scalar tightens
/// every one of them with no call site changing. A tolerance that did not move with the scalar
/// was guessed. The members are the four policies the crate shipped before this type existed,
/// each keeping the shape its own check needs, and the family delegates to those implementations
/// rather than restating them.
///
/// | Member | Value | Where it ships |
/// |---|---|---|
/// | `Commutator` | `C·(‖ρ_j‖·b_k + ‖ρ_k‖·b_j + 2·γ_n·‖ρ_j‖·‖ρ_k‖)` | `CommutatorTolerance` |
/// | `Validation` | `√ε` | `Projection::default_tolerance` |
/// | `NumericalRank` | `D·ε·scale` | `Projection::range_projector` |
/// | `State` | `√ε·max(1, ‖M‖_F)` | `DensityMatrix::with_tolerance` |
/// | `ShotNoise` | `√(p(1−p)/n)` | the read-out decision over shots |
///
/// The commutator member takes a pair of operators and answers through
/// [`commutator_threshold`](Self::commutator_threshold); the other three take an operator's
/// dimension and scale and answer through [`threshold`](Self::threshold). Neither form answers the
/// other's question, and the `Option` says so rather than inventing a number.
///
/// # There is no integer member, and there will not be one
///
/// The family is parameterised over the real carrier alone. Widening `FloatType` buys accuracy,
/// and its failure mode is rounding, bounded by `epsilon()`. Widening `IntType` buys headroom, and
/// its failure mode is overflow, which nothing bounds: a count is right or overflowed. So the
/// integer axis gets checked arithmetic, `NaturalNumber::checked_difference` and `monus`, rather
/// than a looser tolerance. A design that reaches for a tolerance on a count is answering the
/// wrong question, and the bound refuses it:
///
/// ```compile_fail
/// use deep_causality_quantum::Tolerance;
/// // u64 is not a RealField, and no member is defined over the integer axis.
/// let _ = Tolerance::<u64>::validation();
/// ```
#[derive(Debug, Clone)]
pub enum Tolerance<R: RealField> {
    /// Q-TOL, the depth-aware commutator policy over a product of two operators.
    #[cfg(feature = "qcm")]
    Commutator(CommutatorTolerance<R>),
    /// `√ε`: a Hermitian-idempotent residual, as `Projection::new` applies it.
    Validation {
        /// `R::epsilon()`, the one input every member is derived from.
        epsilon: R,
    },
    /// `D·ε·scale`: a numerical-rank cutoff, as `Projection::range_projector` applies it. Not the
    /// `√ε` validation value, which would discard genuine range directions.
    NumericalRank {
        /// `R::epsilon()`.
        epsilon: R,
    },
    /// `√ε·max(1, ‖M‖_F)`: Hermiticity, positivity and trace on one operator, as
    /// `DensityMatrix::with_tolerance` applies it.
    State {
        /// `R::epsilon()`.
        epsilon: R,
    },
    /// `√(p(1−p)/n)`: the width of a sampled read-out, which reads from the budget rather than
    /// from `R::epsilon()`. The one member whose input is a count, and the count is an *input*
    /// to the width: the family stays over the real carrier, and there is still no integer
    /// member.
    ShotNoise,
}

impl<R> Tolerance<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The Q-TOL commutator policy at its defaults, `C = 8` and `u = ε`.
    #[cfg(feature = "qcm")]
    pub fn q_tol() -> Self {
        Self::Commutator(CommutatorTolerance::default())
    }

    /// The commutator policy with an explicit configuration.
    #[cfg(feature = "qcm")]
    pub fn commutator(policy: CommutatorTolerance<R>) -> Self {
        Self::Commutator(policy)
    }

    /// The validation member.
    pub fn validation() -> Self {
        Self::Validation {
            epsilon: R::epsilon(),
        }
    }

    /// The numerical-rank member.
    pub fn numerical_rank() -> Self {
        Self::NumericalRank {
            epsilon: R::epsilon(),
        }
    }

    /// The state-validation member.
    pub fn state() -> Self {
        Self::State {
            epsilon: R::epsilon(),
        }
    }

    /// The shot-noise member.
    pub fn shot_noise() -> Self {
        Self::ShotNoise
    }

    /// The shot-noise width `√(p(1−p)/n)` at an estimate `p` over `n` shots. `None` for every
    /// other member, and `None` at zero shots, where there is no width.
    pub fn shot_noise_width(&self, estimate: R, shots: u64) -> Option<R> {
        match self {
            Self::ShotNoise if shots > 0 => {
                let n = R::from_u64(shots)?;
                Some((estimate * (R::one() - estimate) / n).sqrt())
            }
            _ => None,
        }
    }

    /// The scalar's unit roundoff, which every member is a function of.
    pub fn epsilon(&self) -> R {
        R::epsilon()
    }

    /// The member's name, for reports.
    pub fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "qcm")]
            Self::Commutator(_) => "commutator",
            Self::Validation { .. } => "validation",
            Self::NumericalRank { .. } => "numerical-rank",
            Self::State { .. } => "state",
            Self::ShotNoise => "shot-noise",
        }
    }

    /// The threshold for a residual on one operator of dimension `dim` and scale `scale`.
    ///
    /// `Validation` ignores both, as `Projection::new` does: a projection's norm is at most `√D`
    /// and the shipped check compares its defects against bare `√ε`. `NumericalRank` multiplies
    /// by `dim` and by `max(1, scale)`, where `scale` is the largest eigenvalue as
    /// `range_projector` uses it. `State` multiplies `√ε` by `max(1, scale)`, where `scale` is the
    /// Frobenius norm as `DensityMatrix::with_tolerance` uses it. The commutator member answers
    /// `None`, because its threshold is a function of a pair.
    pub fn threshold(&self, dim: usize, scale: R) -> Option<R> {
        let clamped = if scale > R::one() { scale } else { R::one() };
        match self {
            #[cfg(feature = "qcm")]
            Self::Commutator(_) => None,
            Self::Validation { epsilon } => Some(epsilon.sqrt()),
            Self::NumericalRank { epsilon } => {
                let d = R::from_usize(dim).unwrap_or_else(R::one);
                Some(*epsilon * d * clamped)
            }
            Self::State { .. } => Some(DensityMatrix::<R>::default_tolerance() * clamped),
            Self::ShotNoise => None,
        }
    }

    /// The Q-TOL threshold for the pair `(node_j, node_k)` embedded on a common support of
    /// dimension `dim` with Frobenius norms `norm_j` and `norm_k`, delegating to
    /// [`CommutatorTolerance::threshold`]. `None` for every other member.
    #[cfg(feature = "qcm")]
    pub fn commutator_threshold(
        &self,
        node_j: usize,
        node_k: usize,
        dim: usize,
        norm_j: R,
        norm_k: R,
    ) -> Option<R> {
        match self {
            Self::Commutator(policy) => Some(policy.threshold(node_j, node_k, dim, norm_j, norm_k)),
            _ => None,
        }
    }
}
