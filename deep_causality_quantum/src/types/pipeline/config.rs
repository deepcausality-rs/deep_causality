/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use crate::types::carriers::{Observable, QuantumPlant};
use crate::types::design::Experiment;
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qcm::process_factors::{FactorSupports, ProcessFactors};
#[cfg(feature = "qpu")]
use crate::types::qpu::evidence::Evidence;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use deep_causality::CausableGraph;
use deep_causality_algebra::RealField;
use deep_causality_homology::ChainComplex;
use deep_causality_num::{FromPrimitive, NaturalNumber};

/// The one origin of every configuration, and the entry to each stage.
pub struct QclBuilder;

impl QclBuilder {
    /// A configuration at the two working types: `R` for accuracy, `N` for headroom. This is the
    /// single site where they are named; every tolerance in the run derives from `R::epsilon()`
    /// and every count is bounded on `N`.
    pub fn config<R, N>() -> ConfigBuilder<R, N, NoSubject>
    where
        R: RealField,
        N: NaturalNumber,
    {
        ConfigBuilder {
            subject: NoSubject,
            probes: Vec::new(),
            baseline: None,
            seed: 0,
            #[cfg(feature = "qpu")]
            evidence: None,
            _n: PhantomData,
        }
    }
}

/// The builder before a subject is named.
pub struct NoSubject;

/// Marker: the plant's candidates are mechanisms, which carry no structural claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mechanisms;

/// Marker: the plant's candidates are structural, and must be screened before `control`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Structural;

/// Marker: the plant has no candidates yet.
pub struct NoCandidates;

/// A system that evolves and is measured, with the observables it exposes and its candidates.
#[derive(Debug, Clone)]
pub struct PlantSubject<R: RealField, const D: usize, K> {
    plant: QuantumPlant<R>,
    observables: Vec<Observable<R, D>>,
    candidates: Vec<Hypothesis<R>>,
    _kind: PhantomData<K>,
}

impl<R: RealField, const D: usize, K> PlantSubject<R, D, K> {
    /// The plant.
    pub fn plant(&self) -> &QuantumPlant<R> {
        &self.plant
    }

    /// The observables the plant exposes.
    pub fn observables(&self) -> &[Observable<R, D>] {
        &self.observables
    }

    /// The candidates.
    pub fn candidates(&self) -> &[Hypothesis<R>] {
        &self.candidates
    }
}

/// A Choi–Jamiołkowski factorization over a frozen graph: the degenerate plant with one structural
/// candidate and no evidence, and what the shipped `freeze_quantum` callers map onto.
#[derive(Clone)]
pub struct ModelSubject<R: RealField, G, T> {
    graph: G,
    factors: ProcessFactors<R>,
    supports: FactorSupports,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    _t: PhantomData<T>,
}

impl<R: RealField, G, T> ModelSubject<R, G, T> {
    /// The frozen graph.
    pub fn graph(&self) -> &G {
        &self.graph
    }

    /// The factor store.
    pub fn factors(&self) -> &ProcessFactors<R> {
        &self.factors
    }

    /// The support registry.
    pub fn supports(&self) -> &FactorSupports {
        &self.supports
    }

    /// The declared input systems.
    pub fn inputs(&self) -> &[usize] {
        &self.inputs
    }

    /// The declared output systems.
    pub fn outputs(&self) -> &[usize] {
        &self.outputs
    }
}

/// A chain complex, evaluated exactly. No probes, no evidence.
#[derive(Clone)]
pub struct CodeSubject<K> {
    complex: K,
}

impl<K> CodeSubject<K> {
    /// The complex.
    pub fn complex(&self) -> &K {
        &self.complex
    }
}

/// A configuration under construction.
pub struct ConfigBuilder<R: RealField, N: NaturalNumber, S> {
    subject: S,
    probes: Vec<Experiment<R>>,
    baseline: Option<Experiment<R>>,
    seed: u64,
    #[cfg(feature = "qpu")]
    evidence: Option<Evidence<N>>,
    _n: PhantomData<N>,
}

