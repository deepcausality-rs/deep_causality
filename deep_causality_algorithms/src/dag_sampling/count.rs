/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Clique-Picking counting recursion.
//!
//! Ported from the authoritative `cliquepicking_rs::count`, made generic over the
//! count type `T`. This is the polynomial-time counter of acyclic moral
//! orientations (AMOs) of a chordal graph from Wienöbst, Bannach & Liśkiewicz,
//! "Polynomial-Time Algorithms for Counting and Sampling Markov Equivalence
//! Classes" (AAAI 2021) — the same routine the upstream `cliquepicking` package
//! exposes, here ported in-tree with no `num-bigint` dependency.
//!
//! ## Generic count type
//!
//! The reference counts in `num_bigint::BigUint`; this port counts in a generic
//! `T: RealField + FromPrimitive` so the very same code instantiates at `f64`
//! (fast) and at `deep_causality_num::Float106` (higher precision for large
//! classes). The `rho` recurrence subtracts, so the count must stay in linear
//! (not log) space — this is preserved exactly.
//!
//! The reference's `BigUint::ZERO`-as-"uncomputed" sentinel is replaced by
//! `Option<T>` (in both [`Memoization`] and the per-flower `pre` table), which is
//! cleaner and avoids confusing a legitimate zero count with "not computed".
//!
//! ## Precondition
//!
//! Inputs are *assumed* to satisfy the structural properties (chordality for
//! [`count_amos`]). They are not checked; an invalid input may yield a wrong
//! result, exactly as documented for the reference and for `brcd_mec`.

use crate::dag_sampling::clique_tree::CliqueTree;
use crate::dag_sampling::combinatorics;
use crate::dag_sampling::graph::Graph;
use crate::dag_sampling::index_set::IndexSet;
use crate::dag_sampling::lazy_tokens::LazyTokens;
use crate::dag_sampling::memoization::Memoization;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Converts a `usize` into the count type `T`.
#[inline]
fn from_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}

/// Recursively accumulates the product of sub-tree counts while traversing the
/// flower rooted at clique `i`, lazily computing and caching each crossed-edge
/// sub-count in `pre`.
#[allow(clippy::too_many_arguments)]
fn count_traversal<T: RealField + FromPrimitive>(
    i: usize,
    visited: &mut LazyTokens,
    considered: &mut LazyTokens,
    pre: &mut Vec<Option<T>>,
    flower: &IndexSet,
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
                let next_flower_result = count(
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
                let remaining_subtree_result = count_traversal(
                    j,
                    visited,
                    considered,
                    pre,
                    flower,
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
            product *= count_traversal(
                j,
                visited,
                considered,
                pre,
                flower,
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

/// Computes (and memoizes) the count for one subproblem — a clique-tree flower
/// identified by `subproblem`. The whole-tree subproblem is the last index.
#[allow(clippy::too_many_arguments)]
fn count<T: RealField + FromPrimitive>(
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
    if flower.len() == 1 {
        // Flower consists of a single clique.
        let res = combinatorics::factorial(
            clique_tree.cliques[flower.first().unwrap()].len() - separator.len(),
            &mut memoization.factorial,
        );
        memoization.count[subproblem] = Some(res);
        return res;
    }

    let mut sum = T::zero();
    let mut pre: Vec<Option<T>> = vec![None; 2 * (clique_tree.tree.n - 1)];
    let flower_cliques: Vec<usize> = flower.iter().copied().collect();
    for clique_id in flower_cliques {
        let mut forbidden_sizes = Vec::new();
        forbidden_sizes.push(clique_tree.cliques[clique_id].len() - separator.len());
        for &(u, v, size) in &forbidden_sets[clique_id] {
            if !flower.contains(u) || !flower.contains(v) {
                break;
            }
            // `forbidden_sets` is sorted by descending `size` (separator length of
            // the crossed edge), so once a size no longer exceeds the current
            // subproblem's separator, no later one will either: stop. This mirrors
            // the sampler (`sample.rs`) exactly. Using an early stop here rather
            // than `assert!(size > separator.len())` avoids a release-mode panic
            // (and a `usize` underflow in the push) on the boundary case.
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

        let product = count_traversal(
            clique_id,
            visited,
            considered,
            &mut pre,
            flower,
            memoization,
            clique_tree,
            separators,
            flowers,
            forbidden_sets,
        );

        visited.restore();
        considered.restore();

        sum += phi * product;
    }
    memoization.count[subproblem] = Some(sum);
    sum
}

/// Returns the number of acyclic moral orientations (AMOs) of one **connected,
/// chordal** undirected graph `g`, as a value of `T`.
///
/// This is the in-tree, polynomial-time Clique-Picking counter. Several dense /
/// sparse special cases (from He, Jia & Yu 2015, Thm 3) are handled in closed
/// form; otherwise the clique-tree recursion runs.
///
/// # Precondition
///
/// `g` is assumed connected and chordal; this is **not** checked. An input that
/// violates the assumption may produce a wrong count or panic, mirroring the
/// reference and `brcd_mec`.
pub fn count_amos<T: RealField + FromPrimitive>(g: &Graph) -> T {
    // Closed-form special cases (He, Jia & Yu 2015, Thm 3).
    if g.m == g.n - 1 {
        return from_usize::<T>(g.n);
    }
    if g.m == g.n {
        return from_usize::<T>(2 * g.n);
    }
    let num_possible_edges = g.n * (g.n - 1) / 2;
    if g.m == num_possible_edges - 2 {
        let mut fac = vec![None; g.n + 1];
        return from_usize::<T>(g.n * (g.n - 1) - 4) * combinatorics::factorial(g.n - 3, &mut fac);
    }
    if g.m == num_possible_edges - 1 {
        let mut fac = vec![None; g.n + 1];
        return from_usize::<T>(2 * g.n - 3) * combinatorics::factorial(g.n - 2, &mut fac);
    }
    if g.m == num_possible_edges {
        let mut fac = vec![None; g.n + 1];
        return combinatorics::factorial(g.n, &mut fac);
    }

    // Compute the clique tree of g.
    let clique_tree = CliqueTree::from(g);

    // Separators (one per directed edge) plus a baked-in empty whole-tree separator.
    let mut separators = clique_tree.separators();
    separators.push(IndexSet::from_sorted(Vec::new()));

    // Flowers, plus the baked-in whole-tree flower (all clique nodes).
    let mut flowers = clique_tree.flowers(&separators);
    flowers.push(IndexSet::from_sorted((0..clique_tree.tree.n).collect()));

    let forbidden_sets = clique_tree.forbidden_sets(&separators, &flowers);

    let mut memoization = Memoization::<T>::new(clique_tree.tree.n, g.n);
    let mut visited = LazyTokens::new(clique_tree.tree.n);
    let mut considered = LazyTokens::new(clique_tree.tree.n);

    count(
        flowers.len() - 1,
        &mut visited,
        &mut considered,
        &mut memoization,
        &clique_tree,
        &separators,
        &flowers,
        &forbidden_sets,
    )
}

/// Returns the number of AMOs of a (possibly disconnected) **chordal** undirected
/// graph `g`, as the product of [`count_amos`] over its connected components.
///
/// # Precondition
///
/// Each connected component is assumed chordal; this is not checked.
pub fn count_chordal<T: RealField + FromPrimitive>(g: &Graph) -> T {
    let mut product = T::one();
    for component in g.connected_components() {
        product *= count_amos::<T>(&component);
    }
    product
}
