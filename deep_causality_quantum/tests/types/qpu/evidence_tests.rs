/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qpu")]

//! The shot budget: reproducible at a seed, drawn down in checked ℕ arithmetic, refused at zero.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    DensityMatrix, Evidence, Projection, QuantumErrorEnum, ShotBudget, ShotHistogram,
    sample_within_budget,
};
use deep_causality_tensor::CausalTensor;

/// Counts are ℕ; the width is named once, here, with the unsigned alias.
type Count = u64;

fn ket(a: f64, b: f64) -> CausalTensor<Complex<f64>> {
    CausalTensor::from_slice(&[Complex::new(a, 0.), Complex::new(b, 0.)], &[2])
}

#[test]
fn test_a_declaration_funds_a_budget_and_zero_is_refused() {
    let budget = Evidence::<Count>::shots(1024)
        .seed(20260821)
        .into_budget()
        .unwrap();
    assert_eq!(budget.remaining(), 1024);
    assert_eq!(budget.spent(), 0);
    assert_eq!(budget.seed(), 20260821);
    assert!(!budget.is_exhausted());

    let err = Evidence::<Count>::shots(0).into_budget().unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => assert!(msg.contains("zero"), "{msg}"),
        other => panic!("expected CalculationError, got {other:?}"),
    }
    assert!(ShotBudget::<Count>::new(0, 1).is_err());
}

#[test]
fn test_an_overdrawn_budget_names_the_shortfall_and_records_nothing() {
    let budget = ShotBudget::<Count>::new(100, 1).unwrap();
    let err = budget.draw(150).unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("150") && msg.contains("100"), "{msg}");
            assert!(msg.contains("shortfall 50"), "names the shortfall: {msg}");
        }
        other => panic!("expected CalculationError, got {other:?}"),
    }
    // The receiver is unchanged: nothing was recorded.
    assert_eq!(budget.remaining(), 100);
    assert_eq!(budget.spent(), 0);
}

#[test]
fn test_the_draw_down_is_checked_arithmetic_and_exhausts_exactly() {
    let budget = ShotBudget::<Count>::new(100, 1).unwrap();
    let (first, after) = budget.draw(60).unwrap();
    assert_eq!(first.shots, 60);
    assert_eq!((after.remaining(), after.spent()), (40, 60));
    let (second, after) = after.draw(40).unwrap();
    assert_eq!(second.shots, 40);
    assert!(after.is_exhausted());
    assert!(after.draw(1).is_err());
    // Successive draws at one run seed carry distinct draw seeds.
    assert_ne!(first.seed, second.seed);
}

#[test]
fn test_the_draw_seed_reads_the_full_width_of_the_spent_count() {
    // Two ledgers whose spent counts agree in their low 64 bits and differ above 2^64: one has
    // spent 2^64 shots, the other 2^65. Their next draws are seeded differently, and a ledger
    // replayed at the same count reproduces its seed.
    let seed = 20260821;
    let next_seed_after = |spent: u128| {
        let budget = ShotBudget::<u128>::new(spent + 10, seed).unwrap();
        let (_, after) = budget.draw(spent).unwrap();
        assert_eq!(after.spent(), spent);
        after.draw(10).unwrap().0.seed
    };
    let at_2_64 = next_seed_after(1u128 << 64);
    let at_2_65 = next_seed_after(1u128 << 65);
    assert_ne!(at_2_64, at_2_65);
    assert_eq!(at_2_64, next_seed_after(1u128 << 64));
    // And a count below 2^64 still differs from both.
    assert_ne!(next_seed_after(0), at_2_64);
}

#[test]
fn test_two_runs_at_one_seed_agree_exactly() {
    let rho = DensityMatrix::from_ket(&ket(0.6, 0.8)).unwrap();
    let p = Projection::<f64, 2>::from_ket(&ket(0., 1.)).unwrap();
    let run = || {
        let budget = Evidence::<Count>::shots(1024)
            .seed(20260821)
            .into_budget()
            .unwrap();
        let (h1, budget) = sample_within_budget(&rho, &p, &budget, 512).unwrap();
        let (h2, budget) = sample_within_budget(&rho, &p, &budget, 512).unwrap();
        (h1, h2, budget)
    };
    let (a1, a2, ab) = run();
    let (b1, b2, bb) = run();
    assert_eq!(a1, b1);
    assert_eq!(a2, b2);
    assert_eq!(ab, bb);
    assert!(ab.is_exhausted());
    // The two draws within a run are different samples, not one repeated.
    assert_ne!(a1, a2);
    assert_eq!(a1.total() + a2.total(), 1024);
}

#[test]
fn test_a_malformed_read_out_spends_nothing() {
    let three = CausalTensor::from_slice(
        &[
            Complex::new(1., 0.),
            Complex::new(0., 0.),
            Complex::new(0., 0.),
        ],
        &[3],
    );
    let rho = DensityMatrix::from_ket(&three).unwrap();
    let p = Projection::<f64, 2>::from_ket(&ket(0., 1.)).unwrap();
    let budget = ShotBudget::<Count>::new(1000, 5).unwrap();
    let err = sample_within_budget(&rho, &p, &budget, 100).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    assert_eq!(budget.remaining(), 1000);
    assert_eq!(budget.spent(), 0);
}
