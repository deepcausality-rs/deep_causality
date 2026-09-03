/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The C₃-exclusion criterion (van der Lugt & Lorenz, arXiv:2508.11762,
//! Definition 3.1 and Theorem 3.2). A causal structure `G` — a bipartite
//! influence relation between input and output systems — implies unitary
//! causally faithful decompositions **iff** it has the C₃-exclusion property:
//! no 3×3 induced sub-relation is `C₃`.
//!
//! "Faithful" is the Lorenz–Barrett sense throughout: a circuit decomposition
//! whose connectivity equals the unitary's causal structure, `G_U = G_C`
//! (Definition 2.11). It is not Pearl's, where the word names a distribution
//! with no independences beyond those its graph implies. The two properties are
//! unrelated and this module decides only the first.
//!
//! `C₃` is the causal structure of two commuting CNOTs (Example 2.12): exactly
//! two input–output pairs carry no influence, and they share neither an input
//! nor an output. Seven edges of nine, one input reaching every output and one
//! output reached by every input. Up to relabelling it is the unique obstruction
//! (Theorem 3.2, (iii) ⇒ (i) proved as Proposition 6.1), and the paper's
//! Remark 3.3 fixes what the theorem does not say: a *particular* unitary whose
//! structure is `C₃` may still decompose faithfully, and every unitary with
//! structure inside `C₃` has a *routed* decomposition (Lorenz & Barrett,
//! arXiv:2001.07774, Theorem 3). So the check rejects a structure rather than a
//! unitary, and only for the traditional, non-routed circuit paradigm.
//!
//! This module once tested for a different relation, the bipartite 6-cycle
//! `K₃,₃` minus a perfect matching, and so accepted the paper's `C₃` while
//! rejecting a structure the paper admits. The record is
//! `openspec/notes/quantum/qcl-corrections.md`, X-16.

use crate::QuantumError;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality::CausableGraph;

/// A causal structure: a bipartite influence relation between input systems and
/// output systems. `contains(i, o)` holds when input `i` influences output `o`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CausalStructure {
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    edges: BTreeSet<(usize, usize)>,
}

impl CausalStructure {
    /// A structure over the given input and output system ids (deduplicated,
    /// sorted). No influence edges yet.
    pub fn new(inputs: &[usize], outputs: &[usize]) -> Self {
        let mut inputs = inputs.to_vec();
        inputs.sort_unstable();
        inputs.dedup();
        let mut outputs = outputs.to_vec();
        outputs.sort_unstable();
        outputs.dedup();
        Self {
            inputs,
            outputs,
            edges: BTreeSet::new(),
        }
    }

    /// Declares that input `i` influences output `o`.
    pub fn add_influence(&mut self, i: usize, o: usize) -> &mut Self {
        self.edges.insert((i, o));
        self
    }

    /// Whether input `i` influences output `o`.
    pub fn influences(&self, i: usize, o: usize) -> bool {
        self.edges.contains(&(i, o))
    }

    /// The input system ids.
    pub fn inputs(&self) -> &[usize] {
        &self.inputs
    }

    /// The output system ids.
    pub fn outputs(&self) -> &[usize] {
        &self.outputs
    }

    /// Derives the causal structure from a **frozen** causal graph's
    /// reachability over caller-declared input and output nodes: input `i`
    /// influences output `o` iff there is a directed path `i → … → o` (or
    /// `i == o`), computed from the public `contains_edge` adjacency. This is
    /// the traditional-circuit influence relation the C₃ criterion is stated on.
    ///
    /// The graph **must be frozen**: only the static representation guarantees a
    /// dense node-id space `0..number_nodes()`, which the BFS relies on. On a
    /// dynamic graph a `remove_node` tombstones a slot without compacting, so a
    /// live node can have an id `≥ number_nodes()` and its edges would be
    /// silently skipped — an unsound false negative in a faithfulness gate.
    /// An unfrozen graph is therefore rejected.
    ///
    /// # Errors
    /// Returns [`QuantumError::CalculationError`] if the graph is not frozen.
    pub fn from_graph_reachability<T, G>(
        graph: &G,
        inputs: &[usize],
        outputs: &[usize],
    ) -> Result<Self, QuantumError>
    where
        T: Clone,
        G: CausableGraph<T>,
    {
        if !graph.is_frozen() {
            return Err(QuantumError::CalculationError(
                "CausalStructure::from_graph_reachability requires a frozen graph (dense node ids); \
                 freeze the graph before deriving the causal structure"
                    .into(),
            ));
        }
        let n = graph.number_nodes();
        // A declared system id outside `0..n` names no node; its reachability would
        // be silently empty, detaching the derived structure from the graph (and
        // hiding a real C₃). Reject it so the freeze hook rolls back.
        if let Some(&bad) = inputs.iter().chain(outputs).find(|&&id| id >= n) {
            return Err(QuantumError::CalculationError(format!(
                "declared system id {} is out of range for a frozen graph with {} node(s) \
                 (valid ids are 0..{})",
                bad, n, n
            )));
        }
        let mut me = Self::new(inputs, outputs);
        let out_set: BTreeSet<usize> = outputs.iter().copied().collect();
        for &i in &me.inputs.clone() {
            // BFS from i over forward edges.
            let mut seen = vec![false; n];
            let mut stack = vec![i];
            if i < n {
                seen[i] = true;
            }
            while let Some(node) = stack.pop() {
                if out_set.contains(&node) {
                    me.add_influence(i, node);
                }
                for (succ, seen_succ) in seen.iter_mut().enumerate() {
                    if !*seen_succ && graph.contains_edge(node, succ) {
                        *seen_succ = true;
                        stack.push(succ);
                    }
                }
            }
        }
        Ok(me)
    }

