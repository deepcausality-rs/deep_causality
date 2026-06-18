/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Clique-Picking **uniform MEC DAG sampler**.
//!
//! Ported from the authoritative `cliquepicking_rs::sample` (Wienöbst, Bannach &
//! Liśkiewicz, "Polynomial-Time Algorithms for Counting and Sampling Markov
//! Equivalence Classes", AAAI 2021). This is the polynomial-time uniform sampler
//! the reference BRCD uses (`cliquepicking::MecSampler.sample_dag`); it is the
//! drop-in replacement for the exponential AMO enumeration in
//! `brcd_mec::mec_sample_dag` / `representative_dag`.
//!
//! ## What it does
//!
//! A CPDAG decomposes into compelled arcs plus chain components — the connected
//! components of the undirected subgraph, each chordal. The members of the Markov
//! equivalence class are exactly: the compelled arcs together with one *acyclic
//! moral orientation* (AMO) chosen independently per chain component. A uniform
//! class member is therefore drawn by sampling one **uniform AMO per component**.
//!
//! Clique-Picking samples a uniform AMO by sampling a uniform *vertex ordering*
//! over the component's clique tree (the [`ComponentSampler`]) and then orienting
//! each undirected edge from the earlier to the later vertex in that ordering —
//! exactly the reference's `sample_dag` step.
//!
//! ## Reuse of the landed counter
//!
//! The sampler needs, per clique-tree flower, the AMO sub-count of each candidate
//! clique (the per-clique weights an inverse-CDF draw selects from). The
//! reference's sampler carries its **own** counting recursion separate from
//! `count.rs` for exactly this reason; this port mirrors that
//! ([`ComponentSampler::rec_count_init`] / [`ComponentSampler::rec_count_traversal`]),
//! so the landed public counter in [`count`](crate::dag_sampling::count) is left
//! completely untouched. The shared lower-level machinery
//! ([`CliqueTree`](crate::dag_sampling::clique_tree::CliqueTree),
//! [`combinatorics`](crate::dag_sampling::combinatorics),
//! [`Memoization`](crate::dag_sampling::memoization), etc.) is reused as-is.
//!
//! ## Generics
//!
//! The count/weight type is the same generic `T: RealField + FromPrimitive` the
//! counter uses (instantiates at `f64` and `deep_causality_num::Float106`).
//! Randomness is a generic `R: Rng` from `deep_causality_rand` (no external
//! `rand`). The reference's `BigUint` alias table — which relies on
//! `gen_biguint_below` for an O(1) exact draw — is replaced by an exact
//! **inverse-CDF (cumulative weight) draw** over the `T` weights: the selection
//! probabilities are exactly proportional to the integer AMO sub-counts; only the
//! single uniform variate is an `f64`, which does not bias which member is drawn.
//!
//! ## Preconditions
//!
//! Same contract as `brcd_mec`: the input is expected to be a valid CPDAG — every
//! edge is a directed arc or an undirected edge, the arc projection is acyclic,
//! and each chain component is chordal. Violations yield [`BrcdError`]
//! ([`NotACpdag`](BrcdErrorEnum::NotACpdag) / [`NotAcyclic`](BrcdErrorEnum::NotAcyclic)).
//! Chordality of a chain component is not separately checked; a non-chordal
//! component yields a wrong sample or a panic, exactly as documented for the
//! reference and the counter.

use crate::causal_discovery::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use crate::dag_sampling::chordal::mcs;
use crate::dag_sampling::clique_tree::CliqueTree;
use crate::dag_sampling::combinatorics;
use crate::dag_sampling::graph::Graph;
use crate::dag_sampling::index_set::IndexSet;
use crate::dag_sampling::lazy_tokens::LazyTokens;
use crate::dag_sampling::memoization::Memoization;
use crate::dag_sampling::utils::inverse_permutation;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_rand::Rng;
use deep_causality_topology::{EdgeKind, MixedGraph};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Converts an `f64` (a uniform variate in `[0, 1)`) into the count type `T`.
#[inline]
fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("a finite f64 is representable in every RealField")
}

