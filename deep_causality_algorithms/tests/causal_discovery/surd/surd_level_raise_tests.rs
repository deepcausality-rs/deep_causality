/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_algorithms::causal_discovery::surd::{MaxOrder, surd_states, surd_states_cdl};
use deep_causality_tensor::CausalTensor;

// These tests drive the monotonicity-repair ("level-raising") loop inside
// `analyze_single_target_state` / `analyze_single_target_state_cdl`.
//
// After the specific informations are sorted ascending, the loop walks label
// lengths `l = 1..max_len` and, for every length-`l+1` term whose value is
// strictly BELOW the maximum value seen among length-`l` terms, raises it to
// that maximum (`if *val < max_prev_level { *val = max_prev_level; }`). The
// raise body only executes when a higher-order combination has LOWER specific
// information than the strongest lower-order combination.
//
// The construction below makes that happen: T is almost perfectly determined by
// S1 (large single-variable specific info), while S2/S3 are only weakly coupled
// to T, so several pair/triple combinations land below the strong S1 term and
// must be raised. This exercises:
//   surd_algo.rs     line ~410 (`*val = max_prev_level;`)
//   surd_algo_cdl.rs line ~410 (same, `Option<T>` variant)

fn strong_single_weak_higher_order() -> Vec<f64> {
    let mut data = vec![0.001_f64; 16];
    let idx = |t: usize, s1: usize, s2: usize, s3: usize| ((t * 2 + s1) * 2 + s2) * 2 + s3;
    // T tracks S1 almost perfectly (strong unique S1 information).
    data[idx(0, 0, 0, 0)] += 0.20;
    data[idx(0, 0, 1, 1)] += 0.20;
    data[idx(0, 0, 1, 0)] += 0.05;
    data[idx(1, 1, 0, 0)] += 0.20;
    data[idx(1, 1, 1, 1)] += 0.20;
    data[idx(1, 1, 0, 1)] += 0.05;
    // A pinch of noise keeps S2/S3 only weakly coupled to T.
    data[idx(0, 1, 0, 0)] += 0.02;
    data[idx(1, 0, 1, 1)] += 0.02;
    data
}

#[test]
fn test_level_raise_strong_single_weak_pairs() {
    let p_raw: CausalTensor<f64> =
        CausalTensor::new(strong_single_weak_higher_order(), vec![2, 2, 2, 2]).unwrap();
    let result = surd_states(&p_raw, MaxOrder::Max).unwrap();

    // The decomposition must succeed and contain higher-order (3-variable) terms,
    // which is what makes the level-raising loop run with `max_len == 3`.
    assert!(!result.mutual_info().is_empty());
    assert!(result.mutual_info().keys().any(|k| k.len() == 3));

    // S1 carries the dominant information, so its mutual information is the
    // largest among the single-variable terms (a stable, value-level invariant
    // of this distribution that the level-raising preserves).
    let mi_s1 = *result.mutual_info().get(&vec![1]).unwrap();
    let mi_s2 = *result.mutual_info().get(&vec![2]).unwrap();
    let mi_s3 = *result.mutual_info().get(&vec![3]).unwrap();
    assert!(mi_s1 > mi_s2);
    assert!(mi_s1 > mi_s3);
}

#[test]
fn test_level_raise_strong_single_weak_pairs_cdl() {
    let opt: Vec<Option<f64>> = strong_single_weak_higher_order()
        .into_iter()
        .map(Some)
        .collect();
    let p_raw = CausalTensor::new(opt, vec![2, 2, 2, 2]).unwrap();
    let result = surd_states_cdl(&p_raw, MaxOrder::Max).unwrap();

    assert!(!result.mutual_info().is_empty());
    assert!(result.mutual_info().keys().any(|k| k.len() == 3));

    let mi_s1 = *result.mutual_info().get(&vec![1]).unwrap();
    let mi_s2 = *result.mutual_info().get(&vec![2]).unwrap();
    let mi_s3 = *result.mutual_info().get(&vec![3]).unwrap();
    assert!(mi_s1 > mi_s2);
    assert!(mi_s1 > mi_s3);
}
