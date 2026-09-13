/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Barrett–Lorenz–Oreshkov dilation of a circuit model: the process-operator factorization a
//! circuit with broken wires induces (arXiv:1906.10726, Theorem on unitary circuits with broken
//! wires), in the crate's node-keyed store.
//!
//! # What the factors are
//!
//! A node `A` of the model's grouping is a laboratory with an input space `A^in`, the state of its
//! wires as they enter, and an output space `A^out`, their state as they leave. The mechanism
//! `ρ_{A|Pa(A)}` delivers `A^in` from the parents' outputs: it is the Choi operator of the wiring,
//! the identity channel on each wire a parent hands to `A`, a `|0⟩⟨0|` preparation on each fresh
//! line, and `I/d` on each declared free input. The boxes of `A` are what happens between `A^in`
//! and `A^out`: the default *instrument* at `A`, carried beside the factors as the Choi operator of
//! the node's channel. `Tr(σ · ⊗_A τ_A)` with `σ = ∏ ρ_{A|Pa(A)}` is then the Born rule of the
//! process-operator formalism, and it reproduces the circuit's own probabilities, which is what the
//! tests check against the Kraus-level semantics.
//!
//! # The leg convention
//!
//! `FactorSupports` names one leg per node. Leg `A` has dimension `d_A · d_A`, the input index
//! outer and the output index inner in the row-major layout, each index row-major over the node's
//! wires in ascending order. `ρ_{A|Pa(A)}` is embedded as the identity on `A^out`, on every parent's
//! input half, and on every parent-output wire that does not feed `A`. Under that convention the
//! factors commute pairwise by construction, which is what the Markov check certifies on every
//! fixture.

use crate::QuantumError;
use crate::types::circuit_model::qc_morphism::leg_index_map;
use crate::types::circuit_model::{
    CircuitBox, CircuitModel, NodeId, NumericCaps, QcMorphism, WireId, gate_unitary,
};
use crate::types::qcm::hypothesis::Hypothesis;
use crate::types::qcm::process_factors::{FactorSupports, ProcessFactors};
use crate::types::qgates::channel::choi_from_kraus;
use crate::types::qgates::operator_linalg::embed_on_legs;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

/// The most entries one factor is allowed to occupy: a factor on legs of dimension `d²` each is
/// dense on their product, and three two-qubit nodes already reach `2^24`.
pub const DILATION_ENTRY_CAP: u64 = 1 << 24;

/// Where a wire enters a node from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireSource {
    /// Handed over by a parent node's output.
    Parent(NodeId),
    /// A fresh line, prepared in `|0⟩`.
    Fresh,
    /// A declared free input, carried as `I/d`.
    Input,
}

/// One node's leg: its wires ascending, their dimensions, and the node dimension `d`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLeg {
    wires: Vec<WireId>,
    dims: Vec<usize>,
    d: usize,
}

impl NodeLeg {
    /// The node's wires, ascending.
    pub fn wires(&self) -> &[WireId] {
        &self.wires
    }

    /// The node dimension `d`, so the leg has dimension `d²`.
    pub fn d(&self) -> usize {
        self.d
    }

    fn digit(&self, mut idx: usize) -> Vec<usize> {
        let mut out = vec![0usize; self.dims.len()];
        for k in (0..self.dims.len()).rev() {
            out[k] = idx % self.dims[k];
            idx /= self.dims[k];
        }
        out
    }
}

/// A dilated circuit: the factorization, its supports, the default instruments and the wiring.
#[derive(Debug, Clone, PartialEq)]
pub struct Dilated<R: RealField> {
    factors: ProcessFactors<R>,
    supports: FactorSupports,
    instruments: BTreeMap<NodeId, CausalTensor<Complex<R>>>,
    legs: Vec<NodeLeg>,
    sources: BTreeMap<(NodeId, WireId), WireSource>,
}

