/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The three query rewirings on a two-rotation chain whose answers have closed forms.
//!
//! `R_y(θ₁)` then `R_y(θ₂)` on `|0⟩` reads `|1⟩` with probability `sin²((θ₁ + θ₂)/2)`; with
//! `θ₁ = 0.7`, `θ₂ = 0.9` that is `sin²(0.8)`, and on `|1⟩` it is `cos²(0.8)`. Opening the second
//! node leaves the identity on its fresh input; opening the first leaves `R_y(0.9)` on it; an
//! interchange on the first node feeds the copy's state in, so the main input is discarded and the
//! answer depends on the copy's input alone. Corner cases: (A) opening every node, (B) an
//! interchange on a model with no inputs, (E) a node out of range, (K) an interchange on a chain.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, Channel, CircuitBox, CircuitModel, NumericCaps, QcMorphism, QuantumErrorEnum,
    QubitOperator, WireType, apply_kraus, swap_channel,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ry(wire: usize, theta: f64) -> CircuitBox<f64> {
    CircuitBox::Channel {
        wires: vec![wire],
        channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, theta).unwrap()).unwrap(),
    }
}

fn chain(with_input: bool) -> CircuitModel<f64> {
    let inputs = if with_input { vec![0] } else { vec![] };
    CircuitModel::ungrouped(
        vec![WireType::qubit()],
        vec![ry(0, 0.7), ry(0, 0.9)],
        inputs,
        vec![0],
    )
    .unwrap()
}

fn caps() -> NumericCaps {
    NumericCaps::default()
}

fn block(m: &QcMorphism<f64>) -> Vec<CausalTensor<C>> {
    m.blocks().get(&(vec![], vec![])).unwrap().clone()
}

fn ket(bits: &[f64]) -> CausalTensor<C> {
    let d = bits.len();
    let mut data = vec![C::new(0.0, 0.0); d * d];
    for i in 0..d {
        for j in 0..d {
            data[i * d + j] = C::new(bits[i] * bits[j], 0.0);
        }
    }
    CausalTensor::from_slice(&data, &[d, d])
}

#[test]
fn test_opening_the_last_node_leaves_the_identity_on_the_fresh_input() {
    let m = chain(false);
    let (opened, map) = m.opened_with_map(&[1]).unwrap();
    assert_eq!(map, vec![(0, 1)]);
    assert_eq!(m.opened_inputs(&[1]), vec![1]);
    assert_eq!(opened.wires().len(), 2);
    assert_eq!(opened.boxes().len(), 1);
    assert_eq!(opened.inputs(), &[1]);
    assert_eq!(opened.outputs(), &[1]);
    let sem = opened.numeric_semantics(&caps()).unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    assert!(sem.frobenius_distance(&id, &caps()).unwrap().0 < 1e-12);
}

#[test]
fn test_opening_the_first_node_leaves_the_second_rotation() {
    let opened = chain(false).opened(&[0]).unwrap();
    assert_eq!(opened.inputs(), &[1]);
    assert_eq!(
        opened.boxes()[0].quantum_wires(),
        &[1],
        "the surviving box moved to the fresh line"
    );
    let sem = opened.numeric_semantics(&caps()).unwrap();
    let expect = QcMorphism::from_channel(
        &Channel::unitary(&QubitOperator::rotation(Axis::Y, 0.9).unwrap()).unwrap(),
    )
    .unwrap();
    assert!(sem.frobenius_distance(&expect, &caps()).unwrap().0 < 1e-12);
    // Opening every node leaves a wire-only model: identity on the fresh input.
    let all = chain(false).opened(&[0, 1]).unwrap();
    assert!(all.boxes().is_empty());
    assert_eq!(all.nodes().len(), 0);
    let id = QcMorphism::<f64>::identity(2).unwrap();
    assert!(
        all.numeric_semantics(&caps())
            .unwrap()
            .frobenius_distance(&id, &caps())
            .unwrap()
            .0
            < 1e-12
    );
}

