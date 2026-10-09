/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::decision::{Check, CheckItem, CheckReport, Tolerance};
use crate::types::design::cover_mode::CoverMode;
use crate::types::design::instrument_time::InstrumentTime;
use crate::types::qpu::shot_estimate::{bhattacharyya_bits_per_shot, separation_bits};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, NaturalNumber};

/// The default cap on the hypotheses `design` will cover exactly. `2^C(7,2) = 2^21` subsets.
pub const DEFAULT_MAX_HYPOTHESES: usize = 7;

/// The most experiments a combining `design` enumerates: `2^16` subsets, each summed over every
/// pair.
pub const MAX_COMBINED_EXPERIMENTS: usize = 16;

/// An experiment a plan may choose: its cost, the shots it would take, and the read-out each
/// hypothesis predicts for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Experiment<R> {
    name: String,
    cost: R,
    shots: u64,
    predictions: Vec<R>,
}

impl<R: RealField + core::fmt::Debug> Experiment<R> {
    /// An experiment with one predicted accepting probability per hypothesis.
    ///
    /// # Errors
    ///
    /// [`QuantumError::NonFiniteValue`] on a non-finite or negative cost;
    /// [`QuantumError::NormalizationError`] on a zero shot count or a prediction outside `[0, 1]`.
    pub fn new(
        name: impl Into<String>,
        cost: R,
        shots: u64,
        predictions: Vec<R>,
    ) -> Result<Self, QuantumError> {
        check_cost_and_shots(cost, shots)?;
        if predictions
            .iter()
            .any(|p| !p.is_finite() || *p < R::zero() || *p > R::one())
        {
            return Err(QuantumError::NormalizationError(
                "a predicted read-out must be a probability in [0, 1]".into(),
            ));
        }
        Ok(Self {
            name: name.into(),
            cost,
            shots,
            predictions,
        })
    }

    /// The name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The cost, on the real axis.
    pub fn cost(&self) -> R {
        self.cost
    }

    /// The shots it would take.
    pub fn shots(&self) -> u64 {
        self.shots
    }

    /// The predicted read-out per hypothesis.
    pub fn predictions(&self) -> &[R] {
        &self.predictions
    }

    /// The same experiment predicting for the hypotheses at `slots` only, in that order; `None`
    /// when a slot is past the last prediction. The control stage reads probes through this.
    #[cfg(feature = "qcm")]
    pub(crate) fn at_slots(&self, slots: &[usize]) -> Option<Self> {
        let predictions = slots
            .iter()
            .map(|&slot| self.predictions.get(slot).copied())
            .collect::<Option<Vec<R>>>()?;
        Some(Self {
            name: self.name.clone(),
            cost: self.cost,
            shots: self.shots,
            predictions,
        })
    }
}

/// The rules every experiment's cost and shots keep: a finite, non-negative cost and at least one
/// shot.
///
/// # Errors
///
/// [`QuantumError::NonFiniteValue`] on a non-finite or negative cost;
/// [`QuantumError::NormalizationError`] on zero shots.
pub(crate) fn check_cost_and_shots<R: RealField + core::fmt::Debug>(
    cost: R,
    shots: u64,
) -> Result<(), QuantumError> {
    if !cost.is_finite() || cost < R::zero() {
        return Err(QuantumError::NonFiniteValue(format!(
            "experiment cost must be finite and non-negative, got {cost:?}"
        )));
    }
    if shots == 0 {
        return Err(QuantumError::NormalizationError(
            "an experiment of zero shots predicts nothing".into(),
        ));
    }
    Ok(())
}

/// The objective `design` solves: cover every hypothesis pair at `floor_bits` of separation at
/// least cost, refusing above `max_hypotheses`.
///
/// The [`mode`](Self::mode) says how a pair is covered: one experiment alone at its own shots and
/// cost by default, the chosen experiments' bits added up ([`combining`](Self::combining)), or
/// one experiment alone with its shots sized and priced in seconds ([`timed`](Self::timed)). An
/// objective has exactly one mode; each of the two builders sets it, replacing the one before.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinCostCover<R> {
    /// The separation, in bits, at which an experiment resolves a pair.
    pub floor_bits: R,
    /// The largest hypothesis count the exact solve attempts. The cap is a decision: the table
    /// is `2^C(n,2)` entries and a caller raising it is choosing to pay for them.
    pub max_hypotheses: usize,
    /// How a pair is covered.
    pub mode: CoverMode<R>,
}

