/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::decision::{Check, CheckItem, CheckReport, Tolerance};
use crate::types::qpu::shot_estimate::ShotEstimate;
use crate::types::verdict::projection::Projection;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use deep_causality_algebra::{RealField, Verdict};
use deep_causality_haft::Either;
use deep_causality_num::FromPrimitive;

/// The verdict a forked world came back with. Verdicts are extracted at the measurement
/// boundary, so there is no constructor from an operator: a world that reaches `adjudicate`
/// carrying an operator no `observe` turned into a verdict has nothing to fold.
#[derive(Debug, Clone, PartialEq)]
pub enum WorldVerdict<R: RealField, const D: usize> {
    /// A proposition in the orthomodular lattice, from a projective measurement.
    Projection(Projection<R, D>),
    /// A read-out judged against a real-valued spec: a classical proposition, Boolean.
    ReadOut(CheckReport<R>),
}

/// One live world after a fork: the hypothesis it ran under, the verdict it came back with, and
/// the read-out the verdict was taken from.
#[derive(Debug, Clone, PartialEq)]
pub struct World<R: RealField, const D: usize> {
    name: String,
    verdict: WorldVerdict<R, D>,
    read_out: ShotEstimate<R>,
}

impl<R: RealField, const D: usize> World<R, D> {
    /// A world whose verdict is a projection.
    pub fn projection(
        name: impl Into<String>,
        projection: Projection<R, D>,
        read_out: ShotEstimate<R>,
    ) -> Self {
        Self {
            name: name.into(),
            verdict: WorldVerdict::Projection(projection),
            read_out,
        }
    }

    /// A world whose verdict is a read-out against a real-valued spec.
    pub fn read_out(
        name: impl Into<String>,
        report: CheckReport<R>,
        read_out: ShotEstimate<R>,
    ) -> Self {
        Self {
            name: name.into(),
            verdict: WorldVerdict::ReadOut(report),
            read_out,
        }
    }

    /// The hypothesis this world ran under.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The verdict.
    pub fn verdict(&self) -> &WorldVerdict<R, D> {
        &self.verdict
    }

    /// The read-out the verdict was taken from.
    pub fn estimate(&self) -> &ShotEstimate<R> {
        &self.read_out
    }
}

/// The hypothesis that survived, and how far it stood from its nearest rival.
#[derive(Debug, Clone, PartialEq)]
pub struct Survivor<R> {
    /// Its name.
    pub name: String,
    /// The smallest separation, in bits, between it and any other world.
    pub separation_bits: R,
}

/// Why no single hypothesis survived.
#[derive(Debug, Clone, PartialEq)]
pub enum Ambiguity<R> {
    /// Two projection-valued verdicts do not commute, so the lattice they live in is not
    /// distributive on them and no joint verdict exists. Names the pair and how many pairs were
    /// tested before it.
    NonCommuting {
        /// The offending pair of worlds.
        pair: (usize, usize),
        /// Pairs tested up to and including it.
        pairs_tested: usize,
    },
    /// One world was folded, so nothing was discriminated.
    Vacuous {
        /// The world count, one.
        worlds: usize,
    },
    /// No world's verdict held.
    NoSurvivor {
        /// The worlds folded.
        worlds: usize,
    },
    /// More than one world's verdict held.
    SeveralSurvive {
        /// Their names.
        survivors: Vec<String>,
    },
    /// One world survived, but a rival's read-out lies within the shot-noise separation.
    Unseparated {
        /// The survivor.
        survivor: String,
        /// The tightest pair.
        tightest: (usize, usize),
        /// Its separation in bits.
        separation_bits: R,
        /// The floor it fell short of.
        floor_bits: R,
    },
}

/// The projection lattice's fold of a commuting family: the meet and the join of every verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionFold<R: RealField, const D: usize> {
    /// `⋀ P_i`, what every world asserts.
    pub meet: Projection<R, D>,
    /// `⋁ P_i`, what some world asserts.
    pub join: Projection<R, D>,
}

/// What `adjudicate` examined and concluded.
#[derive(Debug, Clone, PartialEq)]
pub struct Adjudication<R: RealField, const D: usize> {
    /// How many worlds were folded.
    pub worlds_folded: usize,
    /// How many pairs of projection-valued verdicts were tested for commutation; zero on the
    /// read-out path, where no test runs.
    pub commutation_pairs_tested: usize,
    /// One record per pair of worlds: their separation at the taken shots against the floor.
    pub report: CheckReport<R>,
    /// The lattice fold, when the verdicts were projections that all commute.
    pub fold: Option<ProjectionFold<R, D>>,
    /// One surviving hypothesis, or the residual ambiguity.
    pub outcome: Either<Survivor<R>, Ambiguity<R>>,
}

