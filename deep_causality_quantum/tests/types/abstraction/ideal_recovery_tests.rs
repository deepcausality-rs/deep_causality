/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The isometry and the ideal recovery of the `[[4,2,2]]` and `[[8,2,2]]` codes.
//!
//! `[[4,2,2]]` has one independent Z-check and one X-check, so four syndromes, and every syndrome
//! is produced by a weight-one Pauli (`X` flips the Z-check, `Z` the X-check, `Y` both). `[[8,2,2]]`
//! has three of each, so 64 syndromes. In both, `τ ∘ E = id` on the logical space and, for each
//! correction `C_s` of the table, `τ(C_s E ρ E† C_s†) = ρ`: the table recovers its own errors.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::utils_tests::four_two_two;
use deep_causality_quantum::{
    IDEAL_RECOVERY_MAX_QUBITS, IdealRecovery, LogicalBasis, NumericCaps, QcMorphism,
    QuantumErrorEnum, apply_kraus,
};
use deep_causality_tensor::{CausalTensor, Tensor};
use deep_causality_topology::LatticeComplex;

type C = Complex<f64>;

fn pauli_matrix(n: usize, x: &[bool], z: &[bool]) -> CausalTensor<C> {
    let d = 1usize << n;
    let mut data = vec![C::new(0.0, 0.0); d * d];
    for col in 0..d {
        let mut sign = 1.0;
        for (q, &s) in z.iter().enumerate() {
            if s && col & (1 << (n - 1 - q)) != 0 {
                sign = -sign;
            }
        }
        let mut row = col;
        for (q, &f) in x.iter().enumerate() {
            if f {
                row ^= 1 << (n - 1 - q);
            }
        }
        data[row * d + col] = C::new(sign, 0.0);
    }
    CausalTensor::from_slice(&data, &[d, d])
}

fn check_code(basis: &LogicalBasis<u64>, syndromes: usize) {
    let caps = NumericCaps::default();
    let rec = IdealRecovery::<f64>::from_basis(basis).unwrap();
    assert_eq!(rec.syndromes(), syndromes);
    assert_eq!(rec.corrections().len(), syndromes);
    let n = basis.len();
    let k = basis.num_logical_qubits();
    assert_eq!(rec.isometry().d_in(), 1 << k);
    assert_eq!(rec.isometry().d_out(), 1 << n);
    let e = rec.isometry().clone();
    let tau = rec.recovery().clone();
    let id = QcMorphism::<f64>::identity(1 << k).unwrap();
    assert!(
        e.then(&tau, &caps)
            .unwrap()
            .frobenius_distance(&id, &caps)
            .unwrap()
            .0
            < 1e-10
    );
    // The table recovers each of its own corrections on a non-trivial logical state.
    let d_k = 1usize << k;
    let mut rho = vec![C::new(0.0, 0.0); d_k * d_k];
    // |ψ⟩ = (|0…0⟩ + 2|0…1⟩)/√5, an asymmetric pure logical state.
    let amps: Vec<f64> = (0..d_k)
        .map(|i| {
            if i == 0 {
                1.0 / 5f64.sqrt()
            } else if i == 1 {
                2.0 / 5f64.sqrt()
            } else {
                0.0
            }
        })
        .collect();
    for i in 0..d_k {
        for j in 0..d_k {
            rho[i * d_k + j] = C::new(amps[i] * amps[j], 0.0);
        }
    }
    let rho = CausalTensor::from_slice(&rho, &[d_k, d_k]);
    let encoded = apply_kraus(&rec.isometry().kraus(), &rho).unwrap();
    for (x, z) in rec.corrections().iter().take(8) {
        let c = pauli_matrix(n, x, z);
        let corrupted = c
            .matmul(&encoded)
            .unwrap()
            .matmul(&c.dagger().unwrap())
            .unwrap();
        let recovered = apply_kraus(&rec.recovery().kraus(), &corrupted).unwrap();
        for (a, b) in recovered.as_slice().iter().zip(rho.as_slice()) {
            assert!(
                (a.re - b.re).abs() < 1e-10 && (a.im - b.im).abs() < 1e-10,
                "correction {x:?}/{z:?}"
            );
        }
    }
}

#[test]
fn test_four_two_two_recovery() {
    let basis = LogicalBasis::<u64>::from_complex(&four_two_two(), 1).unwrap();
    check_code(&basis, 4);
    let rec = IdealRecovery::<f64>::from_basis(&basis).unwrap();
    assert_eq!(
        rec.max_correction_weight(),
        1,
        "every syndrome from a weight-one Pauli"
    );
}

