/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Type alignments: the section check and the assembly of `τ` on a type in wire order.
//!
//! `H · H = I` makes `(τ, E) = (H, H)` a valid entry and `(H, id)` an invalid one with residual
//! `‖J(H) − J(I)‖_F = √8`: each unitary Choi operator has squared Frobenius norm `d² = 4`, and the
//! cross term `Tr J(H)†J(I) = |Tr H|²` is zero.
//! The partial trace `Tr_B` with the preparation `|0⟩_B` as section is the code-like case.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    AlignmentSide, NumericCaps, QcMorphism, QuantumErrorEnum, QubitOperator, Query, TypeAlignment,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn id() -> QcMorphism<f64> {
    QcMorphism::identity(2).unwrap()
}

fn h() -> QcMorphism<f64> {
    QcMorphism::from_kraus(&[QubitOperator::<f64>::hadamard().matrix().clone()]).unwrap()
}

/// `Tr_B` on two qubits as a channel `4 → 2`: Kraus operators `⟨0|_B` and `⟨1|_B`.
fn trace_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    let k0 = CausalTensor::from_slice(&[one, zero, zero, zero, zero, zero, one, zero], &[2, 4]);
    let k1 = CausalTensor::from_slice(&[zero, one, zero, zero, zero, zero, zero, one], &[2, 4]);
    QcMorphism::from_kraus(&[k0, k1]).unwrap()
}

/// Prepare `|0⟩_B` beside the input: a channel `2 → 4`.
fn prepare_b() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[CausalTensor::from_slice(
        &[one, zero, zero, zero, zero, one, zero, zero],
        &[4, 2],
    )])
    .unwrap()
}

#[test]
fn test_a_section_that_inverts_its_channel_is_admitted() {
    let a = TypeAlignment::new(vec![(vec![0], vec![0], h(), h())]).unwrap();
    assert_eq!(a.entries().len(), 1);
    assert_eq!(a.entries()[0].high(), &[0]);
    assert_eq!(a.entries()[0].tau().d_in(), 2);
    let code_like =
        TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap();
    assert_eq!(code_like.entries()[0].low(), &[0, 1]);
    assert_eq!(code_like.low_for(&[0]).unwrap(), vec![0, 1]);
}

#[test]
fn test_a_section_that_does_not_invert_is_refused_with_the_residual() {
    let err = TypeAlignment::new(vec![(vec![0], vec![0], h(), id())]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::SectionNotInverse(msg) => {
            assert!(msg.contains("2.828427124746"), "√8, {msg}")
        }
        other => panic!("{other:?}"),
    }
    let mismatch = TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), id())]).unwrap_err();
    assert!(matches!(mismatch.0, QuantumErrorEnum::DimensionMismatch(_)));
    let twice = TypeAlignment::new(vec![
        (vec![0], vec![0], id(), id()),
        (vec![0], vec![1], id(), id()),
    ])
    .unwrap_err();
    assert!(
        matches!(twice.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("high-level wire 0"))
    );
    let twice_low = TypeAlignment::new(vec![
        (vec![0], vec![0], id(), id()),
        (vec![1], vec![0], id(), id()),
    ])
    .unwrap_err();
    assert!(
        matches!(twice_low.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("low-level wire 0"))
    );
}

#[test]
fn test_tau_is_assembled_in_ascending_wire_order() {
    let caps = NumericCaps::default();
    // Entries given out of order: high 1 ↔ low 1 with H, high 0 ↔ low 0 with the identity.
    let a = TypeAlignment::new(vec![
        (vec![1], vec![1], h(), h()),
        (vec![0], vec![0], id(), id()),
    ])
    .unwrap();
    let tau = a.tau_for(&[0, 1], &caps).unwrap();
    let expect = id().tensor(&h(), &caps).unwrap();
    assert!(tau.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15);
    let section = a.section_for(&[1, 0], &caps).unwrap();
    assert!(
        section.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15,
        "H is its own section here"
    );
    // Crossed wire order: high 0 ↔ low 1, high 1 ↔ low 0 both with H; τ on (0,1) must swap legs.
    let crossed = TypeAlignment::new(vec![
        (vec![0], vec![1], h(), h()),
        (vec![1], vec![0], id(), id()),
    ])
    .unwrap();
    let tau = crossed.tau_for(&[0, 1], &caps).unwrap();
    // Low legs ascending (0, 1) carry (id, H) but land on high (1, 0): the morphism is swap ∘ (id ⊗ H).
    let swap =
        QcMorphism::from_channel(&deep_causality_quantum::swap_channel::<f64>(2).unwrap()).unwrap();
    let expect = id()
        .tensor(&h(), &caps)
        .unwrap()
        .then(&swap, &caps)
        .unwrap();
    assert!(tau.frobenius_distance(&expect, &caps).unwrap().0 < 1e-15);
    // A partial type and an unaligned wire are refused; the empty type is the trivial morphism.
    let code_like =
        TypeAlignment::new(vec![(vec![0, 1], vec![0, 1, 2], id_on(4).0, id_on(4).1)]).unwrap();
    assert!(
        matches!(code_like.tau_for(&[0], &caps).unwrap_err().0, QuantumErrorEnum::CalculationError(ref m) if m.contains("as a whole"))
    );
    assert!(
        matches!(a.tau_for(&[7], &caps).unwrap_err().0, QuantumErrorEnum::CalculationError(ref m) if m.contains("not aligned"))
    );
    assert_eq!(a.tau_for(&[], &caps).unwrap().d_in(), 1);
}