impl<R: RealField, N: NaturalNumber> ConfigBuilder<R, N, NoSubject> {
    /// A plant and the observables it exposes. The observables' dimension is what `build()`
    /// checks the plant against, so a probe naming an observable the plant does not expose is
    /// refused there.
    pub fn over_plant<const D: usize>(
        self,
        plant: QuantumPlant<R>,
        observables: &[Observable<R, D>],
    ) -> ConfigBuilder<R, N, PlantSubject<R, D, NoCandidates>> {
        self.with_subject(PlantSubject {
            plant,
            observables: observables.to_vec(),
            candidates: Vec::new(),
            _kind: PhantomData,
        })
    }

    /// A factorization over a graph, which `build()` requires to be frozen.
    pub fn over_model<G, T>(
        self,
        graph: G,
        factors: ProcessFactors<R>,
        supports: FactorSupports,
    ) -> ConfigBuilder<R, N, ModelSubject<R, G, T>>
    where
        G: CausableGraph<T>,
        T: Clone,
    {
        self.with_subject(ModelSubject {
            graph,
            factors,
            supports,
            inputs: Vec::new(),
            outputs: Vec::new(),
            _t: PhantomData,
        })
    }

    /// A chain complex.
    pub fn over_code<K: ChainComplex>(self, complex: K) -> ConfigBuilder<R, N, CodeSubject<K>> {
        self.with_subject(CodeSubject { complex })
    }

    fn with_subject<S>(self, subject: S) -> ConfigBuilder<R, N, S> {
        ConfigBuilder {
            subject,
            probes: self.probes,
            baseline: self.baseline,
            seed: self.seed,
            #[cfg(feature = "qpu")]
            evidence: self.evidence,
            _n: PhantomData,
        }
    }
}

impl<R: RealField, N: NaturalNumber, const D: usize>
    ConfigBuilder<R, N, PlantSubject<R, D, NoCandidates>>
{
    /// Structural candidates. The config then reaches `control` only through `validate`.
    pub fn candidates(
        self,
        candidates: &[Hypothesis<R>],
    ) -> ConfigBuilder<R, N, PlantSubject<R, D, Structural>> {
        self.with_candidates(candidates)
    }

    /// Mechanism candidates. The config reaches `control` directly, since a mechanism carries no
    /// structural claim for `validate` to screen.
    pub fn mechanisms(
        self,
        candidates: &[Hypothesis<R>],
    ) -> ConfigBuilder<R, N, PlantSubject<R, D, Mechanisms>> {
        self.with_candidates(candidates)
    }

    fn with_candidates<K>(
        self,
        candidates: &[Hypothesis<R>],
    ) -> ConfigBuilder<R, N, PlantSubject<R, D, K>> {
        ConfigBuilder {
            subject: PlantSubject {
                plant: self.subject.plant,
                observables: self.subject.observables,
                candidates: candidates.to_vec(),
                _kind: PhantomData,
            },
            probes: self.probes,
            baseline: self.baseline,
            seed: self.seed,
            #[cfg(feature = "qpu")]
            evidence: self.evidence,
            _n: PhantomData,
        }
    }
}

impl<R: RealField, N: NaturalNumber, G, T> ConfigBuilder<R, N, ModelSubject<R, G, T>> {
    /// The input and output systems the decomposability check is stated over.
    pub fn declare_systems(mut self, inputs: &[usize], outputs: &[usize]) -> Self {
        self.subject.inputs = inputs.to_vec();
        self.subject.outputs = outputs.to_vec();
        self
    }
}

impl<R: RealField, N: NaturalNumber, S> ConfigBuilder<R, N, S> {
    /// The probe family the design stage chooses from.
    pub fn probes(mut self, probes: &[Experiment<R>]) -> Self {
        self.probes = probes.to_vec();
        self
    }

    /// The baseline experiment.
    pub fn baseline(mut self, baseline: Experiment<R>) -> Self {
        self.baseline = Some(baseline);
        self
    }