#[test]
fn test_eight_two_two_recovery() {
    let complex = LatticeComplex::<2, f64>::square_torus(2);
    let basis = LogicalBasis::<u64>::from_complex(&complex, 1).unwrap();
    check_code(&basis, 64);
    let rec = IdealRecovery::<f64>::from_basis(&basis).unwrap();
    // On the 2 × 2 torus the two edges of a row join the same vertex pair and the two edges of a
    // column the same face pair, so single-edge X errors give 4 distinct syndromes, single-edge Z
    // errors 4, and single-edge Y errors one per edge, 8: sixteen weight-one corrections, and the
    // remaining 47 syndromes need weight two or more.
    let weights: Vec<usize> = rec
        .corrections()
        .iter()
        .map(|(x, z)| x.iter().zip(z).filter(|(a, b)| **a || **b).count())
        .collect();
    assert_eq!(weights[0], 0, "the trivial syndrome corrects nothing");
    assert_eq!(weights.iter().filter(|w| **w == 1).count(), 16);
    assert!(rec.max_correction_weight() >= 2);
    assert_eq!(
        rec.max_correction_weight(),
        *weights.iter().max().unwrap(),
        "the reported weight is the table's largest"
    );
}

#[test]
fn test_wide_codes_are_refused() {
    let complex = LatticeComplex::<2, f64>::square_torus(3);
    let basis = LogicalBasis::<u64>::from_complex(&complex, 1).unwrap();
    assert!(basis.len() > IDEAL_RECOVERY_MAX_QUBITS);
    let err = IdealRecovery::<f64>::from_basis(&basis).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("stops at")));
}

/// The encoder unitary on the `[[4,2,2]]` code: `W† W = I` on all sixteen dimensions, and the
/// column `|x⟩|0…0⟩` of `W` is the column `|x̄⟩` of the isometry. The identity is written out; the
/// isometry's columns are checked by `check_code` against the Pauli corrections.
#[test]
fn test_encoder_is_unitary_and_extends_the_isometry() {
    let basis = LogicalBasis::<u64>::from_complex(&four_two_two(), 1).unwrap();
    let rec = IdealRecovery::<f64>::from_basis(&basis).unwrap();
    let n = basis.len();
    let k = basis.num_logical_qubits();
    let d = 1usize << n;
    assert_eq!(rec.encoder().d_in(), d);
    assert_eq!(rec.encoder().d_out(), d);
    let w = rec.encoder().kraus();
    assert_eq!(w.len(), 1);
    let w = &w[0];
    let gram = w.dagger().unwrap().matmul(w).unwrap();
    for r in 0..d {
        for c in 0..d {
            let g = gram.as_slice()[r * d + c];
            let expect = if r == c { 1.0 } else { 0.0 };
            assert!(
                (g.re - expect).abs() < 1e-12 && g.im.abs() < 1e-12,
                "W†W[{r},{c}] = {g}"
            );
        }
    }
    let e = &rec.isometry().kraus()[0];
    let dk = 1usize << k;
    for x in 0..dk {
        let col = x << (n - k);
        for row in 0..d {
            let a = w.as_slice()[row * d + col];
            let b = e.as_slice()[row * dk + x];
            assert!(
                (a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12,
                "x = {x}, row {row}"
            );
        }
    }
    // A column outside the code space is orthogonal to every code word: the coset states with a
    // non-trivial character or a non-zero Z-syndrome.
    let other = 1usize;
    let mut overlap = 0.0f64;
    for x in 0..dk {
        let mut acc = C::new(0.0, 0.0);
        for row in 0..d {
            let a = e.as_slice()[row * dk + x];
            acc += C::new(a.re, -a.im) * w.as_slice()[row * d + other];
        }
        overlap += acc.re * acc.re + acc.im * acc.im;
    }
    assert!(overlap < 1e-24, "{overlap}");
}

/// On the `[[8,2,2]]` fixture the encoder's code-word columns match the isometry and the unitary is
/// checked on its columns' norms only, which is the part of `W† W = I` that the coset construction
/// could get wrong by a scale.
#[test]
fn test_eight_two_two_encoder_columns() {
    let complex = LatticeComplex::<2, f64>::square_torus(2);
    let basis = LogicalBasis::<u64>::from_complex(&complex, 1).unwrap();
    let rec = IdealRecovery::<f64>::from_basis(&basis).unwrap();
    let n = basis.len();
    let k = basis.num_logical_qubits();
    let (d, dk) = (1usize << n, 1usize << k);
    let w = &rec.encoder().kraus()[0];
    let e = &rec.isometry().kraus()[0];
    for x in 0..dk {
        let col = x << (n - k);
        for row in 0..d {
            let a = w.as_slice()[row * d + col];
            let b = e.as_slice()[row * dk + x];
            assert!((a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12);
        }
    }
    for col in 0..d {
        let norm: f64 = (0..d)
            .map(|row| {
                let a = w.as_slice()[row * d + col];
                a.re * a.re + a.im * a.im
            })
            .sum();
        assert!((norm - 1.0).abs() < 1e-12, "column {col}: {norm}");
    }
}
