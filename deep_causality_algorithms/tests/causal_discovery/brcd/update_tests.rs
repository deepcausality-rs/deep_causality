/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_config::BrcdConfig;
use deep_causality_algorithms::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_algorithms::brcd::brcd_run;
use deep_causality_rand::{Distribution, Normal, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

/// Linear-Gaussian chain X → Y → Z: `X=εx`, `Y=y_intercept+1.5X+εy`, `Z=2Y+εz`.
/// Columns are [X, Y, Z]; `n` rows.
fn chain_data(n: usize, y_intercept: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = dist.sample(&mut rng);
        let y = y_intercept + 1.5 * x + dist.sample(&mut rng);
        let z = 2.0 * y + dist.sample(&mut rng);
        data.push(x);
        data.push(y);
        data.push(z);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// A discrete chain X → Y → Z with integer states in `{0, 1, 2}`. `shift`
/// perturbs Y's mechanism between the two regimes.
fn discrete_chain(n: usize, shift: f64, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let bucket = |v: f64| -> f64 {
        if v < -0.5 {
            0.0
        } else if v < 0.5 {
            1.0
        } else {
            2.0
        }
    };
    let mut data = Vec::with_capacity(n * 3);
    for _ in 0..n {
        let x = bucket(dist.sample(&mut rng));
        let y = bucket(0.8 * x - 0.8 + shift + dist.sample(&mut rng));
        let z = bucket(0.8 * y - 0.8 + dist.sample(&mut rng));
        data.push(x);
        data.push(y);
        data.push(z);
    }
    CausalTensor::new(data, vec![n, 3]).unwrap()
}

/// The undirected chain CPDAG X — Y — Z (arcs = [], edges = {(X,Y),(Y,Z)}).
fn chain_cpdag() -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); 3], vec![3]).unwrap();
    let mut g = MixedGraph::new(3, data, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g
}

#[test]
fn recovers_the_perturbed_mechanism() {
    // The anomaly perturbs p(Y | X) (Y's intercept jumps), so BRCD should rank
    // Y (index 1) as the top root cause.
    let normal = chain_data(120, 0.0, 1);
    let anomalous = chain_data(120, 4.0, 2);
    let cpdag = chain_cpdag();
    let config = BrcdConfig::continuous(7);

    let result = brcd_run(&normal, &anomalous, Some(&cpdag), &config).unwrap();
    assert_eq!(result.top(), Some(&[1][..]), "ranks: {:?}", result.ranks());
}

#[test]
fn produces_a_normalized_ranking_over_all_candidates() {
    let normal = chain_data(80, 0.0, 11);
    let anomalous = chain_data(80, 3.0, 12);
    let cpdag = chain_cpdag();
    let result = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &BrcdConfig::continuous(3),
    )
    .unwrap();

    // k = 1 over 3 variables → 3 single-element candidate sets.
    assert_eq!(result.ranks().len(), 3);
    assert!(result.ranks().iter().all(|c| c.len() == 1));
    // The three variables each appear exactly once.
    let mut seen: Vec<usize> = result.ranks().iter().map(|c| c[0]).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![0, 1, 2]);
    // Posterior is in descending order and finite.
    let post = result.posterior();
    assert!(post.iter().all(|p| p.is_finite()));
    assert!(post.windows(2).all(|w| w[0] >= w[1]));
}

#[test]
fn is_deterministic_under_a_fixed_seed() {
    let normal = chain_data(60, 0.0, 21);
    let anomalous = chain_data(60, 3.0, 22);
    let cpdag = chain_cpdag();
    let a = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &BrcdConfig::continuous(9),
    )
    .unwrap();
    let b = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &BrcdConfig::continuous(9),
    )
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn display_renders_the_ranking() {
    let normal = chain_data(40, 0.0, 31);
    let anomalous = chain_data(40, 3.0, 32);
    let cpdag = chain_cpdag();
    let result = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &BrcdConfig::continuous(1),
    )
    .unwrap();
    let text = format!("{result}");
    assert!(text.contains("BRCD Root-Cause Ranking"));
    assert!(text.contains("posterior="));
}

