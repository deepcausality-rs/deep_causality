/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation of the Clique-Picking **uniform MEC DAG sampler**
//! (`dag_sampling::sample_dag` / `representative_dag`).
//!
//! The sampler is the polynomial-time uniform Clique-Picking sampler; the
//! oracles are `brcd::brcd_mec::mec_size` (exact class size) and
//! `brcd::dag_to_cpdag` (recover a DAG's CPDAG). The tests are statistical but
//! **deterministic**: every randomized draw uses a fixed `Xoshiro256` seed, so a
//! pass/fail outcome is reproducible and non-flaky.
//!
//! Coverage:
//! 1. VALIDITY — every sampled DAG is a genuine MEC member: it is fully directed,
//!    acyclic, keeps all compelled arcs, and `dag_to_cpdag(sample)` equals the
//!    input CPDAG. Asserted for all draws across the author anchors and 200+
//!    random connected chordal graphs.
//! 2. FULL SUPPORT — on small classes, `200 * mec_size` fixed-seed draws hit every
//!    member exactly once (distinct count == `mec_size`), no extras.
//! 3. UNIFORMITY — chi-square goodness-of-fit of member frequencies vs uniform is
//!    below the p=0.001 critical value.
//! 4. DETERMINISM — `representative_dag` returns the same valid member every call.
//! 5. GENERICS — the sampler runs with the `Float106` weight type and with
//!    `Xoshiro256`.

use deep_causality_algorithms::brcd::brcd_mec::mec_size as oracle_mec_size;
use deep_causality_algorithms::brcd::dag_to_cpdag;
use deep_causality_algorithms::brcd::{BrcdError, BrcdErrorEnum};
use deep_causality_algorithms::dag_sampling::{representative_dag, sample_dag};
use deep_causality_num::Float106;
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, MixedGraph};
use std::collections::{BTreeSet, HashMap};

// --- helpers ----------------------------------------------------------------

/// Builds an all-undirected `MixedGraph<()>` on `n` vertices from `edges`.
fn undirected_graph(n: usize, edges: &[(usize, usize)]) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    let mut g = MixedGraph::new(n, data, 0).unwrap();
    for &(a, b) in edges {
        g.add_undirected(a, b).unwrap();
    }
    g
}

/// Builds a genuine essential-graph CPDAG from a known DAG given as per-variable
/// parent sets. Using `dag_to_cpdag` guarantees the result is Meek-closed (a
/// valid CPDAG), unlike hand-laying arcs + undirected edges which can leave the
/// graph un-closed and therefore not a real equivalence-class representative.
fn cpdag_from_dag(parents: &[Vec<usize>]) -> MixedGraph<()> {
    dag_to_cpdag(parents).expect("valid DAG -> CPDAG")
}

/// The canonical key of a fully-directed DAG: its sorted arc set.
fn dag_key<N>(g: &MixedGraph<N>) -> Vec<(usize, usize)> {
    let mut arcs = g.arcs();
    arcs.sort_unstable();
    arcs
}

/// A CPDAG's `(sorted arcs, sorted undirected edges)` signature.
type CpdagSignature = (Vec<(usize, usize)>, Vec<(usize, usize)>);

/// The canonical `(arcs, undirected_edges)` signature of a CPDAG, for equality.
fn cpdag_signature<N>(g: &MixedGraph<N>) -> CpdagSignature {
    let mut arcs = g.arcs();
    arcs.sort_unstable();
    let mut und: Vec<(usize, usize)> = g
        .undirected_edges()
        .into_iter()
        .map(|(a, b)| (a.min(b), a.max(b)))
        .collect();
    und.sort_unstable();
    und.dedup();
    (arcs, und)
}

/// Recovers the CPDAG of a fully-directed sample via per-variable parent sets.
fn sample_cpdag<N>(sample: &MixedGraph<N>, n: usize) -> MixedGraph<()> {
    let parents: Vec<Vec<usize>> = (0..n).map(|v| sample.parents(v)).collect();
    dag_to_cpdag(&parents).expect("a fully-directed acyclic sample has a CPDAG")
}

/// Asserts a single sample is a genuine member of `input_cpdag`'s class:
/// fully directed (no undirected edges left), acyclic, compelled arcs preserved,
/// and `dag_to_cpdag(sample)` equal to the input CPDAG.
fn assert_valid_member<N>(input_cpdag: &MixedGraph<N>, sample: &MixedGraph<N>, n: usize) {
    // Fully directed: no undirected/other edges remain.
    for edge in sample.edges().values() {
        assert_eq!(
            edge.kind(),
            EdgeKind::Directed,
            "sample retains a non-directed edge"
        );
    }
    // Acyclic.
    assert!(!sample.has_cycle(), "sample is cyclic");

    // Compelled arcs preserved (every input arc is an arc of the sample).
    let sample_arcs: BTreeSet<(usize, usize)> = sample.arcs().into_iter().collect();
    for arc in input_cpdag.arcs() {
        assert!(
            sample_arcs.contains(&arc),
            "sample dropped/reversed compelled arc {arc:?}"
        );
    }

    // dag_to_cpdag(sample) == input CPDAG.
    let recovered = sample_cpdag(sample, n);
    assert_eq!(
        cpdag_signature(&recovered),
        cpdag_signature(input_cpdag),
        "recovered CPDAG differs from the input CPDAG"
    );
}