/// An inverse-CDF weighted selector over a flower's candidate cliques.
///
/// Replaces the reference's `BigUint` alias table. The weights are the per-clique
/// AMO sub-counts in the count type `T`; [`sample`](WeightedChoice::sample) draws
/// an index with probability exactly proportional to its weight by a cumulative
/// comparison against a single uniform variate. For a flower this candidate list
/// is small, so the linear scan is not a bottleneck.
#[derive(Debug, Clone)]
struct WeightedChoice<T> {
    /// Cumulative weight prefix sums; `cumulative[i]` is the sum of weights `0..=i`.
    cumulative: Vec<T>,
    /// Total weight (`cumulative.last()`), cached for the uniform draw.
    total: T,
}

impl<T: RealField + FromPrimitive> WeightedChoice<T> {
    /// Builds an empty selector (never sampled; placeholder for unfilled flowers).
    fn empty() -> Self {
        WeightedChoice {
            cumulative: Vec::new(),
            total: T::zero(),
        }
    }

    /// Builds a selector from per-candidate weights.
    fn new(weights: &[T]) -> Self {
        let mut cumulative = Vec::with_capacity(weights.len());
        let mut running = T::zero();
        for &w in weights {
            running += w;
            cumulative.push(running);
        }
        WeightedChoice {
            cumulative,
            total: running,
        }
    }

    /// Draws a candidate index with probability proportional to its weight.
    ///
    /// A single uniform `f64` in `[0, 1)` scales the total weight; the result is
    /// located by linear scan over the cumulative prefix sums. The fallback to
    /// the last index covers the measure-zero `target == total` rounding edge.
    fn sample<R: Rng>(&self, rng: &mut R) -> usize {
        let u: f64 = rng.random::<f64>();
        let target = self.total * from_f64::<T>(u);
        for (i, c) in self.cumulative.iter().enumerate() {
            if target < *c {
                return i;
            }
        }
        self.cumulative.len() - 1
    }
}

/// The per-component uniform AMO ordering sampler.
///
/// Built from one connected, chordal undirected [`Graph`]. It precomputes, for
/// every clique-tree flower, a [`WeightedChoice`] over that flower's candidate
/// cliques (weighted by AMO sub-count) and the forbidden-prefix data; a draw
/// produces a uniform topological vertex ordering of the component.
#[derive(Debug)]
struct ComponentSampler<T> {
    /// Number of original-graph vertices in the component.
    n: usize,
    /// The component's clique tree.
    clique_tree: CliqueTree,
    /// Separator per directed clique-tree edge, plus the empty whole-tree separator.
    separators: Vec<IndexSet>,
    /// Flower per directed clique-tree edge, plus the whole-tree flower.
    flowers: Vec<IndexSet>,
    /// Per-flower weighted clique selector.
    choices: Vec<WeightedChoice<T>>,
    /// Per-clique forbidden `(u, v, separator_size)` triples (descending size).
    forbidden_sets: Vec<Vec<(usize, usize, usize)>>,
}

impl<T: RealField + FromPrimitive> ComponentSampler<T> {
    /// Builds the sampler for one connected, chordal component `g`.
    fn init(g: &Graph) -> Self {
        let clique_tree = CliqueTree::from(g);
        let mut separators = clique_tree.separators();
        separators.push(IndexSet::from_sorted(Vec::new()));

        let mut flowers = clique_tree.flowers(&separators);
        flowers.push(IndexSet::from_sorted((0..clique_tree.tree.n).collect()));

        let num_subproblems = flowers.len();
        let mut choices = vec![WeightedChoice::empty(); num_subproblems];
        let forbidden_sets = clique_tree.forbidden_sets(&separators, &flowers);

        let mut visited = LazyTokens::new(clique_tree.tree.n);
        let mut considered = LazyTokens::new(clique_tree.tree.n);
        let mut memoization = Memoization::<T>::new(clique_tree.tree.n, g.n);

        // Populate `choices` (and the memoized counts) bottom-up from the
        // whole-tree subproblem.
        Self::rec_count_init(
            &mut choices,
            num_subproblems - 1,
            &mut visited,
            &mut considered,
            &mut memoization,
            &clique_tree,
            &separators,
            &flowers,
            &forbidden_sets,
        );

        ComponentSampler {
            n: g.n,
            clique_tree,
            separators,
            flowers,
            choices,
            forbidden_sets,
        }
    }