impl<R> Dilated<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The factor store, one factor per node.
    pub fn factors(&self) -> &ProcessFactors<R> {
        &self.factors
    }

    /// The support registry, leg `A` of dimension `d_A²` on `{A} ∪ Pa(A)`.
    pub fn supports(&self) -> &FactorSupports {
        &self.supports
    }

    /// The default instrument at each node: the Choi operator of the node's boxes.
    pub fn instruments(&self) -> &BTreeMap<NodeId, CausalTensor<Complex<R>>> {
        &self.instruments
    }

    /// The legs, one per node.
    pub fn legs(&self) -> &[NodeLeg] {
        &self.legs
    }

    /// Where wire `w` enters node `n` from.
    pub fn source(&self, node: NodeId, wire: WireId) -> Option<WireSource> {
        self.sources.get(&(node, wire)).copied()
    }

    /// The factorization as a structural hypothesis with provenance `Rederived`.
    ///
    /// # Errors
    ///
    /// As `Hypothesis::structural`.
    pub fn hypothesis(&self, name: impl Into<String>) -> Result<Hypothesis<R>, QuantumError> {
        Hypothesis::structural(name, self.factors.clone(), self.supports.clone())
    }

    /// The joint instrument `⊗_A τ_A` on the union of the legs, with the named nodes' instruments
    /// replaced. A measurement is expressed by replacing the last node's instrument with the Choi
    /// operator of the projector composed onto its channel.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CalculationError`] when the joint operator on the union of the legs would
    /// have more entries than [`DILATION_ENTRY_CAP`], before anything is embedded, since the
    /// per-factor cap bounds each factor on its own support only;
    /// [`QuantumError::DimensionMismatch`] if an override is not `d_A² × d_A²`; the embedding's
    /// errors.
    pub fn joint_instrument(
        &self,
        overrides: &BTreeMap<NodeId, CausalTensor<Complex<R>>>,
    ) -> Result<CausalTensor<Complex<R>>, QuantumError> {
        let dimension = self.legs.iter().fold(1u64, |acc, leg| {
            acc.saturating_mul((leg.d as u64).saturating_mul(leg.d as u64))
        });
        let entries = dimension.saturating_mul(dimension);
        if entries > DILATION_ENTRY_CAP {
            return Err(QuantumError::CalculationError(format!(
                "the joint instrument on the union of the legs has dimension {dimension} and would \
                 have {entries} entries, above the dilation cap of {DILATION_ENTRY_CAP}"
            )));
        }
        let all: BTreeSet<usize> = (0..self.legs.len()).collect();
        let space = self.supports.space_map(&all);
        let mut joint: Option<CausalTensor<Complex<R>>> = None;
        for node in 0..self.legs.len() {
            let tau = overrides
                .get(&node)
                .or_else(|| self.instruments.get(&node))
                .ok_or_else(|| {
                    QuantumError::CalculationError(format!("node {node} has no instrument"))
                })?;
            let d2 = self.legs[node].d * self.legs[node].d;
            if tau.shape() != [d2, d2] {
                return Err(QuantumError::DimensionMismatch(format!(
                    "the instrument at node {node} has shape {:?}, its leg is {d2} × {d2}",
                    tau.shape()
                )));
            }
            let legs: BTreeSet<usize> = [node].into_iter().collect();
            let embedded = embed_on_legs(tau, &legs, &space)?;
            joint = Some(match joint {
                None => embedded,
                Some(acc) => acc
                    .matmul(&embedded)
                    .map_err(|e| QuantumError::CalculationError(format!("matmul: {e:?}")))?,
            });
        }
        joint.ok_or_else(|| QuantumError::CalculationError("the model has no nodes".into()))
    }

    /// `Re Tr(σ · τ)`: the process-operator Born rule for the joint instrument `τ`.
    ///
    /// # Errors
    ///
    /// As `Hypothesis::evaluate`.
    pub fn predict(&self, instrument: &CausalTensor<Complex<R>>) -> Result<R, QuantumError> {
        self.hypothesis("dilation")?.evaluate(instrument)
    }
}