/// Fold the worlds' verdicts under the verdict law, and separate the survivor from its rivals.
///
/// The fold is chosen by the kind of verdict the worlds carry. Projection-valued verdicts are
/// tested pairwise with `Projection::commutes_with` before anything combines, because
/// `Projection<R, D>` is orthomodular and fails distributivity outside the commuting family; a
/// non-commuting pair yields [`Ambiguity::NonCommuting`] and no survivor. Within a commuting
/// family the verdicts combine through `Verdict::meet` and `Verdict::join`, and a world's verdict
/// holds when its projection is not the bottom. Read-outs against a real-valued spec are classical
/// propositions in a Boolean algebra, so no commutation test runs, and a world's verdict holds
/// when its report accepted something.
///
/// The pairwise separation report is measured before the commutation test, so a non-commuting
/// fold carries the report over every pair it examined rather than a vacuous one.
///
/// A survivor is a lone world whose verdict holds and whose read-out separates from every other
/// world's by at least `floor_bits`, the shot-scaled Bhattacharyya distance compared with the
/// state member of the tolerance family as slack. Anything else is the residual ambiguity, which
/// says why.
///
/// # Errors
///
/// [`QuantumError::CalculationError`] on a `floor_bits` that is not finite or is negative, on no
/// worlds, or on worlds carrying verdicts of both kinds, since one fold cannot serve two
/// lattices.
pub fn adjudicate<R, const D: usize>(
    worlds: &[World<R, D>],
    floor_bits: R,
) -> Result<Adjudication<R, D>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    if !floor_bits.is_finite() || floor_bits < R::zero() {
        return Err(QuantumError::CalculationError(format!(
            "adjudicate needs a finite, non-negative floor in bits, got {floor_bits:?}"
        )));
    }
    if worlds.is_empty() {
        return Err(QuantumError::CalculationError(
            "adjudicate needs at least one world".into(),
        ));
    }
    let projective = worlds
        .iter()
        .filter(|w| matches!(w.verdict, WorldVerdict::Projection(_)))
        .count();
    if projective != 0 && projective != worlds.len() {
        return Err(QuantumError::CalculationError(format!(
            "adjudicate cannot fold {} projection-valued and {} read-out verdicts together",
            projective,
            worlds.len() - projective
        )));
    }
    let n = worlds.len();

    // Separation over every pair, at the taken shots. Measured before the commutation test, so
    // a non-commuting fold still reports what it examined.
    let slack = Tolerance::<R>::state()
        .threshold(1, floor_bits)
        .expect("the state member answers the single-operator form");
    let mut checks = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let sep = worlds[i].read_out.separation_bits(&worlds[j].read_out);
            checks.push(Check::at_least(
                CheckItem::Pair(i, j),
                sep,
                floor_bits,
                slack,
            ));
        }
    }
    let report = CheckReport::from_checks(checks);

    // The projection path: commutation first, then the lattice fold.
    let mut commutation_pairs_tested = 0usize;
    let mut fold = None;
    if projective == n {
        let projections: Vec<&Projection<R, D>> = worlds
            .iter()
            .map(|w| match &w.verdict {
                WorldVerdict::Projection(p) => p,
                WorldVerdict::ReadOut(_) => unreachable!("counted above"),
            })
            .collect();
        for i in 0..n {
            for j in (i + 1)..n {
                commutation_pairs_tested += 1;
                if !projections[i].commutes_with(projections[j]) {
                    return Ok(Adjudication {
                        worlds_folded: n,
                        commutation_pairs_tested,
                        report,
                        fold: None,
                        outcome: Either::Right(Ambiguity::NonCommuting {
                            pair: (i, j),
                            pairs_tested: commutation_pairs_tested,
                        }),
                    });
                }
            }
        }
        let meet = projections
            .iter()
            .fold(Projection::<R, D>::top(), |acc, p| acc.meet((*p).clone()));
        let join = projections
            .iter()
            .fold(Projection::<R, D>::bottom(), |acc, p| {
                acc.join((*p).clone())
            });
        fold = Some(ProjectionFold { meet, join });
    }

    let holds = |w: &World<R, D>| match &w.verdict {
        WorldVerdict::Projection(p) => p.rank() > 0,
        WorldVerdict::ReadOut(r) => r.accepted() && !r.is_vacuous(),
    };
    let survivors: Vec<usize> = (0..n).filter(|&i| holds(&worlds[i])).collect();

    let outcome = if n == 1 {
        Either::Right(Ambiguity::Vacuous { worlds: 1 })
    } else if survivors.is_empty() {
        Either::Right(Ambiguity::NoSurvivor { worlds: n })
    } else if survivors.len() > 1 {
        Either::Right(Ambiguity::SeveralSurvive {
            survivors: survivors.iter().map(|&i| worlds[i].name.clone()).collect(),
        })
    } else {
        let s = survivors[0];
        // The tightest pair involving the survivor.
        let mut tightest: Option<(usize, usize, R, bool)> = None;
        for c in report.checks() {
            if let CheckItem::Pair(i, j) = c.item
                && (i == s || j == s)
                && tightest.is_none_or(|t| c.measured < t.2)
            {
                tightest = Some((i, j, c.measured, c.accepted));
            }
        }
        let (i, j, sep, accepted) = tightest.expect("two or more worlds give the survivor a pair");
        if accepted {
            Either::Left(Survivor {
                name: worlds[s].name.clone(),
                separation_bits: sep,
            })
        } else {
            Either::Right(Ambiguity::Unseparated {
                survivor: worlds[s].name.clone(),
                tightest: (i, j),
                separation_bits: sep,
                floor_bits,
            })
        }
    };

    Ok(Adjudication {
        worlds_folded: n,
        commutation_pairs_tested,
        report,
        fold,
        outcome,
    })
}