    /// Computes (and memoizes) the AMO count for one flower `subproblem`, and as a
    /// side effect builds the [`WeightedChoice`] over that flower's candidate
    /// cliques. Mirrors the reference `Sampler::rec_count_init`.
    #[allow(clippy::too_many_arguments)]
    fn rec_count_init(
        choices: &mut [WeightedChoice<T>],
        subproblem: usize,
        visited: &mut LazyTokens,
        considered: &mut LazyTokens,
        memoization: &mut Memoization<T>,
        clique_tree: &CliqueTree,
        separators: &[IndexSet],
        flowers: &[IndexSet],
        forbidden_sets: &[Vec<(usize, usize, usize)>],
    ) -> T {
        if let Some(res) = memoization.count[subproblem] {
            return res;
        }
        let flower = &flowers[subproblem];
        let separator = &separators[subproblem];

        let mut sum = T::zero();
        let mut amos_per_clique: Vec<T> = Vec::with_capacity(flower.len());
        let mut pre: Vec<Option<T>> = vec![None; 2 * (clique_tree.tree.n - 1)];
        let flower_cliques: Vec<usize> = flower.iter().copied().collect();

        for clique_id in flower_cliques {
            let mut forbidden_sizes = Vec::new();
            forbidden_sizes.push(clique_tree.cliques[clique_id].len() - separator.len());
            for &(u, v, size) in &forbidden_sets[clique_id] {
                if !flower.contains(u) || !flower.contains(v) {
                    break;
                }
                if size > separator.len() {
                    forbidden_sizes.push(size - separator.len());
                } else {
                    break;
                }
            }
            let phi = combinatorics::rho(&forbidden_sizes, memoization);

            visited.prepare();
            considered.prepare();
            visited.set(clique_id);
            considered.set(clique_id);

            let product = phi
                * Self::rec_count_traversal(
                    choices,
                    flower,
                    clique_id,
                    &mut pre,
                    visited,
                    considered,
                    memoization,
                    clique_tree,
                    separators,
                    flowers,
                    forbidden_sets,
                );

            visited.restore();
            considered.restore();

            sum += product;
            amos_per_clique.push(product);
        }

        memoization.count[subproblem] = Some(sum);
        choices[subproblem] = WeightedChoice::new(&amos_per_clique);
        sum
    }

    /// Accumulates the product of sub-tree counts while traversing the flower
    /// rooted at clique `i`, lazily computing nested flower counts via
    /// [`rec_count_init`](ComponentSampler::rec_count_init). Mirrors the reference
    /// `Sampler::rec_count_traversal`.
    #[allow(clippy::too_many_arguments)]
    fn rec_count_traversal(
        choices: &mut [WeightedChoice<T>],
        flower: &IndexSet,
        i: usize,
        pre: &mut Vec<Option<T>>,
        visited: &mut LazyTokens,
        considered: &mut LazyTokens,
        memoization: &mut Memoization<T>,
        clique_tree: &CliqueTree,
        separators: &[IndexSet],
        flowers: &[IndexSet],
        forbidden_sets: &[Vec<(usize, usize, usize)>],
    ) -> T {
        visited.set(i);
        let mut product = T::one();
        let neighbors: Vec<usize> = clique_tree.tree.neighbors(i).copied().collect();
        for j in neighbors {
            if !flower.contains(j) {
                continue;
            }
            let edge_id = clique_tree.get_edge_id(i, j);
            if !visited.check(j) && !considered.check(j) {
                if let Some(pre_val) = pre[edge_id] {
                    visited.set(j);
                    product *= pre_val;
                } else {
                    let next_flower_result = Self::rec_count_init(
                        choices,
                        edge_id,
                        visited,
                        considered,
                        memoization,
                        clique_tree,
                        separators,
                        flowers,
                        forbidden_sets,
                    );
                    for &new_clique_id in &flowers[edge_id] {
                        considered.set(new_clique_id);
                    }
                    let remaining_subtree_result = Self::rec_count_traversal(
                        choices,
                        flower,
                        j,
                        pre,
                        visited,
                        considered,
                        memoization,
                        clique_tree,
                        separators,
                        flowers,
                        forbidden_sets,
                    );
                    let combined = next_flower_result * remaining_subtree_result;
                    pre[edge_id] = Some(combined);
                    product *= combined;
                }
            } else if !visited.check(j) {
                product *= Self::rec_count_traversal(
                    choices,
                    flower,
                    j,
                    pre,
                    visited,
                    considered,
                    memoization,
                    clique_tree,
                    separators,
                    flowers,
                    forbidden_sets,
                );
            }
        }
        product
    }

