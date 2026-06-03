/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared helpers for the BRCD verification examples. Each example uses a subset.

#![allow(dead_code)]

use deep_causality_algorithms::brcd::{BrcdConfig, BrcdResult, brcd_run};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::path::{Path, PathBuf};

/// Parses a numeric CSV (an optional non-numeric header row is skipped) into a
/// row-major `(data, n_rows, n_cols)`.
pub fn load_csv(path: &Path) -> std::io::Result<(Vec<f64>, usize, usize)> {
    let text = std::fs::read_to_string(path)?;
    let mut rows: Vec<Vec<f64>> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parsed: Result<Vec<f64>, _> =
            line.split(',').map(|c| c.trim().parse::<f64>()).collect();
        if let Ok(vals) = parsed {
            rows.push(vals);
        }
        // A row that fails to parse (e.g. the header) is skipped.
    }
    let n_rows = rows.len();
    let n_cols = rows.first().map_or(0, Vec::len);
    let data: Vec<f64> = rows.into_iter().flatten().collect();
    Ok((data, n_rows, n_cols))
}

/// Wraps row-major data into a 2-D [`CausalTensor`].
pub fn tensor(data: Vec<f64>, n_rows: usize, n_cols: usize) -> CausalTensor<f64> {
    CausalTensor::new(data, vec![n_rows, n_cols]).expect("valid 2-D tensor shape")
}

/// Builds a CPDAG over `num_vars` variables from undirected and directed edge
/// lists (0-based variable indices).
pub fn cpdag(
    num_vars: usize,
    undirected: &[(usize, usize)],
    arcs: &[(usize, usize)],
) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); num_vars], vec![num_vars]).expect("unit payload");
    let mut g = MixedGraph::new(num_vars, data, 0).expect("valid graph");
    for &(a, b) in undirected {
        g.add_undirected(a, b).expect("undirected edge");
    }
    for &(a, b) in arcs {
        g.add_arc(a, b).expect("directed arc");
    }
    g
}

/// Parses a CPDAG description file: line 1 is `num_vars`; each subsequent line is
/// `U i j` (undirected `i — j`) or `D i j` (directed `i → j`), 0-based.
pub fn load_cpdag(path: &Path) -> std::io::Result<MixedGraph<()>> {
    let text = std::fs::read_to_string(path)?;
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let num_vars: usize = lines
        .next()
        .and_then(|l| l.trim().parse().ok())
        .expect("CPDAG file: first line must be num_vars");
    let mut undirected = Vec::new();
    let mut arcs = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        let i: usize = parts[1].parse().expect("CPDAG edge index");
        let j: usize = parts[2].parse().expect("CPDAG edge index");
        match parts[0] {
            "U" | "u" => undirected.push((i, j)),
            "D" | "d" => arcs.push((i, j)),
            other => panic!("CPDAG file: unknown edge kind '{other}' (use U or D)"),
        }
    }
    Ok(cpdag(num_vars, &undirected, &arcs))
}

/// Parses an expected-ranks file: each non-empty line is a variable index
/// (0-based), best first.
pub fn load_expected(path: &Path) -> std::io::Result<Vec<usize>> {
    let text = std::fs::read_to_string(path)?;
    Ok(text
        .lines()
        .filter_map(|l| l.trim().parse::<usize>().ok())
        .collect())
}

/// A pass/fail reporter that tracks an overall verdict and exits non-zero on
/// failure (so the example is CI-usable).
pub struct Report {
    label: String,
    failed: usize,
    total: usize,
}

impl Report {
    pub fn new(label: &str) -> Self {
        println!("=== BRCD verification: {label} ===");
        Self {
            label: label.to_string(),
            failed: 0,
            total: 0,
        }
    }

    pub fn check(&mut self, name: &str, ok: bool) {
        println!("  [{}] {name}", if ok { "PASS" } else { "FAIL" });
        self.total += 1;
        if !ok {
            self.failed += 1;
        }
    }