#[test]
fn misaligned_datasets_are_rejected() {
    let normal = chain_data(20, 0.0, 41);
    // Anomalous with only 2 columns → mismatch.
    let anomalous = CausalTensor::new(vec![0.0_f64; 20 * 2], vec![20, 2]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(
            &normal,
            &anomalous,
            Some(&cpdag),
            &BrcdConfig::continuous(0)
        )
        .err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn too_many_root_causes_is_rejected() {
    let normal = chain_data(20, 0.0, 51);
    let anomalous = chain_data(20, 3.0, 52);
    let cpdag = chain_cpdag();
    let mut config = BrcdConfig::continuous(0);
    config.num_root_causes = 4; // > 3 variables
    assert_eq!(
        brcd_run(&normal, &anomalous, Some(&cpdag), &config).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn zero_root_causes_is_rejected() {
    let normal = chain_data(20, 0.0, 61);
    let anomalous = chain_data(20, 3.0, 62);
    let cpdag = chain_cpdag();
    let mut config = BrcdConfig::continuous(0);
    config.num_root_causes = 0; // k must be >= 1
    assert_eq!(
        brcd_run(&normal, &anomalous, Some(&cpdag), &config).err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn empty_datasets_are_rejected() {
    let normal = CausalTensor::new(Vec::<f64>::new(), vec![0, 3]).unwrap();
    let anomalous = CausalTensor::new(Vec::<f64>::new(), vec![0, 3]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(
            &normal,
            &anomalous,
            Some(&cpdag),
            &BrcdConfig::continuous(0)
        )
        .err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
}

#[test]
fn scores_the_discrete_family() {
    // Exercises the FamilyKind::Discrete branch: state binning, the Dirichlet
    // likelihood, and the full structural + posterior pipeline on discrete data.
    let normal = discrete_chain(120, 0.0, 71);
    let anomalous = discrete_chain(120, 1.5, 72);
    let cpdag = chain_cpdag();

    let result = brcd_run(&normal, &anomalous, Some(&cpdag), &BrcdConfig::discrete(7)).unwrap();

    // k = 1 over 3 variables → 3 single-element candidate sets, each variable once.
    assert_eq!(result.ranks().len(), 3);
    assert!(result.ranks().iter().all(|c| c.len() == 1));
    let mut seen: Vec<usize> = result.ranks().iter().map(|c| c[0]).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![0, 1, 2]);

    // Posterior is finite and ranked descending.
    let post = result.posterior();
    assert!(post.iter().all(|p| p.is_finite()));
    assert!(post.windows(2).all(|w| w[0] >= w[1]));
}

#[test]
fn discrete_family_is_deterministic_under_a_fixed_seed() {
    let normal = discrete_chain(60, 0.0, 81);
    let anomalous = discrete_chain(60, 1.5, 82);
    let cpdag = chain_cpdag();
    let a = brcd_run(&normal, &anomalous, Some(&cpdag), &BrcdConfig::discrete(9)).unwrap();
    let b = brcd_run(&normal, &anomalous, Some(&cpdag), &BrcdConfig::discrete(9)).unwrap();
    assert_eq!(a, b);
}

#[test]
fn absent_cpdag_triggers_boss_and_returns_a_ranking() {
    // `None` makes brcd_run learn the CPDAG from the normal data via BOSS, then
    // rank. The anomaly perturbs p(Y | X), so Y (index 1) should rank first.
    let normal = chain_data(400, 0.0, 91);
    let anomalous = chain_data(400, 4.0, 92);
    let config = BrcdConfig::continuous(7);

    // `None` needs an explicit node type, since there is no graph to infer it.
    let result = brcd_run::<f64, ()>(&normal, &anomalous, None, &config).unwrap();

    assert_eq!(result.ranks().len(), 3, "k=1 over 3 vars");
    let mut seen: Vec<usize> = result.ranks().iter().map(|c| c[0]).collect();
    seen.sort_unstable();
    assert_eq!(seen, vec![0, 1, 2]);
    assert!(result.posterior().iter().all(|p| p.is_finite()));
    assert_eq!(result.top(), Some(&[1][..]), "ranks: {:?}", result.ranks());
}

#[test]
fn supplied_cpdag_is_used_directly_without_structure_learning() {
    // With `Some(cpdag)` the supplied graph is used verbatim — the result is
    // identical to the pre-Option behaviour and independent of any BOSS run.
    let normal = chain_data(120, 0.0, 1);
    let anomalous = chain_data(120, 4.0, 2);
    let cpdag = chain_cpdag();
    let config = BrcdConfig::continuous(7);

    let a = brcd_run(&normal, &anomalous, Some(&cpdag), &config).unwrap();
    let b = brcd_run(&normal, &anomalous, Some(&cpdag), &config).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.top(), Some(&[1][..]));
}

#[test]
fn absent_cpdag_still_rejects_empty_data() {
    // The `None` path validates the data too: no rows → EmptyData (BOSS cannot
    // learn from an empty sample).
    let normal = CausalTensor::new(Vec::<f64>::new(), vec![0, 3]).unwrap();
    let anomalous = CausalTensor::new(Vec::<f64>::new(), vec![0, 3]).unwrap();
    assert_eq!(
        brcd_run::<f64, ()>(&normal, &anomalous, None, &BrcdConfig::continuous(0)).err(),
        Some(BrcdError(BrcdErrorEnum::EmptyData))
    );
}

/// Four-variable linear-Gaussian data, columns [X0, X1, X2, X3]. The mechanism
/// is arbitrary but full-rank so every family scores finitely.
fn data4(n: usize, seed: u64) -> CausalTensor<f64> {
    let mut rng = Xoshiro256::from_seed(seed);
    let dist = Normal::new(0.0_f64, 1.0).unwrap();
    let mut data = Vec::with_capacity(n * 4);
    for _ in 0..n {
        let x0 = dist.sample(&mut rng);
        let x1 = 0.7 * x0 + dist.sample(&mut rng);
        let x2 = 1.3 * x1 + dist.sample(&mut rng);
        let x3 = 0.5 * x2 + dist.sample(&mut rng);
        data.push(x0);
        data.push(x1);
        data.push(x2);
        data.push(x3);
    }
    CausalTensor::new(data, vec![n, 4]).unwrap()
}

/// A 4-vertex CPDAG in which candidate {0} has NO valid cut configuration:
/// arcs 1→2, 2→0, 3→0 and undirected 0—1. (See augment_tests for the proof that
/// both orientations of 0—1 are invalid.) Candidate {0} therefore scores as −∞
/// (the None-plan branch); the other single-element candidates score normally.
fn no_config_cpdag() -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); 4], vec![4]).unwrap();
    let mut g = MixedGraph::new(4, data, 0).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    g.add_arc(3, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    g
}

#[test]
fn candidate_without_a_valid_configuration_scores_neg_inf() {
    // brcd_run must still return a full ranking: candidate {0} has no valid cut
    // configuration, so its plan is None and its log-posterior is −∞ (it sorts
    // last), while the remaining candidates rank normally. This exercises the
    // None-plan branch of the posterior assembly.
    let normal = data4(120, 101);
    let anomalous = data4(120, 102);
    let cpdag = no_config_cpdag();
    let result = brcd_run(
        &normal,
        &anomalous,
        Some(&cpdag),
        &BrcdConfig::continuous(7),
    )
    .unwrap();

    // k = 1 over 4 variables → 4 single-element candidates.
    assert_eq!(result.ranks().len(), 4);
    // Candidate {0} has no valid configuration → its plan is None and its
    // posterior weight collapses to 0.0 (−∞ log-posterior, exp-shifted).
    let idx0 = result
        .ranks()
        .iter()
        .position(|c| c == &vec![0])
        .expect("candidate {0} is present");
    assert_eq!(
        result.posterior()[idx0],
        0.0,
        "candidate {{0}} (no valid config) carries zero posterior; ranks: {:?}, post: {:?}",
        result.ranks(),
        result.posterior()
    );
    // The reported posterior weights are descending.
    let post = result.posterior();
    assert!(post.windows(2).all(|w| w[0] >= w[1]));
}

#[test]
fn multiple_root_causes_enumerate_pairwise_candidates() {
    // num_root_causes = 2 over 4 variables → C(4,2) = 6 candidate pairs. This
    // drives the k ≥ 2 path of the candidate combinations (the inner index-fill
    // loop), and every returned candidate is a sorted 2-element set.
    let normal = data4(120, 111);
    let anomalous = data4(120, 112);
    let cpdag = chain_cpdag4();
    let mut config = BrcdConfig::continuous(7);
    config.num_root_causes = 2;
    let result = brcd_run(&normal, &anomalous, Some(&cpdag), &config).unwrap();

    assert_eq!(result.ranks().len(), 6, "C(4,2) = 6 pairs");
    for cand in result.ranks() {
        assert_eq!(cand.len(), 2);
        assert!(cand[0] < cand[1], "each pair is sorted ascending");
    }
    // All six unordered pairs over {0,1,2,3} appear exactly once.
    let mut seen: Vec<Vec<usize>> = result.ranks().to_vec();
    seen.sort();
    assert_eq!(
        seen,
        vec![
            vec![0, 1],
            vec![0, 2],
            vec![0, 3],
            vec![1, 2],
            vec![1, 3],
            vec![2, 3],
        ]
    );
}

/// The undirected chain CPDAG X0 — X1 — X2 — X3 over four vertices.
fn chain_cpdag4() -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); 4], vec![4]).unwrap();
    let mut g = MixedGraph::new(4, data, 0).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(2, 3).unwrap();
    g
}

