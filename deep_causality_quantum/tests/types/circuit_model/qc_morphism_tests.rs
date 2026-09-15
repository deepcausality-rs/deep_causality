/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Kraus-level morphism carrier and its two caps.
//!
//! Provenance of the literals: the Choi of the identity on one qubit is `Σ_{ik} |ii⟩⟨kk|`, the
//! unnormalised maximally entangled projector, with Frobenius norm `2` and trace `2`; `Z` has the
//! same Choi with signs `(−1)^{i+k}`, so `‖J(id) − J(Z)‖_F = ‖2(|01⟩⟨01|-like block)‖` works out to
//! `√8 = 2√2` (four entries of modulus 2 off the `i = k` diagonal). Corner cases: (A) an empty
//! family, (B) a single scalar block, (D) mismatched types, (F) the entry cap at exactly the limit
//! and one above, (G) a classical value at its outcome count.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, NumericCaps, QcMorphism, QuantumErrorEnum, QubitOperator, choi_from_kraus,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn z() -> CausalTensor<C> {
    CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(-1.0, 0.0),
        ],
        &[2, 2],
    )
}

#[test]
fn test_identity_equals_itself_and_differs_from_z_by_two_root_two() {
    let caps = NumericCaps::default();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let (same, entries) = id.frobenius_distance(&id, &caps).unwrap();
    assert!(same.abs() < 1e-15);
    assert_eq!(entries, 32, "two Choi operators of 16 entries each");
    let (diff, _) = id.frobenius_distance(&zc, &caps).unwrap();
    assert!((diff - 8f64.sqrt()).abs() < 1e-12, "{diff}");
    assert_eq!(id.d_in(), 2);
    assert_eq!(id.d_out(), 2);
    assert_eq!(id.input_qubits(), 1);
    assert_eq!(id.output_qubits(), 1);
    assert_eq!(id.operator_count(), 1);
    assert_eq!(id.choi_entries(), 16);
}

#[test]
fn test_from_channel_agrees_with_choi_from_kraus() {
    let ch = Channel::unitary(&QubitOperator::hadamard()).unwrap();
    let m = QcMorphism::from_channel(&ch).unwrap();
    let (blocks, _) = m.choi_blocks(&NumericCaps::default()).unwrap();
    let j: &CausalTensor<C> = blocks.get(&(vec![], vec![])).unwrap();
    let expect: CausalTensor<C> = choi_from_kraus(ch.kraus().unwrap()).unwrap();
    for (a, b) in j.as_slice().iter().zip(expect.as_slice()) {
        assert!((a.re - b.re).abs() < 1e-15 && (a.im - b.im).abs() < 1e-15);
    }
    // A composed channel holds only its Choi; the Kraus family is recovered.
    let composed = ch.compose(&ch).unwrap();
    let m2 = QcMorphism::from_channel(&composed).unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let (d, _) = m2.frobenius_distance(&id, &NumericCaps::default()).unwrap();
    assert!(d < 1e-12, "H·H = I, {d}");
}

#[test]
fn test_then_composes_kraus_products_and_classical_wiring() {
    let caps = NumericCaps::default();
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let zz = zc.then(&zc, &caps).unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    assert!(zz.frobenius_distance(&id, &caps).unwrap().0 < 1e-15);
    // Classical: a bit-flip stochastic matrix as scalar blocks, composed with itself.
    let one = || CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1]);
    let mut flip = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    flip.push(vec![0], vec![1], vec![one()]).unwrap();
    flip.push(vec![1], vec![0], vec![one()]).unwrap();
    let twice = flip.then(&flip, &caps).unwrap();
    assert!(twice.blocks().contains_key(&(vec![0], vec![0])));
    assert!(twice.blocks().contains_key(&(vec![1], vec![1])));
    assert!(!twice.blocks().contains_key(&(vec![0], vec![1])));
    assert_eq!(twice.classical_in(), &[2]);
    assert_eq!(twice.classical_out(), &[2]);
    // Mismatched types cannot compose or compare.
    let err = flip.then(&zc, &caps).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let err = flip.frobenius_distance(&zc, &caps).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let tight = NumericCaps {
        max_entries: 1 << 24,
        max_operators: 0,
    };
    let err = zc.then(&zc, &tight).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::KrausFamilyExceeded {
            operators: 1,
            cap: 0
        }
    ));
}

#[test]
fn test_entry_cap_is_exact_at_the_boundary() {
    let id = QcMorphism::<f64>::identity(2).unwrap();
    let at = NumericCaps {
        max_entries: 16,
        max_operators: 1 << 12,
    };
    assert!(id.choi_blocks(&at).is_ok());
    let below = NumericCaps {
        max_entries: 15,
        max_operators: 1 << 12,
    };
    let err = id.choi_blocks(&below).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            n: 1,
            k: 1,
            entries: 16,
            cap: 15
        }
    ));
}