/// An identity-like pair `8 → 4` and `4 → 8`: trace out one of three qubits, prepare it back.
fn id_on(_: usize) -> (QcMorphism<f64>, QcMorphism<f64>) {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    // τ: 8 → 4 traces the last qubit; E: 4 → 8 prepares it in |0⟩.
    let mut k0 = vec![zero; 4 * 8];
    let mut k1 = vec![zero; 4 * 8];
    let mut e = vec![zero; 8 * 4];
    for i in 0..4 {
        k0[i * 8 + 2 * i] = one;
        k1[i * 8 + 2 * i + 1] = one;
        e[(2 * i) * 4 + i] = one;
    }
    (
        QcMorphism::from_kraus(&[
            CausalTensor::from_slice(&k0, &[4, 8]),
            CausalTensor::from_slice(&k1, &[4, 8]),
        ])
        .unwrap(),
        QcMorphism::from_kraus(&[CausalTensor::from_slice(&e, &[8, 4])]).unwrap(),
    )
}

#[test]
fn test_extension_follows_a_renaming_as_a_whole() {
    let a = TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap();
    let open = Query::Open(vec![0]);
    let ext = a.extended(&[(0, 5)], &[(0, 7), (1, 8)], &open).unwrap();
    assert_eq!(ext.entries().len(), 2);
    assert_eq!(ext.entries()[1].high(), &[5]);
    assert_eq!(ext.entries()[1].low(), &[7, 8]);
    let partial = a.extended(&[(0, 5)], &[(0, 7)], &open).unwrap_err();
    assert!(
        matches!(partial.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("only in part"))
    );
    let untouched = a.extended(&[(3, 4)], &[], &open).unwrap();
    assert_eq!(untouched.entries().len(), 1);
    // A query that renames nothing leaves the alignment as it is.
    let io = a
        .extended(&[(0, 5)], &[(0, 7), (1, 8)], &Query::Io)
        .unwrap();
    assert_eq!(io.entries().len(), 1);
}

/// Sided entries: the input side aligns wire 0 by the identity and the output side aligns the same
/// high-level wire with the two low-level wires through the trace. Each side sees only its entries;
/// the same wire on the same side twice is refused.
#[test]
fn test_sided_entries_apply_to_their_side_only() {
    let caps = NumericCaps::default();
    let a = TypeAlignment::new_sided(vec![
        (AlignmentSide::Input, (vec![0], vec![0], id(), id())),
        (
            AlignmentSide::Output,
            (vec![0], vec![0, 1], trace_b(), prepare_b()),
        ),
    ])
    .unwrap();
    assert_eq!(a.entries()[0].side(), AlignmentSide::Input);
    assert_eq!(a.entries()[1].side(), AlignmentSide::Output);
    assert_eq!(a.low_for_side(&[0], AlignmentSide::Input).unwrap(), vec![0]);
    assert_eq!(
        a.low_for_side(&[0], AlignmentSide::Output).unwrap(),
        vec![0, 1]
    );
    assert_eq!(
        a.tau_for_side(&[0], AlignmentSide::Input, &caps)
            .unwrap()
            .d_in(),
        2
    );
    assert_eq!(
        a.tau_for_side(&[0], AlignmentSide::Output, &caps)
            .unwrap()
            .d_in(),
        4
    );
    assert_eq!(
        a.section_for_side(&[0], AlignmentSide::Output, &caps)
            .unwrap()
            .d_out(),
        4
    );

    let same_side = TypeAlignment::new_sided(vec![
        (AlignmentSide::Input, (vec![0], vec![0], id(), id())),
        (AlignmentSide::Input, (vec![0], vec![1], id(), id())),
    ])
    .unwrap_err();
    assert!(
        matches!(same_side.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("on one side"))
    );
    let any_overlaps_input = TypeAlignment::new_sided(vec![
        (AlignmentSide::Any, (vec![0], vec![0], id(), id())),
        (AlignmentSide::Input, (vec![1], vec![0], id(), id())),
    ])
    .unwrap_err();
    assert!(
        matches!(any_overlaps_input.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("low-level wire 0"))
    );
    // An unsided alignment answers every side.
    let plain = TypeAlignment::new(vec![(vec![0], vec![0, 1], trace_b(), prepare_b())]).unwrap();
    assert_eq!(plain.entries()[0].side(), AlignmentSide::Any);
    assert_eq!(
        plain.low_for_side(&[0], AlignmentSide::Input).unwrap(),
        vec![0, 1]
    );
    assert_eq!(
        plain.low_for_side(&[0], AlignmentSide::Output).unwrap(),
        vec![0, 1]
    );
}