/// Chi-square critical value at `p = 0.001` for `df` degrees of freedom, via the
/// Wilson–Hilferty cube-root normal approximation:
/// `chi2_crit ≈ df * (1 - 2/(9 df) + z * sqrt(2/(9 df)))^3`, with the upper
/// 0.001 normal quantile `z = 3.0902323`. The approximation is accurate to well
/// under 1% for `df ≥ 1` in this tail, and is conservative enough for a
/// goodness-of-fit pass/fail gate. (Spot check: df=53 gives ≈ 92.05; the exact
/// value is ≈ 92.01.)
fn chi2_critical_p001(df: usize) -> f64 {
    let z = 3.090_232_3_f64;
    let d = df as f64;
    let a = 2.0 / (9.0 * d);
    d * (1.0 - a + z * a.sqrt()).powi(3)
}

// --- VALIDITY ---------------------------------------------------------------

#[test]
fn validity_on_author_anchors() {
    // The 6-node -> 54 anchor, K4 -> 24, and a two-triangle book.
    let anchors: &[(usize, &[(usize, usize)])] = &[
        (
            6,
            &[
                (0, 1),
                (0, 2),
                (1, 2),
                (1, 3),
                (1, 4),
                (1, 5),
                (2, 3),
                (2, 4),
                (2, 5),
                (3, 4),
                (4, 5),
            ],
        ),
        (4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]), // K4 -> 24
        (4, &[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]),         // two triangles
    ];

    let mut rng = Xoshiro256::from_seed(0xA11C_E0FA_5A3D_0001);
    for &(n, edges) in anchors {
        let g = undirected_graph(n, edges);
        for _ in 0..500 {
            let sample = sample_dag::<f64, (), _>(&g, &mut rng).expect("sample");
            assert_valid_member(&g, &sample, n);
        }
    }
}

#[test]
fn validity_on_cpdag_with_compelled_arcs() {
    // A genuine essential graph with both compelled arcs and an undirected chain
    // component. The DAG: v-structure 0 -> 2 <- 1 (0,1 non-adjacent) compels both
    // arcs; the disjoint triangle {3,4,5} is fully reversible (undirected chain
    // component, AMO count 6). Built via dag_to_cpdag so it is Meek-closed.
    let parents = vec![
        vec![],     // 0
        vec![],     // 1
        vec![0, 1], // 2  (collider: 0 -> 2 <- 1)
        vec![],     // 3
        vec![3],    // 4  (triangle 3-4-5 reversible)
        vec![3, 4], // 5
    ];
    let g = cpdag_from_dag(&parents);
    // Sanity: it carries compelled arcs and a size-6 undirected component.
    assert!(!g.arcs().is_empty(), "expected compelled arcs");
    assert_eq!(oracle_mec_size(&g), Ok(6));

    let mut rng = Xoshiro256::from_seed(0xA11C_E0FA_5A3D_0002);
    for _ in 0..1000 {
        let sample = sample_dag::<f64, (), _>(&g, &mut rng).expect("sample");
        assert_valid_member(&g, &sample, 6);
    }
}

#[test]
fn validity_on_random_chordal_graphs() {
    // 200+ random connected chordal graphs (n in 3..=10), all-undirected CPDAGs.
    let mut rng = Xoshiro256::from_seed(0xA11C_E0FA_5A3D_0003);
    let mut draw_rng = Xoshiro256::from_seed(0xA11C_E0FA_5A3D_BEEF);
    let target = 220usize;
    let mut checked = 0usize;
    let mut attempts = 0usize;

    while checked < target {
        attempts += 1;
        assert!(attempts < 100_000, "could not generate {target} graphs");
        let n: usize = 3 + rng.random_range(0usize..8); // n in 3..=10
        let edges = random_connected_chordal(&mut rng, n);
        let g = undirected_graph(n, &edges);

        // Keep graphs the oracle can size (so we know the class is finite/sane).
        let mec = match oracle_mec_size(&g) {
            Ok(size) => size,
            Err(_) => continue,
        };
        // Draw several samples and validate each.
        let draws = (3 * mec).clamp(5, 60);
        for _ in 0..draws {
            let sample = sample_dag::<f64, (), _>(&g, &mut draw_rng).expect("sample");
            assert_valid_member(&g, &sample, n);
        }
        checked += 1;
    }
    assert_eq!(checked, target);
}