#[test]
fn test_push_validates_values_and_shapes() {
    let mut m = QcMorphism::<f64>::new(2, 2, vec![2], vec![]).unwrap();
    let bad_len = m.push(vec![], vec![], vec![z()]).unwrap_err();
    assert!(matches!(bad_len.0, QuantumErrorEnum::DimensionMismatch(_)));
    let at_count = m.push(vec![2], vec![], vec![z()]).unwrap_err();
    assert!(matches!(at_count.0, QuantumErrorEnum::DimensionMismatch(_)));
    let bad_shape = m
        .push(
            vec![0],
            vec![],
            vec![CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1])],
        )
        .unwrap_err();
    assert!(matches!(
        bad_shape.0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    assert!(m.push(vec![1], vec![], vec![z()]).is_ok());
    assert!(QcMorphism::<f64>::from_kraus(&[]).is_err());
    assert!(QcMorphism::<f64>::new(0, 1, vec![], vec![]).is_err());
    assert!(QcMorphism::<f64>::new(1, 1, vec![0], vec![]).is_err());
}

#[test]
fn test_entry_count_multiplies_by_the_block_count() {
    // Two scalar blocks: one entry each, two in all; a cap of one refuses them.
    let one = || CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1]);
    let mut flip = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    flip.push(vec![0], vec![1], vec![one()]).unwrap();
    flip.push(vec![1], vec![0], vec![one()]).unwrap();
    assert_eq!(flip.choi_entries(), 2);
    let caps = NumericCaps {
        max_entries: 1,
        max_operators: 1 << 12,
    };
    let err = flip.choi_blocks(&caps).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            n: 0,
            k: 0,
            entries: 2,
            cap: 1
        }
    ));
    let (_, entries) = flip.choi_blocks(&NumericCaps::default()).unwrap();
    assert_eq!(entries, 2);
}

#[test]
fn test_default_caps_are_the_design_values() {
    let caps = NumericCaps::default();
    assert_eq!(caps.max_entries, 1 << 24);
    assert_eq!(caps.max_operators, 1 << 12);
}

#[test]
fn test_tensor_classical_identity_state_and_leg_permutation() {
    use deep_causality_quantum::{GateOp, gate_unitary};
    let caps = NumericCaps::default();
    let id2 = QcMorphism::<f64>::identity(2).unwrap();
    let id4 = QcMorphism::<f64>::identity(4).unwrap();
    assert!(
        id2.tensor(&id2, &caps)
            .unwrap()
            .frobenius_distance(&id4, &caps)
            .unwrap()
            .0
            < 1e-15
    );
    let cid = QcMorphism::<f64>::classical_identity(&[2, 3]).unwrap();
    assert_eq!(cid.blocks().len(), 6);
    assert!(cid.blocks().keys().all(|(x, y)| x == y));
    assert!(QcMorphism::<f64>::classical_identity(&[0]).is_err());
    // A state then Z: |+⟩ ↦ |−⟩.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let plus = QcMorphism::<f64>::state(&[C::new(s, 0.0), C::new(s, 0.0)]).unwrap();
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let after = plus.then(&zc, &caps).unwrap();
    let k = &after.blocks().get(&(vec![], vec![])).unwrap()[0];
    assert_eq!(k.shape(), &[2, 1]);
    assert!((k.as_slice()[0].re - s).abs() < 1e-15 && (k.as_slice()[1].re + s).abs() < 1e-15);
    assert!(QcMorphism::<f64>::state(&[]).is_err());
    // Permuting the two legs of CNOT(0 → 1) gives CNOT(1 → 0).
    let (_, forward) = gate_unitary::<f64>(&GateOp::Cnot {
        control: 0,
        target: 1,
    })
    .unwrap();
    let (_, backward) = gate_unitary::<f64>(&GateOp::Cnot {
        control: 1,
        target: 0,
    })
    .unwrap();
    let m = QcMorphism::from_kraus(&[forward]).unwrap();
    let swapped = m.permute_legs(&[2, 2], &[1, 0], &[2, 2], &[1, 0]).unwrap();
    let expect = QcMorphism::from_kraus(&[backward]).unwrap();
    assert!(swapped.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15);
    assert!(m.permute_legs(&[2, 2], &[0, 0], &[2, 2], &[0, 1]).is_err());
    assert!(m.permute_legs(&[3, 2], &[0, 1], &[2, 2], &[0, 1]).is_err());
}

