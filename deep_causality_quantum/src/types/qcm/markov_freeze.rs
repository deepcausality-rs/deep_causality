/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The quantum Markov condition (Lorenz 2022, Def 3.3) as a freeze-time
//! commutativity check (R3, Layer-D). The operator-valued process is carried
//! as an external [`ProcessFactors`] store; the check computes `[ρ_j, ρ_k]`
//! only for factor pairs whose Hilbert supports intersect, comparing
//! `‖[ρ_j, ρ_k]‖_F` against a condition-driven forward-error tolerance (Q-TOL,
//! not linear-in-depth). It is sound (never accepts a non-commuting model) and
//! MAY be incomplete.

use crate::QuantumError;
use crate::types::qcm::faithfulness::CausalStructure;
use crate::types::qcm::process_factors::{FactorSupports, ProcessFactors};
use crate::types::qgates::operator_linalg::{embed_on_legs, frobenius_norm, matrix_commutator};
use deep_causality::{CausableGraph, CausalityGraphError};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use std::cell::RefCell;
use std::collections::BTreeSet;

/// The orphan-rule-legal bridge from the crate-local [`QuantumError`] into the
/// engine's `CausalityGraphError`, so a quantum freeze hook can surface a
/// structured cause through the `Result<(), CausalityGraphError>` channel that
/// `freeze_verified_with_check` requires.
impl From<QuantumError> for CausalityGraphError {
    fn from(e: QuantumError) -> Self {
        CausalityGraphError(format!("{}", e))
    }
}

/// The depth-aware, condition-driven commutator tolerance (Q-TOL): an
/// incremental first-order forward-error budget over the real unit roundoff
/// `u`. The pair test accepts `[ρ_j, ρ_k] = 0` iff
/// `‖[ρ_j,ρ_k]‖_F ≤ C · (‖ρ_j‖·budget_k + ‖ρ_k‖·budget_j + 2·γ_n·‖ρ_j‖·‖ρ_k‖)`
/// with `γ_n = n·u/(1−n·u)` and `C` the safety factor. Per-node budgets default
/// to `u·‖ρ‖_F`; deeper/nested models can supply larger budgets that grew
/// through their construction.
#[derive(Debug, Clone)]
pub struct CommutatorTolerance<R: RealField> {
    safety_factor: R,
    unit_roundoff: R,
    budgets: std::collections::BTreeMap<usize, R>,
}

impl<R: RealField + FromPrimitive> Default for CommutatorTolerance<R> {
    fn default() -> Self {
        Self {
            safety_factor: R::from_f64(8.0).unwrap_or_else(R::one),
            unit_roundoff: R::epsilon(),
            budgets: std::collections::BTreeMap::new(),
        }
    }
}

impl<R: RealField + FromPrimitive> CommutatorTolerance<R> {
    /// The default policy: `C = 8`, `u = ε`, seed budgets.
    pub fn new() -> Self {
        Self::default()
    }

    /// Overrides the safety factor `C`.
    pub fn with_safety_factor(mut self, c: R) -> Self {
        self.safety_factor = c;
        self
    }

    /// Overrides the base unit roundoff `u`.
    pub fn with_unit_roundoff(mut self, u: R) -> Self {
        self.unit_roundoff = u;
        self
    }

    /// Supplies an explicit forward-error budget for a node's factor (for a
    /// deeply constructed operator; defaults to `u·‖ρ‖_F`).
    pub fn with_budget(mut self, node: usize, budget: R) -> Self {
        self.budgets.insert(node, budget);
        self
    }

    fn budget_for(&self, node: usize, norm: R) -> R {
        self.budgets
            .get(&node)
            .copied()
            .unwrap_or(self.unit_roundoff * norm)
    }