    /// The run seed, for reproducible draws.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// The evidence policy: naming a shot budget selects the emergent modality, and this method
    /// exists only under the `qpu` feature, so a default build refuses it at compile time.
    #[cfg(feature = "qpu")]
    pub fn evidence(mut self, evidence: Evidence<N>) -> Self {
        self.evidence = Some(evidence);
        self
    }

    /// The configuration, after the subject's preconditions. No stage has run.
    ///
    /// # Errors
    ///
    /// Whatever the subject refuses: see [`BuildSubject`] for each. Under `qpu`, a zero shot
    /// budget is refused here through `Evidence::into_budget`.
    pub fn build(self) -> Result<Config<R, N, S>, QuantumError>
    where
        S: BuildSubject,
    {
        self.subject.check()?;
        #[cfg(feature = "qpu")]
        if let Some(e) = &self.evidence {
            e.into_budget()?;
        }
        Ok(Config {
            subject: self.subject,
            probes: self.probes,
            baseline: self.baseline,
            seed: self.seed,
            #[cfg(feature = "qpu")]
            evidence: self.evidence,
            _n: PhantomData,
        })
    }
}

/// A built configuration: a subject that passed its preconditions, and what the stages share.
#[derive(Clone)]
pub struct Config<R: RealField, N: NaturalNumber, S> {
    subject: S,
    probes: Vec<Experiment<R>>,
    baseline: Option<Experiment<R>>,
    seed: u64,
    #[cfg(feature = "qpu")]
    evidence: Option<Evidence<N>>,
    _n: PhantomData<N>,
}

impl<R: RealField, N: NaturalNumber, S> Config<R, N, S> {
    /// The subject.
    pub fn subject(&self) -> &S {
        &self.subject
    }

    /// The probe family.
    pub fn probes(&self) -> &[Experiment<R>] {
        &self.probes
    }

    /// The baseline experiment.
    pub fn baseline(&self) -> Option<&Experiment<R>> {
        self.baseline.as_ref()
    }

    /// The run seed.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The evidence policy, if one was named.
    #[cfg(feature = "qpu")]
    pub fn evidence(&self) -> Option<&Evidence<N>> {
        self.evidence.as_ref()
    }
}

/// What `build()` checks of a subject before any stage runs.
pub trait BuildSubject {
    /// The subject's preconditions.
    ///
    /// # Errors
    ///
    /// A structured `QuantumError` naming the precondition that failed.
    fn check(&self) -> Result<(), QuantumError>;
}

impl<R, const D: usize> BuildSubject for PlantSubject<R, D, Mechanisms>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn check(&self) -> Result<(), QuantumError> {
        check_plant_common(self)?;
        if let Some(h) = self.candidates.iter().find(|h| !h.is_mechanism()) {
            return Err(QuantumError::CalculationError(format!(
                "candidate '{}' is structural; mechanisms() takes mechanism candidates only",
                h.name()
            )));
        }
        Ok(())
    }
}

impl<R, const D: usize> BuildSubject for PlantSubject<R, D, Structural>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    fn check(&self) -> Result<(), QuantumError> {
        check_plant_common(self)?;
        for h in &self.candidates {
            if !h.is_structural() {
                return Err(QuantumError::CalculationError(format!(
                    "candidate '{}' is a mechanism; candidates() takes structural candidates only",
                    h.name()
                )));
            }
            let (factors, supports) = (
                h.factors().expect("structural"),
                h.supports().expect("structural"),
            );
            if supports_have_cycle(factors, supports) {
                return Err(QuantumError::CyclicStructureUnsupported(format!(
                    "candidate '{}' declares a cyclic causal structure; cyclic causal structures \
                     are outside v1's scope by decision, not because they fail a criterion",
                    h.name()
                )));
            }
        }
        Ok(())
    }
}

