/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # BRCD configuration-sum factorization probe
//!
//! Tests whether the per-candidate posterior contribution
//! `p(D|R) ∝ Σ_b  valid(b) · L_b · Q_b`  (b ranges over the `2^{du}` orientations
//! of the candidate's incident undirected edges, `L_b` the I-CPDAG likelihood,
//! `Q_b` its MEC size) **factorizes** over the cut-orientation bits. If the weight
//! tensor `g(b) = L_b · Q_b` is low-rank across every bipartition of the bits, an
//! exact tensor-train / junction-tree contraction computes the sum in
//! `O(du · r²)` time instead of `2^{du}` — the "treewidth, not degree" thesis.
//! If `g` is full-rank, no such decomposition exists and the thesis is dead.
//!
//! The probe runs on the genuine worst case: a **clique** neighborhood, whose
//! essential graph (the undirected complete graph) is exactly the CPDAG of a
//! transitive tournament — so all `2^{du}` configurations are valid and `g` lives
//! on the full Boolean cube, giving a clean rank spectrum. Data is linear-Gaussian
//! with the candidate (node 0) perturbed between regimes, so node 0 is the true
//! root cause and `F → 0` is the real augmentation, exactly as BRCD scores it.
//!
//! Reported per clique size: the **tensor-train bond dimension** (max numerical
//! rank over the sequential bit cuts) and the **balanced-cut rank**, at two
//! tolerances, against the full-rank reference `2^{⌊du/2⌋}`. Bond dimension that
//! stays small as `du` grows ⇒ exact poly-time DP exists. Bond dimension that
//! tracks the full rank ⇒ irreducible.
//!
//! Run: `cargo run --release -p deep_causality_algorithms --example brcd_factorization_probe`

use deep_causality_algorithms::brcd::brcd_augment::augmented_graph;
use deep_causality_algorithms::brcd::brcd_gaussian::{
    GaussianFamilyConfig, gaussian_family_logdensity,
};
use deep_causality_algorithms::brcd::brcd_mec::{mec_size, representative_dag};
use deep_causality_algorithms::brcd::brcd_meek::meek_complete;
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

// --- linear-Gaussian data on a transitive tournament ------------------------

/// Columns of the concatenated normal+anomalous frame: `num_vars` variable
/// columns followed by the `F` indicator column (length `n_total`), plus the
/// boolean `F` mask. Node `i = intercept_i + Σ_{j<i} w·X_j + ε`; the anomalous
/// regime shifts node 0's intercept, making node 0 the true root cause.
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

    // Fixed forward weights j -> i for j < i (transitive tournament => clique CPDAG).
    let w = 0.8_f64;
    for regime in 0..2 {
        let intercept0 = if regime == 0 { 0.0 } else { perturb }; // perturb node 0 when anomalous
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

/// Total log-likelihood of `dag` under the frame: Σ_node Σ_row family log-density,
/// replicating the continuous `ScoreCtx::score` (F-as-parent per-regime at node 0).
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
        let per_row = gaussian_family_logdensity(
            &frame.columns[node],
            &parent_rows,
            f,
            has_fnode,
            cfg,
        )
        .expect("family logdensity");
        total += per_row.iter().sum::<f64>();
    }
    total
}

/// Builds the clique CPDAG over `c` nodes (undirected complete graph).
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

/// For candidate node 0 in the clique, returns `w[b] = logL_b + ln(Q_b)` for every
/// orientation `b` of the `du = c-1` incident edges (bit j set ⇒ neighbor (j+1) → 0).
fn weight_vector(c: usize, frame: &Frame, cfg: &GaussianFamilyConfig<f64>) -> (usize, Vec<f64>) {
    let du = c - 1;
    let mut w = vec![f64::NEG_INFINITY; 1usize << du];
    for b in 0..(1usize << du) {
        let mut g = clique_cpdag(c);
        for j in 0..du {
            let nb = j + 1;
            if (b >> j) & 1 == 1 {
                g.orient(nb, 0).expect("orient inward"); // neighbor -> 0
            } else {
                g.orient(0, nb).expect("orient outward"); // 0 -> neighbor
            }
        }
        meek_complete(&mut g);
        let aug = augmented_graph(&g, &[0]).expect("augment");
        let q = match mec_size(&aug) {
            Ok(q) => q,
            Err(_) => continue, // MEC cap: treat as 0 weight (does not occur for these sizes)
        };
        let rep = representative_dag(&aug).expect("representative");
        let ll = total_loglik(&rep, frame, cfg);
        w[b] = ll + (q as f64).ln();
    }
    (du, w)
}

// --- ANOVA / Walsh-Hadamard interaction-order analysis ----------------------