    /// Draws a uniform topological vertex ordering of the component, using `rng`
    /// for the per-flower clique choice and for the in-clique permutations.
    fn sample_ordering<R: Rng>(&self, rng: &mut R) -> Vec<usize> {
        let mut pos = vec![0usize; self.n];
        let mut visited = LazyTokens::new(self.clique_tree.tree.n);
        let mut considered = LazyTokens::new(self.clique_tree.tree.n);
        self.rec_sample_ordering(
            self.flowers.len() - 1,
            &mut pos,
            &mut visited,
            &mut considered,
            rng,
        )
    }

    /// Recursively assembles the ordering for flower `subproblem`. Mirrors the
    /// reference `Sampler::rec_sample_ordering`.
    fn rec_sample_ordering<R: Rng>(
        &self,
        subproblem: usize,
        pos: &mut [usize],
        visited: &mut LazyTokens,
        considered: &mut LazyTokens,
        rng: &mut R,
    ) -> Vec<usize> {
        let mut ordering = Vec::new();

        let clique_local_idx = self.choices[subproblem].sample(rng);
        let clique_id = self.flowers[subproblem].get(clique_local_idx);
        let clique = &self.clique_tree.cliques[clique_id];
        let flower = &self.flowers[subproblem];
        let separator = &self.separators[subproblem];

        let remaining_clique_vertices = clique.set_difference(separator);
        let mut forbidden_prefixes = Vec::new();
        for &(u, v, size) in &self.forbidden_sets[clique_id] {
            if !flower.contains(u) || !flower.contains(v) {
                break;
            }
            if size > separator.len() {
                forbidden_prefixes.push(
                    self.separators[self.clique_tree.get_edge_id(u, v)].set_difference(separator),
                );
            } else {
                break;
            }
        }
        let mut clique_ordering =
            Self::draw_allowed_permutation(&remaining_clique_vertices, pos, &forbidden_prefixes, rng);
        ordering.append(&mut clique_ordering);

        let flower = &self.flowers[subproblem];
        let mut queue = VecDeque::new();
        queue.push_back(clique_id);
        visited.prepare();
        considered.prepare();
        visited.set(clique_id);
        considered.set(clique_id);
        while let Some(u) = queue.pop_front() {
            let neighbors: Vec<usize> = self.clique_tree.tree.neighbors(u).copied().collect();
            for v in neighbors {
                if !flower.contains(v) {
                    continue;
                }
                if !visited.check(v) {
                    queue.push_back(v);
                    visited.set(v);
                }
                if !considered.check(v) {
                    let new_flower_id = self.clique_tree.get_edge_id(u, v);
                    let mut sub = self.rec_sample_ordering(
                        new_flower_id,
                        pos,
                        visited,
                        considered,
                        rng,
                    );
                    ordering.append(&mut sub);
                    for &new_clique_id in &self.flowers[new_flower_id] {
                        considered.set(new_clique_id);
                    }
                }
            }
        }
        visited.restore();
        considered.restore();
        ordering
    }

    /// Draws a uniform permutation of `clique`'s remaining vertices that respects
    /// the forbidden separator prefixes (no forbidden prefix may be an exact
    /// initial segment of the permutation). Mirrors the reference
    /// `Sampler::draw_allowed_permutation`, using a generic `Rng` shuffle.
    fn draw_allowed_permutation<R: Rng>(
        clique: &IndexSet,
        helper: &mut [usize],
        forbidden_prefixes: &[IndexSet],
        rng: &mut R,
    ) -> Vec<usize> {
        // For each vertex, the smallest forbidden-prefix length it participates
        // in (default: the full clique length). Forbidden prefixes are ordered
        // largest-to-smallest, so later (smaller) ones overwrite correctly.
        for &u in clique {
            helper[u] = clique.len();
        }
        for forbidden_prefix in forbidden_prefixes {
            for &u in forbidden_prefix {
                helper[u] = forbidden_prefix.len() - 1;
            }
        }

        loop {
            let mut perm = clique.to_vec();
            shuffle(&mut perm, rng);
            if Self::is_allowed(&perm, helper) {
                return perm;
            }
        }
    }

    /// Returns `true` if no forbidden prefix is an exact initial segment of
    /// `perm`. Mirrors the reference `Sampler::is_allowed`.
    fn is_allowed(perm: &[usize], helper: &[usize]) -> bool {
        let mut mx = 0;
        for (i, &u) in perm.iter().enumerate() {
            mx = mx.max(helper[u]);
            if mx == i {
                return false;
            }
            if mx >= perm.len() {
                return true;
            }
        }
        true
    }
}

