/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # BRCD top-k configuration-pruning experiment
//!
//! Tests the accuracy/compute trade-off behind a sample-efficient BRCD: instead
//! of summing all `2^{du}` valid cut configurations per candidate, score only the
//! top few. Two measurements on the worst case (a clique neighborhood, all
//! `2^{du}` configs valid):
//!
//! 1. **Config mass-capture** `k*`: per candidate, the minimal number of
//!    configurations (ranked by weight `g_i = L_i·Q_i`) whose cumulative mass
//!    reaches 99% / 99.9% of `Σ_i g_i`. `k* ≪ 2^{du}` means a tiny frontier
//!    reproduces the candidate's posterior.
//! 2. **Ranking preservation**: BRCD's output is the *ranking* of candidate root
//!    causes by `p(D|R) ∝ Σ_i g_i`. We compare the full-enumeration ranking to the
//!    **top-1-config** approximation `p̂(D|R) ∝ max_i g_i` — does the argmax (true
//!    root cause) and the full candidate order survive when each candidate is
//!    scored by its single best configuration?
//!
//! Data is linear-Gaussian on a transitive tournament (whose essential graph is
//! the undirected clique); node 0's mechanism is perturbed between regimes, so
//! node 0 is the true root cause. Swept over clique size and anomaly strength.
//!
//! Run: `cargo run --release -p deep_causality_algorithms --example brcd_topk_pruning`

use deep_causality_algorithms::brcd::brcd_augment::augmented_graph;
use deep_causality_algorithms::brcd::brcd_gaussian::{
    GaussianFamilyConfig, gaussian_family_logdensity,
};
use deep_causality_algorithms::brcd::brcd_mec::{mec_size, representative_dag};
use deep_causality_algorithms::brcd::brcd_meek::meek_complete;
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

struct Frame {
    columns: Vec<Vec<f64>>, // len num_vars + 1 (last = F as f64)
    f_bool: Vec<bool>,
    n_total: usize,
    num_vars: usize,
}

fn make_frame(num_vars: usize, n_each: usize, perturb: f64, seed: u64) -> Frame {
    let mut rng = Xoshiro256::from_seed(seed);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();
    let n_total = 2 * n_each;
    let mut cols: Vec<Vec<f64>> = vec![Vec::with_capacity(n_total); num_vars];
    let w = 0.8_f64;
    for regime in 0..2 {
        let intercept0 = if regime == 0 { 0.0 } else { perturb };
        for _ in 0..n_each {
            let mut x = vec![0.0_f64; num_vars];
            for i in 0..num_vars {
                let mut mean = if i == 0 { intercept0 } else { 0.0 };
                for j in 0..i {
                    mean += w * x[j];
                }
                x[i] = mean + eps.sample(&mut rng);
            }
            for i in 0..num_vars {
                cols[i].push(x[i]);
            }
        }
    }
    let mut fcol = vec![0.0_f64; n_each];
    fcol.extend(std::iter::repeat_n(1.0_f64, n_each));
    let mut f_bool = vec![false; n_each];
    f_bool.extend(std::iter::repeat_n(true, n_each));
    cols.push(fcol);
    Frame {
        columns: cols,
        f_bool,
        n_total,
        num_vars,
    }
}

fn total_loglik(dag: &MixedGraph<()>, frame: &Frame, cfg: &GaussianFamilyConfig<f64>) -> f64 {
    let fnode_idx = frame.num_vars;
    let mut total = 0.0_f64;
    for node in 0..dag.num_vertices() {
        let parents = dag.parents(node);
        let has_fnode = parents.contains(&fnode_idx);
        let cont: Vec<usize> = parents.into_iter().filter(|&p| p != fnode_idx).collect();
        let parent_rows: Vec<Vec<f64>> = if cont.is_empty() {
            Vec::new()
        } else {
            (0..frame.n_total)
                .map(|r| cont.iter().map(|&p| frame.columns[p][r]).collect())
                .collect()
        };
        let f = if has_fnode {
            Some(frame.f_bool.as_slice())
        } else {
            None
        };
        let per_row =
            gaussian_family_logdensity(&frame.columns[node], &parent_rows, f, has_fnode, cfg)
                .expect("family logdensity");
        total += per_row.iter().sum::<f64>();
    }
    total
}

fn clique_cpdag(c: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); c], vec![c]).expect("unit payload");
    let mut g = MixedGraph::new(c, data, 0).expect("graph");
    for i in 0..c {
        for j in (i + 1)..c {
            g.add_undirected(i, j).expect("undirected");
        }
    }
    g
}