#[test]
fn test_tensor_order_against_the_kronecker_product() {
    // `I ⊗ Z = diag(1, −1, 1, −1)`, `Z ⊗ I = diag(1, 1, −1, −1)`: entry (1, 1) tells them apart.
    let caps = NumericCaps::default();
    let id2 = QcMorphism::<f64>::identity(2).unwrap();
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let iz = id2.tensor(&zc, &caps).unwrap();
    let k = &iz.blocks().get(&(vec![], vec![])).unwrap()[0];
    assert!(
        (k.as_slice()[4 + 1].re + 1.0).abs() < 1e-15,
        "I ⊗ Z has −1 at (1, 1)"
    );
    assert!(
        (k.as_slice()[2 * 4 + 2].re - 1.0).abs() < 1e-15,
        "and +1 at (2, 2)"
    );
    let identity = CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    let expect = identity.kronecker(&z()).unwrap();
    for (a, b) in k.as_slice().iter().zip(expect.as_slice()) {
        assert!((a.re - b.re).abs() < 1e-15 && (a.im - b.im).abs() < 1e-15);
    }
}

#[test]
fn test_then_and_tensor_refuse_products_above_the_entry_cap_before_forming_them() {
    // `Z · Z`: one operator of 2 × 2, four entries. A cap of three refuses it, four admits it.
    let zc = QcMorphism::from_kraus(&[z()]).unwrap();
    let three = NumericCaps {
        max_entries: 3,
        max_operators: 1 << 12,
    };
    let err = zc.then(&zc, &three).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            n: 1,
            k: 1,
            entries: 4,
            cap: 3
        }
    ));
    let four = NumericCaps {
        max_entries: 4,
        max_operators: 1 << 12,
    };
    assert!(zc.then(&zc, &four).is_ok());
    // `I₂ ⊗ I₂`: one operator of 4 × 4, sixteen entries. Fifteen refuses, sixteen admits.
    let id2 = QcMorphism::<f64>::identity(2).unwrap();
    let fifteen = NumericCaps {
        max_entries: 15,
        max_operators: 1 << 12,
    };
    let err = id2.tensor(&id2, &fifteen).unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            n: 2,
            k: 2,
            entries: 16,
            cap: 15
        }
    ));
    let sixteen = NumericCaps {
        max_entries: 16,
        max_operators: 1 << 12,
    };
    assert!(id2.tensor(&id2, &sixteen).is_ok());
    // Two scalar blocks each side: `then` matches blocks on the middle value, so the composite has
    // two operators of one entry; `tensor` pairs every block with every block, four operators.
    let one = || CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1]);
    let mut flip = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    flip.push(vec![0], vec![1], vec![one()]).unwrap();
    flip.push(vec![1], vec![0], vec![one()]).unwrap();
    let one_entry = NumericCaps {
        max_entries: 1,
        max_operators: 1 << 12,
    };
    assert!(matches!(
        flip.then(&flip, &one_entry).unwrap_err().0,
        QuantumErrorEnum::NaturalityDimensionExceeded { entries: 2, .. }
    ));
    let three_entries = NumericCaps {
        max_entries: 3,
        max_operators: 1 << 12,
    };
    assert!(matches!(
        flip.tensor(&flip, &three_entries).unwrap_err().0,
        QuantumErrorEnum::NaturalityDimensionExceeded { entries: 4, .. }
    ));
}

/// (H) A morphism too wide for its natural representation is refused by the entry cap before any
/// arithmetic on the dimensions can overflow: `(2^33)^2` entries per side do not fit `u64`, so the
/// count saturates and the cap refuses it.
#[test]
fn test_induced_norm_refuses_an_oversized_morphism_before_the_dimensions_overflow() {
    let wide = QcMorphism::<f64>::new(1, 1 << 33, vec![], vec![]).unwrap();
    let err = wide
        .frobenius_induced_norm(&NumericCaps::default())
        .unwrap_err();
    match err.0 {
        QuantumErrorEnum::NaturalityDimensionExceeded { entries, cap, .. } => {
            assert_eq!(cap, 1 << 24);
            assert!(entries > cap, "{entries}");
        }
        other => panic!("{other:?}"),
    }
}

/// (H) The qubit count of the widest dimension `usize` holds is the bit width, and it is reached
/// without shifting past the word.
#[test]
fn test_qubit_counts_terminate_at_the_top_of_usize() {
    let widest = QcMorphism::<f64>::new(usize::MAX, 1, vec![], vec![]).unwrap();
    assert_eq!(widest.input_qubits(), usize::BITS as usize);
    assert_eq!(widest.output_qubits(), 0);
    let top = QcMorphism::<f64>::new(1 << 40, 1 << 40, vec![], vec![]).unwrap();
    assert_eq!((top.input_qubits(), top.output_qubits()), (40, 40));
    assert_eq!(top.choi_entries(), u64::MAX, "saturated, not wrapped");
}