    pub fn finish(self) {
        // Zero checks means the example did no real work (missing or empty data).
        // Treat that as a failure so CI cannot go green without running the
        // verification.
        if self.total == 0 {
            println!(
                "=== {} : NO CHECKS RAN (missing or empty dataset) ===",
                self.label
            );
            std::process::exit(1);
        }
        if self.failed == 0 {
            println!("=== {} : ALL PASS ({} checks) ===", self.label, self.total);
        } else {
            println!("=== {} : {} FAILURE(S) ===", self.label, self.failed);
            std::process::exit(1);
        }
    }
}

/// Runs every case under a real-world dataset directory (each case is a subdir
/// with `normal.csv`, `anomalous.csv`, `cpdag.txt`, `expected.txt`) and checks
/// that the Rust top-ranked root cause reproduces the committed Python ranking.
pub fn verify_dataset(report: &mut Report, dataset_dir: &Path, transform_parents: bool, k: usize) {
    let mut cases: Vec<PathBuf> = std::fs::read_dir(dataset_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dataset_dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && p.join("normal.csv").exists())
        .collect();
    cases.sort();
    if cases.is_empty() {
        // Register an explicit failing check so an empty dataset directory fails
        // the run instead of passing with zero checks.
        report.check(
            &format!(
                "dataset {} contains at least one case",
                dataset_dir.display()
            ),
            false,
        );
    }
    for case in &cases {
        verify_case(report, case, transform_parents, k);
    }
}

/// Runs one case and reports whether the Rust top-1 root cause matches the
/// committed Python `expected.txt`.
pub fn verify_case(report: &mut Report, case_dir: &Path, transform_parents: bool, k: usize) {
    let name = case_dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let (nd, nr, nc) = load_csv(&case_dir.join("normal.csv")).expect("read normal.csv");
    let (ad, ar, ac) = load_csv(&case_dir.join("anomalous.csv")).expect("read anomalous.csv");
    assert_eq!(nc, ac, "{name}: normal/anomalous column count differs");
    let cpdag = load_cpdag(&case_dir.join("cpdag.txt")).expect("read cpdag.txt");
    let expected = load_expected(&case_dir.join("expected.txt")).expect("read expected.txt");

    // Replays the reference config: continuous, single root, no node transform
    // (transform_parents is a no-op then, but set for fidelity). Fully-directed
    // CPDAGs make the run deterministic, so the seed is immaterial.
    let mut config = BrcdConfig::continuous(0);
    config.transform_parents = transform_parents;
    config.num_root_causes = k;

    let result = brcd_run(
        &tensor(nd, nr, nc),
        &tensor(ad, ar, ac),
        Some(&cpdag),
        &config,
    )
    .expect("brcd_run on real-world case");
    let got: Vec<usize> = result
        .ranks()
        .iter()
        .filter_map(|c| c.first().copied())
        .collect();

    let n = got.len().min(expected.len());
    let exact = got[..n] == expected[..n];
    let topn = 5.min(n);
    println!(
        "  [{name}] rust top-{topn}: {:?} | python top-{topn}: {:?} | exact full match: {exact}",
        &got[..topn],
        &expected[..topn]
    );
    report.check(
        &format!("{name}: full ranking reproduces python ({n} positions)"),
        exact,
    );
}

// --- supplied vs learned (BOSS) comparison ---------------------------------

/// The top-1 candidate per ranked set, best first.
fn top1_ranking(result: &BrcdResult<f64>) -> Vec<usize> {
    result
        .ranks()
        .iter()
        .filter_map(|c| c.first().copied())
        .collect()
}

/// 1-based position of `v` in `ranking`, or `0` if absent.
fn rank_of(ranking: &[usize], v: usize) -> usize {
    ranking.iter().position(|&x| x == v).map_or(0, |p| p + 1)
}

/// Length of the common leading prefix of two rankings (how deep they agree).
fn agreement_depth(a: &[usize], b: &[usize]) -> usize {
    a.iter().zip(b.iter()).take_while(|(x, y)| x == y).count()
}