impl<R> CircuitModel<R>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    /// The dilation. See the module documentation.
    ///
    /// # Errors
    ///
    /// [`QuantumError::CyclicStructureUnsupported`] if the grouping's induced DAG has a cycle;
    /// [`QuantumError::CalculationError`] if a box is not a unitary or a channel, since a classical
    /// wire has no place in a quantum causal model, if the model has no nodes, or if a factor would
    /// exceed [`DILATION_ENTRY_CAP`]; the embedding's dimension errors.
    pub fn dilation(&self) -> Result<Dilated<R>, QuantumError> {
        let dag = self.induced_dag();
        if dag.has_cycle() {
            return Err(QuantumError::CyclicStructureUnsupported(
                "the grouping's induced DAG has a directed cycle; a cyclic model has no BLO dilation \
                 in this crate"
                    .into(),
            ));
        }
        if self.nodes().is_empty() {
            return Err(QuantumError::CalculationError(
                "a model with no nodes has no dilation".into(),
            ));
        }
        for (b, bx) in self.boxes().iter().enumerate() {
            if !matches!(
                bx,
                CircuitBox::Unitary { .. } | CircuitBox::Channel { .. } | CircuitBox::Kraus { .. }
            ) {
                return Err(QuantumError::CalculationError(format!(
                    "box {b} is a {}; a dilation needs a quantum-only model of unitaries and channels",
                    bx.kind()
                )));
            }
        }

        // Legs: each node's wires ascending.
        let mut legs = Vec::with_capacity(self.nodes().len());
        for members in self.nodes() {
            let mut wires: Vec<WireId> = members
                .iter()
                .flat_map(|&b| self.boxes()[b].quantum_wires().iter().copied())
                .collect();
            wires.sort_unstable();
            wires.dedup();
            let dims: Vec<usize> = wires
                .iter()
                .map(|&w| self.wires()[w].cardinality())
                .collect();
            let d = dims
                .iter()
                .try_fold(1usize, |a, &x| a.checked_mul(x))
                .ok_or_else(|| {
                    QuantumError::DimensionMismatch("a node's dimension overflows usize".into())
                })?;
            legs.push(NodeLeg { wires, dims, d });
        }

        // Sources: the first box of a node to touch a wire says where the wire came from.
        let inputs: BTreeSet<WireId> = self.inputs().iter().copied().collect();
        let mut last_node: Vec<Option<NodeId>> = vec![None; self.wires().len()];
        let mut sources: BTreeMap<(NodeId, WireId), WireSource> = BTreeMap::new();
        for (b, bx) in self.boxes().iter().enumerate() {
            let node = self.node_of(b).expect("validated");
            for &w in bx.quantum_wires() {
                match last_node[w] {
                    Some(prev) if prev == node => {}
                    Some(prev) => {
                        sources.entry((node, w)).or_insert(WireSource::Parent(prev));
                    }
                    None => {
                        let src = if inputs.contains(&w) {
                            WireSource::Input
                        } else {
                            WireSource::Fresh
                        };
                        sources.entry((node, w)).or_insert(src);
                    }
                }
                last_node[w] = Some(node);
            }
        }

        let mut supports = FactorSupports::new();
        let mut factors = ProcessFactors::new();
        for (node, leg) in legs.iter().enumerate() {
            supports.set_leg_dim(node, leg.d * leg.d);
        }
        for node in 0..legs.len() {
            let parents: BTreeSet<NodeId> = legs[node]
                .wires
                .iter()
                .filter_map(|&w| match sources.get(&(node, w)) {
                    Some(WireSource::Parent(p)) => Some(*p),
                    _ => None,
                })
                .collect();
            let mut support: Vec<usize> = parents.iter().copied().collect();
            support.push(node);
            support.sort_unstable();
            supports.declare(node, &support);
            factors.insert(node, factor_for(node, &support, &legs, &sources)?);
        }

        // Default instruments: the Choi operator of each node's boxes on its own wires.
        let mut instruments = BTreeMap::new();
        for (node, members) in self.nodes().iter().enumerate() {
            instruments.insert(node, node_instrument(self, &legs[node], members)?);
        }

        supports.validate(&factors)?;
        Ok(Dilated {
            factors,
            supports,
            instruments,
            legs,
            sources,
        })
    }
}