#[test]
fn test_opening_a_measurement_node_drops_its_classical_output() {
    let m = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![
            ry(0, 0.7),
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 1,
            },
        ],
        vec![],
        vec![1],
    )
    .unwrap();
    let opened = m.opened(&[1]).unwrap();
    assert_eq!(
        opened.outputs(),
        &[] as &[usize],
        "the outcome nobody writes is no output"
    );
    assert!(matches!(
        m.opened(&[7]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_observing_the_wire_gives_the_closed_form_probability() {
    let observed = chain(false).observed(&[0]).unwrap();
    assert_eq!(observed.outputs(), &[1]);
    assert_eq!(observed.wires()[1], WireType::Classical { outcomes: 2 });
    let (blocks, _) = observed
        .numeric_semantics(&caps())
        .unwrap()
        .choi_blocks(&caps())
        .unwrap();
    let p1 = blocks.get(&(vec![], vec![1])).unwrap().as_slice()[0].re;
    assert!((p1 - 0.8f64.sin().powi(2)).abs() < 1e-12);
    assert!(matches!(
        chain(false).observed(&[3]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_interchange_feeds_the_copy_and_discards_the_main_input() {
    let m = chain(true);
    let (inc, map) = m.interchanged_with_map(&[vec![0]]).unwrap();
    assert_eq!(map, vec![(0, 1)]);
    assert_eq!(inc.inputs(), &[0, 1], "the main input and its copy");
    let sem = inc.numeric_semantics(&caps()).unwrap();
    assert_eq!((sem.d_in(), sem.d_out()), (4, 2));
    let family = block(&sem);
    // Main |1⟩, copy |0⟩: the answer is the chain on |0⟩.
    let rho = ket(&[0.0, 1.0]).kronecker(&ket(&[1.0, 0.0])).unwrap();
    let out = apply_kraus(&family, &rho).unwrap();
    assert!(
        (out.as_slice()[3].re - 0.8f64.sin().powi(2)).abs() < 1e-12,
        "{}",
        out.as_slice()[3].re
    );
    // Main |0⟩, copy |1⟩: the chain on |1⟩.
    let rho = ket(&[1.0, 0.0]).kronecker(&ket(&[0.0, 1.0])).unwrap();
    let out = apply_kraus(&family, &rho).unwrap();
    assert!((out.as_slice()[3].re - 0.8f64.cos().powi(2)).abs() < 1e-12);
    // On a model with no inputs the interchange equals the plain semantics.
    let m0 = chain(false);
    let inc0 = m0
        .interchanged(&[vec![0]])
        .unwrap()
        .numeric_semantics(&caps())
        .unwrap();
    let io0 = m0.numeric_semantics(&caps()).unwrap();
    assert!(inc0.frobenius_distance(&io0, &caps()).unwrap().0 < 1e-12);
}

#[test]
fn test_interchange_refuses_a_chain_and_bad_nodes() {
    let m = chain(true);
    let err = m.interchanged(&[vec![0, 1]]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::NotParallelisable(ref msg) if msg.contains("0 → 1")));
    assert!(matches!(
        m.interchanged(&[vec![4]]).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // The swap channel exchanges the two systems.
    let swap = QcMorphism::from_channel(&swap_channel::<f64>(2).unwrap()).unwrap();
    let rho = ket(&[0.0, 1.0]).kronecker(&ket(&[1.0, 0.0])).unwrap();
    let out = apply_kraus(&block(&swap), &rho).unwrap();
    assert!(
        (out.as_slice()[4 + 1].re - 1.0).abs() < 1e-12,
        "|10⟩ ↦ |01⟩"
    );
}

/// A node whose boxes are grouped so that two of them act on one wire is interchanged with one
/// swap on that wire, placed after the node's last box. The copy of the chain runs `R_y(0.7)` then
/// `R_y(0.9)` on the copy's input, so the main output reads `|1⟩` with `sin²(0.8)` on a copy input
/// of `|0⟩` and `cos²(0.8)` on `|1⟩`; the main input is discarded. Two swaps would cancel and hand
/// the untouched main input back.
#[test]
fn test_interchange_of_a_node_with_two_boxes_on_one_wire_swaps_once() {
    let m = CircuitModel::<f64>::new(
        vec![WireType::qubit()],
        vec![ry(0, 0.7), ry(0, 0.9)],
        vec![vec![0, 1]],
        vec![0],
        vec![0],
    )
    .unwrap();
    let inc = m.interchanged(&[vec![0]]).unwrap();
    let swaps = inc
        .boxes()
        .iter()
        .filter(|b| b.quantum_wires().len() == 2)
        .count();
    assert_eq!(swaps, 1, "one swap per wire of the node: {:?}", inc.boxes());
    assert_eq!(inc.inputs(), &[0, 1]);
    let sem = inc.numeric_semantics(&caps()).unwrap();
    assert_eq!((sem.d_in(), sem.d_out()), (4, 2));
    let family = block(&sem);
    let rho = ket(&[0.0, 1.0]).kronecker(&ket(&[1.0, 0.0])).unwrap();
    let out = apply_kraus(&family, &rho).unwrap();
    assert!(
        (out.as_slice()[3].re - 0.8f64.sin().powi(2)).abs() < 1e-12,
        "main |1⟩, copy |0⟩: {}",
        out.as_slice()[3].re
    );
    let rho = ket(&[1.0, 0.0]).kronecker(&ket(&[0.0, 1.0])).unwrap();
    let out = apply_kraus(&family, &rho).unwrap();
    assert!(
        (out.as_slice()[3].re - 0.8f64.cos().powi(2)).abs() < 1e-12,
        "main |0⟩, copy |1⟩: {}",
        out.as_slice()[3].re
    );
}

/// A node that appears in two interchange sets is refused, as `QuerySignature::new` refuses it;
/// the method is public and must hold the same contract on its own.
#[test]
fn test_interchange_refuses_a_node_shared_between_sets() {
    let m = chain(true);
    let err = m.interchanged(&[vec![0], vec![0]]).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref msg) if msg.contains("node 0")),
        "{err}"
    );
    // Disjoint singletons on a two-wire model are fine.
    let two = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![ry(0, 0.7), ry(1, 0.9)],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    assert!(two.interchanged(&[vec![0], vec![1]]).is_ok());
}

/// An interchange set holding a node that writes a classical wire is refused: the copy would
/// write the outcome on its private wire while the main wire is never written. The measurement
/// outcome here is no declared output, so nothing else would have caught it.
#[test]
fn test_interchange_refuses_a_set_holding_a_classical_writer() {
    let m = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![
            ry(0, 0.7),
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 1,
            },
        ],
        vec![0],
        vec![0],
    )
    .unwrap();
    let err = m.interchanged(&[vec![1]]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => assert!(
            msg.contains("node 1") && msg.contains("box 1") && msg.contains("classical wire 1"),
            "{msg}"
        ),
        other => panic!("{other:?}"),
    }
    // The quantum-only node of the same model is still interchangeable.
    assert!(m.interchanged(&[vec![0]]).is_ok());
}

/// Observing a quantum wire that is not a declared output is refused: `Observe(O)` measures
/// output wires, and a traced wire has no place among the outputs for its outcome to take.
#[test]
fn test_observing_a_wire_that_is_not_an_output_is_refused() {
    let m = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![ry(0, 0.7), ry(1, 0.9)],
        vec![0, 1],
        vec![0],
    )
    .unwrap();
    let err = m.observed(&[1]).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref msg) if msg.contains("wire 1") && msg.contains("output")),
        "{err}"
    );
    let ok = m.observed(&[0]).unwrap();
    assert_eq!(ok.outputs(), &[2]);
}