/// Extension along an opening: a fresh input carries the output type of the opened mechanism, so
/// the input-side entry is not copied and the output-side entry is copied on both sides.
#[test]
fn test_extension_copies_output_entries_to_fresh_wires_on_both_sides() {
    let a = TypeAlignment::new_sided(vec![
        (AlignmentSide::Input, (vec![0], vec![0], id(), id())),
        (
            AlignmentSide::Output,
            (vec![0], vec![0, 1], trace_b(), prepare_b()),
        ),
    ])
    .unwrap();
    let e = a
        .extended(&[(0, 1)], &[(0, 2), (1, 3)], &Query::Open(vec![0]))
        .unwrap();
    assert_eq!(e.entries().len(), 3);
    let fresh = &e.entries()[2];
    assert_eq!(fresh.side(), AlignmentSide::Any);
    assert_eq!(fresh.high(), &[1]);
    assert_eq!(fresh.low(), &[2, 3]);
    assert_eq!(
        e.low_for_side(&[1], AlignmentSide::Input).unwrap(),
        vec![2, 3]
    );
    assert_eq!(
        e.low_for_side(&[1], AlignmentSide::Output).unwrap(),
        vec![2, 3]
    );
    // The original input entry still answers the original wire and only that.
    assert_eq!(e.low_for_side(&[0], AlignmentSide::Input).unwrap(), vec![0]);
    assert!(e.low_for_side(&[1, 0], AlignmentSide::Input).is_ok());
}

/// A wire listed twice within one entry names no type; both sides are checked.
#[test]
fn test_a_wire_repeated_within_an_entry_is_refused() {
    let high_twice =
        TypeAlignment::new(vec![(vec![0, 0], vec![0, 1], trace_b(), prepare_b())]).unwrap_err();
    assert!(
        matches!(high_twice.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("high-level wire 0")),
        "{high_twice:?}"
    );
    let low_twice =
        TypeAlignment::new(vec![(vec![0], vec![1, 1], trace_b(), prepare_b())]).unwrap_err();
    assert!(
        matches!(low_twice.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("low-level wire 1")),
        "{low_twice:?}"
    );
}

/// `Tr` on one qubit as a channel `2 → 1`, Kraus operators `⟨0|` and `⟨1|`.
fn trace_one() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[
        CausalTensor::from_slice(&[one, zero], &[1, 2]),
        CausalTensor::from_slice(&[zero, one], &[1, 2]),
    ])
    .unwrap()
}

/// `|0⟩` on one qubit as a channel `1 → 2`.
fn prepare_one() -> QcMorphism<f64> {
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    QcMorphism::from_kraus(&[CausalTensor::from_slice(&[one, zero], &[2, 1])]).unwrap()
}