/// (H) The identity on a dimension whose square does not fit `usize` is refused, not allocated.
#[test]
fn test_identity_refuses_a_dimension_whose_square_overflows() {
    let err = QcMorphism::<f64>::identity(1 << 33).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

/// The induced norm of a morphism with classical wires is the largest singular value of the whole
/// natural representation over the direct sums, not the largest block norm. A fair coin from the
/// trivial system, two output blocks with scalar Kraus `√½`, sends `ρ` to `(½ρ, ½ρ)` with
/// Frobenius norm `√½ ‖ρ‖`, so its norm is `√½` where the block maximum is `½`. The sum of two
/// classical values into one, two input blocks with scalar Kraus `1`, sends `(a, b)` to `a + b`,
/// norm `√2` where either per-input stacking gives `1`. A diagonal of scalar blocks `1` and `2`
/// stays at the block maximum `4`.
#[test]
fn test_induced_norm_over_classical_blocks_is_the_norm_of_the_whole_natural_representation() {
    let caps = NumericCaps::default();
    let scalar = |v: f64| CausalTensor::from_slice(&[C::new(v, 0.0)], &[1, 1]);
    let half = 0.5f64.sqrt();
    let mut coin = QcMorphism::<f64>::new(1, 1, vec![], vec![2]).unwrap();
    coin.push(vec![], vec![0], vec![scalar(half)]).unwrap();
    coin.push(vec![], vec![1], vec![scalar(half)]).unwrap();
    let norm = coin.frobenius_induced_norm(&caps).unwrap();
    assert!((norm - half).abs() < 1e-12, "fair coin: {norm}");

    let mut sum = QcMorphism::<f64>::new(1, 1, vec![2], vec![]).unwrap();
    sum.push(vec![0], vec![], vec![scalar(1.0)]).unwrap();
    sum.push(vec![1], vec![], vec![scalar(1.0)]).unwrap();
    let norm = sum.frobenius_induced_norm(&caps).unwrap();
    assert!(
        (norm - 2f64.sqrt()).abs() < 1e-12,
        "sum of two values: {norm}"
    );

    let mut diagonal = QcMorphism::<f64>::new(1, 1, vec![2], vec![2]).unwrap();
    diagonal.push(vec![0], vec![0], vec![scalar(1.0)]).unwrap();
    diagonal.push(vec![1], vec![1], vec![scalar(2.0)]).unwrap();
    let norm = diagonal.frobenius_induced_norm(&caps).unwrap();
    assert!((norm - 4.0).abs() < 1e-12, "diagonal: {norm}");

    // A computational-basis measurement, one quantum input to two classical blocks with Kraus
    // `|0⟩⟨0|` and `|1⟩⟨1|`, keeps the diagonal of `ρ` and has norm one; its entry count under the
    // cap is the stacked `(2 · 4) × 4` representation.
    let one = C::new(1.0, 0.0);
    let zero = C::new(0.0, 0.0);
    let p0 = CausalTensor::from_slice(&[one, zero, zero, zero], &[2, 2]);
    let p1 = CausalTensor::from_slice(&[zero, zero, zero, one], &[2, 2]);
    let mut measure = QcMorphism::<f64>::new(2, 1, vec![], vec![2]).unwrap();
    measure
        .push(
            vec![],
            vec![0],
            vec![CausalTensor::from_slice(&[one, zero], &[1, 2])],
        )
        .unwrap();
    measure
        .push(
            vec![],
            vec![1],
            vec![CausalTensor::from_slice(&[zero, one], &[1, 2])],
        )
        .unwrap();
    let norm = measure.frobenius_induced_norm(&caps).unwrap();
    assert!((norm - 1.0).abs() < 1e-12, "measurement: {norm}");
    let mut dephase = QcMorphism::<f64>::new(2, 2, vec![], vec![2]).unwrap();
    dephase.push(vec![], vec![0], vec![p0]).unwrap();
    dephase.push(vec![], vec![1], vec![p1]).unwrap();
    let norm = dephase.frobenius_induced_norm(&caps).unwrap();
    assert!((norm - 1.0).abs() < 1e-12, "dephasing instrument: {norm}");
    let tight = NumericCaps {
        max_entries: 31,
        max_operators: 1 << 12,
    };
    assert!(matches!(
        dephase.frobenius_induced_norm(&tight).unwrap_err().0,
        QuantumErrorEnum::NaturalityDimensionExceeded {
            entries: 32,
            cap: 31,
            ..
        }
    ));
}
