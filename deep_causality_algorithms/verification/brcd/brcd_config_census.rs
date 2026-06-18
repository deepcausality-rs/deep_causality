/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # BRCD configuration census
//!
//! Quantifies, with implementation-independent counts, how BRCD's worst-case
//! `Σ_V 2^{du(V)}` configuration enumeration actually behaves — both on the real
//! verification CPDAGs and on controlled synthetic neighborhoods. For each
//! candidate root cause it tracks four shrinking quantities:
//!
//! 1. `2^{du}`              — raw cut-edge orientations the worst case enumerates.
//! 2. `valid`              — orientations surviving Meek completion + validity
//!                            (the driver's actual per-candidate loop count).
//! 3. `distinct I-CPDAGs`  — distinct completed configurations (duplicates collapsed).
//! 4. `distinct decomps`   — distinct representative-DAG family decompositions, i.e.
//!                            the number of distinct per-configuration likelihoods
//!                            (data-independent, since all DAGs in one I-MEC share a
//!                            likelihood and a config's likelihood is its decomposition).
//!
//! and, across a whole run, the family-cache compression:
//!
//! * `raw family evals`    — `Σ_candidate Σ_config (#families)` a no-cache scorer pays.
//! * `unique families`     — distinct `(node, parents)` families actually scored.
//!
//! Everything is computed from the public BRCD structural API
//! (`get_configurations_multi`, `augmented_graph`, `mec_size`, `representative_dag`),
//! so it mirrors exactly what the driver enumerates and scores — no timing, no
//! language-dependent numbers.
//!
//! Run: `cargo run -p deep_causality_algorithms --example brcd_config_census`

mod common;

use common::{cpdag, load_cpdag};
use deep_causality_algorithms::brcd::brcd_augment::{augmented_graph, get_configurations_multi};
use deep_causality_algorithms::brcd::brcd_mec::{mec_size, representative_dag};
use deep_causality_topology::MixedGraph;
use std::collections::BTreeSet;
use std::path::Path;

/// Canonical, data-independent family signature of one DAG: each node paired with
/// its sorted parent set. Two DAGs with the same signature induce the same
/// per-family factors, hence the same total likelihood for any data.
fn decomposition_signature<N>(dag: &MixedGraph<N>) -> Vec<(usize, Vec<usize>)> {
    let mut sig: Vec<(usize, Vec<usize>)> = (0..dag.num_vertices())
        .map(|node| {
            let mut ps = dag.parents(node);
            ps.sort_unstable();
            (node, ps)
        })
        .collect();
    sig.sort_unstable();
    sig
}

/// Per-candidate census counters.
#[derive(Default, Clone)]
struct Cand {
    du: usize,
    enumerated: u128,
    valid: usize,
    distinct_icpdags: usize,
    distinct_decomps: usize,
    mec_capped: bool,
}

/// Runs the census for one candidate root cause against `graph`, folding the
/// candidate's families into the run-wide cache sets.
fn census_candidate(
    graph: &MixedGraph<()>,
    candidate: usize,
    global_families: &mut BTreeSet<(usize, Vec<usize>)>,
    raw_family_evals: &mut u128,
) -> Cand {
    let du = graph.undirected_neighbors(candidate).len();
    let mut c = Cand {
        du,
        enumerated: 1u128 << (du.min(127) as u32),
        ..Default::default()
    };

    let configs = match get_configurations_multi(graph, &[candidate]) {
        Ok(cfgs) => cfgs,
        Err(_) => return c, // ConfigSpaceTooLarge (du > 16): leave counts at 0/enumerated.
    };
    c.valid = configs.len();

    let mut icpdag_sigs: BTreeSet<String> = BTreeSet::new();
    let mut decomp_sigs: BTreeSet<Vec<(usize, Vec<usize>)>> = BTreeSet::new();

    for cfg in &configs {
        icpdag_sigs.insert(format!("{:?}", cfg.edges()));
        let aug = match augmented_graph(cfg, &[candidate]) {
            Ok(a) => a,
            Err(_) => continue,
        };
        if mec_size(&aug).is_err() {
            c.mec_capped = true;
        }
        match representative_dag(&aug) {
            Ok(rep) => {
                let sig = decomposition_signature(&rep);
                *raw_family_evals += rep.num_vertices() as u128;
                for (node, ps) in &sig {
                    global_families.insert((*node, ps.clone()));
                }
                decomp_sigs.insert(sig);
            }
            Err(_) => c.mec_capped = true,
        }
    }
    c.distinct_icpdags = icpdag_sigs.len();
    c.distinct_decomps = decomp_sigs.len();
    c
}

/// Aggregate census over every single-node candidate of `graph`.
struct RunCensus {
    label: String,
    num_vars: usize,
    undirected_edges: usize,
    max_du: usize,
    sum_enumerated: u128,
    sum_valid: u128,
    sum_distinct_icpdags: u128,
    sum_distinct_decomps: u128,
    raw_family_evals: u128,
    unique_families: usize,
    any_mec_cap: bool,
}

fn run_census(label: &str, graph: &MixedGraph<()>) -> RunCensus {
    let n = graph.num_vertices();
    let mut global_families: BTreeSet<(usize, Vec<usize>)> = BTreeSet::new();
    let mut raw_family_evals: u128 = 0;
    let (mut sum_e, mut sum_v, mut sum_i, mut sum_d) = (0u128, 0u128, 0u128, 0u128);
    let mut max_du = 0usize;
    let mut any_cap = false;
    for cand in 0..n {
        let c = census_candidate(graph, cand, &mut global_families, &mut raw_family_evals);
        sum_e += c.enumerated;
        sum_v += c.valid as u128;
        sum_i += c.distinct_icpdags as u128;
        sum_d += c.distinct_decomps as u128;
        max_du = max_du.max(c.du);
        any_cap |= c.mec_capped;
    }
    RunCensus {
        label: label.to_string(),
        num_vars: n,
        undirected_edges: graph.undirected_edges().len(),
        max_du,
        sum_enumerated: sum_e,
        sum_valid: sum_v,
        sum_distinct_icpdags: sum_i,
        sum_distinct_decomps: sum_d,
        raw_family_evals,
        unique_families: global_families.len(),
        any_mec_cap: any_cap,
    }
}