/// In-place fast Walsh-Hadamard transform (unnormalized). Coefficient at index
/// `S` (a bit subset of the `du` cut variables) is the ANOVA interaction term for
/// that subset; `|S|` is its interaction order.
fn fwht(a: &mut [f64]) {
    let n = a.len();
    let mut h = 1;
    while h < n {
        let mut i = 0;
        while i < n {
            for j in i..(i + h) {
                let x = a[j];
                let y = a[j + h];
                a[j] = x + y;
                a[j + h] = x - y;
            }
            i += 2 * h;
        }
        h *= 2;
    }
}

/// Fraction of `w`'s interaction energy (orders ≥ 1) carried by each order, plus
/// the highest order holding ≥ 1% of that energy.
fn order_energy(w: &[f64], du: usize) -> (Vec<f64>, usize) {
    let mut coeff = w.to_vec();
    fwht(&mut coeff);
    let mut by_order = vec![0.0_f64; du + 1];
    for (s, &c) in coeff.iter().enumerate() {
        by_order[(s as u32).count_ones() as usize] += c * c;
    }
    let interaction: f64 = by_order[1..].iter().sum();
    let frac: Vec<f64> = by_order
        .iter()
        .map(|&e| if interaction > 0.0 { e / interaction } else { 0.0 })
        .collect();
    let max_order = (1..=du).rev().find(|&k| frac[k] >= 0.01).unwrap_or(0);
    (frac, max_order)
}

/// Effective number of configurations carrying the posterior mass:
/// `(Σ g)² / Σ g²` with `g = exp(w − max)`. ≈ 1 ⇒ one config dominates.
fn participation_ratio(w: &[f64]) -> f64 {
    let wmax = w.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let g: Vec<f64> = w
        .iter()
        .map(|&x| if x.is_finite() { (x - wmax).exp() } else { 0.0 })
        .collect();
    let s1: f64 = g.iter().sum();
    let s2: f64 = g.iter().map(|x| x * x).sum();
    if s2 > 0.0 { s1 * s1 / s2 } else { 0.0 }
}

fn main() {
    println!("==================================================================");
    println!(" BRCD configuration-sum factorization probe (clique worst case)");
    println!("==================================================================\n");
    println!("  Tests TWO things on the full 2^du cube of cut orientations:");
    println!("  (1) interaction order of the LOG-weights w(b)=logL_b+lnQ_b");
    println!("      (ANOVA via Walsh-Hadamard). Low max-order ⇒ w has a sparse");
    println!("      interaction graph ⇒ exact treewidth/DP contraction exists.");
    println!("  (2) participation ratio of g=exp(w): how many configs actually");
    println!("      carry posterior mass (≈1 ⇒ argmax suffices, no sum needed).\n");
    let cfg = GaussianFamilyConfig::<f64>::default();

    println!(
        "  {:<7} | {:>3} | {:>5} | {:>6} | {:>7} {:>7} {:>7} {:>7} | {:>8} | {:>5}",
        "clique", "du", "cfgs", "anomaly", "ord1", "ord2", "ord3", "ord≥4", "PR(eff)", "maxO"
    );
    for &perturb in &[0.5_f64, 5.0] {
        for c in 4..=7usize {
            let frame = make_frame(c, 120, perturb, 0xC0FFEE + c as u64);
            let (du, w) = weight_vector(c, &frame, &cfg);
            let (frac, max_order) = order_energy(&w, du);
            let pr = participation_ratio(&w);
            let o3plus: f64 = frac[3.min(du)..].iter().sum();
            println!(
                "  {:<7} | {:>3} | {:>5} | {:>7.1} | {:>6.1}% {:>6.1}% {:>6.1}% {:>6.1}% | {:>8.2} | {:>5}",
                format!("K_{c}"),
                du,
                1usize << du,
                perturb,
                100.0 * frac.get(1).copied().unwrap_or(0.0),
                100.0 * frac.get(2).copied().unwrap_or(0.0),
                100.0 * frac.get(3).copied().unwrap_or(0.0),
                100.0 * o3plus,
                pr,
                max_order,
            );
        }
    }

    println!("\n  Reading:");
    println!("  - max-order (maxO) stays small (≤2) as du grows ⇒ w is low-interaction");
    println!("    ⇒ exact junction-tree DP in exp(treewidth) is possible (THESIS ALIVE).");
    println!("  - maxO grows with du / energy spreads to ord≥4 ⇒ dense coupling (THESIS DEAD).");
    println!("  - PR≈1 ⇒ posterior concentrates on one config ⇒ exact summation is moot,");
    println!("    prune/argmax suffices (points to config-pruning, not exact DP).");
    println!("==================================================================");
}