/// `ρ_{A|Pa(A)}` on the legs `support` (ascending), dense, by the delta rule of the module doc.
fn factor_for<R>(
    node: NodeId,
    support: &[usize],
    legs: &[NodeLeg],
    sources: &BTreeMap<(NodeId, WireId), WireSource>,
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let leg_dims: Vec<usize> = support.iter().map(|&l| legs[l].d * legs[l].d).collect();
    let total = leg_dims
        .iter()
        .try_fold(1u64, |a, &d| a.checked_mul(d as u64))
        .ok_or_else(|| {
            QuantumError::DimensionMismatch("the factor's dimension overflows".into())
        })?;
    let entries = total.saturating_mul(total);
    if entries > DILATION_ENTRY_CAP {
        return Err(QuantumError::CalculationError(format!(
            "the factor at node {node} would have {entries} entries, above the dilation cap of {DILATION_ENTRY_CAP}"
        )));
    }
    let total = total as usize;
    let position = |l: usize| support.iter().position(|&s| s == l).expect("in support");

    // Decompose a joint index into, per leg in `support`, (input digits, output digits).
    let decompose = |mut idx: usize| -> Vec<(Vec<usize>, Vec<usize>)> {
        let mut out = vec![(Vec::new(), Vec::new()); support.len()];
        for k in (0..support.len()).rev() {
            let leg = &legs[support[k]];
            let d2 = leg.d * leg.d;
            let li = idx % d2;
            idx /= d2;
            let (in_idx, out_idx) = (li / leg.d, li % leg.d);
            out[k] = (leg.digit(in_idx), leg.digit(out_idx));
        }
        out
    };
    let a = position(node);
    let leg_a = &legs[node];
    // Which wire of a parent feeds A: (parent position, wire position in parent) -> wire position in A.
    let mut feeds: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for (wa, &w) in leg_a.wires.iter().enumerate() {
        if let Some(WireSource::Parent(p)) = sources.get(&(node, w)) {
            let wp = legs[*p]
                .wires
                .iter()
                .position(|&x| x == w)
                .expect("parent carries the wire");
            feeds.insert((position(*p), wp), wa);
        }
    }
    let mut weight_inputs = R::one();
    for (wa, &w) in leg_a.wires.iter().enumerate() {
        if sources.get(&(node, w)) == Some(&WireSource::Input) {
            let d = R::from_usize(leg_a.dims[wa]).ok_or_else(|| {
                QuantumError::CalculationError(
                    "the scalar cannot represent a wire dimension".into(),
                )
            })?;
            weight_inputs /= d;
        }
    }

    let zero = Complex::new(R::zero(), R::zero());
    let mut data = vec![zero; total * total];
    for r in 0..total {
        let rd = decompose(r);
        for c in 0..total {
            let cd = decompose(c);
            let mut ok = true;
            // A's output half is identity.
            ok &= rd[a].1 == cd[a].1;
            // Each wire of A's input half by its source.
            for (wa, &w) in leg_a.wires.iter().enumerate() {
                match sources.get(&(node, w)) {
                    Some(WireSource::Parent(p)) => {
                        let pp = position(*p);
                        let wp = legs[*p].wires.iter().position(|&x| x == w).expect("wire");
                        ok &= rd[a].0[wa] == rd[pp].1[wp] && cd[a].0[wa] == cd[pp].1[wp];
                    }
                    Some(WireSource::Fresh) => ok &= rd[a].0[wa] == 0 && cd[a].0[wa] == 0,
                    Some(WireSource::Input) | None => ok &= rd[a].0[wa] == cd[a].0[wa],
                }
                if !ok {
                    break;
                }
            }
            if !ok {
                continue;
            }
            // Parents: input half identity, non-feeding output wires identity.
            for (k, &l) in support.iter().enumerate() {
                if l == node {
                    continue;
                }
                ok &= rd[k].0 == cd[k].0;
                for wp in 0..legs[l].wires.len() {
                    if !feeds.contains_key(&(k, wp)) {
                        ok &= rd[k].1[wp] == cd[k].1[wp];
                    }
                }
                if !ok {
                    break;
                }
            }
            if ok {
                data[r * total + c] = Complex::new(weight_inputs, R::zero());
            }
        }
    }
    Ok(CausalTensor::from_slice(&data, &[total, total]))
}