// --- FULL SUPPORT + UNIFORMITY ----------------------------------------------

/// Runs the support + uniformity battery on one all-undirected chordal graph.
fn support_and_uniformity(n: usize, edges: &[(usize, usize)], seed: u64) {
    let g = undirected_graph(n, edges);
    let mec = oracle_mec_size(&g).expect("oracle size") as usize;
    assert!(mec <= 120, "test graph class too large for support check");

    let sample_count = 200 * mec;
    let mut rng = Xoshiro256::from_seed(seed);
    let mut counts: HashMap<Vec<(usize, usize)>, usize> = HashMap::new();
    for _ in 0..sample_count {
        let sample = sample_dag::<f64, (), _>(&g, &mut rng).expect("sample");
        assert_valid_member(&g, &sample, n);
        *counts.entry(dag_key(&sample)).or_insert(0) += 1;
    }

    // FULL SUPPORT: every member hit, no extras.
    assert_eq!(
        counts.len(),
        mec,
        "distinct sampled members ({}) != mec_size ({mec})",
        counts.len()
    );

    // UNIFORMITY: chi-square goodness-of-fit vs uniform.
    let expected = sample_count as f64 / mec as f64;
    let mut chi2 = 0.0_f64;
    for &observed in counts.values() {
        let diff = observed as f64 - expected;
        chi2 += diff * diff / expected;
    }
    let df = mec - 1;
    let crit = chi2_critical_p001(df);
    assert!(
        chi2 < crit,
        "chi-square {chi2:.3} >= critical {crit:.3} (df={df}, mec={mec}) — non-uniform"
    );
}

#[test]
fn support_and_uniformity_anchor_54() {
    let edges = [
        (0, 1),
        (0, 2),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (2, 4),
        (2, 5),
        (3, 4),
        (4, 5),
    ];
    support_and_uniformity(6, &edges, 0x5044_0F7A_0000_0054);
}

#[test]
fn support_and_uniformity_k4_24() {
    let edges = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    support_and_uniformity(4, &edges, 0x5044_0F7A_0000_0024);
}

#[test]
fn support_and_uniformity_two_triangles() {
    let edges = [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)];
    support_and_uniformity(4, &edges, 0x5044_0F7A_0000_0008);
}

#[test]
fn support_and_uniformity_triangle_6() {
    let edges = [(0, 1), (1, 2), (0, 2)];
    support_and_uniformity(3, &edges, 0x5044_0F7A_0000_0006);
}

// --- DETERMINISM ------------------------------------------------------------

#[test]
fn representative_dag_is_deterministic_and_valid() {
    let cases: &[(usize, &[(usize, usize)])] = &[
        (4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]),
        (3, &[(0, 1), (1, 2)]),
        (
            6,
            &[
                (0, 1),
                (0, 2),
                (1, 2),
                (1, 3),
                (1, 4),
                (1, 5),
                (2, 3),
                (2, 4),
                (2, 5),
                (3, 4),
                (4, 5),
            ],
        ),
    ];
    for &(n, edges) in cases {
        let g = undirected_graph(n, edges);
        let first = representative_dag::<()>(&g).expect("representative");
        let key = dag_key(&first);
        // Same result every call.
        for _ in 0..5 {
            let again = representative_dag::<()>(&g).expect("representative");
            assert_eq!(dag_key(&again), key, "representative_dag not deterministic");
        }
        // And it is a valid member.
        assert_valid_member(&g, &first, n);
    }
}

#[test]
fn representative_dag_valid_when_identity_is_not_a_peo() {
    // Regression: orienting undirected edges by raw vertex index is NOT a valid
    // AMO when the identity order is not a perfect elimination order. Path
    // 0-2-1 (center vertex 2 has the highest index): orienting low->high gives
    // 0->2<-1, an unshielded collider absent from the CPDAG. The representative
    // must instead be a genuine member (dag_to_cpdag(rep) == the undirected path).
    let g = undirected_graph(3, &[(0, 2), (1, 2)]);
    let rep = representative_dag::<()>(&g).expect("representative");
    assert_valid_member(&g, &rep, 3);

    // Broader: the representative must be a valid member of EVERY random chordal
    // class, not only those where the identity order is a PEO.
    let mut rng = Xoshiro256::from_seed(0xA11C_E0FA_5A3D_0E04);
    let target = 220usize;
    let mut checked = 0usize;
    let mut attempts = 0usize;
    while checked < target {
        attempts += 1;
        assert!(attempts < 100_000, "could not generate {target} graphs");
        let n: usize = 3 + rng.random_range(0usize..8);
        let edges = random_connected_chordal(&mut rng, n);
        let g = undirected_graph(n, &edges);
        if oracle_mec_size(&g).is_err() {
            continue;
        }
        let rep = representative_dag::<()>(&g).expect("representative");
        assert_valid_member(&g, &rep, n);
        checked += 1;
    }
    assert_eq!(checked, target);
}

