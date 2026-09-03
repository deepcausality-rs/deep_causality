/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::decision::{Check, CheckItem, CheckReport};
use crate::types::pipeline::config::{
    CodeSubject, Config, ModelSubject, PlantSubject, QclBuilder, Structural,
};
use crate::types::qcm::faithfulness::CausalStructure;
use crate::types::qcm::hypothesis::{Hypothesis, Marginalised};
use crate::types::qcm::markov_freeze::{
    CommutatorTolerance, QuantumMarkovReport, freeze_quantum, markov_certificate,
    quantum_markov_check_report,
};
use crate::types::qcm::process_factors::{FactorSupports, ProcessFactors};
use crate::types::qcode::css_code::{CssCode, LdpcWeights, check_ldpc_weights, derive_code};
use crate::types::qcode::diagonal_phase::DiagonalPhase;
use crate::types::qcode::logical_equivalence::LogicalBasis;
use crate::types::qgates::gates_haruna::logical_hadamard;
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality::CausableGraph;
use deep_causality_algebra::RealField;
use deep_causality_homology::ChainComplex;
use deep_causality_num::{FromPrimitive, NaturalNumber};
use deep_causality_num_complex::Complex;
use deep_causality_tensor::CausalTensor;

/// The word width of the 𝔽₂ chains the code stages carry.
pub type Word = u64;

/// Why a screened report is or is not current.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScreenStatus<R> {
    /// The report describes the subject as it stands.
    Current,
    /// The subject was marginalised after screening; the report's margins are stale, and the
    /// amplification is what a carried-forward margin would be degraded by.
    Invalidated {
        /// `√(d_B)` from the boundary warrant.
        amplification: R,
    },
}

/// What `validate` terminates in: the config, the admitted candidates, and the report.
#[derive(Clone)]
pub struct Screened<R: RealField, N: NaturalNumber, S> {
    config: Config<R, N, S>,
    admitted: Vec<Hypothesis<R>>,
    stages: Vec<(&'static str, CheckReport<R>)>,
    report: CheckReport<R>,
    status: ScreenStatus<R>,
}

impl<R: RealField, N: NaturalNumber, S> Screened<R, N, S> {
    /// The configuration that was screened.
    pub fn config(&self) -> &Config<R, N, S> {
        &self.config
    }

    /// The candidates that passed every check.
    pub fn admitted(&self) -> &[Hypothesis<R>] {
        &self.admitted
    }

    /// Each stage's report, in order.
    pub fn stages(&self) -> &[(&'static str, CheckReport<R>)] {
        &self.stages
    }

    /// The folded report, when it is current. `None` after a marginalisation: the pre-trace
    /// margins are not readable as current margins.
    pub fn report(&self) -> Option<&CheckReport<R>> {
        match self.status {
            ScreenStatus::Current => Some(&self.report),
            ScreenStatus::Invalidated { .. } => None,
        }
    }

    /// The stale report's worst margin degraded by the amplification, after a marginalisation.
    pub fn stale_report_degraded(&self) -> Option<R> {
        match self.status {
            ScreenStatus::Current => None,
            ScreenStatus::Invalidated { amplification } => {
                self.report.worst_margin().map(|m| m * amplification)
            }
        }
    }

    /// Whether the report is current.
    pub fn status(&self) -> ScreenStatus<R> {
        self.status
    }
}

impl<R, N, G, T> Screened<R, N, ModelSubject<R, G, T>>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber,
    G: Clone,
    T: Clone,
{
    /// Marginalise the screened factorization, gated on the boundary warrant, invalidating the
    /// report. The traced operator and its warrant come back beside the invalidated screen.
    ///
    /// # Errors
    ///
    /// As `Hypothesis::marginalise`.
    pub fn marginalise(
        &self,
        kept_legs: usize,
        z: &CausalTensor<Complex<R>>,
        tolerance: R,
    ) -> Result<(Marginalised<R>, Self), QuantumError> {
        let subject = self.config.subject();
        let h = Hypothesis::structural(
            "model",
            subject.factors().clone(),
            subject.supports().clone(),
        )?;
        let m = h.marginalise(kept_legs, z, tolerance)?;
        let invalidated = Self {
            config: self.config.clone(),
            admitted: self.admitted.clone(),
            stages: self.stages.clone(),
            report: self.report.clone(),
            status: ScreenStatus::Invalidated {
                amplification: m.warrant.amplification,
            },
        };
        Ok((m, invalidated))
    }
}

/// The validate stage: checks accumulate, the first failure is sticky, and `finalize` terminates
/// in a [`Screened`] or carries the structured error out.
pub struct Validate<'c, R: RealField, N: NaturalNumber, S> {
    cfg: &'c Config<R, N, S>,
    stages: Vec<(&'static str, CheckReport<R>)>,
    admitted: Vec<Hypothesis<R>>,
    failure: Option<QuantumError>,
    code: Option<CssCode<Word>>,
    ldpc: Option<LdpcWeights<R>>,
}

impl QclBuilder {
    /// The validate stage over a built configuration. Nothing in the config is mutated by any
    /// check, so a failed validation leaves the subject exactly as it was.
    pub fn validate<R, N, S>(cfg: &Config<R, N, S>) -> Validate<'_, R, N, S>
    where
        R: RealField,
        N: NaturalNumber,
    {
        Validate {
            cfg,
            stages: Vec::new(),
            admitted: Vec::new(),
            failure: None,
            code: None,
            ldpc: None,
        }
    }