fn print_run_header() {
    println!(
        "  {:<26} | {:>5} | {:>4} | {:>5} | {:>10} | {:>8} | {:>8} | {:>8} | cache(raw→uniq)",
        "graph", "vars", "uEdg", "maxDu", "Σ2^du", "Σvalid", "Σicpdag", "Σdecomp"
    );
}

fn print_run(r: &RunCensus) {
    let compression = if r.unique_families == 0 {
        0.0
    } else {
        r.raw_family_evals as f64 / r.unique_families as f64
    };
    println!(
        "  {:<26} | {:>5} | {:>4} | {:>5} | {:>10} | {:>8} | {:>8} | {:>8} | {:>7}→{:<6} ({:.1}x){}",
        r.label,
        r.num_vars,
        r.undirected_edges,
        r.max_du,
        r.sum_enumerated,
        r.sum_valid,
        r.sum_distinct_icpdags,
        r.sum_distinct_decomps,
        r.raw_family_evals,
        r.unique_families,
        compression,
        if r.any_mec_cap { "  [MEC CAP]" } else { "" },
    );
}

/// A single undirected component (plus the candidate) of a chosen shape.
fn star(du: usize) -> MixedGraph<()> {
    let undirected: Vec<(usize, usize)> = (1..=du).map(|i| (0, i)).collect();
    cpdag(du + 1, &undirected, &[])
}

fn clique(c: usize) -> MixedGraph<()> {
    let mut undirected = Vec::new();
    for i in 0..c {
        for j in (i + 1)..c {
            undirected.push((i, j));
        }
    }
    cpdag(c, &undirected, &[])
}

fn path(k: usize) -> MixedGraph<()> {
    let undirected: Vec<(usize, usize)> = (0..k - 1).map(|i| (i, i + 1)).collect();
    cpdag(k, &undirected, &[])
}

/// Census one synthetic component, reporting the worst-affected candidate (the
/// one with the highest undirected degree).
fn census_shape(label: &str, graph: &MixedGraph<()>, candidate: usize) {
    let mut fams = BTreeSet::new();
    let mut raw = 0u128;
    let c = census_candidate(graph, candidate, &mut fams, &mut raw);
    let collapse = if c.distinct_decomps == 0 {
        0.0
    } else {
        c.enumerated as f64 / c.distinct_decomps as f64
    };
    println!(
        "  {:<14} | du={:>2} | 2^du={:>7} | valid={:>6} | icpdag={:>6} | decomp={:>6} | 2^du/decomp={:>8.1}x{}",
        label,
        c.du,
        c.enumerated,
        c.valid,
        c.distinct_icpdags,
        c.distinct_decomps,
        collapse,
        if c.mec_capped { "  [MEC CAP]" } else { "" },
    );
}

fn main() {
    println!("=================================================================");
    println!(" BRCD configuration census — implementation-independent counts");
    println!("=================================================================\n");

    // --- Part 1: real verification CPDAGs (service-map / BOSS output) ---------
    println!("PART 1 — real verification CPDAGs (single root cause, k=1)\n");
    print_run_header();
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/verification/brcd/data");
    let cases = [
        ("online-boutique/adservice_cpu_1", "OB/adservice_cpu_1"),
        ("online-boutique/adservice_cpu_2", "OB/adservice_cpu_2"),
        ("sock-shop-2/carts_cpu_1", "SS/carts_cpu_1"),
        ("sock-shop-2/carts_cpu_2", "SS/carts_cpu_2"),
    ];
    for (rel, label) in cases {
        let p = base.join(rel).join("cpdag.txt");
        match load_cpdag(&p) {
            Ok(g) => print_run(&run_census(label, &g)),
            Err(e) => println!("  {label}: could not load cpdag.txt: {e}"),
        }
    }
    println!(
        "\n  Reading: on fully-directed CPDAGs du=0, so Σ2^du = #vars (one config each).\n  \
         The cache column is the family reuse across the n F→candidate augmentations.\n"
    );

    // --- Part 2: synthetic undirected neighborhoods (where 2^du fires) --------
    println!("PART 2 — synthetic undirected neighborhoods (worst-affected candidate)\n");
    println!("  STAR K(1,du): candidate = hub. Non-adjacent leaves ⇒ collider rule bites.");
    for du in [2usize, 4, 6, 8, 10, 12, 14] {
        census_shape(&format!("star(du={du})"), &star(du), 0);
    }
    println!();
    println!("  CLIQUE K(c): candidate ∈ clique, du=c-1. Shielded ⇒ collider rule does NOT bite.");
    for c in [3usize, 4, 5, 6, 7, 8] {
        census_shape(&format!("clique(c={c})"), &clique(c), 0);
    }
    println!();
    println!("  PATH P(k): candidate = interior node, du=2 regardless of length.");
    for k in [3usize, 5, 9, 15] {
        census_shape(&format!("path(k={k})"), &path(k), k / 2);
    }

    println!("\n  Reading: 'decomp' is the count of distinct per-configuration likelihoods");
    println!("  (the true cost of the candidate). '2^du/decomp' is the worst-case overcount.");
    println!("=================================================================");
}