impl<R: RealField> MinCostCover<R> {
    /// The objective at `floor_bits` with the default cap, in [`CoverMode::Fixed`].
    pub fn new(floor_bits: R) -> Self {
        Self {
            floor_bits,
            max_hypotheses: DEFAULT_MAX_HYPOTHESES,
            mode: CoverMode::Fixed,
        }
    }

    /// The same objective in [`CoverMode::Combining`], covering a pair by the bits the chosen
    /// experiments add up to: the Bhattacharyya distance is additive over independent draws, so
    /// two experiments that each fall short of the floor on a pair may reach it together.
    pub fn combining(mut self) -> Self {
        self.mode = CoverMode::Combining;
        self
    }

    /// The same objective in [`CoverMode::Timed`], priced in time: an experiment's cost is its
    /// setup time, and each chosen experiment takes the fewest shots that reach the floor on the
    /// pairs it covers, at `time`'s shot time, never past its white-noise range.
    pub fn timed(mut self, time: InstrumentTime<R>) -> Self {
        self.mode = CoverMode::Timed(time);
        self
    }

    /// The same objective with an explicit cap.
    pub fn with_max_hypotheses(mut self, max_hypotheses: usize) -> Self {
        self.max_hypotheses = max_hypotheses;
        self
    }
}

/// One chosen experiment and the pairs it resolves.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanEntry<R> {
    /// Index into the offered experiments.
    pub experiment: usize,
    /// The experiment's name.
    pub name: String,
    /// Its cost: the experiment's own, or, priced in time, its setup time plus its shots at the
    /// shot time.
    pub cost: R,
    /// The shots it takes: the experiment's own, or the fewest that reach the floor, priced in
    /// time.
    pub shots: u64,
    /// The hypothesis pairs it separates at the floor alone, ascending. A pair a combining plan
    /// covers only across experiments appears under none of them.
    pub resolves: Vec<(usize, usize)>,
}

/// What `design` returns: the ordered experiments, their total cost, what each resolves, what no
/// experiment resolves, and the separation report over every pair examined.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignPlan<R> {
    entries: Vec<PlanEntry<R>>,
    total_cost: R,
    uncovered: Vec<(usize, usize)>,
    hypotheses: usize,
    report: CheckReport<R>,
}

impl<R: RealField> DesignPlan<R> {
    /// The chosen experiments, in declared order.
    pub fn entries(&self) -> &[PlanEntry<R>] {
        &self.entries
    }

    /// The total cost of the chosen experiments.
    pub fn total_cost(&self) -> R {
        self.total_cost
    }

    /// The pairs no offered experiment resolves at the floor.
    pub fn uncovered(&self) -> &[(usize, usize)] {
        &self.uncovered
    }

    /// Whether every pair is resolved.
    pub fn is_complete(&self) -> bool {
        self.uncovered.is_empty()
    }

    /// The hypotheses the plan ranges over.
    pub fn hypotheses(&self) -> usize {
        self.hypotheses
    }

    /// `C(n, 2)`, the pairs examined.
    pub fn pairs_examined(&self) -> usize {
        self.report.examined()
    }

    /// One record per pair: its best separation against the floor, so the worst record is the
    /// pair closest to the floor and an uncovered pair is a rejecting record.
    pub fn report(&self) -> &CheckReport<R> {
        &self.report
    }

    /// The number of chosen experiments.
    pub fn experiment_count(&self) -> usize {
        self.entries.len()
    }