    /// The shipped freeze on a dynamic graph: freezes under the Markov check and, when systems are
    /// declared, the decomposability check, rolling the graph back to its dynamic state on failure
    /// and carrying the structured error out. This is the path a caller holding an unfrozen graph
    /// takes; the model subject of a configuration takes a frozen one and mutates nothing.
    ///
    /// # Errors
    ///
    /// As `freeze_quantum`.
    pub fn freeze_model<T, G, R>(
        graph: &mut G,
        state_writers: &[usize],
        factors: &ProcessFactors<R>,
        supports: &FactorSupports,
        tolerance: &CommutatorTolerance<R>,
        systems: Option<(&[usize], &[usize])>,
    ) -> Result<QuantumMarkovReport<R>, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
        R: RealField + FromPrimitive + Default,
    {
        freeze_quantum(graph, state_writers, factors, supports, tolerance, systems)
    }
}

impl<'c, R, N, S> Validate<'c, R, N, S>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber,
    S: Clone,
{
    fn record(&mut self, name: &'static str, report: CheckReport<R>) {
        self.stages.push((name, report));
    }

    fn fail(&mut self, e: QuantumError) {
        if self.failure.is_none() {
            self.failure = Some(e);
        }
    }

    fn folded(&self) -> CheckReport<R> {
        self.stages
            .iter()
            .fold(CheckReport::vacuous(), |acc, (_, r)| acc.fold(r.clone()))
    }

    /// The screen, or the first structured error a stage raised.
    ///
    /// # Errors
    ///
    /// The first stage failure, as the structured `QuantumError` it produced.
    pub fn finalize(self) -> Result<Screened<R, N, S>, QuantumError> {
        if let Some(e) = self.failure {
            return Err(e);
        }
        let report = self.folded();
        Ok(Screened {
            config: self.cfg.clone(),
            admitted: self.admitted,
            stages: self.stages,
            report,
            status: ScreenStatus::Current,
        })
    }

    /// The CSS code derived so far, on a code subject.
    pub fn code(&self) -> Option<&CssCode<Word>> {
        self.code.as_ref()
    }

    /// The LDPC weights measured so far, on a code subject.
    pub fn ldpc(&self) -> Option<&LdpcWeights<R>> {
        self.ldpc.as_ref()
    }
}

// ---------------------------------------------------------------------------
// The model subject: the two level checks freeze_quantum runs, on a frozen graph.
// ---------------------------------------------------------------------------

impl<'c, R, N, G, T> Validate<'c, R, N, ModelSubject<R, G, T>>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber,
    G: CausableGraph<T> + Clone,
    T: Clone,
{
    /// The Markov commutativity check over intersecting supports. A rejecting pair is the
    /// structured `CommutatorNonZero` the shipped check raises.
    pub fn check_markov(mut self, tolerance: &CommutatorTolerance<R>) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let s = self.cfg.subject();
        match quantum_markov_check_report(s.factors(), s.supports(), tolerance) {
            Ok(report) => {
                if let Err(e) = markov_certificate(&report) {
                    self.fail(e);
                }
                self.record("check_markov", report);
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// C₃-exclusion over the frozen graph's reachability between the declared systems.
    pub fn check_decomposable(mut self) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let s = self.cfg.subject();
        if s.inputs().is_empty() || s.outputs().is_empty() {
            self.fail(QuantumError::CalculationError(
                "check_decomposable needs declared input and output systems; call declare_systems"
                    .into(),
            ));
            return self;
        }
        match CausalStructure::from_graph_reachability::<T, G>(s.graph(), s.inputs(), s.outputs())
            .and_then(|cs| cs.check_c3_exclusion().map(|()| cs))
        {
            Ok(cs) => {
                let blocks = choose3(cs.inputs().len()) * choose3(cs.outputs().len());
                let check = Check::new(CheckItem::Whole, R::zero(), R::zero());
                self.record("check_decomposable", CheckReport::new(vec![check], blocks));
            }
            Err(e) => self.fail(e),
        }
        self
    }
}

// ---------------------------------------------------------------------------
// The plant subject with structural candidates: each candidate screened, the admitted kept.
// ---------------------------------------------------------------------------

impl<'c, R, N, const D: usize> Validate<'c, R, N, PlantSubject<R, D, Structural>>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber,
{
    /// The Markov check on every structural candidate. A candidate whose factors fail is not
    /// admitted; a structural failure of the check itself is the stage's failure.
    pub fn check_markov(mut self, tolerance: &CommutatorTolerance<R>) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let mut admitted = Vec::new();
        let mut folded = CheckReport::vacuous();
        for h in self.cfg.subject().candidates() {
            match h.check_markov(tolerance) {
                Ok(certified) => {
                    let report = certified.certificate().cloned().expect("just certified");
                    if report.accepted() {
                        admitted.push(certified);
                    }
                    folded = folded.fold(report);
                }
                Err(e) => {
                    self.fail(e);
                    return self;
                }
            }
        }
        self.admitted = admitted;
        self.record("check_markov", folded);
        self
    }

    /// C₃-exclusion for every admitted candidate over the structure its own supports encode,
    /// between the declared systems. A candidate containing a `C₃` is not admitted. Each
    /// structural candidate implies a structure of its own, and the supports carry it, so no
    /// graph is needed here.
    pub fn check_decomposable(mut self, inputs: &[usize], outputs: &[usize]) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let pool: Vec<Hypothesis<R>> = if self.stages.is_empty() {
            self.cfg.subject().candidates().to_vec()
        } else {
            core::mem::take(&mut self.admitted)
        };
        let mut admitted = Vec::new();
        let mut folded = CheckReport::vacuous();
        for h in pool {
            match h.check_decomposable_from_supports(inputs, outputs) {
                Ok(report) => {
                    folded = folded.fold(report);
                    admitted.push(h);
                }
                Err(QuantumError(crate::QuantumErrorEnum::NotFaithfullyRepresentable(_))) => {}
                Err(e) => {
                    self.fail(e);
                    return self;
                }
            }
        }
        self.admitted = admitted;
        self.record("check_decomposable", folded);
        self
    }

