/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation of the opt-in `ConfigStrategy::MapPrune` MAP-config finder against
//! the exact `ConfigStrategy::Full` enumeration.
//!
//! Two regimes are checked:
//! * **du = 0 exactness** — on fully-directed CPDAGs the finder returns the one
//!   valid config, so MapPrune must reproduce Full's ranking *and* posteriors to
//!   1e-9 (it is the identical code path with one config per candidate).
//! * **du > 0 ranking fidelity** — on synthetic linear-Gaussian CPDAGs with
//!   undirected structure and a planted, detectable root cause, MapPrune's top-1
//!   must equal Full's top-1; full-order (Kendall-τ / exact) agreement is reported.
//!
//! The unit-level `find_map_configs` budget check (O(du) ≪ 2^du) lives at the end.

use deep_causality_algorithms::brcd::brcd_augment::augmented_graph;
use deep_causality_algorithms::brcd::brcd_boss_cpdag::dag_to_cpdag;
use deep_causality_algorithms::brcd::brcd_config::{BrcdConfig, ConfigStrategy};
use deep_causality_algorithms::brcd::brcd_mapconfig::find_map_configs;
use deep_causality_algorithms::brcd::brcd_mec::mec_size;
use deep_causality_algorithms::brcd::brcd_run;
use deep_causality_rand::{Distribution, Normal, Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

// --- shared data generation (mirrors the verification probe) -----------------

/// A linear-Gaussian frame as two `n × num_vars` row-major tensors (normal,
/// anomalous), the root cause, and the CPDAG it was sampled from.
struct Case {
    normal: CausalTensor<f64>,
    anomalous: CausalTensor<f64>,
    cpdag: MixedGraph<()>,
    rc: usize,
}

/// Random DAG (parents from lower indices), its CPDAG, and a linear-Gaussian
/// dataset with root cause `rc` (the first node with an incident undirected edge)
/// perturbed between regimes. Mirrors `brcd_heuristic_mapconfig::make_case`.
fn make_case(n: usize, p_edge: f64, n_each: usize, perturb: f64, seed: u64) -> Option<Case> {
    let mut rng = Xoshiro256::from_seed(seed);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();

    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut weight: Vec<Vec<f64>> = vec![Vec::new(); n];
    for i in 0..n {
        for j in 0..i {
            if rng.random_range(0.0..1.0) < p_edge {
                parents[i].push(j);
                let sign = if rng.random_range(0.0..1.0) < 0.5 {
                    -1.0
                } else {
                    1.0
                };
                weight[i].push(sign * (0.5 + rng.random_range(0.0..1.0)));
            }
        }
    }
    let cpdag = dag_to_cpdag(&parents).ok()?;
    let rc = (0..n)
        .find(|&v| !cpdag.undirected_neighbors(v).is_empty())
        .unwrap_or(0);

    let mut normal = Vec::with_capacity(n_each * n);
    let mut anomalous = Vec::with_capacity(n_each * n);
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
            let dst = if regime == 0 {
                &mut normal
            } else {
                &mut anomalous
            };
            dst.extend_from_slice(&x);
        }
    }
    Some(Case {
        normal: CausalTensor::new(normal, vec![n_each, n]).unwrap(),
        anomalous: CausalTensor::new(anomalous, vec![n_each, n]).unwrap(),
        cpdag,
        rc,
    })
}

