/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::decision::{Check, CheckItem, CheckReport, Tolerance};
use crate::types::qpu::shot_estimate::separation_bits;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, NaturalNumber};

/// The default cap on the hypotheses `design` will cover exactly. `2^C(7,2) = 2^21` subsets.
pub const DEFAULT_MAX_HYPOTHESES: usize = 7;

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
}

/// The objective `design` solves: cover every hypothesis pair at `floor_bits` of separation at
/// least cost, refusing above `max_hypotheses`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinCostCover<R> {
    /// The separation, in bits, at which an experiment resolves a pair.
    pub floor_bits: R,
    /// The largest hypothesis count the exact solve attempts. The cap is a decision: the table
    /// is `2^C(n,2)` entries and a caller raising it is choosing to pay for them.
    pub max_hypotheses: usize,
}

impl<R: RealField> MinCostCover<R> {
    /// The objective at `floor_bits` with the default cap.
    pub fn new(floor_bits: R) -> Self {
        Self {
            floor_bits,
            max_hypotheses: DEFAULT_MAX_HYPOTHESES,
        }
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
    /// Its cost.
    pub cost: R,
    /// The hypothesis pairs it separates at the floor, ascending.
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
///
/// # Errors
///
/// [`QuantumError::HypothesisCountExceeded`] naming `n` and `C(n, 2)` when `n` exceeds
/// `objective.max_hypotheses` or `C(n, 2)` exceeds the pair mask, before the pairs or the table
/// are allocated; `C(n, 2)` is computed in checked arithmetic, and an overflow is reported as
/// `usize::MAX` pairs. [`QuantumError::CalculationError`] if fewer than two hypotheses are
/// offered, since there is then no pair to cover, or if `objective.floor_bits` is not finite or is
/// negative. [`QuantumError::DimensionMismatch`] if an experiment predicts for a different number
/// of hypotheses.
pub fn design<R>(
    hypotheses: usize,
    experiments: &[Experiment<R>],
    objective: MinCostCover<R>,
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

    let pairs: Vec<(usize, usize)> = (0..n)
        .flat_map(|i| ((i + 1)..n).map(move |j| (i, j)))
        .collect();
    debug_assert_eq!(pairs.len(), p);

    // Coverage masks and, per pair, the best separation any experiment achieves.
    let slack = Tolerance::<R>::state()
        .threshold(1, objective.floor_bits)
        .expect("the state member answers the single-operator form");
    let mut masks = vec![0usize; experiments.len()];
    let mut best = vec![R::zero(); p];
    for (ei, e) in experiments.iter().enumerate() {
        for (pi, &(i, j)) in pairs.iter().enumerate() {
            let sep = separation_bits(e.predictions[i], e.predictions[j], e.shots);
            if sep > best[pi] {
                best[pi] = sep;
            }
            if sep + slack >= objective.floor_bits {
                masks[ei] |= 1 << pi;
            }
        }
    }
    let coverable = masks.iter().fold(0usize, |acc, m| acc | m);
    let full = if p == 0 { 0 } else { (1usize << p) - 1 };

    // The table: cost, the experiment that reached each state, and the state it came from.
    let states = 1usize << p;
    let mut cost: Vec<Option<R>> = vec![None; states];
    let mut last: Vec<usize> = vec![usize::MAX; states];
    let mut prev: Vec<usize> = vec![0; states];
    cost[0] = Some(R::zero());
    for s in 0..states {
        let Some(c) = cost[s] else { continue };
        for (ei, &m) in masks.iter().enumerate() {
            let t = s | m;
            if t == s {
                continue;
            }
            let candidate = c + experiments[ei].cost;
            let better = match cost[t] {
                None => true,
                Some(existing) => candidate < existing,
            };
            if better {
                cost[t] = Some(candidate);
                last[t] = ei;
                prev[t] = s;
            }
        }
    }

    // Reconstruct the cover of the coverable pairs.
    let mut chosen: Vec<usize> = Vec::new();
    let mut state = coverable;
    while state != 0 {
        let ei = last[state];
        chosen.push(ei);
        state = prev[state];
    }
    chosen.sort_unstable();
    let total_cost = cost[coverable].unwrap_or_else(R::zero);

    let entries = chosen
        .iter()
        .map(|&ei| PlanEntry {
            experiment: ei,
            name: experiments[ei].name.clone(),
            cost: experiments[ei].cost,
            resolves: pairs
                .iter()
                .enumerate()
                .filter(|(pi, _)| masks[ei] & (1 << pi) != 0)
                .map(|(_, &pair)| pair)
                .collect(),
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
        .map(|(pi, &(i, j))| {
            Check::at_least(CheckItem::Pair(i, j), best[pi], objective.floor_bits, slack)
        })
        .collect();

    Ok(DesignPlan {
        entries,
        total_cost,
        uncovered,
        hypotheses: n,
        report: CheckReport::from_checks(checks),
    })
}
