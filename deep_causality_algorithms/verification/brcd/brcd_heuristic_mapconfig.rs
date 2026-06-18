/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # BRCD heuristic MAP-configuration finder vs. oracle
//!
//! The load-bearing claim of the pruned-BRCD thesis: a candidate's dominant cut
//! configuration can be **found cheaply, without enumerating `2^{du}`**, and
//! scoring only that one configuration reproduces the full-enumeration candidate
//! ranking. This experiment tests it on realistic asymmetric CPDAGs (a random DAG
//! passed through `dag_to_cpdag`, not the symmetric clique toy), with one
//! perturbed root cause.
//!
//! For every candidate `R` it compares:
//! * **oracle** — enumerate all valid cut configs, score `Σ` (full BRCD) and the
//!   argmax config; the ground-truth ranking.
//! * **H0** — a fixed default orientation (all edges out of `R`, else all in);
//!   `O(1)` config evaluations, no search.
//! * **H1** — one greedy coordinate pass over the cut edges; `O(du)` evaluations.
//! * **H2** — hill-climb to a local optimum; `O(du · iters)` evaluations.
//!
//! Reported (aggregated over many random graphs, two anomaly strengths):
//! oracle recovery of the true root cause; **heuristic top-1 == oracle top-1**
//! (the decision-preserving metric); MAP-hit rate (did the heuristic find the best
//! config); and the evaluation budget vs. `valid` configs vs. `2^{du}`.
//!
//! Run: `cargo run --release -p deep_causality_algorithms --example brcd_heuristic_mapconfig`

use deep_causality_algorithms::brcd::brcd_augment::augmented_graph;
use deep_causality_algorithms::brcd::brcd_boss_cpdag::dag_to_cpdag;
use deep_causality_algorithms::brcd::brcd_gaussian::{
    GaussianFamilyConfig, gaussian_family_logdensity,
};
use deep_causality_algorithms::brcd::brcd_mec::{mec_size, representative_dag};
use deep_causality_algorithms::brcd::brcd_validity::{baseline_parents, is_valid_configuration};
use deep_causality_rand::{Distribution, Normal, Rng, Xoshiro256};
use deep_causality_topology::MixedGraph;
use std::collections::{BTreeMap, BTreeSet};

const DU_CAP: usize = 9; // skip candidates with du > cap from the oracle comparison

struct Frame {
    columns: Vec<Vec<f64>>, // num_vars + 1 (last = F as f64)
    f_bool: Vec<bool>,
    n_total: usize,
    num_vars: usize,
}

/// Random DAG (parents from lower indices), its CPDAG, a linear-Gaussian frame
/// with root cause `rc` perturbed between regimes, and `rc`.
fn make_case(
    n: usize,
    p_edge: f64,
    n_each: usize,
    perturb: f64,
    seed: u64,
) -> Option<(MixedGraph<()>, Frame, usize)> {
    let mut rng = Xoshiro256::from_seed(seed);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();

    // Structure: parents[i] ⊂ 0..i.  Weights in ±[0.5,1.5].
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut weight: Vec<Vec<f64>> = vec![Vec::new(); n];
    for i in 0..n {
        for j in 0..i {
            if rng.random_range(0.0..1.0) < p_edge {
                parents[i].push(j);
                let sign = if rng.random_range(0.0..1.0) < 0.5 { -1.0 } else { 1.0 };
                weight[i].push(sign * (0.5 + rng.random_range(0.0..1.0)));
            }
        }
    }
    let cpdag = dag_to_cpdag(&parents).ok()?;

    // Root cause: a node with at least one incident undirected edge if possible.
    let rc = (0..n)
        .find(|&v| !cpdag.undirected_neighbors(v).is_empty())
        .unwrap_or(0);

    // Data: ancestral sampling; anomalous shifts rc's intercept.
    let n_total = 2 * n_each;
    let mut cols: Vec<Vec<f64>> = vec![Vec::with_capacity(n_total); n];
    for regime in 0..2 {
        for _ in 0..n_each {
            let mut x = vec![0.0_f64; n];
            for i in 0..n {
                let mut mean = if regime == 1 && i == rc { perturb } else { 0.0 };
                for (k, &j) in parents[i].iter().enumerate() {
                    mean += weight[i][k] * x[j];
                }
                x[i] = mean + eps.sample(&mut rng);
            }
            for i in 0..n {
                cols[i].push(x[i]);
            }
        }
    }
    let mut fcol = vec![0.0_f64; n_each];
    fcol.extend(std::iter::repeat_n(1.0_f64, n_each));
    let mut f_bool = vec![false; n_each];
    f_bool.extend(std::iter::repeat_n(true, n_each));
    cols.push(fcol);
    Some((cpdag, Frame { columns: cols, f_bool, n_total, num_vars: n }, rc))
}