/// In-place Fisher-Yates shuffle using a generic [`Rng`] (the reference uses
/// `SliceRandom::shuffle`; this is the dependency-free equivalent).
fn shuffle<R: Rng>(slice: &mut [usize], rng: &mut R) {
    let len = slice.len();
    if len <= 1 {
        return;
    }
    for i in (1..len).rev() {
        let j: usize = rng.random_range(0..(i + 1));
        slice.swap(i, j);
    }
}

// --- public API (mirrors brcd_mec) ------------------------------------------

/// Draws a DAG uniformly at random from the Markov equivalence class of `graph`,
/// using the polynomial-time Clique-Picking sampler. The compelled arcs are kept
/// and, for each chain component (a connected component of the undirected
/// subgraph), one **uniform AMO** is sampled by clique-picking (using the AMO
/// sub-counts as weights) and its undirected edges are oriented accordingly. The
/// result is a fully-directed `MixedGraph` over the same vertices.
///
/// This is the poly-time drop-in for
/// [`brcd_mec::mec_sample_dag`](crate::brcd::brcd_mec::mec_sample_dag): same
/// semantics (a uniform class member), no class-size bound, no enumeration.
///
/// The count/weight type `T` is the same generic the counter uses (`f64` or
/// `Float106`); it controls only the precision of the internal AMO weights, not
/// the returned graph.
///
/// # Errors
/// * [`BrcdErrorEnum::NotACpdag`] if any edge is bidirected or partially oriented.
/// * [`BrcdErrorEnum::NotAcyclic`] if the arc projection contains a cycle.
///
/// # Preconditions
/// Each chain component is assumed chordal (a valid CPDAG); this is not checked.
pub fn sample_dag<T, N, R>(graph: &MixedGraph<N>, rng: &mut R) -> Result<MixedGraph<N>, BrcdError>
where
    T: RealField + FromPrimitive,
    N: Clone,
    R: Rng,
{
    validate_cpdag(graph)?;
    let mut dag = graph.clone();

    for component in chain_components(graph) {
        // Build the internal undirected graph over the component's vertices,
        // renumbered to a dense `0..k` range; `local_to_global` maps back.
        let (g, local_to_global) = component.to_internal_graph();
        let sampler = ComponentSampler::<T>::init(&g);
        let ordering = sampler.sample_ordering(rng);

        // Inverse ordering: position of each local vertex in the sampled order.
        let mut order_pos = vec![0usize; g.n];
        for (i, &local) in ordering.iter().enumerate() {
            order_pos[local] = i;
        }

        // Orient each undirected edge of the component from the earlier to the
        // later vertex in the sampled ordering — this realizes the sampled AMO.
        for &(lu, lv) in &component.edges {
            let (gu, gv) = (local_to_global[lu], local_to_global[lv]);
            let (parent, child) = if order_pos[lu] < order_pos[lv] {
                (gu, gv)
            } else {
                (gv, gu)
            };
            dag.orient(parent, child)
                .map_err(|_| BrcdError(BrcdErrorEnum::NotACpdag))?;
        }
    }

    if dag.has_cycle() {
        return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
    }
    Ok(dag)
}

/// Returns a **deterministic** representative DAG of the Markov equivalence class
/// of `graph`: the compelled arcs plus, for each chain component, the canonical
/// acyclic moral orientation obtained from the component's maximum-cardinality
/// search (MCS) order.
///
/// MCS yields a perfect elimination order in which each vertex's *earlier*
/// neighbours form a clique; orienting every undirected edge from the earlier to
/// the later MCS vertex therefore creates no new unshielded collider (a collider
/// `a → c ← b` would need the two earlier neighbours `a, b` of `c` non-adjacent,
/// which the PEO forbids), so the result is a genuine class member. Note that
/// orienting by **raw vertex index** does *not* work — the identity order is not
/// a perfect elimination order in general (e.g. the path `0 − 2 − 1` would yield
/// the spurious collider `0 → 2 ← 1`). Mirrors
/// [`brcd_mec::representative_dag`](crate::brcd::brcd_mec::representative_dag) as a
/// deterministic single member; the *which* member differs (a fixed choice),
/// which is all `representative_dag` promises.
///
/// # Errors
/// As [`sample_dag`].
///
/// # Preconditions
/// As [`sample_dag`].
pub fn representative_dag<N>(graph: &MixedGraph<N>) -> Result<MixedGraph<N>, BrcdError>
where
    N: Clone,
{
    validate_cpdag(graph)?;
    let mut dag = graph.clone();
    for component in chain_components(graph) {
        let (g, local_to_global) = component.to_internal_graph();
        // MCS order: each vertex's earlier neighbours form a clique, so orienting
        // earlier -> later is a valid AMO (introduces no new unshielded collider).
        let order = mcs(&g);
        let rank = inverse_permutation(&order);
        for &(lu, lv) in &component.edges {
            let (parent, child) = if rank[lu] < rank[lv] {
                (local_to_global[lu], local_to_global[lv])
            } else {
                (local_to_global[lv], local_to_global[lu])
            };
            dag.orient(parent, child)
                .map_err(|_| BrcdError(BrcdErrorEnum::NotACpdag))?;
        }
    }
    if dag.has_cycle() {
        return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
    }
    Ok(dag)
}