    /// The acceptance threshold for the pair `(node_j, node_k)` whose factors
    /// have been embedded on their common support as `dim × dim` operators with
    /// Frobenius norms `norm_j`, `norm_k`.
    pub fn threshold(&self, node_j: usize, node_k: usize, dim: usize, norm_j: R, norm_k: R) -> R {
        let u = self.unit_roundoff;
        let nu = R::from_usize(dim).unwrap_or_else(R::one) * u;
        let one = R::one();
        // γ_n = n·u/(1−n·u); guard the degenerate n·u ≥ 1 by capping at n·u.
        let gamma = if nu < one { nu / (one - nu) } else { nu };
        let budget_j = self.budget_for(node_j, norm_j);
        let budget_k = self.budget_for(node_k, norm_k);
        let two = one + one;
        self.safety_factor * (norm_j * budget_k + norm_k * budget_j + two * gamma * norm_j * norm_k)
    }
}

/// One recorded commutator check (the shared B1(c)/Q-TOL telemetry).
#[derive(Debug, Clone)]
pub struct CommutatorCheck<R: RealField> {
    pub node_j: usize,
    pub node_k: usize,
    /// `‖[ρ_j, ρ_k]‖_F` on the common support.
    pub norm: R,
    /// The Q-TOL acceptance threshold.
    pub threshold: R,
    /// `norm / threshold` — `≤ 1` accepts.
    pub margin: R,
    /// Whether the pair passed (commuting within tolerance).
    pub commutes: bool,
}

/// The instrumented report of a Markov check: one entry per intersecting-support
/// factor pair.
#[derive(Debug, Clone, Default)]
pub struct QuantumMarkovReport<R: RealField> {
    pub checks: Vec<CommutatorCheck<R>>,
}

impl<R: RealField> QuantumMarkovReport<R> {
    /// The number of pairs actually tested (intersecting supports).
    pub fn tested_pairs(&self) -> usize {
        self.checks.len()
    }

    /// The worst (largest) recorded margin, or `None` if nothing was tested.
    pub fn worst_margin(&self) -> Option<R> {
        self.checks.iter().map(|c| c.margin).fold(None, |acc, m| {
            Some(acc.map_or(m, |a: R| if m > a { m } else { a }))
        })
    }
}

/// Runs the quantum Markov commutativity check over the external factor store,
/// pair by pair, on intersecting Hilbert supports only. Returns the instrumented
/// report on success, or a [`QuantumError::CommutatorNonZero`] naming the first
/// offending pair. Sound: it never accepts a pair whose commutator exceeds the
/// Q-TOL threshold.
pub fn quantum_markov_check<R>(
    factors: &ProcessFactors<R>,
    supports: &FactorSupports,
    tolerance: &CommutatorTolerance<R>,
) -> Result<QuantumMarkovReport<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default,
{
    supports.validate(factors)?;

    let nodes: Vec<usize> = factors.nodes().collect();
    let mut report = QuantumMarkovReport::default();

    for (a, &node_j) in nodes.iter().enumerate() {
        let legs_j: BTreeSet<usize> = supports
            .support(node_j)
            .expect("validated above")
            .iter()
            .copied()
            .collect();
        for &node_k in nodes.iter().skip(a + 1) {
            let legs_k: BTreeSet<usize> = supports
                .support(node_k)
                .expect("validated above")
                .iter()
                .copied()
                .collect();

            // Disjoint supports impose no commutativity obligation.
            if legs_j.is_disjoint(&legs_k) {
                continue;
            }

            let union: BTreeSet<usize> = legs_j.union(&legs_k).copied().collect();
            let space = supports.space_map(&union);

            let e_j = embed_on_legs(factors.get(node_j).expect("factor j"), &legs_j, &space)?;
            let e_k = embed_on_legs(factors.get(node_k).expect("factor k"), &legs_k, &space)?;

            let comm = matrix_commutator(&e_j, &e_k)?;
            let norm = frobenius_norm(&comm);
            let norm_j = frobenius_norm(&e_j);
            let norm_k = frobenius_norm(&e_k);
            let dim = space.values().product::<usize>();
            let threshold = tolerance.threshold(node_j, node_k, dim, norm_j, norm_k);

            let commutes = norm <= threshold;
            let margin = if threshold > R::zero() {
                norm / threshold
            } else if norm > R::zero() {
                // A zero threshold with a non-zero norm is an infinite margin;
                // represent it as the norm itself (strictly > 0 ⇒ rejected).
                norm
            } else {
                R::zero()
            };

            report.checks.push(CommutatorCheck {
                node_j,
                node_k,
                norm,
                threshold,
                margin,
                commutes,
            });

            if !commutes {
                return Err(QuantumError::CommutatorNonZero(
                    node_j,
                    node_k,
                    format!(
                        "‖[ρ_{}, ρ_{}]‖_F exceeds the Q-TOL threshold (margin > 1)",
                        node_j, node_k
                    ),
                ));
            }
        }
    }

    Ok(report)
}

