/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The shot budget, behind the `qpu` feature.
//!
//! Naming a budget is what selects the emergent modality, and the modality split is a compiler
//! guarantee: [`Evidence`], [`ShotBudget`] and anything that accepts them are compiled only with
//! `qpu`, so a configuration naming shots in a default build fails to compile. No runtime path
//! anywhere in the crate accepts a budget and then rejects it. The sampler itself lives in the
//! default build, because a seeded draw from the shipped Born probability is reproducible data;
//! what is gated is the declaration that a pipeline will spend it.
//!
//! # Counts are ℕ, and the draw-down is checked arithmetic
//!
//! A shot count is bounded on `NaturalNumber`, whose width a program names once with an unsigned
//! alias such as `NumberType`. ℕ is a commutative semiring with no additive inverse, so there is
//! no `Sub` to reach for: [`ShotBudget::draw`] uses `checked_difference`, which returns `None` on
//! an overdraw, and the shortfall is reported through `monus`. Widening the alias buys headroom,
//! not accuracy, and the family of tolerances has no member here.

use crate::QuantumError;
use crate::types::density_matrix::DensityMatrix;
use crate::types::qpu::born_sampler::sample_projector;
use crate::types::qpu::histogram::CountHistogram;
use crate::types::qpu::prng::SplitMix64;
use crate::types::verdict::projection::Projection;
use alloc::format;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, NaturalNumber, ToPrimitive};

/// What a configuration declares: how many shots, at which seed.
///
/// The declaration is data; [`into_budget`](Self::into_budget) turns it into the ledger a run
/// draws from, and that is where a zero count is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Evidence<N: NaturalNumber> {
    shots: N,
    seed: u64,
}

impl<N: NaturalNumber> Evidence<N> {
    /// A declaration of `shots`, seeded at zero until [`seed`](Self::seed) is called.
    pub fn shots(shots: N) -> Self {
        Self { shots, seed: 0 }
    }

    /// The same declaration at `seed`.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// The declared count.
    pub fn shot_count(&self) -> N {
        self.shots
    }

    /// The declared seed.
    pub fn seed_value(&self) -> u64 {
        self.seed
    }

    /// The ledger this declaration funds.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the zero budget when `shots` is zero, since a
    /// budget that buys no evidence is a construction error rather than a vacuous pass.
    pub fn into_budget(self) -> Result<ShotBudget<N>, QuantumError> {
        ShotBudget::new(self.shots, self.seed)
    }
}

/// One draw against a budget: the shots granted and the seed they are drawn at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Draw<N: NaturalNumber> {
    /// The shots granted.
    pub shots: N,
    /// The seed for this draw, mixed from the run seed and the shots already spent, so successive
    /// draws at one run seed are distinct and a repeated run reproduces them all.
    pub seed: u64,
}

/// The ledger of a shot budget: what remains, what was spent, and the run seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShotBudget<N: NaturalNumber> {
    remaining: N,
    spent: N,
    seed: u64,
}

impl<N: NaturalNumber> ShotBudget<N> {
    /// A budget of `shots` at `seed`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the zero budget when `shots` is zero.
    pub fn new(shots: N, seed: u64) -> Result<Self, QuantumError> {
        if shots.is_zero() {
            return Err(QuantumError::CalculationError(
                "a shot budget of zero buys no evidence; refused at construction".into(),
            ));
        }
        Ok(Self {
            remaining: shots,
            spent: N::zero(),
            seed,
        })
    }

    /// What remains.
    pub fn remaining(&self) -> N {
        self.remaining
    }

    /// What was drawn so far.
    pub fn spent(&self) -> N {
        self.spent
    }

    /// The run seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Whether nothing remains.
    pub fn is_exhausted(&self) -> bool {
        self.remaining.is_zero()
    }

    /// `request` shots drawn down, as the draw and the budget after it. The receiver is unchanged.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the request, the remainder and the shortfall when
    /// `request` exceeds what remains. `checked_difference` decides it, and nothing is recorded.
    pub fn draw(&self, request: N) -> Result<(Draw<N>, Self), QuantumError>
    where
        N: ToPrimitive + core::fmt::Debug,
    {
        let Some(left) = self.remaining.checked_difference(request) else {
            return Err(QuantumError::CalculationError(format!(
                "shot budget overdrawn: requested {:?} against {:?} remaining, shortfall {:?}",
                request,
                self.remaining,
                request.monus(self.remaining)
            )));
        };
        let spent_so_far = self.spent.to_u64().unwrap_or(u64::MAX);
        let seed = SplitMix64::new(
            self.seed
                .wrapping_add(spent_so_far.wrapping_mul(0x9E37_79B9_7F4A_7C15)),
        )
        .next_u64();
        let after = Self {
            remaining: left,
            spent: self.spent + request,
            seed: self.seed,
        };
        Ok((
            Draw {
                shots: request,
                seed,
            },
            after,
        ))
    }
}

/// `request` shots of a projector read-out, drawn against `budget`.
///
/// Returns the histogram and the budget after the draw. On any error the caller's budget is
/// unchanged and nothing was drawn: a malformed read-out spends nothing.
///
/// # Errors
///
/// The overdraw error of [`ShotBudget::draw`], the dimension error of the Born call, or
/// [`QuantumError::CalculationError`] if the count does not fit a `u64`.
pub fn sample_within_budget<R, N, const D: usize>(
    rho: &DensityMatrix<R>,
    projection: &Projection<R, D>,
    budget: &ShotBudget<N>,
    request: N,
) -> Result<(CountHistogram, ShotBudget<N>), QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber + ToPrimitive + core::fmt::Debug,
{
    let (draw, after) = budget.draw(request)?;
    let shots = draw
        .shots
        .to_u64()
        .ok_or_else(|| QuantumError::CalculationError("shot count does not fit a u64".into()))?;
    let hist = sample_projector(rho, projection, shots, draw.seed)?;
    Ok((hist, after))
}