/// The Choi operator of a node's boxes as one channel on the node's wires.
fn node_instrument<R>(
    model: &CircuitModel<R>,
    leg: &NodeLeg,
    members: &[usize],
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let space: BTreeMap<usize, usize> = leg
        .wires
        .iter()
        .copied()
        .zip(leg.dims.iter().copied())
        .collect();
    let d = leg.d;
    let mut identity = vec![Complex::new(R::zero(), R::zero()); d * d];
    for i in 0..d {
        identity[i * d + i] = Complex::new(R::one(), R::zero());
    }
    let mut family: Vec<CausalTensor<Complex<R>>> =
        vec![CausalTensor::from_slice(&identity, &[d, d])];
    let caps = NumericCaps::default();
    let mut ordered = members.to_vec();
    ordered.sort_unstable();
    for b in ordered {
        match &model.boxes()[b] {
            CircuitBox::Unitary { wires, program } => {
                for op in program {
                    let (local, m) = gate_unitary::<R>(op)?;
                    let in_gate_order: Vec<WireId> = local.iter().map(|&q| wires[q]).collect();
                    let (gate_wires, m) = in_ascending_wire_order(&m, &in_gate_order, &space)?;
                    let embedded = embed_on_legs(&m, &gate_wires, &space)?;
                    for k in &mut family {
                        *k = embedded.matmul(k).map_err(|e| {
                            QuantumError::CalculationError(format!("matmul: {e:?}"))
                        })?;
                    }
                }
            }
            CircuitBox::Channel { wires, channel } => {
                let kraus = QcMorphism::from_channel(channel)?.kraus();
                family = compose_kraus(family, &kraus, wires, &space, &caps)?;
            }
            CircuitBox::Kraus { wires, kraus } => {
                family = compose_kraus(family, kraus, wires, &space, &caps)?;
            }
            other => {
                return Err(QuantumError::CalculationError(format!(
                    "box {b} is a {}; a dilation needs a quantum-only model",
                    other.kind()
                )));
            }
        }
    }
    choi_from_kraus(&family)
}

/// Every product `K_j · F_i` of a box's Kraus operators, embedded on the leg, with the family so
/// far; the operator count is capped.
fn compose_kraus<R>(
    family: Vec<CausalTensor<Complex<R>>>,
    kraus: &[CausalTensor<Complex<R>>],
    wires: &[usize],
    space: &BTreeMap<usize, usize>,
    caps: &NumericCaps,
) -> Result<Vec<CausalTensor<Complex<R>>>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    let count = (family.len() as u64).saturating_mul(kraus.len() as u64);
    if count > caps.max_operators {
        return Err(QuantumError::KrausFamilyExceeded(count, caps.max_operators));
    }
    let mut next = Vec::with_capacity(count as usize);
    for k in kraus {
        let (box_wires, k) = in_ascending_wire_order(k, wires, space)?;
        let embedded = embed_on_legs(&k, &box_wires, space)?;
        for f in &family {
            next.push(
                embedded
                    .matmul(f)
                    .map_err(|e| QuantumError::CalculationError(format!("matmul: {e:?}")))?,
            );
        }
    }
    Ok(next)
}

/// An operator given on legs in a box's wire order, permuted to ascending wire order, which is the
/// order `embed_on_legs` reads its legs in. A box may list its wires in any order, the first being
/// its most significant leg; the numeric semantics reads that order and the dilation must agree.
fn in_ascending_wire_order<R>(
    op: &CausalTensor<Complex<R>>,
    wires: &[WireId],
    space: &BTreeMap<usize, usize>,
) -> Result<(BTreeSet<WireId>, CausalTensor<Complex<R>>), QuantumError>
where
    R: RealField,
{
    let dims: Vec<usize> = wires
        .iter()
        .map(|w| {
            space.get(w).copied().ok_or_else(|| {
                QuantumError::DimensionMismatch(format!("wire {w} is not on the node's leg"))
            })
        })
        .collect::<Result<_, _>>()?;
    let d: usize = dims.iter().product();
    if op.shape() != [d, d] {
        return Err(QuantumError::DimensionMismatch(format!(
            "an operator of shape {:?} on wires {wires:?} of dimension {d}",
            op.shape()
        )));
    }
    let legs: BTreeSet<WireId> = wires.iter().copied().collect();
    let mut order: Vec<usize> = (0..wires.len()).collect();
    order.sort_by_key(|&i| wires[i]);
    if order.iter().enumerate().all(|(k, &i)| k == i) {
        return Ok((legs, op.clone()));
    }
    let map = leg_index_map(&dims, &order, d)?;
    let src = op.as_slice();
    let mut data = vec![Complex::new(R::zero(), R::zero()); d * d];
    for r in 0..d {
        for c in 0..d {
            data[map[r] * d + map[c]] = src[r * d + c];
        }
    }
    Ok((legs, CausalTensor::from_slice(&data, &[d, d])))
}