    /// The experiment budget left after this plan, in checked ℕ arithmetic.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] naming the shortfall when the plan needs more experiments
    /// than the budget holds, through `checked_difference` returning `None`; and if the count does
    /// not fit `N`.
    pub fn draw_experiments<N>(&self, budget: N) -> Result<N, QuantumError>
    where
        N: NaturalNumber + FromPrimitive + core::fmt::Debug,
    {
        let needed = N::from_usize(self.entries.len()).ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "{} experiments do not fit the count width",
                self.entries.len()
            ))
        })?;
        budget.checked_difference(needed).ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "experiment budget overdrawn: the plan needs {:?} against {:?} available, \
                 shortfall {:?}",
                needed,
                budget,
                needed.monus(budget)
            ))
        })
    }
}

/// The exact minimum-cost cover of the `C(n, 2)` hypothesis pairs by the offered experiments.
///
/// A dynamic program over subsets of covered pairs: `dp[S | cover(e)] = min(dp[S], dp[S] +
/// cost(e))`, relaxed for every state in ascending order and every experiment in declared order.
/// `O(2^C(n,2) · k)`, linear in the experiments and exponential in the hypotheses; enumerating
/// experiment subsets at `2^k` is the wrong enumeration and is not what runs. Strict relaxation in
/// declared order breaks every tie the same way, so one instance yields one plan.
///
/// An experiment covers a pair when the two hypotheses' predicted read-outs separate by at least
/// `floor_bits` at the experiment's shots, measured as the shot-scaled Bhattacharyya distance and
/// compared with the state member of the tolerance family as slack. Pairs no experiment covers
/// are reported rather than failed: the solve targets the coverable pairs and lists the rest.
/// The report holds each pair's best separation over the offered experiments.
///
/// Priced in time ([`CoverMode::Timed`]), each experiment enters the same program once per shot
/// count that some pair needs: the fewest shots `n` at which that pair reaches the floor, so
/// `n − 1` falls short, kept only when `n` shots fit the white-noise range. The entry at `n`
/// covers every pair that needs no more and costs the setup time plus `n` shot times; taking two
/// entries of one experiment never beats taking its larger one, so the cover picks one shot count
/// per experiment. A pair that needs more shots than the range holds is uncovered, and the report
/// holds each pair's best separation at the most shots the range allows.
///
/// Combining ([`CoverMode::Combining`]), a set of experiments covers a pair when its separations
/// add up to the floor. The solve enumerates every subset of the offered experiments, in
/// ascending order of the subset's bits and keeping the first of equal cost, which is exact and
/// exponential in the experiments; [`MAX_COMBINED_EXPERIMENTS`] caps them. The report holds each
/// pair's summed separation over the chosen experiments.
///
/// # Errors
///
/// [`QuantumError::HypothesisCountExceeded`] naming `n` and `C(n, 2)` when `n` exceeds
/// `objective.max_hypotheses` or `C(n, 2)` exceeds the pair mask, before the pairs or the table
/// are allocated; `C(n, 2)` is computed in checked arithmetic, and an overflow is reported as
/// `usize::MAX` pairs. [`QuantumError::CalculationError`] if fewer than two hypotheses are
/// offered, since there is then no pair to cover, or if `objective.floor_bits` is not finite or is
/// negative, or when a combining objective is offered more than [`MAX_COMBINED_EXPERIMENTS`]
/// experiments. [`QuantumError::DimensionMismatch`] if an experiment predicts for a different
/// number of hypotheses.
pub fn design<R>(
    hypotheses: usize,
    experiments: &[Experiment<R>],
    objective: MinCostCover<R>,
) -> Result<DesignPlan<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    design_after(hypotheses, experiments, objective, None)
}

