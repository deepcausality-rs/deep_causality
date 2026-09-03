/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A hypothesis: a structural candidate as a factorization, or a mechanism candidate as a channel.
//!
//! A structural candidate is the triple `{ name, ProcessFactors<R>, FactorSupports }` and nothing
//! more. Its causal structure is *derived* from a frozen graph on demand through
//! `CausalStructure::from_graph_reachability`, never stored beside the factors, so a stored
//! structure can never disagree with the factorization it describes. A mechanism candidate
//! declares no factorization; it carries the [`Channel`] it hypothesises and reaches `control`
//! without the structural checks.
//!
//! # Two interventions, one built
//!
//! A quantum causal model has two interventions and they differ. The factor `ρ_{A|Pa(A)}` is the
//! *mechanism* delivering A's input from its parents' outputs, and replacing it is the
//! mechanism-level `do()`, the classical analogue; that is [`Hypothesis::intervene_mechanism`].
//! Barrett–Lorenz–Oreshkov's canonical intervention fixes the *instrument* at the node, what
//! happens between A's input and A's output, and `predict` differs under the two. v1 supplies the
//! first only and models a probe as a factor replacement, with that consequence stated on
//! [`Hypothesis::predict`]; `intervene_instrument` is the name reserved for the second and it is
//! not built.
//!
//! # A certificate does not survive a change to what it certified
//!
//! `quantum_markov_check` measures `‖[ρ_j, ρ_k]‖_F` over the factors it is given, so its margins
//! describe the store as it stood. An intervention replaces a factor and a marginalisation traces
//! one away; both drop the certificate, and a stage that needs one runs the check again. A
//! composite of exactly two certified factors may inherit, marked `Inherited`, because there
//! pairwise commutation follows from the hermiticity Def 3.3 assumes of the composite; at any other
//! arity the literature has no theorem and neither does this type.

use crate::QuantumError;
use crate::types::carriers::Channel;
use crate::types::decision::{Check, CheckItem, CheckReport, Factorization};
use crate::types::qcm::faithfulness::CausalStructure;
use crate::types::qcm::markov_freeze::{CommutatorTolerance, quantum_markov_check_report_as};
use crate::types::qcm::process_factors::{CjFactor, FactorSupports, ProcessFactors};
use crate::types::qgates::operator_linalg::{
    BoundaryWarrant, embed_on_legs, partial_trace, partial_trace_preservation_boundary, square_dim,
};
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality::CausableGraph;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

/// A structural or mechanism candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct Hypothesis<R: RealField> {
    name: String,
    candidate: Candidate<R>,
}

#[derive(Debug, Clone, PartialEq)]
enum Candidate<R: RealField> {
    Structural {
        factors: ProcessFactors<R>,
        supports: FactorSupports,
        certificate: Option<CheckReport<R>>,
    },
    Mechanism {
        channel: Channel<R>,
    },
}

/// What a marginalisation returns when its warrant holds.
///
/// The traced operator, the warrant that licensed it, and, if the factorization carried a Markov
/// certificate, that certificate's worst margin degraded by the warrant's amplification. No
/// certificate is carried: the traced model's Markov property is not what the old report stood
/// for, and a stage that needs one runs the check on the traced model.
#[derive(Debug, Clone, PartialEq)]
pub struct Marginalised<R> {
    /// `Tr_B(σ)`, on the kept legs.
    pub operator: CausalTensor<Complex<R>>,
    /// The warrant that licensed the trace, with its `√(d_B)` amplification.
    pub warrant: BoundaryWarrant<R>,
    /// The prior certificate's worst margin times the amplification, if there was a certificate.
    pub prior_margin_degraded: Option<R>,
}