/// `w[b] = logL_b + ln(Q_b)` over all `2^{c-1}` orientations of `candidate`'s
/// incident edges in the clique.
fn candidate_weights(
    c: usize,
    candidate: usize,
    frame: &Frame,
    cfg: &GaussianFamilyConfig<f64>,
) -> Vec<f64> {
    let others: Vec<usize> = (0..c).filter(|&x| x != candidate).collect();
    let du = others.len();
    let mut w = vec![f64::NEG_INFINITY; 1usize << du];
    for b in 0..(1usize << du) {
        let mut g = clique_cpdag(c);
        for (j, &o) in others.iter().enumerate() {
            if (b >> j) & 1 == 1 {
                g.orient(o, candidate).expect("inward"); // o -> candidate
            } else {
                g.orient(candidate, o).expect("outward"); // candidate -> o
            }
        }
        meek_complete(&mut g);
        let aug = augmented_graph(&g, &[candidate]).expect("augment");
        let q = match mec_size(&aug) {
            Ok(q) => q,
            Err(_) => continue,
        };
        let rep = representative_dag(&aug).expect("representative");
        w[b] = total_loglik(&rep, frame, cfg) + (q as f64).ln();
    }
    w
}

fn logsumexp(w: &[f64]) -> f64 {
    let m = w.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if !m.is_finite() {
        return m;
    }
    m + w.iter().map(|&x| (x - m).exp()).sum::<f64>().ln()
}

/// Minimal #configs (weight-sorted) whose cumulative mass reaches `frac` of total.
fn mass_capture(w: &[f64], frac: f64) -> usize {
    let m = w.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mut g: Vec<f64> = w.iter().map(|&x| (x - m).exp()).collect();
    let total: f64 = g.iter().sum();
    g.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let mut acc = 0.0;
    for (k, &v) in g.iter().enumerate() {
        acc += v;
        if acc >= frac * total {
            return k + 1;
        }
    }
    g.len()
}

/// Descending candidate order under a score vector.
fn rank_order(scores: &[f64]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..scores.len()).collect();
    idx.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap());
    idx
}

fn agreement_depth(a: &[usize], b: &[usize]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

fn main() {
    println!("==================================================================");
    println!(" BRCD top-k configuration pruning (clique worst case)");
    println!("==================================================================\n");
    println!("  Per candidate: full = Σ_i g_i (all 2^du configs), top1 = max_i g_i.");
    println!("  k*99 / k*999 = configs needed for 99% / 99.9% of a candidate's mass.\n");
    let cfg = GaussianFamilyConfig::<f64>::default();

    println!(
        "  {:<7} | {:>3} | {:>6} | {:>7} | {:>6} {:>6} | {:>9} | {:>9} | {:>11}",
        "clique", "du", "2^du", "anomaly", "k*99", "k*999", "full top1", "top1 top1", "rank agree"
    );
    for &perturb in &[0.5_f64, 2.0, 5.0] {
        for c in 4..=7usize {
            let du = c - 1;
            let frame = make_frame(c, 120, perturb, 0xBEEF + c as u64);

            // Per-candidate full and top-1-config scores.
            let mut full = vec![0.0_f64; c];
            let mut top1 = vec![0.0_f64; c];
            let mut max_k99 = 0usize;
            let mut max_k999 = 0usize;
            for cand in 0..c {
                let w = candidate_weights(c, cand, &frame, &cfg);
                full[cand] = logsumexp(&w);
                top1[cand] = w.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                max_k99 = max_k99.max(mass_capture(&w, 0.99));
                max_k999 = max_k999.max(mass_capture(&w, 0.999));
            }
            let full_rank = rank_order(&full);
            let top1_rank = rank_order(&top1);
            let depth = agreement_depth(&full_rank, &top1_rank);
            println!(
                "  {:<7} | {:>3} | {:>6} | {:>7.1} | {:>6} {:>6} | {:>9} | {:>9} | {:>4}/{} ({})",
                format!("K_{c}"),
                du,
                1usize << du,
                perturb,
                max_k99,
                max_k999,
                full_rank[0],
                top1_rank[0],
                depth,
                c,
                if full_rank == top1_rank {
                    "exact"
                } else {
                    "partial"
                },
            );
        }
    }

    println!("\n  Reading:");
    println!("  - k*99 ≪ 2^du ⇒ a tiny config frontier reproduces each candidate's posterior.");
    println!("  - 'full top1' == 'top1 top1' and rank agree = c ⇒ scoring ONE config per");
    println!("    candidate reproduces the full BRCD ranking at 1/2^du the config cost.");
    println!("  - The true root cause is node 0; it should be the top1 in both columns.");
    println!("==================================================================");
}