/// [`design`] after experiments have already run. `prior`, when given, holds the separation in
/// bits the run experiments credit to each pair, summed over them as a campaign's adjudication
/// sums it, one entry per pair in the order the plan's report lists the pairs. A pair whose prior
/// reaches the floor needs no experiment: no entry lists it and it is never uncovered. Combining,
/// every other pair's prior adds to the bits the chosen experiments sum to; in the other modes
/// every other pair still needs one experiment that reaches the floor alone. The report holds,
/// combining, each pair's prior plus the chosen experiments' bits, and otherwise the larger of
/// its prior and its best separation by one offered experiment. `None` is [`design`]: nothing
/// has run, and no pair starts covered.
///
/// # Errors
///
/// As [`design`].
pub(crate) fn design_after<R>(
    hypotheses: usize,
    experiments: &[Experiment<R>],
    objective: MinCostCover<R>,
    prior: Option<&[R]>,
) -> Result<DesignPlan<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let n = hypotheses;
    if n < 2 {
        return Err(QuantumError::CalculationError(format!(
            "design needs at least two hypotheses to discriminate, got {n}"
        )));
    }
    if !objective.floor_bits.is_finite() || objective.floor_bits < R::zero() {
        return Err(QuantumError::CalculationError(format!(
            "design needs a finite, non-negative floor in bits, got {:?}",
            objective.floor_bits
        )));
    }
    // `C(n, 2)` in checked arithmetic: an overflow is a count above any cap.
    let p = match n.checked_mul(n - 1) {
        Some(twice) => twice / 2,
        None => return Err(QuantumError::HypothesisCountExceeded(n, usize::MAX)),
    };
    if n > objective.max_hypotheses || p > usize::BITS as usize - 1 {
        return Err(QuantumError::HypothesisCountExceeded(n, p));
    }
    for e in experiments {
        if e.predictions.len() != n {
            return Err(QuantumError::DimensionMismatch(format!(
                "experiment '{}' predicts for {} hypotheses, the plan covers {}",
                e.name,
                e.predictions.len(),
                n
            )));
        }
    }
    debug_assert!(prior.is_none_or(|prior| prior.len() == p));

    let pairs: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
        .collect();
    debug_assert_eq!(pairs.len(), p);
    let slack = Tolerance::<R>::state()
        .threshold(1, objective.floor_bits)
        .expect("the state member answers the single-operator form");
    let floor = objective.floor_bits;
    let credited: Vec<R> = prior.map_or_else(|| vec![R::zero(); p], <[R]>::to_vec);
    // The pairs the run experiments already separate at the floor; none before any ran.
    let start = match prior {
        Some(_) => credited
            .iter()
            .enumerate()
            .filter(|&(_, &bits)| bits + slack >= floor)
            .fold(0usize, |m, (pi, _)| m | (1 << pi)),
        None => 0,
    };
    let separations = |e: &Experiment<R>, shots: u64| -> Vec<R> {
        pairs
            .iter()
            .map(|&(i, j)| separation_bits(e.predictions[i], e.predictions[j], shots))
            .collect()
    };
    let mask_of = |seps: &[R]| -> usize {
        seps.iter()
            .enumerate()
            .filter(|&(_, &sep)| sep + slack >= floor)
            .fold(0usize, |m, (pi, _)| m | (1 << pi))
            & !start
    };
    let full = if p == 0 { 0 } else { (1usize << p) - 1 };

    // One candidate entry per offered experiment, or per experiment and shot count in time; each
    // with the pairs it covers alone and its cost.
    let mut options: Vec<PlanEntry<R>> = Vec::new();
    let mut masks: Vec<usize> = Vec::new();
    let mut best = vec![R::zero(); p];
    match objective.mode {
        CoverMode::Fixed | CoverMode::Combining => {
            for (ei, e) in experiments.iter().enumerate() {
                let seps = separations(e, e.shots);
                for (b, &sep) in best.iter_mut().zip(&seps) {
                    if sep > *b {
                        *b = sep;
                    }
                }
                masks.push(mask_of(&seps));
                options.push(PlanEntry {
                    experiment: ei,
                    name: e.name.clone(),
                    cost: e.cost,
                    shots: e.shots,
                    resolves: Vec::new(),
                });
            }
        }
        CoverMode::Timed(time) => {
            let max_shots = time.max_shots();
            for (ei, e) in experiments.iter().enumerate() {
                for (b, &sep) in best.iter_mut().zip(&separations(e, max_shots)) {
                    if sep > *b {
                        *b = sep;
                    }
                }
                let needs: Vec<Option<u64>> = pairs
                    .iter()
                    .enumerate()
                    .map(|(pi, &(i, j))| {
                        if start & (1 << pi) != 0 {
                            return None;
                        }
                        shots_to_floor(e.predictions[i], e.predictions[j], floor, slack)
                            .filter(|&shots| shots <= max_shots)
                    })
                    .collect();
                let mut levels: Vec<u64> = needs.iter().flatten().copied().collect();
                levels.sort_unstable();
                levels.dedup();
                for shots in levels {
                    let integration = R::from_u64(shots).ok_or_else(|| {
                        QuantumError::CalculationError(format!(
                            "{shots} shots are not representable"
                        ))
                    })? * time.shot_time();
                    masks.push(needs.iter().enumerate().fold(0usize, |m, (pi, need)| {
                        if need.is_some_and(|need| need <= shots) {
                            m | (1 << pi)
                        } else {
                            m
                        }
                    }));
                    options.push(PlanEntry {
                        experiment: ei,
                        name: e.name.clone(),
                        cost: e.cost + integration,
                        shots,
                        resolves: Vec::new(),
                    });
                }
            }
        }
    }

    let (chosen, total_cost, coverable, report_bits) = match objective.mode {
        CoverMode::Combining => {
            if experiments.len() > MAX_COMBINED_EXPERIMENTS {
                return Err(QuantumError::CalculationError(format!(
                    "combining enumerates every subset of the experiments, and {} exceed the cap \
                     of {MAX_COMBINED_EXPERIMENTS}",
                    experiments.len()
                )));
            }
            let seps: Vec<Vec<R>> = experiments
                .iter()
                .map(|e| separations(e, e.shots))
                .collect();
            let (chosen, total_cost, coverable) =
                combined_cover(&seps, &options, &credited, floor, slack, p);
            let summed: Vec<R> = (0..p)
                .map(|pi| {
                    chosen
                        .iter()
                        .fold(credited[pi], |bits, &ei| bits + seps[ei][pi])
                })
                .collect();
            (chosen, total_cost, coverable, summed)
        }
        CoverMode::Fixed | CoverMode::Timed(_) => {
            let (chosen, total_cost, coverable) = exact_cover(&masks, &options, p, start);
            let reached: Vec<R> = credited
                .iter()
                .zip(&best)
                .map(|(&c, &b)| if c > b { c } else { b })
                .collect();
            (chosen, total_cost, coverable, reached)
        }
    };

    let entries = chosen
        .iter()
        .map(|&oi| PlanEntry {
            resolves: pairs
                .iter()
                .enumerate()
                .filter(|(pi, _)| masks[oi] & (1 << pi) != 0)
                .map(|(_, &pair)| pair)
                .collect(),
            ..options[oi].clone()
        })
        .collect();
    let uncovered: Vec<(usize, usize)> = pairs
        .iter()
        .enumerate()
        .filter(|(pi, _)| (full & !coverable) & (1 << pi) != 0)
        .map(|(_, &pair)| pair)
        .collect();
    let checks: Vec<Check<R>> = pairs
        .iter()
        .enumerate()
        .map(|(pi, &(i, j))| Check::at_least(CheckItem::Pair(i, j), report_bits[pi], floor, slack))
        .collect();

    Ok(DesignPlan {
        entries,
        total_cost,
        uncovered,
        hypotheses: n,
        report: CheckReport::from_checks(checks),
    })
}