/// Spearman rank correlation between two full rankings over the same variables.
fn spearman(a: &[usize], b: &[usize], num_vars: usize) -> f64 {
    let n = num_vars as f64;
    if num_vars < 2 {
        return 1.0;
    }
    let d2: f64 = (0..num_vars)
        .map(|v| {
            let d = rank_of(a, v) as f64 - rank_of(b, v) as f64;
            d * d
        })
        .sum();
    1.0 - 6.0 * d2 / (n * (n * n - 1.0))
}

/// Runs every case under a dataset directory **twice** — once with the supplied
/// service-map CPDAG, once with `cpdag = None` so BOSS learns the structure —
/// and prints a side-by-side comparison. The supplied ranking must place the true
/// injected fault in its top 3 (the reference is correct), and the BOSS-learned
/// ranking must still recover it within the top 5 — degraded but useful, since
/// the supplied and learned CPDAGs are different objects (see the README). The
/// divergence below the leading positions is the point the table illustrates.
pub fn compare_dataset(report: &mut Report, dataset_dir: &Path, transform_parents: bool, k: usize) {
    let mut cases: Vec<PathBuf> = std::fs::read_dir(dataset_dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dataset_dir.display()))
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && p.join("normal.csv").exists())
        .collect();
    cases.sort();
    if cases.is_empty() {
        report.check(
            &format!(
                "dataset {} contains at least one case",
                dataset_dir.display()
            ),
            false,
        );
    }
    println!(
        "  {:<18} | {:<17} | {:<17} | {:>5} | {:>8} | fault rank (sup/boss)",
        "case", "supplied top-5", "BOSS top-5", "agree", "spearman"
    );
    for case in &cases {
        compare_case(report, case, transform_parents, k);
    }
}

/// Compares the supplied-CPDAG and BOSS-learned rankings for one case.
pub fn compare_case(report: &mut Report, case_dir: &Path, transform_parents: bool, k: usize) {
    let name = case_dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    let (nd, nr, nc) = load_csv(&case_dir.join("normal.csv")).expect("read normal.csv");
    let (ad, ar, ac) = load_csv(&case_dir.join("anomalous.csv")).expect("read anomalous.csv");
    assert_eq!(nc, ac, "{name}: normal/anomalous column count differs");
    let cpdag = load_cpdag(&case_dir.join("cpdag.txt")).expect("read cpdag.txt");
    let expected = load_expected(&case_dir.join("expected.txt")).expect("read expected.txt");
    let fault = expected
        .first()
        .copied()
        .expect("expected.txt has a top fault");

    let mut config = BrcdConfig::continuous(0);
    config.transform_parents = transform_parents;
    config.num_root_causes = k;

    let normal = tensor(nd, nr, nc);
    let anomalous = tensor(ad, ar, ac);

    let supplied = top1_ranking(
        &brcd_run(&normal, &anomalous, Some(&cpdag), &config).expect("supplied-CPDAG run"),
    );
    let learned = top1_ranking(
        &brcd_run::<f64, ()>(&normal, &anomalous, None, &config).expect("BOSS-learned run"),
    );

    let depth = agreement_depth(&supplied, &learned);
    let rho = spearman(&supplied, &learned, nc);
    let sup_fault = rank_of(&supplied, fault);
    let boss_fault = rank_of(&learned, fault);
    let top = 5.min(supplied.len()).min(learned.len());
    println!(
        "  {:<18} | {:<17} | {:<17} | {:>5} | {:>8.3} | {}/{}",
        name,
        format!("{:?}", &supplied[..top]),
        format!("{:?}", &learned[..top]),
        depth,
        rho,
        sup_fault,
        boss_fault,
    );

    report.check(
        &format!("{name}: supplied recovers the fault in top-3, BOSS within top-5 (degraded)"),
        sup_fault > 0 && sup_fault <= 3 && boss_fault > 0 && boss_fault <= 5,
    );
}