    /// C₃-exclusion for every admitted candidate over `graph`'s reachability between the declared
    /// systems, for candidates whose structure lives in a graph rather than in their supports.
    pub fn check_decomposable_with<T, G>(
        mut self,
        graph: &G,
        inputs: &[usize],
        outputs: &[usize],
    ) -> Self
    where
        T: Clone,
        G: CausableGraph<T>,
    {
        if self.failure.is_some() {
            return self;
        }
        let pool: Vec<Hypothesis<R>> = if self.stages.is_empty() {
            self.cfg.subject().candidates().to_vec()
        } else {
            core::mem::take(&mut self.admitted)
        };
        let mut admitted = Vec::new();
        let mut folded = CheckReport::vacuous();
        for h in pool {
            match h.check_decomposable(graph, inputs, outputs) {
                Ok(report) => {
                    folded = folded.fold(report);
                    admitted.push(h);
                }
                Err(QuantumError(crate::QuantumErrorEnum::NotFaithfullyRepresentable(_))) => {}
                Err(e) => {
                    self.fail(e);
                    return self;
                }
            }
        }
        self.admitted = admitted;
        self.record("check_decomposable", folded);
        self
    }
}

// ---------------------------------------------------------------------------
// The code subject: exact checks over the chain complex.
// ---------------------------------------------------------------------------

impl<'c, R, N, K> Validate<'c, R, N, CodeSubject<K>>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
    N: NaturalNumber,
    K: ChainComplex + Clone,
{
    /// The CSS code: `n`, `k` and the two check families, read off the complex.
    pub fn derive_code(mut self) -> Self {
        if self.failure.is_some() {
            return self;
        }
        match derive_code::<Word, K>(self.cfg.subject().complex()) {
            Ok(code) => {
                let k = R::from_usize(code.k()).unwrap_or_else(R::zero);
                self.record(
                    "derive_code",
                    CheckReport::new(vec![Check::new(CheckItem::Whole, R::zero(), k)], code.n()),
                );
                self.code = Some(code);
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// Both weights of both check matrices against `bound`.
    pub fn check_ldpc_weights(mut self, bound: usize) -> Self {
        if self.failure.is_some() {
            return self;
        }
        if self.code.is_none() {
            self = self.derive_code();
            if self.failure.is_some() {
                return self;
            }
        }
        let code = self.code.as_ref().expect("derived above");
        match check_ldpc_weights::<R, Word>(code, bound) {
            Ok(w) => {
                self.record("check_ldpc_weights", w.report.clone());
                self.ldpc = Some(w);
            }
            Err(e) => self.fail(e),
        }
        self
    }

    /// Class invariance of `Z̄`, `S̄` and `T̄` on every logical qubit, decided over the code space.
    /// One record per `(class, gate)`, exact.
    pub fn check_class_invariance(mut self) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let complex = self.cfg.subject().complex();
        let basis = match LogicalBasis::<Word>::from_complex(complex, 1) {
            Ok(b) => b,
            Err(e) => {
                self.fail(e);
                return self;
            }
        };
        let code = match self.code.clone() {
            Some(c) => c,
            None => match derive_code::<Word, K>(complex) {
                Ok(c) => c,
                Err(e) => {
                    self.fail(e);
                    return self;
                }
            },
        };
        let mut checks = Vec::new();
        for gamma in basis.homology() {
            for (label, gate) in [
                ("Z", DiagonalPhase::z(gamma.clone())),
                ("S", DiagonalPhase::s(gamma.clone())),
                ("T", DiagonalPhase::t(gamma.clone())),
            ] {
                match basis.check_class_invariance(&gate, code.z_generators()) {
                    Ok(r) => {
                        let measured = if r.holds { R::zero() } else { R::one() };
                        checks.push(Check::new(
                            CheckItem::Index(checks.len()),
                            measured,
                            R::zero(),
                        ));
                        let _ = label;
                    }
                    Err(e) => {
                        self.fail(e);
                        return self;
                    }
                }
            }
        }
        self.record("check_class_invariance", CheckReport::from_checks(checks));
        self.code = Some(code);
        self
    }

    /// The Clifford check of `H̄` on every logical qubit, against a dual cochain.
    pub fn check_clifford_action(mut self) -> Self {
        if self.failure.is_some() {
            return self;
        }
        let complex = self.cfg.subject().complex();
        let basis = match LogicalBasis::<Word>::from_complex(complex, 1) {
            Ok(b) => b,
            Err(e) => {
                self.fail(e);
                return self;
            }
        };
        let mut checks = Vec::new();
        for gamma in basis.homology() {
            let dual = match complex.dual_representative::<Word>(gamma, 1) {
                Ok(Some(d)) => d,
                Ok(None) => {
                    self.fail(QuantumError::CalculationError(
                        "a homology class pairs to zero with every cocycle; the pairing is \
                         degenerate on it"
                            .into(),
                    ));
                    return self;
                }
                Err(e) => {
                    self.fail(QuantumError::CalculationError(format!("{e}")));
                    return self;
                }
            };
            let program = match logical_hadamard::<Word, R>(gamma, &dual) {
                Ok((ops, _phase)) => ops,
                Err(e) => {
                    self.fail(e);
                    return self;
                }
            };
            match basis.check_clifford_action(&program, gamma, &dual) {
                Ok(r) => {
                    let measured = if r.holds { R::zero() } else { R::one() };
                    checks.push(Check::new(
                        CheckItem::Index(checks.len()),
                        measured,
                        R::zero(),
                    ));
                }
                Err(e) => {
                    self.fail(e);
                    return self;
                }
            }
        }
        self.record("check_clifford_action", CheckReport::from_checks(checks));
        self
    }
}

fn choose3(n: usize) -> usize {
    if n < 3 { 0 } else { n * (n - 1) * (n - 2) / 6 }
}

#[allow(dead_code)]
fn _name_witness() -> String {
    String::new()
}