/// Planted clique: a transitive tournament on `c` nodes whose CPDAG is the
/// undirected `c`-clique (`du = c-1` for every node). Node 0 is the root cause.
/// Mirrors `brcd_heuristic_mapconfig::make_clique_case`.
fn make_clique_case(c: usize, n_each: usize, perturb: f64, seed: u64) -> Case {
    let mut rng = Xoshiro256::from_seed(seed);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); c];
    let mut weight: Vec<Vec<f64>> = vec![Vec::new(); c];
    for i in 0..c {
        for j in 0..i {
            parents[i].push(j);
            let s = if rng.random_range(0.0..1.0) < 0.5 {
                -1.0
            } else {
                1.0
            };
            weight[i].push(s * (0.5 + rng.random_range(0.0..1.0)));
        }
    }
    let cpdag = dag_to_cpdag(&parents).expect("clique cpdag");
    let mut normal = Vec::with_capacity(n_each * c);
    let mut anomalous = Vec::with_capacity(n_each * c);
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
            let dst = if regime == 0 {
                &mut normal
            } else {
                &mut anomalous
            };
            dst.extend_from_slice(&x);
        }
    }
    Case {
        normal: CausalTensor::new(normal, vec![n_each, c]).unwrap(),
        anomalous: CausalTensor::new(anomalous, vec![n_each, c]).unwrap(),
        cpdag,
        rc: 0,
    }
}

fn cfg(strategy: ConfigStrategy) -> BrcdConfig<f64> {
    let mut c = BrcdConfig::<f64>::continuous(0);
    c.config_strategy = strategy;
    c
}

/// Kendall-τ-b between two rankings of the same candidate label set. Pairs tied
/// in either ranking position are dropped. Returns 1.0 for a degenerate set.
fn kendall_tau(a: &[Vec<usize>], b: &[Vec<usize>]) -> f64 {
    let pos = |order: &[Vec<usize>], cand: &[usize]| order.iter().position(|c| c == cand).unwrap();
    let labels = a;
    let (mut conc, mut disc) = (0i64, 0i64);
    for i in 0..labels.len() {
        for j in (i + 1)..labels.len() {
            let (ci, cj) = (&labels[i], &labels[j]);
            let da = pos(a, ci) as i64 - pos(a, cj) as i64;
            let db = pos(b, ci) as i64 - pos(b, cj) as i64;
            if da == 0 || db == 0 {
                continue;
            }
            if (da > 0) == (db > 0) {
                conc += 1;
            } else {
                disc += 1;
            }
        }
    }
    let denom = conc + disc;
    if denom == 0 {
        1.0
    } else {
        (conc - disc) as f64 / denom as f64
    }
}

// --- du = 0 exactness --------------------------------------------------------

/// A fully-directed CPDAG (a chain with a sink) and Gaussian data. With no
/// undirected edges, MapPrune visits exactly one config per candidate, so its
/// ranking and posteriors must match Full to 1e-9.
#[test]
fn du0_exactness_directed_vstructure() {
    // X0 -> X2 <- X1 : an unshielded collider at X2. Both edges are compelled, so
    // dag_to_cpdag yields a fully-directed CPDAG (du = 0 for every node).
    let parents = vec![vec![], vec![], vec![0, 1]];
    let cpdag = dag_to_cpdag(&parents).expect("cpdag");
    for v in 0..3 {
        assert!(
            cpdag.undirected_neighbors(v).is_empty(),
            "expected a fully-directed CPDAG (du=0 everywhere)"
        );
    }

    let mut rng = Xoshiro256::from_seed(99);
    let eps = Normal::new(0.0_f64, 1.0).unwrap();
    let n_each = 200usize;
    let weight = [vec![], vec![], vec![1.1_f64, -0.9]];
    let rc = 0usize; // planted root cause (a source of the collider)
    let perturb = 3.0;
    let mut normal = Vec::new();
    let mut anomalous = Vec::new();
    for regime in 0..2 {
        for _ in 0..n_each {
            let mut x = vec![0.0_f64; 3];
            for i in 0..3 {
                let mut mean = if regime == 1 && i == rc { perturb } else { 0.0 };
                for (k, &j) in parents[i].iter().enumerate() {
                    mean += weight[i][k] * x[j];
                }
                x[i] = mean + eps.sample(&mut rng);
            }
            let dst = if regime == 0 {
                &mut normal
            } else {
                &mut anomalous
            };
            dst.extend_from_slice(&x);
        }
    }
    let normal = CausalTensor::new(normal, vec![n_each, 3]).unwrap();
    let anomalous = CausalTensor::new(anomalous, vec![n_each, 3]).unwrap();

    let full = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &cfg(ConfigStrategy::Full),
    )
    .unwrap();
    let prune = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &cfg(ConfigStrategy::MapPrune),
    )
    .unwrap();

    assert_eq!(
        full.ranks(),
        prune.ranks(),
        "du=0: MapPrune ranking must equal Full"
    );
    for (a, b) in full.posterior().iter().zip(prune.posterior().iter()) {
        assert!(
            (a - b).abs() < 1e-9,
            "du=0: posteriors must match to 1e-9 (Full {a}, MapPrune {b})"
        );
    }
    assert_eq!(full.top().unwrap(), &[rc], "Full should rank the rc first");
}