/// Entries whose low-level wires interleave, high 0 ↔ low (0, 2) through `Tr_B` and
/// high 1 ↔ low 1 through `H`: the assembled `τ` runs from low legs (0, 1, 2) ascending, so it is
/// `id ⊗ H ⊗ Tr`, and its section is `id ⊗ H ⊗ |0⟩`; the whole-entry order (0, 2, 1) would put
/// the trace on the middle leg.
#[test]
fn test_interleaved_entries_are_assembled_per_wire_leg() {
    let caps = NumericCaps::default();
    let a = TypeAlignment::new(vec![
        (vec![0], vec![0, 2], trace_b(), prepare_b()),
        (vec![1], vec![1], h(), h()),
    ])
    .unwrap();
    assert_eq!(a.low_for(&[0, 1]).unwrap(), vec![0, 1, 2]);
    let tau = a.tau_for(&[0, 1], &caps).unwrap();
    assert_eq!((tau.d_in(), tau.d_out()), (8, 4));
    let expect = id()
        .tensor(&h(), &caps)
        .unwrap()
        .tensor(&trace_one(), &caps)
        .unwrap();
    let wrong = id()
        .tensor(&trace_one(), &caps)
        .unwrap()
        .tensor(&h(), &caps)
        .unwrap();
    assert!(tau.frobenius_distance(&wrong, &caps).unwrap().0 > 1.0);
    assert!(
        tau.frobenius_distance(&expect, &caps).unwrap().0 < 1e-12,
        "{}",
        tau.frobenius_distance(&expect, &caps).unwrap().0
    );
    let section = a.section_for(&[0, 1], &caps).unwrap();
    let expect_section = id()
        .tensor(&h(), &caps)
        .unwrap()
        .tensor(&prepare_one(), &caps)
        .unwrap();
    assert!(
        section
            .frobenius_distance(&expect_section, &caps)
            .unwrap()
            .0
            < 1e-12
    );
    // The same wires on the high side: high (0, 2) ↔ low (0, 2) through H ⊗ H and high 1 ↔ low 1
    // through the identity; the high legs come out ascending as well.
    let hh = h().tensor(&h(), &caps).unwrap();
    let b = TypeAlignment::new(vec![
        (vec![0, 2], vec![0, 2], hh.clone(), hh),
        (vec![1], vec![1], id(), id()),
    ])
    .unwrap();
    let tau = b.tau_for(&[0, 1, 2], &caps).unwrap();
    let expect = h()
        .tensor(&id(), &caps)
        .unwrap()
        .tensor(&h(), &caps)
        .unwrap();
    assert!(tau.frobenius_distance(&expect, &caps).unwrap().0 < 1e-12);
}

/// An interleaving entry whose dimension does not split into equal legs, `6 → 3` on two wires,
/// is refused when it must be split and answered when it need not be.
#[test]
fn test_an_interleaving_entry_of_unequal_leg_dimensions_is_refused() {
    let caps = NumericCaps::default();
    let zero = C::new(0.0, 0.0);
    let one = C::new(1.0, 0.0);
    // τ: 6 → 3 traces a qubit beside a qutrit; E: 3 → 6 prepares it in |0⟩.
    let mut k0 = vec![zero; 3 * 6];
    let mut k1 = vec![zero; 3 * 6];
    let mut e = vec![zero; 6 * 3];
    for i in 0..3 {
        k0[i * 6 + 2 * i] = one;
        k1[i * 6 + 2 * i + 1] = one;
        e[(2 * i) * 3 + i] = one;
    }
    let tau = QcMorphism::from_kraus(&[
        CausalTensor::from_slice(&k0, &[3, 6]),
        CausalTensor::from_slice(&k1, &[3, 6]),
    ])
    .unwrap();
    let section = QcMorphism::from_kraus(&[CausalTensor::from_slice(&e, &[6, 3])]).unwrap();
    let a = TypeAlignment::new(vec![
        (vec![0], vec![0, 2], tau, section),
        (vec![1], vec![1], id(), id()),
    ])
    .unwrap();
    assert_eq!(a.tau_for(&[0], &caps).unwrap().d_in(), 6);
    let err = a.tau_for(&[0, 1], &caps).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("interleaves")),
        "{err:?}"
    );
}

/// Extension along an interchange: a copy of a model input carries the input type, so the
/// input-side entry is copied with its side and the output-side entry is not.
#[test]
fn test_extension_copies_input_entries_to_the_copies_of_an_interchange() {
    let a = TypeAlignment::new_sided(vec![
        (AlignmentSide::Input, (vec![0], vec![0], id(), id())),
        (
            AlignmentSide::Output,
            (vec![0], vec![0, 1], trace_b(), prepare_b()),
        ),
    ])
    .unwrap();
    let e = a
        .extended(&[(0, 1)], &[(0, 2)], &Query::Inc(vec![vec![0]]))
        .unwrap();
    assert_eq!(e.entries().len(), 3);
    let copy = &e.entries()[2];
    assert_eq!(copy.side(), AlignmentSide::Input);
    assert_eq!(copy.high(), &[1]);
    assert_eq!(copy.low(), &[2]);
    assert_eq!(
        e.low_for_side(&[0, 1], AlignmentSide::Input).unwrap(),
        vec![0, 2]
    );
    assert!(e.low_for_side(&[1], AlignmentSide::Output).is_err());
    // The same renaming read as an opening is refused: the output entry is renamed in part.
    let err = a
        .extended(&[(0, 1)], &[(0, 2)], &Query::Open(vec![0]))
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("only in part"))
    );
}