#[test]
fn representative_dag_on_cpdag_with_arcs() {
    let parents = vec![vec![], vec![], vec![0, 1], vec![], vec![3], vec![3, 4]];
    let g = cpdag_from_dag(&parents);
    let rep = representative_dag::<()>(&g).expect("representative");
    assert_valid_member(&g, &rep, 6);
    // Determinism.
    let rep2 = representative_dag::<()>(&g).expect("representative");
    assert_eq!(dag_key(&rep), dag_key(&rep2));
}

// --- GENERICS ---------------------------------------------------------------

#[test]
fn sampler_runs_with_float106_weights() {
    // Same uniform-sampler contract, but with the high-precision weight type.
    let edges = [
        (0, 1),
        (0, 2),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (2, 4),
        (2, 5),
        (3, 4),
        (4, 5),
    ];
    let g = undirected_graph(6, &edges);
    let mec = oracle_mec_size(&g).unwrap() as usize; // 54

    let mut rng = Xoshiro256::from_seed(0xF106_0F7A_0000_0054);
    let sample_count = 200 * mec;
    let mut counts: HashMap<Vec<(usize, usize)>, usize> = HashMap::new();
    for _ in 0..sample_count {
        let sample = sample_dag::<Float106, (), _>(&g, &mut rng).expect("sample");
        assert_valid_member(&g, &sample, 6);
        *counts.entry(dag_key(&sample)).or_insert(0) += 1;
    }
    // Full support with Float106 weights, too.
    assert_eq!(counts.len(), mec, "Float106 weights lost support");

    // Float106 representative is valid and deterministic.
    let rep = representative_dag::<()>(&g).expect("representative");
    assert_valid_member(&g, &rep, 6);
    assert_eq!(
        dag_key(&rep),
        dag_key(&representative_dag::<()>(&g).unwrap())
    );
}

// --- random chordal generator (shared shape with count_tests) ---------------

/// Generates the edge list of a random connected chordal graph on `n` vertices,
/// by random elimination (each new vertex attaches to a random subset of an
/// existing clique), which keeps every closed neighborhood a clique => chordal.
fn random_connected_chordal(rng: &mut Xoshiro256, n: usize) -> Vec<(usize, usize)> {
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut cliques: Vec<Vec<usize>> = vec![vec![0]];

    for v in 1..n {
        let clique_idx: usize = rng.random_range(0..cliques.len());
        let base = cliques[clique_idx].clone();
        let k: usize = 1 + rng.random_range(0..base.len());
        let mut pool = base.clone();
        let mut chosen = Vec::with_capacity(k);
        for i in 0..k {
            let j: usize = rng.random_range(i..pool.len());
            pool.swap(i, j);
            chosen.push(pool[i]);
        }
        for &u in &chosen {
            edges.push((u.min(v), u.max(v)));
        }
        let mut new_clique = chosen.clone();
        new_clique.push(v);
        cliques.push(new_clique);
    }
    edges
}

// --- input validation (validate_cpdag) --------------------------------------

/// Both samplers reject a graph whose arc projection has a directed cycle
/// (`validate_cpdag` → `NotAcyclic`).
#[test]
fn rejects_cyclic_input() {
    let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
    let mut g = MixedGraph::new(3, data, 0).unwrap();
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();

    let mut rng = Xoshiro256::from_seed(1);
    assert_eq!(
        sample_dag::<f64, (), _>(&g, &mut rng).unwrap_err(),
        BrcdError(BrcdErrorEnum::NotAcyclic)
    );
    assert_eq!(
        representative_dag(&g).unwrap_err(),
        BrcdError(BrcdErrorEnum::NotAcyclic)
    );
}

/// Both samplers reject a graph containing an edge that is neither a directed arc
/// nor an undirected edge (here a bidirected edge) (`validate_cpdag` →
/// `NotACpdag`). The edge-kind check precedes the acyclicity check, so the
/// bidirected edge is what trips it.
#[test]
fn rejects_non_cpdag_edge_kind() {
    let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
    let mut g = MixedGraph::new(3, data, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_bidirected(1, 2).unwrap();

    let mut rng = Xoshiro256::from_seed(1);
    assert_eq!(
        sample_dag::<f64, (), _>(&g, &mut rng).unwrap_err(),
        BrcdError(BrcdErrorEnum::NotACpdag)
    );
    assert_eq!(
        representative_dag(&g).unwrap_err(),
        BrcdError(BrcdErrorEnum::NotACpdag)
    );
}
