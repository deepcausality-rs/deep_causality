/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The C₃-exclusion faithfulness criterion (van der Lugt & Lorenz,
//! arXiv:2508.11762, Thm 3.2). A causal structure `G` — a bipartite relation
//! between input and output systems — is faithfully representable by a
//! traditional (non-routed) circuit **iff** it has the C₃-exclusion property:
//! it contains no `C₃` sub-relation between three inputs and three outputs.
//!
//! Up to relabelling there is a unique such obstruction: `C₃` is the bipartite
//! 6-cycle `K_{3,3}` minus a perfect matching — a 3×3 induced sub-relation in
//! which every one of the three inputs relates to exactly two of the three
//! outputs and every output to exactly two of the inputs (canonically two
//! commuting CNOTs, `U₃`). A `C₃`-containing structure provably has no
//! traditional-circuit faithful decomposition and is rejected at freeze.

use crate::QuantumError;
use deep_causality::CausableGraph;
use std::collections::BTreeSet;

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
    /// A 3×3 induced sub-relation is a `C₃` iff every chosen input relates to
    /// exactly two of the chosen outputs and every chosen output to exactly two
    /// of the chosen inputs — which forces the three non-edges to form a perfect
    /// matching, i.e. `K_{3,3}` minus a matching = the bipartite 6-cycle.
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

    /// Whether the 3×3 induced block on `rows × cols` is a `C₃`: every row has
    /// exactly two edges into `cols`, and every column exactly two edges from
    /// `rows`.
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
        row_deg.iter().all(|&d| d == 2) && col_deg.iter().all(|&d| d == 2)
    }

    /// The freeze-time faithfulness check: `Ok(())` if the structure is
    /// C₃-exclusion (faithfully representable by a traditional circuit),
    /// otherwise [`QuantumError::NotFaithfullyRepresentable`] identifying the C₃
    /// obstruction.
    pub fn check_c3_exclusion(&self) -> Result<(), QuantumError> {
        match self.find_c3() {
            None => Ok(()),
            Some((rows, cols)) => Err(QuantumError::NotFaithfullyRepresentable(format!(
                "causal structure contains a C₃ sub-relation between inputs {:?} and outputs {:?}; \
                 it has no traditional-circuit causally faithful decomposition",
                rows, cols
            ))),
        }
    }
}