/// Planted clique: a transitive tournament on `c` nodes, whose CPDAG is the
/// undirected `c`-clique (`du = c-1` for every node). Node 0 is the root cause.
fn make_clique_case(c: usize, n_each: usize, perturb: f64, seed: u64) -> (MixedGraph<()>, Frame, usize) {
    let mut rng = Xoshiro256::from_seed(seed);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); c];
    let mut weight: Vec<Vec<f64>> = vec![Vec::new(); c];
    for i in 0..c {
        for j in 0..i {
            parents[i].push(j);
            let s = if rng.random_range(0.0..1.0) < 0.5 { -1.0 } else { 1.0 };
            weight[i].push(s * (0.5 + rng.random_range(0.0..1.0)));
        }
    }
    let cpdag = dag_to_cpdag(&parents).expect("clique cpdag");
    let n_total = 2 * n_each;
    let mut cols: Vec<Vec<f64>> = vec![Vec::with_capacity(n_total); c];
    for regime in 0..2 {
        for _ in 0..n_each {
            let mut x = vec![0.0_f64; c];
            for i in 0..c {
                let mut mean = if regime == 1 && i == 0 { perturb } else { 0.0 };
                for (k, &j) in parents[i].iter().enumerate() {
                    mean += weight[i][k] * x[j];
                }
                x[i] = mean + eps.sample(&mut rng);
            }
            for i in 0..c {
                cols[i].push(x[i]);
            }
        }
    }
    let mut fcol = vec![0.0_f64; n_each];
    fcol.extend(std::iter::repeat_n(1.0_f64, n_each));
    let mut f_bool = vec![false; n_each];
    f_bool.extend(std::iter::repeat_n(true, n_each));
    cols.push(fcol);
    (cpdag, Frame { columns: cols, f_bool, n_total, num_vars: c }, 0)
}

fn total_loglik(dag: &MixedGraph<()>, frame: &Frame, cfg: &GaussianFamilyConfig<f64>) -> f64 {
    let fnode_idx = frame.num_vars;
    let mut total = 0.0_f64;
    for node in 0..dag.num_vertices() {
        let parents = dag.parents(node);
        let has_fnode = parents.contains(&fnode_idx);
        let cont: Vec<usize> = parents.into_iter().filter(|&q| q != fnode_idx).collect();
        let rows: Vec<Vec<f64>> = if cont.is_empty() {
            Vec::new()
        } else {
            (0..frame.n_total)
                .map(|r| cont.iter().map(|&q| frame.columns[q][r]).collect())
                .collect()
        };
        let f = if has_fnode { Some(frame.f_bool.as_slice()) } else { None };
        let per = gaussian_family_logdensity(&frame.columns[node], &rows, f, has_fnode, cfg)
            .expect("logdensity");
        total += per.iter().sum::<f64>();
    }
    total
}