/// The fewest shots at which read-outs `p` and `q` separate by `floor_bits`, with `slack`, so that
/// one shot fewer falls short; `None` when no shot count does, the read-outs being equal, or when
/// the count does not fit a `u64`. One shot is checked first: a floor within the slack, or read-outs
/// at `0` and `1` whose per-shot separation is infinite, reach it there.
fn shots_to_floor<R>(p: R, q: R, floor_bits: R, slack: R) -> Option<u64>
where
    R: RealField + FromPrimitive,
{
    let reaches = |shots: u64| separation_bits(p, q, shots) + slack >= floor_bits;
    if reaches(1) {
        return Some(1);
    }
    let per_shot = bhattacharyya_bits_per_shot(p, q);
    if !per_shot.is_finite() || per_shot <= R::zero() {
        return None;
    }
    // One shot falls short, so `floor_bits − slack` exceeds the per-shot bits and the estimate is
    // positive; it is then corrected against the separation itself, which rounds once more.
    let mut shots = ((floor_bits - slack) / per_shot).ceil().to_u64()?.max(1);
    while !reaches(shots) {
        shots = shots.checked_add(1)?;
    }
    while shots > 1 && reaches(shots - 1) {
        shots -= 1;
    }
    Some(shots)
}

/// The minimum-cost cover of the coverable pairs by `options`, each covering `masks[o]` alone,
/// from the pairs already covered at `start`: a dynamic program over the subsets of covered pairs,
/// relaxed in ascending state order and in declared option order with strict improvement. The
/// chosen options, in declared order, their total cost, and the coverable pairs, `start`
/// included.
fn exact_cover<R: RealField>(
    masks: &[usize],
    options: &[PlanEntry<R>],
    p: usize,
    start: usize,
) -> (Vec<usize>, R, usize) {
    let coverable = masks.iter().fold(start, |acc, m| acc | m);
    let states = 1usize << p;
    let mut cost: Vec<Option<R>> = vec![None; states];
    let mut last: Vec<usize> = vec![usize::MAX; states];
    let mut prev: Vec<usize> = vec![0; states];
    cost[start] = Some(R::zero());
    for s in 0..states {
        let Some(c) = cost[s] else { continue };
        for (oi, &m) in masks.iter().enumerate() {
            let t = s | m;
            if t == s {
                continue;
            }
            let candidate = c + options[oi].cost;
            let better = match cost[t] {
                None => true,
                Some(existing) => candidate < existing,
            };
            if better {
                cost[t] = Some(candidate);
                last[t] = oi;
                prev[t] = s;
            }
        }
    }
    let mut chosen: Vec<usize> = Vec::new();
    let mut state = coverable;
    while state != start {
        chosen.push(last[state]);
        state = prev[state];
    }
    chosen.sort_unstable();
    (chosen, cost[coverable].unwrap_or_else(R::zero), coverable)
}