#[test]
fn non_2d_normal_tensor_is_rejected() {
    // A 1-D (non-matrix) normal tensor fails the shape_2d guard → DimensionMismatch.
    let normal = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap();
    let anomalous = chain_data(20, 3.0, 121);
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(
            &normal,
            &anomalous,
            Some(&cpdag),
            &BrcdConfig::continuous(0)
        )
        .err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn non_2d_anomalous_tensor_is_rejected() {
    // A 3-D anomalous tensor fails the shape_2d guard → DimensionMismatch.
    let normal = chain_data(20, 0.0, 131);
    let anomalous = CausalTensor::new(vec![0.0_f64; 8], vec![2, 2, 2]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(
            &normal,
            &anomalous,
            Some(&cpdag),
            &BrcdConfig::continuous(0)
        )
        .err(),
        Some(BrcdError(BrcdErrorEnum::DimensionMismatch))
    );
}

#[test]
fn discrete_family_rejects_negative_states() {
    // The discrete family rounds each value to a non-negative integer state; a
    // negative value (here -1.0) is outside the 0..K range and must be rejected.
    let normal = CausalTensor::new(vec![-1.0_f64, 0.0, 1.0, 0.0, 1.0, 2.0], vec![2, 3]).unwrap();
    let anomalous = CausalTensor::new(vec![0.0_f64, 1.0, 2.0, 1.0, 2.0, 0.0], vec![2, 3]).unwrap();
    let cpdag = chain_cpdag();
    assert_eq!(
        brcd_run(&normal, &anomalous, Some(&cpdag), &BrcdConfig::discrete(0)).err(),
        Some(BrcdError(BrcdErrorEnum::StateOutOfRange))
    );
}