/// du = 0 exactness across many random fully-directed CPDAGs: only graphs whose
/// CPDAG happens to be fully directed are checked, but for those the match must be
/// exact.
#[test]
fn du0_exactness_many_random_directed() {
    let mut checked = 0usize;
    for seed in 0..200u64 {
        let Some(case) = make_case(7, 0.45, 120, 3.0, 0xD00 + seed) else {
            continue;
        };
        let fully_directed = (0..7).all(|v| case.cpdag.undirected_neighbors(v).is_empty());
        if !fully_directed {
            continue;
        }
        let full = brcd_run(
            &case.normal,
            &case.anomalous,
            Some(&case.cpdag),
            &cfg(ConfigStrategy::Full),
        )
        .unwrap();
        let prune = brcd_run(
            &case.normal,
            &case.anomalous,
            Some(&case.cpdag),
            &cfg(ConfigStrategy::MapPrune),
        )
        .unwrap();
        assert_eq!(full.ranks(), prune.ranks(), "seed {seed}: ranks differ");
        for (a, b) in full.posterior().iter().zip(prune.posterior().iter()) {
            assert!((a - b).abs() < 1e-9, "seed {seed}: posterior {a} vs {b}");
        }
        checked += 1;
    }
    assert!(checked > 0, "no fully-directed CPDAG was produced to check");
}

// --- du > 0 ranking fidelity -------------------------------------------------

/// du > 0: on random asymmetric CPDAGs with a detectable planted root cause,
/// MapPrune's top-1 must equal Full's top-1. Full-order agreement is reported.
#[test]
fn du_pos_top1_fidelity_random_cpdags() {
    let n = 8usize;
    let n_graphs = 80usize;
    let perturb = 3.0; // detectable regime (≥ 2.0)

    let mut trials = 0usize;
    let mut top1_agree = 0usize;
    let mut exact_order = 0usize;
    let mut tau_sum = 0.0_f64;
    let mut with_undirected = 0usize;
    let mut full_recovers_rc = 0usize;

    for gi in 0..n_graphs {
        let Some(case) = make_case(n, 0.25, 150, perturb, 0xA11CE + gi as u64) else {
            continue;
        };
        let has_undirected = (0..n).any(|v| !case.cpdag.undirected_neighbors(v).is_empty());
        if has_undirected {
            with_undirected += 1;
        }

        let full = brcd_run(
            &case.normal,
            &case.anomalous,
            Some(&case.cpdag),
            &cfg(ConfigStrategy::Full),
        )
        .unwrap();
        let prune = brcd_run(
            &case.normal,
            &case.anomalous,
            Some(&case.cpdag),
            &cfg(ConfigStrategy::MapPrune),
        )
        .unwrap();

        trials += 1;
        if full.top() == Some([case.rc].as_slice()) {
            full_recovers_rc += 1;
        }
        if full.top() == prune.top() {
            top1_agree += 1;
        } else {
            panic!(
                "du>0 top-1 mismatch (smallest failing case):\n\
                 seed gi={gi}, n={n}, perturb={perturb}\n\
                 Full ranks:    {:?}\n  posteriors: {:?}\n\
                 MapPrune ranks:{:?}\n  posteriors: {:?}",
                full.ranks(),
                full.posterior(),
                prune.ranks(),
                prune.posterior(),
            );
        }
        if full.ranks() == prune.ranks() {
            exact_order += 1;
        }
        tau_sum += kendall_tau(full.ranks(), prune.ranks());
    }

    assert!(trials > 0, "no cases generated");
    assert!(
        with_undirected > 0,
        "test must include CPDAGs with undirected structure (du>0)"
    );
    let t = trials as f64;
    eprintln!(
        "[du>0 random] trials={trials} (with-undirected {with_undirected})  \
         Full→rc={full_recovers_rc}/{trials}  top1-agree={top1_agree}/{trials}  \
         exact-order={exact_order}/{trials}  mean Kendall-τ={:.4}",
        tau_sum / t
    );
    assert_eq!(top1_agree, trials, "MapPrune top-1 must equal Full top-1");
}