impl<R> Hypothesis<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// A structural candidate, admitted only when its supports validate against its factors.
    ///
    /// # Errors
    ///
    /// The shipped `FactorSupports::validate` error: a non-square, empty or mis-dimensioned
    /// factor, a factor on an undeclared node, or a support whose dimensions overflow.
    pub fn structural(
        name: impl Into<String>,
        factors: ProcessFactors<R>,
        supports: FactorSupports,
    ) -> Result<Self, QuantumError> {
        supports.validate(&factors)?;
        Ok(Self {
            name: name.into(),
            candidate: Candidate::Structural {
                factors,
                supports,
                certificate: None,
            },
        })
    }

    /// A structural candidate whose supports are derived from a **frozen** graph under the
    /// single-system-per-node convention, `support(Aᵢ) = {Aᵢ} ∪ Pa(Aᵢ)`.
    ///
    /// # Errors
    ///
    /// `FactorSupports::from_graph`'s errors, an unfrozen graph or a factor keyed past
    /// `number_nodes()`, and then the validation errors of [`structural`](Self::structural).
    pub fn structural_from_graph<T, G>(
        name: impl Into<String>,
        graph: &G,
        factors: ProcessFactors<R>,
    ) -> Result<Self, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
    {
        let supports = FactorSupports::from_graph(graph, &factors)?;
        Self::structural(name, factors, supports)
    }

    /// A mechanism candidate: a plant modification, carried as a channel. No factorization, no
    /// derived structure, no structural checks.
    pub fn mechanism(name: impl Into<String>, channel: Channel<R>) -> Self {
        Self {
            name: name.into(),
            candidate: Candidate::Mechanism { channel },
        }
    }

    /// The name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether this is a structural candidate.
    pub fn is_structural(&self) -> bool {
        matches!(self.candidate, Candidate::Structural { .. })
    }

    /// Whether this is a mechanism candidate.
    pub fn is_mechanism(&self) -> bool {
        matches!(self.candidate, Candidate::Mechanism { .. })
    }

    /// The factor store, for a structural candidate.
    pub fn factors(&self) -> Option<&ProcessFactors<R>> {
        match &self.candidate {
            Candidate::Structural { factors, .. } => Some(factors),
            Candidate::Mechanism { .. } => None,
        }
    }

    /// The support registry, for a structural candidate.
    pub fn supports(&self) -> Option<&FactorSupports> {
        match &self.candidate {
            Candidate::Structural { supports, .. } => Some(supports),
            Candidate::Mechanism { .. } => None,
        }
    }

    /// The channel, for a mechanism candidate.
    pub fn channel(&self) -> Option<&Channel<R>> {
        match &self.candidate {
            Candidate::Mechanism { channel } => Some(channel),
            Candidate::Structural { .. } => None,
        }
    }

    /// The Markov certificate, if one has been established for the factors as they stand.
    pub fn certificate(&self) -> Option<&CheckReport<R>> {
        match &self.candidate {
            Candidate::Structural { certificate, .. } => certificate.as_ref(),
            Candidate::Mechanism { .. } => None,
        }
    }

    fn structural_parts(&self) -> Result<(&ProcessFactors<R>, &FactorSupports), QuantumError> {
        match &self.candidate {
            Candidate::Structural {
                factors, supports, ..
            } => Ok((factors, supports)),
            Candidate::Mechanism { .. } => Err(QuantumError::CalculationError(format!(
                "hypothesis '{}' is a mechanism candidate and declares no factorization",
                self.name
            ))),
        }
    }

    /// The causal structure, derived from `graph`'s reachability over the declared systems.
    /// Derived each time; nothing is stored.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a mechanism candidate, and the shipped errors of
    /// `CausalStructure::from_graph_reachability`, an unfrozen graph or an out-of-range system id.
    pub fn structure<T, G>(
        &self,
        graph: &G,
        inputs: &[usize],
        outputs: &[usize],
    ) -> Result<CausalStructure, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
    {
        self.structural_parts()?;
        CausalStructure::from_graph_reachability::<T, G>(graph, inputs, outputs)
    }

    /// The decomposability gate: `check_c3_exclusion` on the freshly derived structure.
    ///
    /// A structure containing `C₃` is rejected with the shipped
    /// [`QuantumError::NotFaithfullyRepresentable`], which names the witnessing inputs and
    /// outputs a record could not carry; the check is exact, so it reports the obstruction rather
    /// than a margin. A structure without one returns a report whose examined count is the number
    /// of `3 × 3` blocks the search ranged over, `C(m, 3) · C(n, 3)`, so a structure too small to
    /// contain a `C₃` reads as a vacuous pass rather than a certified one.
    ///
    /// # Errors
    ///
    /// As [`structure`](Self::structure), and `NotFaithfullyRepresentable` on a `C₃`.
    pub fn check_decomposable<T, G>(
        &self,
        graph: &G,
        inputs: &[usize],
        outputs: &[usize],
    ) -> Result<CheckReport<R>, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
    {
        let structure = self.structure(graph, inputs, outputs)?;
        structure.check_c3_exclusion()?;
        let blocks = choose3(structure.inputs().len()) * choose3(structure.outputs().len());
        let check = Check::new(CheckItem::Whole, R::zero(), R::zero());
        Ok(CheckReport::new(vec![check], blocks))
    }

    /// The causal structure the supports encode, with no graph in sight.
    ///
    /// Under the flat convention `support(A) = {A} ∪ Pa(A)`, every leg of a node's support that is
    /// itself a factor node is a parent of that node, so the supports carry the DAG and its
    /// reachability. Input `i` influences output `o` when `o` is reachable from `i` along child
    /// edges, or `i == o`. This is what lets a structural candidate be screened for
    /// decomposability without an external graph, which matters when each candidate implies a
    /// structure of its own.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a mechanism candidate.
    pub fn structure_from_supports(
        &self,
        inputs: &[usize],
        outputs: &[usize],
    ) -> Result<CausalStructure, QuantumError> {
        let (factors, supports) = self.structural_parts()?;
        let nodes: BTreeSet<usize> = factors.nodes().collect();
        let children = |node: usize| -> Vec<usize> {
            nodes
                .iter()
                .copied()
                .filter(|&child| {
                    child != node
                        && supports
                            .support(child)
                            .is_some_and(|legs| legs.contains(&node))
                })
                .collect()
        };
        let out_set: BTreeSet<usize> = outputs.iter().copied().collect();
        let mut structure = CausalStructure::new(inputs, outputs);
        for &i in inputs {
            let mut seen = BTreeSet::new();
            let mut stack = vec![i];
            while let Some(node) = stack.pop() {
                if !seen.insert(node) {
                    continue;
                }
                if out_set.contains(&node) {
                    structure.add_influence(i, node);
                }
                stack.extend(children(node));
            }
        }
        Ok(structure)
    }

    /// The decomposability gate on the structure the supports encode; see
    /// [`check_decomposable`](Self::check_decomposable) for the report and the rejection.
    ///
    /// # Errors
    ///
    /// As [`structure_from_supports`](Self::structure_from_supports), and
    /// `NotFaithfullyRepresentable` on a `C₃`.
    pub fn check_decomposable_from_supports(
        &self,
        inputs: &[usize],
        outputs: &[usize],
    ) -> Result<CheckReport<R>, QuantumError> {
        let structure = self.structure_from_supports(inputs, outputs)?;
        structure.check_c3_exclusion()?;
        let blocks = choose3(structure.inputs().len()) * choose3(structure.outputs().len());
        let check = Check::new(CheckItem::Whole, R::zero(), R::zero());
        Ok(CheckReport::new(vec![check], blocks))
    }

    /// The Markov check on the factors as they stand, returning the hypothesis carrying its
    /// certificate. The receiver is unchanged.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a mechanism candidate, and the structural errors of
    /// `quantum_markov_check_report`. A non-commuting pair is not an error here; it is a rejecting
    /// record in the certificate, and `markov_certificate` turns it into one.
    pub fn check_markov(&self, tolerance: &CommutatorTolerance<R>) -> Result<Self, QuantumError> {
        self.check_markov_as(tolerance, Factorization::Rederived)
    }

    /// [`check_markov`](Self::check_markov) with the provenance of the factors stated, for a
    /// re-check on a composite's inherited factors.
    ///
    /// # Errors
    ///
    /// As [`check_markov`](Self::check_markov).
    pub fn check_markov_as(
        &self,
        tolerance: &CommutatorTolerance<R>,
        factorization: Factorization,
    ) -> Result<Self, QuantumError> {
        let (factors, supports) = self.structural_parts()?;
        let report = quantum_markov_check_report_as(factors, supports, tolerance, factorization)?;
        Ok(Self {
            name: self.name.clone(),
            candidate: Candidate::Structural {
                factors: factors.clone(),
                supports: supports.clone(),
                certificate: Some(report),
            },
        })
    }

    /// `do(node ← factor)`: the mechanism-level intervention, as a keyed replacement followed by
    /// revalidation. The receiver is unchanged, and the result carries no certificate.
    ///
    /// The surgery touches one key and leaves every other factor identical. The revalidation is
    /// what makes it safe: a replacement of the wrong dimension would otherwise sit in a store
    /// that the Markov check embeds through `embed_on_legs`.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a mechanism candidate;
    /// [`QuantumError::DimensionMismatch`] from `FactorSupports::validate` when the replacement's
    /// dimension disagrees with the node's declared support, or when the node has no declared
    /// support. On either the hypothesis passed in is left as it was.
    pub fn intervene_mechanism(
        &self,
        node: usize,
        factor: CjFactor<R>,
    ) -> Result<Self, QuantumError> {
        let (factors, supports) = self.structural_parts()?;
        let mut replaced = factors.clone();
        replaced.insert(node, factor);
        supports.validate(&replaced)?;
        Ok(Self {
            name: self.name.clone(),
            candidate: Candidate::Structural {
                factors: replaced,
                supports: supports.clone(),
                certificate: None,
            },
        })
    }

    /// The joint operator `σ = ∏ᵢ ρ_{Aᵢ|Pa(Aᵢ)}` on the union of the supports, built by embedding.
    ///
    /// `space_map` gives the leg-to-dimension map of the union, `embed_on_legs` lifts each factor
    /// onto it as the identity elsewhere, and the product in ascending node order is the joint
    /// operator. The evaluation stays on the full union of legs, so it invokes no partial trace
    /// and needs no warrant. For a Markov factorization the factors commute and the order is
    /// immaterial; for one that is not, the product is not a process operator and the Markov
    /// check is what says so.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a mechanism candidate or an empty store, and the
    /// embedding's dimension errors.
    pub fn joint_operator(&self) -> Result<CausalTensor<Complex<R>>, QuantumError> {
        let (factors, supports) = self.structural_parts()?;
        let (union, space) = union_space(factors, supports)?;
        let mut joint: Option<CausalTensor<Complex<R>>> = None;
        for node in factors.nodes() {
            let legs: BTreeSet<usize> = supports
                .support(node)
                .expect("validated at construction")
                .iter()
                .copied()
                .collect();
            let embedded =
                embed_on_legs(factors.get(node).expect("node from nodes()"), &legs, &space)?;
            joint = Some(match joint {
                None => embedded,
                Some(acc) => acc
                    .matmul(&embedded)
                    .map_err(|e| QuantumError::CalculationError(format!("matmul: {e:?}")))?,
            });
        }
        let _ = union;
        joint.ok_or_else(|| {
            QuantumError::CalculationError(format!(
                "hypothesis '{}' has no factors to contract",
                self.name
            ))
        })
    }

    /// The ascending legs the joint operator acts on, and their dimensions.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] on a mechanism candidate.
    pub fn legs(&self) -> Result<BTreeMap<usize, usize>, QuantumError> {
        let (factors, supports) = self.structural_parts()?;
        Ok(union_space(factors, supports)?.1)
    }

    /// Model evaluation: `Re Tr(σ · τ)` for an instrument operator `τ` on the joint space.
    ///
    /// # Errors
    ///
    /// The errors of [`joint_operator`](Self::joint_operator);
    /// [`QuantumError::DimensionMismatch`] if `instrument` is not square of the joint dimension;
    /// [`QuantumError::NonFiniteValue`] if the trace has a non-negligible imaginary part.
    pub fn evaluate(&self, instrument: &CausalTensor<Complex<R>>) -> Result<R, QuantumError> {
        let joint = self.joint_operator()?;
        let d = square_dim(&joint)?;
        if square_dim(instrument)? != d {
            return Err(QuantumError::DimensionMismatch(format!(
                "instrument is {} × {} but the joint operator is {} × {}",
                square_dim(instrument)?,
                square_dim(instrument)?,
                d,
                d
            )));
        }
        // Tr(σ τ) = Σ_ij σ_ij τ_ji.
        let s = joint.as_slice();
        let t = instrument.as_slice();
        let (mut re, mut im) = (R::zero(), R::zero());
        for i in 0..d {
            for j in 0..d {
                let a = s[i * d + j];
                let b = t[j * d + i];
                re += a.re * b.re - a.im * b.im;
                im += a.re * b.im + a.im * b.re;
            }
        }
        if im.abs() > R::epsilon().sqrt() {
            return Err(QuantumError::NonFiniteValue(format!(
                "model evaluation has a non-negligible imaginary part ({im:?})"
            )));
        }
        Ok(re)
    }

    /// `predict`: the world under the probe, evaluated. The probe is a **mechanism-level**
    /// intervention, `do(node ← factor)` through [`intervene_mechanism`](Self::intervene_mechanism),
    /// and then [`evaluate`](Self::evaluate) with `instrument`. v1 models every probe this way; a
    /// probe that is physically an instrument choice at the node is approximated by the factor
    /// replacement, and `intervene_instrument` is reserved for the exact form.
    ///
    /// The receiver is unchanged. Counting the evaluation on a ledger is the pipeline's business.
    ///
    /// # Errors
    ///
    /// As the two operations it composes.
    pub fn predict(
        &self,
        node: usize,
        factor: CjFactor<R>,
        instrument: &CausalTensor<Complex<R>>,
    ) -> Result<R, QuantumError> {
        self.intervene_mechanism(node, factor)?.evaluate(instrument)
    }

    /// The boundary warrant for tracing every leg past the first `kept_legs` out of the joint
    /// operator, with `z` the operator on the kept legs whose commutation is to survive.
    ///
    /// `z` is passed to `partial_trace_preservation_boundary`, which constructs `Z ⊗ 1_B` itself,
    /// so the theorem's shape hypothesis holds by construction. This is the typed form of what
    /// [`marginalise`](Self::marginalise) decides on.
    ///
    /// # Errors
    ///
    /// The errors of [`joint_operator`](Self::joint_operator), and the boundary function's shape
    /// errors, including `kept_legs` of zero or at least the leg count.
    pub fn boundary_warrant(
        &self,
        kept_legs: usize,
        z: &CausalTensor<Complex<R>>,
        tolerance: R,
    ) -> Result<BoundaryWarrant<R>, QuantumError> {
        let (joint, dims) = self.split(kept_legs)?;
        partial_trace_preservation_boundary(z, &joint, dims, tolerance)
    }

    /// Marginalise the factorization onto its first `kept_legs` legs, gated on the boundary
    /// warrant. Refuses when the warrant does not hold, before any call to `partial_trace`.
    ///
    /// A held warrant travels with the result, degraded by its amplification: the traced
    /// commutator is bounded by `√(d_B) · residual`, and a prior certificate's worst margin is
    /// reported times that factor, while the certificate itself is not carried.
    ///
    /// # Errors
    ///
    /// [`QuantumError::BoundaryNotHeld`] carrying the residual, the tolerance and the
    /// amplification when the warrant fails; otherwise as
    /// [`boundary_warrant`](Self::boundary_warrant).
    pub fn marginalise(
        &self,
        kept_legs: usize,
        z: &CausalTensor<Complex<R>>,
        tolerance: R,
    ) -> Result<Marginalised<R>, QuantumError> {
        let (joint, dims) = self.split(kept_legs)?;
        let warrant = partial_trace_preservation_boundary(z, &joint, dims, tolerance)?;
        if !warrant.holds {
            return Err(QuantumError::BoundaryNotHeld(format!(
                "‖[Z ⊗ 1_B, σ]‖_F = {:?} exceeds the tolerance {:?}; the traced commutator could \
                 only be bounded by {:?} × that residual, and nothing was traced",
                warrant.hypothesis_residual, warrant.tolerance, warrant.amplification
            )));
        }
        let operator = partial_trace(&joint, &dims, &[1])?;
        let prior_margin_degraded = self
            .certificate()
            .and_then(|c| c.worst_margin())
            .map(|m| m * warrant.amplification);
        Ok(Marginalised {
            operator,
            warrant,
            prior_margin_degraded,
        })
    }

    /// The joint operator and its `[d_kept, d_traced]` split at the first `kept_legs` legs.
    fn split(
        &self,
        kept_legs: usize,
    ) -> Result<(CausalTensor<Complex<R>>, [usize; 2]), QuantumError> {
        let legs = self.legs()?;
        if kept_legs == 0 || kept_legs >= legs.len() {
            return Err(QuantumError::PartialTraceShape(format!(
                "cannot keep {} of {} legs: a marginalisation keeps at least one and traces at \
                 least one",
                kept_legs,
                legs.len()
            )));
        }
        let dims: Vec<usize> = legs.values().copied().collect();
        let d_kept = dims[..kept_legs]
            .iter()
            .try_fold(1usize, |a, &d| a.checked_mul(d))
            .ok_or_else(|| {
                QuantumError::DimensionMismatch("kept leg dimensions overflow usize".into())
            })?;
        let d_traced = dims[kept_legs..]
            .iter()
            .try_fold(1usize, |a, &d| a.checked_mul(d))
            .ok_or_else(|| {
                QuantumError::DimensionMismatch("traced leg dimensions overflow usize".into())
            })?;
        Ok((self.joint_operator()?, [d_kept, d_traced]))
    }

    /// The composite of two structural candidates over disjoint node keys.
    ///
    /// The factor stores and support registries are unioned. A certificate is inherited only when
    /// the composite has exactly two factors and both parts were certified: there, pairwise
    /// commutation follows from the hermiticity Def 3.3 assumes of the composite, and the
    /// inherited certificate is the parts' reports folded and marked `Inherited`. At any other
    /// arity the composite carries no certificate and a stage that needs one re-runs the check,
    /// because Lorenz (2022) footnote 11 and Lorenz & Barrett leave that case open.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] if either is a mechanism candidate or the node keys
    /// overlap; [`QuantumError::DimensionMismatch`] if the two registries disagree on a leg's
    /// dimension.
    pub fn compose(&self, other: &Self) -> Result<Self, QuantumError> {
        let (fa, sa) = self.structural_parts()?;
        let (fb, sb) = other.structural_parts()?;
        let mut factors = ProcessFactors::new();
        let mut supports = FactorSupports::new();
        for (store, registry) in [(fa, sa), (fb, sb)] {
            for node in store.nodes() {
                if factors.get(node).is_some() {
                    return Err(QuantumError::CalculationError(format!(
                        "cannot compose: both hypotheses carry a factor at node {node}"
                    )));
                }
                factors.insert(node, store.get(node).expect("node from nodes()").clone());
                let legs = registry.support(node).expect("validated at construction");
                for &leg in legs {
                    let dim = registry.leg_dim(leg);
                    let known = supports.leg_dim(leg);
                    if supports.support(node).is_none() && known != dim && known != 2 {
                        return Err(QuantumError::DimensionMismatch(format!(
                            "cannot compose: leg {leg} has dimension {known} in one part and {dim} in the other"
                        )));
                    }
                    supports.set_leg_dim(leg, dim);
                }
                supports.declare(node, legs);
            }
        }
        let certificate = match (self.certificate(), other.certificate()) {
            (Some(a), Some(b)) if factors.len() == 2 => Some(
                a.clone()
                    .fold(b.clone())
                    .with_factorization(Factorization::Inherited),
            ),
            _ => None,
        };
        let name = format!("{}∘{}", self.name, other.name);
        Self::structural(name.clone(), factors.clone(), supports.clone())?;
        Ok(Self {
            name,
            candidate: Candidate::Structural {
                factors,
                supports,
                certificate,
            },
        })
    }
}

/// The union of the supports of every factor, and its leg-to-dimension map.
fn union_space<R: RealField>(
    factors: &ProcessFactors<R>,
    supports: &FactorSupports,
) -> Result<(BTreeSet<usize>, BTreeMap<usize, usize>), QuantumError> {
    let mut union = BTreeSet::new();
    for node in factors.nodes() {
        let legs = supports.support(node).ok_or_else(|| {
            QuantumError::DimensionMismatch(format!(
                "node {node} has a factor but no declared support"
            ))
        })?;
        union.extend(legs.iter().copied());
    }
    let space = supports.space_map(&union);
    Ok((union, space))
}

/// `C(n, 3)`, the number of three-element subsets, as the block count a `C₃` search ranges over.
fn choose3(n: usize) -> usize {
    if n < 3 { 0 } else { n * (n - 1) * (n - 2) / 6 }
}