/// Freezes `graph` with the built-in Stage-4 checks **and** the quantum Markov
/// commutativity check as the level-specific hook, returning the instrumented
/// report on success. On a quantum-check failure the graph is rolled back to the
/// dynamic state (the hook's `unfreeze`) and the **structured** `QuantumError`
/// is recovered via an internal stash — not merely its `Display` message. A
/// failure in the built-in checks (acyclicity / single-writer) surfaces as a
/// [`QuantumError::CalculationError`] carrying the graph error's message.
///
/// When `faithfulness` is `Some((inputs, outputs))`, the declared causal
/// structure is additionally derived from the frozen graph's reachability and
/// checked for C₃-exclusion (spec quantum-markov-freeze): a model whose
/// structure contains a `C₃` is **rejected at freeze** with
/// [`QuantumError::NotFaithfullyRepresentable`] and rolled back. Pass `None` to
/// run the commutativity check alone.
pub fn freeze_quantum<T, G, R>(
    graph: &mut G,
    state_writers: &[usize],
    factors: &ProcessFactors<R>,
    supports: &FactorSupports,
    tolerance: &CommutatorTolerance<R>,
    faithfulness: Option<(&[usize], &[usize])>,
) -> Result<QuantumMarkovReport<R>, QuantumError>
where
    T: Clone,
    G: CausableGraph<T>,
    R: RealField + FromPrimitive + Default,
{
    // Validate the operator layout up front so a shape error is reported as
    // itself rather than as a downstream freeze failure.
    supports.validate(factors)?;

    // The stash preserves the structured error across the CausalityGraphError
    // bridge (the hook can only return CausalityGraphError). RefCell gives the
    // `Fn` hook the interior mutability it needs.
    let stash: RefCell<Option<QuantumError>> = RefCell::new(None);
    let report: RefCell<Option<QuantumMarkovReport<R>>> = RefCell::new(None);

    let outcome = graph.freeze_verified_with_check(state_writers, |g| {
        // Level check 1: the quantum Markov commutativity check.
        let rep = match quantum_markov_check(factors, supports, tolerance) {
            Ok(rep) => rep,
            Err(e) => {
                let bridged = CausalityGraphError::from(e.clone());
                *stash.borrow_mut() = Some(e);
                return Err(bridged);
            }
        };
        // Level check 2: C₃-exclusion faithfulness over the declared input/output
        // systems, derived from the (now-frozen) graph's reachability.
        if let Some((inputs, outputs)) = faithfulness
            && let Err(e) = CausalStructure::from_graph_reachability::<T, G>(g, inputs, outputs)
                .and_then(|cs| cs.check_c3_exclusion())
        {
            let bridged = CausalityGraphError::from(e.clone());
            *stash.borrow_mut() = Some(e);
            return Err(bridged);
        }
        *report.borrow_mut() = Some(rep);
        Ok(())
    });

    match outcome {
        Ok(()) => Ok(report
            .into_inner()
            .expect("the hook stores a report on success")),
        Err(graph_err) => match stash.into_inner() {
            Some(quantum_err) => Err(quantum_err),
            None => Err(QuantumError::CalculationError(format!(
                "freeze failed in a built-in check before the quantum hook: {}",
                graph_err
            ))),
        },
    }
}