/// du > 0 high-stress: planted cliques (du = c-1) at a strong anomaly. The MAP
/// finder's top-1 must still equal Full's top-1 (node 0).
#[test]
fn du_pos_top1_fidelity_cliques() {
    let perturb = 4.0;
    let mut trials = 0usize;
    let mut top1_agree = 0usize;
    let mut exact_order = 0usize;
    let mut tau_sum = 0.0_f64;

    for c in 4..=6usize {
        for gi in 0..15usize {
            let seed = 0x000C_116E + (c as u64) * 7919 + gi as u64;
            let case = make_clique_case(c, 150, perturb, seed);
            // confirm the clique really is undirected (du = c-1)
            assert_eq!(
                case.cpdag.undirected_neighbors(0).len(),
                c - 1,
                "clique node 0 should have du = c-1"
            );
            let full = brcd_run(
                &case.normal,
                &case.anomalous,
                Some(&case.cpdag),
                &cfg(ConfigStrategy::Full),
            )
            .unwrap();
            let prune = brcd_run(
                &case.normal,
                &case.anomalous,
                Some(&case.cpdag),
                &cfg(ConfigStrategy::MapPrune),
            )
            .unwrap();
            trials += 1;
            if full.top() == prune.top() {
                top1_agree += 1;
            } else {
                panic!(
                    "clique top-1 mismatch: c={c} gi={gi}\n\
                     Full: {:?} {:?}\nPrune: {:?} {:?}",
                    full.ranks(),
                    full.posterior(),
                    prune.ranks(),
                    prune.posterior()
                );
            }
            if full.ranks() == prune.ranks() {
                exact_order += 1;
            }
            tau_sum += kendall_tau(full.ranks(), prune.ranks());
        }
    }
    let t = trials as f64;
    eprintln!(
        "[du>0 clique] trials={trials}  top1-agree={top1_agree}/{trials}  \
         exact-order={exact_order}/{trials}  mean Kendall-τ={:.4}",
        tau_sum / t
    );
    assert_eq!(
        top1_agree, trials,
        "clique: MapPrune top-1 must equal Full top-1"
    );
}

// --- finder budget (O(du) ≪ 2^du) -------------------------------------------