    /// Searches for a `C₃` sub-relation. Returns the witnessing three inputs and
    /// three outputs (each ascending) if one exists, else `None`.
    ///
    /// Definition 3.1 quantifies over every choice of three inputs and three
    /// outputs, so the search is over the induced 3×3 blocks and each block is
    /// decided by [`is_c3_block`](Self::is_c3_block).
    pub fn find_c3(&self) -> Option<([usize; 3], [usize; 3])> {
        let ins = &self.inputs;
        let outs = &self.outputs;
        if ins.len() < 3 || outs.len() < 3 {
            return None;
        }
        for a in 0..ins.len() {
            for b in (a + 1)..ins.len() {
                for c in (b + 1)..ins.len() {
                    let row = [ins[a], ins[b], ins[c]];
                    for x in 0..outs.len() {
                        for y in (x + 1)..outs.len() {
                            for z in (y + 1)..outs.len() {
                                let col = [outs[x], outs[y], outs[z]];
                                if self.is_c3_block(&row, &col) {
                                    return Some((row, col));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Whether the 3×3 induced block on `rows × cols` is `C₃`.
    ///
    /// `C₃` has seven edges, and its two non-edges lie in distinct rows and
    /// distinct columns (Example 2.12: `A₁ ↛ B₃` and `A₃ ↛ B₁`, influence
    /// everywhere else). Read as degree sequences that is `{2, 2, 3}` on both
    /// sides, and the reading is exact: seven edges force the row degrees to be
    /// `{3, 2, 2}` or `{3, 3, 1}`, and only the first leaves the two non-edges in
    /// different rows; the same on the columns. Checked against all 512 relations
    /// on three inputs and three outputs, this agrees with isomorphism to `C₃`
    /// and with Theorem 4.9(v) on every one.
    ///
    /// The 6-cycle, every degree exactly two, is not `C₃`. It has six edges and
    /// Theorem 4.9(v) admits it: any two outputs share exactly one parent, so the
    /// parent sets of overlapping output pairs are disjoint.
    fn is_c3_block(&self, rows: &[usize; 3], cols: &[usize; 3]) -> bool {
        let mut row_deg = [0u8; 3];
        let mut col_deg = [0u8; 3];
        for (ri, &i) in rows.iter().enumerate() {
            for (ci, &o) in cols.iter().enumerate() {
                if self.influences(i, o) {
                    row_deg[ri] += 1;
                    col_deg[ci] += 1;
                }
            }
        }
        row_deg.sort_unstable();
        col_deg.sort_unstable();
        row_deg == [2, 2, 3] && col_deg == [2, 2, 3]
    }

    /// The freeze-time decomposability check: `Ok(())` if the structure has the
    /// C₃-exclusion property and so implies unitary causally faithful
    /// decompositions in the traditional circuit paradigm (Theorem 3.2),
    /// otherwise [`QuantumError::NotFaithfullyRepresentable`] identifying the C₃
    /// obstruction. The error's "faithfully" is Lorenz–Barrett's `G_U = G_C`,
    /// not Pearl's; see the module docs.
    pub fn check_c3_exclusion(&self) -> Result<(), QuantumError> {
        match self.find_c3() {
            None => Ok(()),
            Some((rows, cols)) => Err(QuantumError::NotFaithfullyRepresentable(format!(
                "causal structure contains a C₃ sub-relation between inputs {:?} and outputs {:?}; \
                 it does not imply a unitary causally faithful decomposition in the traditional \
                 circuit paradigm (van der Lugt & Lorenz, Theorem 3.2)",
                rows, cols
            ))),
        }
    }
}
