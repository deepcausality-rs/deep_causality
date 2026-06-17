/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_fft::{DctPlan, DctType, FftError, naive_dct_i, naive_dct_ii, naive_dct_iii};
use deep_causality_num::{Complex, Float106};

fn rbuf(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| ((i as f64) * 0.37).sin() + 0.25 * ((i as f64) * 1.7).cos())
        .collect()
}

fn run_plan(plan: &DctPlan<f64>, input: &[f64], inverse: bool) -> Vec<f64> {
    let n = input.len();
    let mut out = vec![0.0; n];
    let mut rs = vec![0.0; plan.scratch_real_len()];
    let mut cs = vec![Complex::new(0.0, 0.0); plan.scratch_complex_len()];
    if inverse {
        plan.execute_inverse(input, &mut out, &mut rs, &mut cs)
            .unwrap();
    } else {
        plan.execute(input, &mut out, &mut rs, &mut cs).unwrap();
    }
    out
}

fn assert_close(a: &[f64], b: &[f64], tol: f64, what: &str) {
    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        assert!((x - y).abs() < tol, "{what}[{i}]: {x} vs {y}");
    }
}

#[test]
fn dct_ii_matches_naive() {
    // Power-of-two, odd, even-composite, and prime lengths (Bluestein).
    for n in [1usize, 2, 4, 8, 9, 12, 16, 17, 31, 64] {
        let plan = DctPlan::<f64>::new(n, DctType::II).unwrap();
        assert_eq!(plan.len(), n);
        assert!(!plan.is_empty());
        assert_eq!(plan.dct_type(), DctType::II);
        let x = rbuf(n);
        let got = run_plan(&plan, &x, false);
        assert_close(&got, &naive_dct_ii(&x), 1e-11, &format!("dct2 n={n}"));
    }
}

#[test]
fn dct_iii_matches_naive() {
    for n in [1usize, 2, 4, 8, 9, 16, 17, 64] {
        let plan = DctPlan::<f64>::new(n, DctType::III).unwrap();
        let x = rbuf(n);
        let got = run_plan(&plan, &x, false);
        assert_close(&got, &naive_dct_iii(&x), 1e-11, &format!("dct3 n={n}"));
    }
}

#[test]
fn dct_i_matches_naive() {
    for n in [2usize, 3, 4, 8, 9, 16, 17, 33, 65] {
        let plan = DctPlan::<f64>::new(n, DctType::I).unwrap();
        let x = rbuf(n);
        let got = run_plan(&plan, &x, false);
        assert_close(&got, &naive_dct_i(&x), 1e-11, &format!("dct1 n={n}"));
    }
}

#[test]
fn all_types_round_trip() {
    for ty in [DctType::I, DctType::II, DctType::III] {
        for n in [2usize, 4, 8, 9, 16, 17, 64] {
            let plan = DctPlan::<f64>::new(n, ty).unwrap();
            let x = rbuf(n);
            let fwd = run_plan(&plan, &x, false);
            let back = run_plan(&plan, &fwd, true);
            assert_close(&back, &x, 1e-11, &format!("{ty:?} round trip n={n}"));
        }
    }
}

#[test]
fn ii_iii_pairing_identity() {
    // DCT-III(DCT-II(x)) = (n/2)·x under the unnormalized conventions.
    let n = 16usize;
    let plan2 = DctPlan::<f64>::new(n, DctType::II).unwrap();
    let plan3 = DctPlan::<f64>::new(n, DctType::III).unwrap();
    let x = rbuf(n);
    let mid = run_plan(&plan2, &x, false);
    let out = run_plan(&plan3, &mid, false);
    let scaled: Vec<f64> = x.iter().map(|v| v * (n as f64) / 2.0).collect();
    assert_close(&out, &scaled, 1e-10, "II∘III pairing");
}

#[test]
fn float106_accuracy() {
    let n = 16usize;
    let plan = DctPlan::<Float106>::new(n, DctType::II).unwrap();
    let x: Vec<Float106> = (0..n)
        .map(|i| {
            let v = Float106::from_f64(i as f64);
            let a = Float106::from_f64(0.37);
            deep_causality_num::Real::sin(v * a)
        })
        .collect();
    let mut out = vec![Float106::from_f64(0.0); n];
    let mut rs = vec![Float106::from_f64(0.0); plan.scratch_real_len()];
    let mut cs = vec![
        Complex::new(Float106::from_f64(0.0), Float106::from_f64(0.0));
        plan.scratch_complex_len()
    ];
    plan.execute(&x, &mut out, &mut rs, &mut cs).unwrap();
    let reference = naive_dct_ii(&x);
    for (a, b) in out.iter().zip(reference.iter()) {
        let d = *a - *b;
        assert!(d.hi().abs() < 1e-28, "Float106 dct2 err {:e}", d.hi());
    }
}

#[test]
fn f32_accuracy() {
    let n = 32usize;
    let plan = DctPlan::<f32>::new(n, DctType::II).unwrap();
    let x: Vec<f32> = (0..n).map(|i| ((i as f32) * 0.37).sin()).collect();
    let mut out = vec![0.0f32; n];
    let mut rs = vec![0.0f32; plan.scratch_real_len()];
    let mut cs = vec![Complex::new(0.0f32, 0.0); plan.scratch_complex_len()];
    plan.execute(&x, &mut out, &mut rs, &mut cs).unwrap();
    let reference = naive_dct_ii(&x);
    for (a, b) in out.iter().zip(reference.iter()) {
        assert!((a - b).abs() < 1e-3);
    }
}

#[test]
fn invalid_lengths_rejected() {
    assert_eq!(
        DctPlan::<f64>::new(0, DctType::II).unwrap_err(),
        FftError::InvalidLength(0)
    );
    assert_eq!(
        DctPlan::<f64>::new(1, DctType::I).unwrap_err(),
        FftError::InvalidLength(1)
    );
}

#[test]
fn buffer_validation() {
    let plan = DctPlan::<f64>::new(8, DctType::II).unwrap();
    let x = rbuf(8);
    let mut out = vec![0.0; 8];
    let mut rs = vec![0.0; plan.scratch_real_len()];
    let mut cs = vec![Complex::new(0.0, 0.0); plan.scratch_complex_len()];

    let bad_in = rbuf(4);
    let err = plan
        .execute(&bad_in, &mut out, &mut rs, &mut cs)
        .unwrap_err();
    assert!(matches!(err, FftError::LengthMismatch { .. }));

    let mut bad_out = vec![0.0; 4];
    let err = plan
        .execute(&x, &mut bad_out, &mut rs, &mut cs)
        .unwrap_err();
    assert!(matches!(err, FftError::LengthMismatch { .. }));

    let mut tiny_rs: Vec<f64> = Vec::new();
    let err = plan
        .execute(&x, &mut out, &mut tiny_rs, &mut cs)
        .unwrap_err();
    assert!(matches!(err, FftError::ScratchTooSmall { .. }));

    let mut tiny_cs: Vec<Complex<f64>> = Vec::new();
    let err = plan
        .execute(&x, &mut out, &mut rs, &mut tiny_cs)
        .unwrap_err();
    assert!(matches!(err, FftError::ScratchTooSmall { .. }));
}

#[test]
fn plan_is_cloneable_and_debuggable() {
    let plan = DctPlan::<f64>::new(16, DctType::I).unwrap();
    let cloned = plan.clone();
    assert_eq!(cloned.len(), 16);
    assert!(format!("{plan:?}").contains("DctPlan"));
}