// --- internals --------------------------------------------------------------

/// Validates that every edge is a directed arc or an undirected edge and that the
/// arc projection is acyclic (same check as `brcd_mec`).
fn validate_cpdag<N>(graph: &MixedGraph<N>) -> Result<(), BrcdError> {
    for edge in graph.edges().values() {
        match edge.kind() {
            EdgeKind::Directed | EdgeKind::Undirected => {}
            _ => return Err(BrcdError(BrcdErrorEnum::NotACpdag)),
        }
    }
    if graph.has_cycle() {
        return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
    }
    Ok(())
}

/// A chain component: its global vertices and the undirected edges within it,
/// both ready to be translated into the dense internal [`Graph`] the sampler
/// consumes.
struct Component {
    /// The component's vertices, in ascending global-index order. Position `i` is
    /// local vertex `i`.
    vertices: Vec<usize>,
    /// Undirected edges of the component as **local** vertex-index pairs
    /// (indices into `vertices`), each edge listed once.
    edges: Vec<(usize, usize)>,
}

impl Component {
    /// Builds the dense internal undirected [`Graph`] (`0..k`) for this component
    /// and the local→global vertex map.
    fn to_internal_graph(&self) -> (Graph, Vec<usize>) {
        let edge_list: Vec<(usize, usize)> = self.edges.clone();
        let g = Graph::from_edge_list(edge_list, self.vertices.len());
        (g, self.vertices.clone())
    }
}

/// Returns the chain components of `graph`: the connected components of the
/// undirected subgraph, with vertices renumbered to a dense `0..k` local range
/// and the within-component undirected edges in local indices. Vertices incident
/// to no undirected edge form no component (they are fixed by the compelled arcs
/// alone). Mirrors `brcd_mec::chain_components`, but also captures edges and the
/// local renumbering the sampler needs.
fn chain_components<N>(graph: &MixedGraph<N>) -> Vec<Component> {
    // Vertices incident to at least one undirected edge.
    let mut undirected_vertices: BTreeSet<usize> = BTreeSet::new();
    for &(a, b) in graph.undirected_edges().iter() {
        undirected_vertices.insert(a);
        undirected_vertices.insert(b);
    }

    let mut seen: BTreeSet<usize> = BTreeSet::new();
    let mut components = Vec::new();

    for &start in undirected_vertices.iter() {
        if seen.contains(&start) {
            continue;
        }
        // BFS over undirected edges to collect this component's vertices.
        let mut members: BTreeSet<usize> = BTreeSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);
        seen.insert(start);
        members.insert(start);
        while let Some(v) = queue.pop_front() {
            for nb in graph.undirected_neighbors(v) {
                if seen.insert(nb) {
                    members.insert(nb);
                    queue.push_back(nb);
                }
            }
        }

        // Dense local renumbering (ascending global order).
        let vertices: Vec<usize> = members.iter().copied().collect();
        let global_to_local: BTreeMap<usize, usize> =
            vertices.iter().enumerate().map(|(i, &g)| (g, i)).collect();

        // Within-component undirected edges, each once, in local indices.
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for &v in &vertices {
            for nb in graph.undirected_neighbors(v) {
                if v < nb {
                    edges.push((global_to_local[&v], global_to_local[&nb]));
                }
            }
        }

        components.push(Component { vertices, edges });
    }
    components
}