/// Evaluates one cut orientation `bits` of candidate `r` (bit j ⇒ neighbor j → r):
/// validity-check (Meek in place), then `w = logL + ln Q`. `None` if invalid.
/// Increments `budget` (one config evaluation).
#[allow(clippy::too_many_arguments)]
fn evaluate(
    cpdag: &MixedGraph<()>,
    r: usize,
    neighbors: &[usize],
    bits: usize,
    baseline: &BTreeMap<usize, BTreeSet<usize>>,
    frame: &Frame,
    cfg: &GaussianFamilyConfig<f64>,
    budget: &mut usize,
) -> Option<f64> {
    *budget += 1;
    let mut g = cpdag.clone();
    for (j, &x) in neighbors.iter().enumerate() {
        if (bits >> j) & 1 == 1 {
            g.orient(x, r).ok()?;
        } else {
            g.orient(r, x).ok()?;
        }
    }
    if !is_valid_configuration(&mut g, &[r], baseline) {
        return None;
    }
    let aug = augmented_graph(&g, &[r]).ok()?;
    let q = mec_size(&aug).ok()?;
    let rep = representative_dag(&aug).ok()?;
    Some(total_loglik(&rep, frame, cfg) + (q as f64).ln())
}

fn logsumexp(v: &[f64]) -> f64 {
    let m = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    if !m.is_finite() {
        return m;
    }
    m + v.iter().map(|&x| (x - m).exp()).sum::<f64>().ln()
}

/// Oracle: enumerate all `2^du` orientations; returns (full logsumexp score, max
/// single-config score, #valid configs).
struct Oracle {
    full: f64,
    maxw: f64,
    valid: usize,
}

fn oracle(
    cpdag: &MixedGraph<()>,
    r: usize,
    neighbors: &[usize],
    baseline: &BTreeMap<usize, BTreeSet<usize>>,
    frame: &Frame,
    cfg: &GaussianFamilyConfig<f64>,
    budget: &mut usize,
) -> Oracle {
    let du = neighbors.len();
    let mut ws = Vec::new();
    for bits in 0..(1usize << du) {
        if let Some(w) = evaluate(cpdag, r, neighbors, bits, baseline, frame, cfg, budget) {
            ws.push(w);
        }
    }
    let maxw = ws.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    Oracle { full: logsumexp(&ws), maxw, valid: ws.len() }
}

/// Finds a valid starting orientation: all-out, then all-in, then first valid by
/// single-bit flips from all-out.
fn valid_start(
    cpdag: &MixedGraph<()>,
    r: usize,
    nb: &[usize],
    base: &BTreeMap<usize, BTreeSet<usize>>,
    frame: &Frame,
    cfg: &GaussianFamilyConfig<f64>,
    budget: &mut usize,
) -> Option<(usize, f64)> {
    let du = nb.len();
    for &bits in &[0usize, (1usize << du) - 1] {
        if let Some(w) = evaluate(cpdag, r, nb, bits, base, frame, cfg, budget) {
            return Some((bits, w));
        }
    }
    for j in 0..du {
        let bits = 1usize << j;
        if let Some(w) = evaluate(cpdag, r, nb, bits, base, frame, cfg, budget) {
            return Some((bits, w));
        }
    }
    None
}

/// Result of a finder run: `max` is the best single-config weight (the top-1/MAP
/// estimate, what the old `greedy` returned); `frontier_lse` is the `logsumexp`
/// over **all distinct valid configs the finder visited** — a lower bound on the
/// full marginal `Σ_b g_b` that captures the visited mass (Step 2: sum a frontier
/// instead of taking the max). Both come from the same `O(du)`/`O(du·iters)` walk.
#[derive(Clone, Copy)]
struct Finder {
    max: f64,
    frontier_lse: f64,
}