/// `find_map_configs` on a `c`-clique candidate evaluates `O(du)` configs, far
/// fewer than the `2^du` the full enumeration would. The weight here just sizes
/// the augmented MEC (structure-only) — enough to exercise the walk and budget.
#[test]
fn finder_budget_is_subexponential() {
    for c in 5..=9usize {
        let mut parents: Vec<Vec<usize>> = vec![Vec::new(); c];
        for (i, row) in parents.iter_mut().enumerate() {
            row.extend(0..i);
        }
        let cpdag = dag_to_cpdag(&parents).expect("clique cpdag");
        let du = c - 1;
        assert_eq!(cpdag.undirected_neighbors(0).len(), du);

        let pruned = find_map_configs::<f64, (), _>(&cpdag, &[0], |g| {
            let aug = augmented_graph(g, &[0])?;
            Ok(mec_size(&aug)? as f64)
        })
        .expect("finder");

        let full_space = 1usize << du;
        // Hill-climb is O(du^2) worst case; assert it is well under the 2^du wall.
        assert!(
            pruned.evals <= du * du + 2 * du + 2,
            "c={c} du={du}: evals {} exceeded the O(du^2) budget",
            pruned.evals
        );
        assert!(
            pruned.evals < full_space,
            "c={c} du={du}: evals {} not below 2^du={full_space}",
            pruned.evals
        );
        assert!(
            !pruned.configs.is_empty(),
            "c={c}: expected ≥1 valid config"
        );
        eprintln!(
            "[budget] c={c} du={du}: evals={} (2^du={full_space}, configs={})",
            pruned.evals,
            pruned.configs.len()
        );
    }
}

/// The MapPrune finder accepts undirected degree ABOVE the full path's
/// `MAX_CONFIG_EDGES` (16) cap: it is `O(du)`, so its only limit is the `usize`
/// orientation-label width, not the `2^{du}` wall the full enumeration hits.
#[test]
fn mapprune_accepts_du_above_full_path_cap() {
    use deep_causality_algorithms::brcd::brcd_augment::get_configurations_multi;
    use deep_causality_algorithms::dag_sampling::mec_size as poly_mec_size;

    // c = 19 clique -> du = 18 > MAX_CONFIG_EDGES (16).
    let c = 19usize;
    let mut parents: Vec<Vec<usize>> = vec![Vec::new(); c];
    for (i, row) in parents.iter_mut().enumerate() {
        row.extend(0..i);
    }
    let cpdag = dag_to_cpdag(&parents).expect("clique cpdag");
    let du = c - 1;
    assert_eq!(cpdag.undirected_neighbors(0).len(), du);

    // The full path refuses: it would enumerate 2^{du} = 2^18 configs.
    assert!(
        get_configurations_multi(&cpdag, &[0]).is_err(),
        "full path should refuse du={du} > MAX_CONFIG_EDGES"
    );

    // The MapPrune finder accepts it, in O(du) evaluations, using the polynomial
    // (uncapped) counter for the structure-only weight.
    let pruned = find_map_configs::<f64, (), _>(&cpdag, &[0], |g| {
        let aug = augmented_graph(g, &[0])?;
        Ok(poly_mec_size::<f64, ()>(&aug))
    })
    .expect("finder must accept du beyond the full-path cap");
    assert!(!pruned.configs.is_empty(), "expected >= 1 valid config");
    assert!(
        pruned.evals <= du * du + 2 * du + 2,
        "du={du}: evals {} exceeded the O(du^2) budget",
        pruned.evals
    );
    eprintln!(
        "[cap-removed] du={du}: full path refuses, MapPrune evals={} (2^du would be {})",
        pruned.evals,
        1u64 << du
    );
}

/// du = 0 unit check: the finder returns exactly one config and evaluates once.
#[test]
fn finder_du0_returns_single_config() {
    // X0 -> X2 <- X1 : a v-structure; every edge compelled, so du = 0 everywhere.
    let parents = vec![vec![], vec![], vec![0, 1]];
    let cpdag = dag_to_cpdag(&parents).expect("cpdag");
    for v in 0..3 {
        assert!(cpdag.undirected_neighbors(v).is_empty());
    }
    let pruned = find_map_configs::<f64, (), _>(&cpdag, &[1], |g| {
        let aug = augmented_graph(g, &[1])?;
        Ok(mec_size(&aug)? as f64)
    })
    .expect("finder");
    assert_eq!(
        pruned.configs.len(),
        1,
        "du=0 must return exactly one config"
    );
    assert_eq!(pruned.evals, 1, "du=0 must evaluate exactly once");
}