fn check_plant_common<R, const D: usize, K>(s: &PlantSubject<R, D, K>) -> Result<(), QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    if s.candidates.is_empty() {
        return Err(QuantumError::CalculationError(
            "a plant configuration needs at least one candidate".into(),
        ));
    }
    if let Some(o) = s.observables.iter().find(|o| o.dim() != s.plant.dim()) {
        return Err(QuantumError::DimensionMismatch(format!(
            "observable '{}' is {}-dimensional but the plant is {}-dimensional: the plant does \
             not expose it",
            o.name(),
            o.dim(),
            s.plant.dim()
        )));
    }
    Ok(())
}

impl<R, G, T> BuildSubject for ModelSubject<R, G, T>
where
    R: RealField,
    G: CausableGraph<T>,
    T: Clone,
{
    fn check(&self) -> Result<(), QuantumError> {
        if self.factors.is_empty() {
            return Err(QuantumError::CalculationError(
                "the model subject has no factors; a factorization with no factor cannot be \
                 evaluated"
                    .into(),
            ));
        }
        if !self.graph.is_frozen() {
            return Err(QuantumError::CalculationError(
                "the model subject requires a frozen graph (dense node ids); freeze the graph \
                 before building the configuration"
                    .into(),
            ));
        }
        let n = self.graph.number_nodes();
        if let Some(bad) = self.factors.nodes().find(|&node| node >= n) {
            return Err(QuantumError::CalculationError(format!(
                "factor keyed by node {bad} but the frozen graph has {n} nodes (valid ids 0..{n})"
            )));
        }
        self.supports.validate(&self.factors)?;
        if let Some(bad) = self.inputs.iter().chain(&self.outputs).find(|&&id| id >= n) {
            return Err(QuantumError::CalculationError(format!(
                "declared system id {bad} is out of range for a graph with {n} nodes"
            )));
        }
        if graph_has_cycle(&self.graph) {
            return Err(QuantumError::CyclicStructureUnsupported(
                "the model's graph contains a directed cycle; cyclic causal structures are outside \
                 v1's scope by decision, not because they fail a criterion"
                    .into(),
            ));
        }
        Ok(())
    }
}

impl<K: ChainComplex> BuildSubject for CodeSubject<K> {
    fn check(&self) -> Result<(), QuantumError> {
        if self.complex.num_cells(1) == 0 {
            return Err(QuantumError::DimensionMismatch(
                "the code subject has no 1-cells, so no qubits".into(),
            ));
        }
        Ok(())
    }
}

/// Whether a frozen graph has a directed cycle: some node reaches itself along at least one edge.
fn graph_has_cycle<T: Clone, G: CausableGraph<T>>(graph: &G) -> bool {
    let n = graph.number_nodes();
    for start in 0..n {
        let mut seen = vec![false; n];
        let mut stack: Vec<usize> = (0..n).filter(|&s| graph.contains_edge(start, s)).collect();
        while let Some(node) = stack.pop() {
            if node == start {
                return true;
            }
            if seen[node] {
                continue;
            }
            seen[node] = true;
            for s in 0..n {
                if graph.contains_edge(node, s) {
                    stack.push(s);
                }
            }
        }
    }
    false
}

/// Whether the parent relation a support registry encodes has a cycle. Under the flat
/// convention `support(A) = {A} ∪ Pa(A)`, every leg of a node's support that is itself a factor
/// node is one of its parents.
fn supports_have_cycle<R: RealField>(
    factors: &ProcessFactors<R>,
    supports: &FactorSupports,
) -> bool {
    let nodes: BTreeSet<usize> = factors.nodes().collect();
    let parents = |node: usize| -> Vec<usize> {
        supports
            .support(node)
            .map(|legs| {
                legs.iter()
                    .copied()
                    .filter(|l| *l != node && nodes.contains(l))
                    .collect()
            })
            .unwrap_or_default()
    };
    for &start in &nodes {
        let mut seen = BTreeSet::new();
        let mut stack = parents(start);
        while let Some(node) = stack.pop() {
            if node == start {
                return true;
            }
            if !seen.insert(node) {
                continue;
            }
            stack.extend(parents(node));
        }
    }
    false
}