/// H1: one greedy coordinate pass. H2: hill-climb to a local optimum (`climb`).
/// Returns the MAP estimate and the visited-frontier sum (see [`Finder`]).
fn greedy(
    cpdag: &MixedGraph<()>,
    r: usize,
    nb: &[usize],
    base: &BTreeMap<usize, BTreeSet<usize>>,
    frame: &Frame,
    cfg: &GaussianFamilyConfig<f64>,
    climb: bool,
    budget: &mut usize,
) -> Option<Finder> {
    let du = nb.len();
    let (mut bits, mut cur) = valid_start(cpdag, r, nb, base, frame, cfg, budget)?;
    // Distinct valid configs seen, keyed by orientation bits (dedup so a config
    // re-evaluated across hill-climb iterations is not double-counted in the sum).
    let mut seen: BTreeMap<usize, f64> = BTreeMap::new();
    seen.insert(bits, cur);
    loop {
        let mut best = None;
        for j in 0..du {
            let cand = bits ^ (1usize << j);
            if let Some(w) = evaluate(cpdag, r, nb, cand, base, frame, cfg, budget) {
                seen.insert(cand, w);
                if w > cur + 1e-12 && best.is_none_or(|(_, bw)| w > bw) {
                    best = Some((cand, w));
                }
            }
        }
        match best {
            Some((cand, w)) => {
                bits = cand;
                cur = w;
                if !climb {
                    break;
                }
            }
            None => break,
        }
    }
    let frontier: Vec<f64> = seen.values().copied().collect();
    Some(Finder { max: cur, frontier_lse: logsumexp(&frontier) })
}

fn h0(
    cpdag: &MixedGraph<()>,
    r: usize,
    nb: &[usize],
    base: &BTreeMap<usize, BTreeSet<usize>>,
    frame: &Frame,
    cfg: &GaussianFamilyConfig<f64>,
    budget: &mut usize,
) -> Option<f64> {
    let du = nb.len();
    for &bits in &[0usize, (1usize << du) - 1] {
        if let Some(w) = evaluate(cpdag, r, nb, bits, base, frame, cfg, budget) {
            return Some(w);
        }
    }
    None
}

fn argmax(v: &[f64]) -> usize {
    let mut bi = 0;
    for i in 1..v.len() {
        if v[i] > v[bi] {
            bi = i;
        }
    }
    bi
}

/// Descending order of the candidate indices in `cand` by their score in `s`.
fn order_of(cand: &[usize], s: &[f64]) -> Vec<usize> {
    let mut idx: Vec<usize> = cand.to_vec();
    idx.sort_by(|&a, &b| s[b].partial_cmp(&s[a]).unwrap_or(std::cmp::Ordering::Equal));
    idx
}

/// Kendall-τ-b between the candidate order under `oracle` and under `approx`,
/// over the comparable candidate set `cand`. Pairs tied in either score are
/// dropped (so trivial du=0 ties do not inflate it). Returns `1.0` when there is
/// no orderable pair (degenerate; counted as agreement).
fn kendall_tau(cand: &[usize], oracle: &[f64], approx: &[f64]) -> f64 {
    let (mut conc, mut disc) = (0i64, 0i64);
    for i in 0..cand.len() {
        for j in (i + 1)..cand.len() {
            let (a, b) = (cand[i], cand[j]);
            let do_ = oracle[a] - oracle[b];
            let da = approx[a] - approx[b];
            if do_ == 0.0 || da == 0.0 {
                continue;
            }
            if (do_ > 0.0) == (da > 0.0) {
                conc += 1;
            } else {
                disc += 1;
            }
        }
    }
    let denom = conc + disc;
    if denom == 0 { 1.0 } else { (conc - disc) as f64 / denom as f64 }
}

#[derive(Default)]
struct Agg {
    trials: usize,
    cand_n: usize,       // total comparable candidates scored
    oracle_rc: usize,    // oracle top-1 == true rc
    h0_top1: usize,      // heuristic top-1 == oracle top-1
    h1_top1: usize,
    h2_top1: usize,
    h0_rc: usize,
    h1_rc: usize,
    h2_rc: usize,
    map_hit_h1: usize,   // over du>=2 candidates
    map_hit_h2: usize,
    du2_cands: usize,
    bud_oracle: u64,
    bud_h0: u64,
    bud_h1: u64,
    bud_h2: u64,
    sum_2du: u64,
    sum_valid: u64,
    // Full-order (ranks 2..n) fidelity vs the oracle Σ ranking: Kendall-τ sums
    // and exact-full-order match counts, for each finder's MAP (max) and the
    // visited-frontier sum.
    tau_h1m: f64,
    tau_h1s: f64,
    tau_h2m: f64,
    tau_h2s: f64,
    exact_h1m: usize,
    exact_h1s: usize,
    exact_h2m: usize,
    exact_h2s: usize,
}