/// The least-cost subset of the experiments whose summed separations, on top of each pair's
/// `prior`, reach the floor on every pair that all of them together reach: subsets in ascending
/// order of their bits, the first of equal cost kept. The chosen experiments, ascending, their
/// total cost, and the coverable pairs.
fn combined_cover<R: RealField>(
    seps: &[Vec<R>],
    options: &[PlanEntry<R>],
    prior: &[R],
    floor_bits: R,
    slack: R,
    p: usize,
) -> (Vec<usize>, R, usize) {
    let k = seps.len();
    let reaches = |members: &[usize], pi: usize| {
        members
            .iter()
            .fold(prior[pi], |bits, &ei| bits + seps[ei][pi])
            + slack
            >= floor_bits
    };
    let everyone: Vec<usize> = (0..k).collect();
    let coverable = (0..p)
        .filter(|&pi| reaches(&everyone, pi))
        .fold(0usize, |m, pi| m | (1 << pi));
    let mut best: Option<(Vec<usize>, R)> = None;
    for subset in 0..(1usize << k) {
        let members: Vec<usize> = (0..k).filter(|&ei| subset & (1 << ei) != 0).collect();
        let cost = members
            .iter()
            .fold(R::zero(), |c, &ei| c + options[ei].cost);
        if best.as_ref().is_some_and(|(_, b)| cost >= *b) {
            continue;
        }
        if (0..p).all(|pi| coverable & (1 << pi) == 0 || reaches(&members, pi)) {
            best = Some((members, cost));
        }
    }
    let (chosen, cost) = best.unwrap_or_else(|| (Vec::new(), R::zero()));
    (chosen, cost, coverable)
}