fn main() {
    println!("==================================================================");
    println!(" BRCD heuristic MAP-config finder vs. oracle (asymmetric CPDAGs)");
    println!("==================================================================\n");
    println!("  Random DAG -> dag_to_cpdag; one perturbed root cause; n=10.");
    println!("  Candidates with du>{DU_CAP} are skipped from the comparison.\n");
    let cfg = GaussianFamilyConfig::<f64>::default();
    let n = 10usize;
    let n_graphs = 150usize;

    println!(
        "  {:>7} | {:>9} | {:>10} {:>10} {:>10} | {:>9} {:>9} | {:>16}",
        "anomaly", "oracle→rc", "H0 top1", "H1 top1", "H2 top1", "H1 MAP", "H2 MAP", "evals/cand (2^du|val)"
    );

    for &perturb in &[1.0_f64, 4.0] {
        let mut a = Agg::default();
        for gi in 0..n_graphs {
            let seed = 0x5EED + (perturb as u64) * 1_000_000 + gi as u64;
            let Some((cpdag, frame, rc)) = make_case(n, 0.25, 100, perturb, seed) else {
                continue;
            };
            // Comparable candidate set: du in 0..=DU_CAP.
            let cand: Vec<usize> = (0..n)
                .filter(|&v| cpdag.undirected_neighbors(v).len() <= DU_CAP)
                .collect();
            if !cand.contains(&rc) {
                continue;
            }

            // Per-candidate scores: oracle marginal Σ, plus each finder's MAP
            // (max single config) and frontier-SUM (Σ over visited valid configs).
            let mk = || vec![f64::NEG_INFINITY; n];
            let (mut s_oracle, mut s_h0) = (mk(), mk());
            let (mut s_h1m, mut s_h1s, mut s_h2m, mut s_h2s) = (mk(), mk(), mk(), mk());
            for &r in &cand {
                let nb = cpdag.undirected_neighbors(r);
                let du = nb.len();
                a.cand_n += 1;
                let base = baseline_parents(&cpdag, &[r]);
                let mut bo = 0usize;
                let o = oracle(&cpdag, r, &nb, &base, &frame, &cfg, &mut bo);
                s_oracle[r] = o.full;
                a.bud_oracle += bo as u64;
                a.sum_2du += 1u64 << du;
                a.sum_valid += o.valid as u64;

                let mut b0 = 0;
                s_h0[r] = h0(&cpdag, r, &nb, &base, &frame, &cfg, &mut b0)
                    .unwrap_or(f64::NEG_INFINITY);
                a.bud_h0 += b0 as u64;
                let mut b1 = 0;
                let w1 = greedy(&cpdag, r, &nb, &base, &frame, &cfg, false, &mut b1);
                s_h1m[r] = w1.map_or(f64::NEG_INFINITY, |f| f.max);
                s_h1s[r] = w1.map_or(f64::NEG_INFINITY, |f| f.frontier_lse);
                a.bud_h1 += b1 as u64;
                let mut b2 = 0;
                let w2 = greedy(&cpdag, r, &nb, &base, &frame, &cfg, true, &mut b2);
                s_h2m[r] = w2.map_or(f64::NEG_INFINITY, |f| f.max);
                s_h2s[r] = w2.map_or(f64::NEG_INFINITY, |f| f.frontier_lse);
                a.bud_h2 += b2 as u64;

                if du >= 2 {
                    a.du2_cands += 1;
                    if let Some(f) = w1
                        && (f.max - o.maxw).abs() < 1e-6
                    {
                        a.map_hit_h1 += 1;
                    }
                    if let Some(f) = w2
                        && (f.max - o.maxw).abs() < 1e-6
                    {
                        a.map_hit_h2 += 1;
                    }
                }
            }

            // Top-1 (rank-1) agreement over the comparable candidate set.
            let mask = |s: &[f64]| {
                let mut v = vec![f64::NEG_INFINITY; n];
                for &r in &cand {
                    v[r] = s[r];
                }
                v
            };
            let o_top = argmax(&mask(&s_oracle));
            a.trials += 1;
            if o_top == rc {
                a.oracle_rc += 1;
            }
            if argmax(&mask(&s_h0)) == o_top {
                a.h0_top1 += 1;
            }
            if argmax(&mask(&s_h1m)) == o_top {
                a.h1_top1 += 1;
            }
            if argmax(&mask(&s_h2m)) == o_top {
                a.h2_top1 += 1;
            }
            if argmax(&mask(&s_h0)) == rc {
                a.h0_rc += 1;
            }
            if argmax(&mask(&s_h1m)) == rc {
                a.h1_rc += 1;
            }
            if argmax(&mask(&s_h2m)) == rc {
                a.h2_rc += 1;
            }

            // Full-order (ranks 2..n) fidelity vs the oracle Σ ranking.
            let o_order = order_of(&cand, &s_oracle);
            let acc = |s: &[f64], tau: &mut f64, exact: &mut usize| {
                *tau += kendall_tau(&cand, &s_oracle, s);
                if order_of(&cand, s) == o_order {
                    *exact += 1;
                }
            };
            acc(&s_h1m, &mut a.tau_h1m, &mut a.exact_h1m);
            acc(&s_h1s, &mut a.tau_h1s, &mut a.exact_h1s);
            acc(&s_h2m, &mut a.tau_h2m, &mut a.exact_h2m);
            acc(&s_h2s, &mut a.tau_h2s, &mut a.exact_h2s);
        }

        let t = a.trials.max(1) as f64;
        let d2 = a.du2_cands.max(1) as f64;
        let nc = a.cand_n.max(1) as f64;
        println!(
            "  {:>7.1} | {:>8.0}% | {:>9.0}% {:>9.0}% {:>9.0}% | {:>8.0}% {:>8.0}% | 2^du {:.1} / valid {:.1}",
            perturb,
            100.0 * a.oracle_rc as f64 / t,
            100.0 * a.h0_top1 as f64 / t,
            100.0 * a.h1_top1 as f64 / t,
            100.0 * a.h2_top1 as f64 / t,
            100.0 * a.map_hit_h1 as f64 / d2,
            100.0 * a.map_hit_h2 as f64 / d2,
            a.sum_2du as f64 / nc,
            a.sum_valid as f64 / nc,
        );
        println!(
            "          | mean evals/candidate — oracle {:.1}  H0 {:.1}  H1 {:.1}  H2 {:.1}  | rc-recovery H0 {:.0}% H1 {:.0}% H2 {:.0}%",
            a.bud_oracle as f64 / nc,
            a.bud_h0 as f64 / nc,
            a.bud_h1 as f64 / nc,
            a.bud_h2 as f64 / nc,
            100.0 * a.h0_rc as f64 / t,
            100.0 * a.h1_rc as f64 / t,
            100.0 * a.h2_rc as f64 / t,
        );
        println!(
            "          | full-order vs oracleΣ — Kendall-τ  H1 max {:.3} sum {:.3} | H2 max {:.3} sum {:.3}",
            a.tau_h1m / t, a.tau_h1s / t, a.tau_h2m / t, a.tau_h2s / t,
        );
        println!(
            "          |                        exact-order  H1 max {:.0}% sum {:.0}% | H2 max {:.0}% sum {:.0}%",
            100.0 * a.exact_h1m as f64 / t,
            100.0 * a.exact_h1s as f64 / t,
            100.0 * a.exact_h2m as f64 / t,
            100.0 * a.exact_h2s as f64 / t,
        );
    }

    // --- high-du stress: planted clique (transitive tournament -> clique CPDAG) ---
    // Full-order fidelity (Kendall-τ vs the oracle marginal Σ) for the finder MAP
    // (max) vs the visited-frontier SUM, swept over weak and strong anomaly. The
    // weak row is where top-1/max is known to break (cf. brcd_topk_pruning K_7/0.5);
    // the question is whether the frontier SUM rescues the ranks-2..n order.
    println!("\n  High-du stress (planted clique, du = c-1, root cause = node 0, 60 graphs each):");
    println!(
        "  {:>5} | {:>4} | {:>3} | {:>6} | {:>9} | {:>15} | {:>15} | {:>11}",
        "anom", "c", "du", "2^du", "oracle→rc", "τ H1 max/sum", "τ H2 max/sum", "evals H1/H2"
    );
    for &perturb in &[0.5_f64, 4.0] {
        for c in 4..=9usize {
            let du = c - 1;
            let (mut orc, mut nt, mut cands) = (0usize, 0usize, 0usize);
            let (mut b1, mut b2) = (0u64, 0u64);
            let (mut t1m, mut t1s, mut t2m, mut t2s) = (0.0_f64, 0.0, 0.0, 0.0);
            for gi in 0..60usize {
                let seed = 0xC11_6E + (perturb as u64) * 1_000_000 + (c as u64) * 7919 + gi as u64;
                let (cpdag, frame, rc) = make_clique_case(c, 100, perturb, seed);
                let mk = || vec![f64::NEG_INFINITY; c];
                let (mut so, mut s1m, mut s1s, mut s2m, mut s2s) = (mk(), mk(), mk(), mk(), mk());
                for r in 0..c {
                    let nb = cpdag.undirected_neighbors(r);
                    let base = baseline_parents(&cpdag, &[r]);
                    let mut bo = 0;
                    let o = oracle(&cpdag, r, &nb, &base, &frame, &cfg, &mut bo);
                    so[r] = o.full;
                    let mut e1 = 0;
                    let w1 = greedy(&cpdag, r, &nb, &base, &frame, &cfg, false, &mut e1);
                    s1m[r] = w1.map_or(f64::NEG_INFINITY, |f| f.max);
                    s1s[r] = w1.map_or(f64::NEG_INFINITY, |f| f.frontier_lse);
                    b1 += e1 as u64;
                    let mut e2 = 0;
                    let w2 = greedy(&cpdag, r, &nb, &base, &frame, &cfg, true, &mut e2);
                    s2m[r] = w2.map_or(f64::NEG_INFINITY, |f| f.max);
                    s2s[r] = w2.map_or(f64::NEG_INFINITY, |f| f.frontier_lse);
                    b2 += e2 as u64;
                    cands += 1;
                }
                let all: Vec<usize> = (0..c).collect();
                t1m += kendall_tau(&all, &so, &s1m);
                t1s += kendall_tau(&all, &so, &s1s);
                t2m += kendall_tau(&all, &so, &s2m);
                t2s += kendall_tau(&all, &so, &s2s);
                nt += 1;
                if argmax(&so) == rc {
                    orc += 1;
                }
            }
            let t = nt.max(1) as f64;
            let nc = cands.max(1) as f64;
            println!(
                "  {:>5.1} | {:>4} | {:>3} | {:>6} | {:>8.0}% | {:>6.3}/{:<7.3} | {:>6.3}/{:<7.3} | {:>5.1}/{:<5.1}",
                perturb,
                c,
                du,
                1usize << du,
                100.0 * orc as f64 / t,
                t1m / t,
                t1s / t,
                t2m / t,
                t2s / t,
                b1 as f64 / nc,
                b2 as f64 / nc,
            );
        }
    }

    println!("\n  Reading:");
    println!("  - 'H* top1' = heuristic ranking's top candidate equals the oracle's (rank-1).");
    println!("  - Kendall-τ / exact-order vs oracleΣ = whether ranks 2..n survive, NOT just");
    println!("    rank-1. τ=1.0 ⇒ the finder reproduces the full marginal order.");
    println!("  - max = score each candidate by its single best config; sum = logsumexp over");
    println!("    the configs the finder visited (a lower bound on the marginal Σ_b g_b).");
    println!("  - If τ(sum) > τ(max) at weak anomaly, summing the frontier rescues the tail");
    println!("    that top-1/max distorts — at the same O(du) « 2^du evaluation budget.");
    println!("==================================================================");
}
