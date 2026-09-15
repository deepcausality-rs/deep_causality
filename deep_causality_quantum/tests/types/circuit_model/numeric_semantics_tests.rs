/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Kraus-level evaluation of a circuit model.
//!
//! Provenance of the literals: the Choi of a diagonal unitary `U = diag(u)` is
//! `J[(i,i),(k,k)] = u_i ū_k` and zero elsewhere (the crate's convention
//! `J[(i,j),(k,l)] = U[j,i] conj(U[l,k])` with `U` diagonal), so for `CZ = diag(1, 1, 1, −1)` the
//! entry at `((3,3),(0,0))` is `−1` and at `((0,1),(0,1))` zero; a perfect classical channel has
//! probability one on `y = x`; the amplitude-damping channel with `γ = 0.3` has Kraus operators
//! `[[1, 0], [0, √0.7]]` and `[[0, √0.3], [0, 0]]` (Nielsen & Chuang §8.3.5). Corner cases: (A) a
//! circuit with no boxes, (B) one box, (D) an 18-qubit request whose working storage, `2^18` state
//! vectors of `2^18` amplitudes, would be `2^36` entries, refused with that count, (E) an encoder reading an unwritten wire, (F) the operator cap on a
//! chain of measurements, (I) the mask length mismatch.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Channel, CircuitBox, CircuitModel, GateOp, NumericCaps, QcMorphism, QuantumErrorEnum, WireType,
    apply_kraus, choi_from_kraus,
};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn re(v: &[f64], shape: &[usize]) -> CausalTensor<C> {
    CausalTensor::from_slice(
        &v.iter().map(|&a| C::new(a, 0.0)).collect::<Vec<_>>(),
        shape,
    )
}

fn close(a: &CausalTensor<C>, b: &CausalTensor<C>) -> bool {
    a.as_slice()
        .iter()
        .zip(b.as_slice())
        .all(|(x, y)| (x.re - y.re).abs() < 1e-12 && (x.im - y.im).abs() < 1e-12)
}

fn amplitude_damping() -> Vec<CausalTensor<C>> {
    vec![
        re(&[1.0, 0.0, 0.0, 0.7f64.sqrt()], &[2, 2]),
        re(&[0.0, 0.3f64.sqrt(), 0.0, 0.0], &[2, 2]),
    ]
}

fn single_block(m: &QcMorphism<f64>) -> Vec<CausalTensor<C>> {
    m.blocks().get(&(vec![], vec![])).unwrap().clone()
}

#[test]
fn test_channel_box_agrees_with_apply_kraus_on_one_qubit() {
    let kraus = amplitude_damping();
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![CircuitBox::Channel {
            wires: vec![0],
            channel: Channel::from_kraus(&kraus).unwrap(),
        }],
        vec![0],
        vec![0],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    let family = single_block(&m);
    let rho = re(&[0.25, 0.25, 0.25, 0.75], &[2, 2]);
    let direct = apply_kraus(&kraus, &rho).unwrap();
    let through = apply_kraus(&family, &rho).unwrap();
    assert!(close(&direct, &through));
    let (blocks, entries) = m.choi_blocks(&NumericCaps::default()).unwrap();
    assert_eq!(entries, 16);
    assert!(close(
        blocks.get(&(vec![], vec![])).unwrap(),
        &choi_from_kraus(&kraus).unwrap()
    ));
}

#[test]
fn test_cz_choi_matches_the_hand_computed_entries() {
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Unitary {
            wires: vec![0, 1],
            program: vec![GateOp::Cz {
                control: 0,
                target: 1,
            }],
        }],
        vec![0, 1],
        vec![0, 1],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    let (blocks, entries) = m.choi_blocks(&NumericCaps::default()).unwrap();
    assert_eq!(entries, 256);
    let j = blocks.get(&(vec![], vec![])).unwrap();
    let d = 16;
    let at = |i: usize, jj: usize, k: usize, l: usize| j.as_slice()[(i * 4 + jj) * d + (k * 4 + l)];
    assert!((at(3, 3, 0, 0).re + 1.0).abs() < 1e-12);
    assert!((at(0, 0, 0, 0).re - 1.0).abs() < 1e-12);
    assert!((at(3, 3, 3, 3).re - 1.0).abs() < 1e-12);
    assert!((at(2, 2, 3, 3).re + 1.0).abs() < 1e-12);
    assert!(at(0, 1, 0, 1).re.abs() < 1e-12 && at(0, 1, 0, 1).im.abs() < 1e-12);
    // The single Kraus operator is the CZ matrix itself.
    let family = single_block(&m);
    assert_eq!(family.len(), 1);
    assert!(close(
        &family[0],
        &re(
            &[
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, -1.0
            ],
            &[4, 4]
        )
    ));
}

#[test]
fn test_gate_on_a_subset_of_a_wider_register_embeds_by_position() {
    // X on wire 2 of three; input basis |000⟩ maps to |001⟩ (wire 2 least significant).
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Unitary {
            wires: vec![2],
            program: vec![GateOp::X(0)],
        }],
        vec![0, 1, 2],
        vec![0, 1, 2],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    let u = &single_block(&m)[0];
    let s = u.as_slice();
    assert!((s[8].re - 1.0).abs() < 1e-12, "column 0 lands on row 1");
    assert!((s[6 * 8 + 7].re - 1.0).abs() < 1e-12, "|111⟩ ↦ |110⟩");
    // A wire listed out of order in the box: Cnot with control wire 2, target wire 0.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Unitary {
            wires: vec![2, 0],
            program: vec![GateOp::Cnot {
                control: 0,
                target: 1,
            }],
        }],
        vec![0, 1, 2],
        vec![0, 1, 2],
    )
    .unwrap();
    let u = &single_block(&model.numeric_semantics(&NumericCaps::default()).unwrap())[0];
    // |001⟩ (wire 2 set) flips wire 0: ↦ |101⟩ = index 5.
    assert!((u.as_slice()[5 * 8 + 1].re - 1.0).abs() < 1e-12);
    assert!(
        (u.as_slice()[2 * 8 + 2].re - 1.0).abs() < 1e-12,
        "|010⟩ untouched"
    );
}

#[test]
fn test_encoder_and_measurement_form_a_perfect_classical_channel() {
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit(), WireType::bit()],
        vec![
            CircuitBox::Encoder {
                input: 1,
                outputs: vec![0],
                states: vec![re(&[1.0, 0.0], &[2]), re(&[0.0, 1.0], &[2])],
            },
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 2,
            },
        ],
        vec![],
        vec![2],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    assert_eq!(m.classical_in(), &[2]);
    assert_eq!(m.classical_out(), &[2]);
    assert_eq!((m.d_in(), m.d_out()), (1, 1));
    let (blocks, _) = m.choi_blocks(&NumericCaps::default()).unwrap();
    // An outcome the state cannot produce has no block: its operators are exactly zero and the
    // semantics drops them, so a missing block reads as probability zero.
    for x in 0..2 {
        for y in 0..2 {
            let p = blocks
                .get(&(vec![x], vec![y]))
                .map_or(0.0, |b| b.as_slice()[0].re);
            let expect = if x == y { 1.0 } else { 0.0 };
            assert!((p - expect).abs() < 1e-12, "P(y={y}|x={x}) = {p}");
        }
    }
    assert_eq!(
        m.blocks().len(),
        2,
        "the two impossible outcomes carry no block"
    );
    // With a Hadamard between, every probability is one half.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit(), WireType::bit()],
        vec![
            CircuitBox::Encoder {
                input: 1,
                outputs: vec![0],
                states: vec![re(&[1.0, 0.0], &[2]), re(&[0.0, 1.0], &[2])],
            },
            CircuitBox::Unitary {
                wires: vec![0],
                program: vec![GateOp::H(0)],
            },
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 2,
            },
        ],
        vec![],
        vec![2],
    )
    .unwrap();
    let (blocks, _) = model
        .numeric_semantics(&NumericCaps::default())
        .unwrap()
        .choi_blocks(&NumericCaps::default())
        .unwrap();
    for (_, j) in blocks {
        assert!((j.as_slice()[0].re - 0.5).abs() < 1e-12);
    }
}

#[test]
fn test_instrument_branches_by_outcome() {
    let p0 = re(&[1.0, 0.0, 0.0, 0.0], &[2, 2]);
    let p1 = re(&[0.0, 0.0, 0.0, 1.0], &[2, 2]);
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Instrument {
            wires: vec![0],
            outcome: 1,
            kraus: vec![vec![p0.clone()], vec![p1.clone()]],
        }],
        vec![0],
        vec![0, 1],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    let (blocks, _) = m.choi_blocks(&NumericCaps::default()).unwrap();
    assert!(close(
        blocks.get(&(vec![], vec![0])).unwrap(),
        &choi_from_kraus(&[p0]).unwrap()
    ));
    assert!(close(
        blocks.get(&(vec![], vec![1])).unwrap(),
        &choi_from_kraus(&[p1]).unwrap()
    ));
}

#[test]
fn test_no_boxes_is_the_identity_and_traced_legs_are_summed() {
    let model =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0]).unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    let id = QcMorphism::<f64>::identity(2).unwrap();
    assert!(
        m.frobenius_distance(&id, &NumericCaps::default())
            .unwrap()
            .0
            < 1e-15
    );
    // Two qubits in, one kept: the discarded leg is traced, giving `Tr_B` as a channel 4 → 2.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![],
        vec![0, 1],
        vec![0],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    assert_eq!((m.d_in(), m.d_out()), (4, 2));
    let family = single_block(&m);
    assert_eq!(family.len(), 2, "one Kraus operator per traced basis state");
    let rho = re(
        &[
            0.5, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.5,
        ],
        &[4, 4],
    );
    let out = apply_kraus(&family, &rho).unwrap();
    // Tr_B of the Bell-like state above is I/2 on the kept qubit.
    assert!(close(&out, &re(&[0.5, 0.0, 0.0, 0.5], &[2, 2])));
}

#[test]
fn test_eighteen_qubits_are_refused_with_the_exact_entry_count() {
    // The composite Choi with two logical qubits would be 2^40; the working storage alone is 2^36.
    let wires = vec![WireType::qubit(); 18];
    let inputs: Vec<usize> = (0..18).collect();
    let model = CircuitModel::<f64>::ungrouped(
        wires,
        vec![CircuitBox::Unitary {
            wires: inputs.clone(),
            program: vec![],
        }],
        inputs,
        vec![0, 1],
    )
    .unwrap();
    let err = model
        .numeric_semantics(&NumericCaps::default())
        .unwrap_err();
    assert!(
        matches!(
            err.0,
            QuantumErrorEnum::NaturalityDimensionExceeded {
                n: 18,
                k: 2,
                entries,
                cap
            } if entries == 1u64 << 36 && cap == 1u64 << 24
        ),
        "{err}"
    );
}

#[test]
fn test_operator_cap_stops_a_chain_of_measurements() {
    // Thirteen measured qubits give 2^13 branches, above the default 2^12.
    let n = 13;
    let mut wires = vec![WireType::qubit(); n];
    wires.extend(vec![WireType::bit(); n]);
    let boxes: Vec<CircuitBox<f64>> = (0..n)
        .map(|q| CircuitBox::Measurement {
            wires: vec![q],
            outcome: n + q,
        })
        .collect();
    let outputs: Vec<usize> = (n..2 * n).collect();
    let model = CircuitModel::<f64>::ungrouped(wires, boxes, vec![], outputs).unwrap();
    // The entry cap would also fire on 2^13 blocks of one entry each; loosen it to see the operator cap.
    let caps = NumericCaps {
        max_entries: 1 << 40,
        max_operators: 1 << 12,
    };
    let err = model.numeric_semantics(&caps).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::KrausFamilyExceeded { operators, cap } if operators > cap && cap == 1 << 12),
        "{err}"
    );
}

#[test]
fn test_encoder_reading_an_unwritten_wire_and_a_bad_mask_are_errors() {
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Encoder {
            input: 1,
            outputs: vec![0],
            states: vec![re(&[1.0, 0.0], &[2]), re(&[0.0, 1.0], &[2])],
        }],
        vec![],
        vec![0],
    )
    .unwrap();
    // The encoder's input is a classical input here, so evaluation enumerates it: two blocks.
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    assert_eq!(m.blocks().len(), 2);
    let err = model
        .evaluate(&[true, true], &[], &[0], &NumericCaps::default())
        .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    // Deactivating every box leaves a classical output no box wrote.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    let err = model
        .evaluate(&[false], &[0], &[1], &NumericCaps::default())
        .unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("never written"))
    );
}

/// A Kraus box evaluates as the same map as `apply_kraus` with the same operators, and as the
/// `Channel` box built from them; the asymmetric amplitude-damping family and a state with
/// off-diagonal weight make a dropped or transposed operator visible.
#[test]
fn test_kraus_box_agrees_with_apply_kraus_on_one_qubit() {
    let kraus = amplitude_damping();
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit()],
        vec![CircuitBox::Kraus {
            wires: vec![0],
            kraus: kraus.clone(),
        }],
        vec![0],
        vec![0],
    )
    .unwrap();
    let m = model.numeric_semantics(&NumericCaps::default()).unwrap();
    let family = single_block(&m);
    assert_eq!(family.len(), 2);
    let rho = re(&[0.25, 0.25, 0.25, 0.75], &[2, 2]);
    let direct = apply_kraus(&kraus, &rho).unwrap();
    let through = apply_kraus(&family, &rho).unwrap();
    assert!(close(&direct, &through));
    // Hand values: the damped state is [[0.25 + 0.3·0.75, 0.25·√0.7], [0.25·√0.7, 0.7·0.75]].
    let expect = re(
        &[0.475, 0.25 * 0.7f64.sqrt(), 0.25 * 0.7f64.sqrt(), 0.525],
        &[2, 2],
    );
    assert!(close(&through, &expect));
}

fn basis_encoder() -> CircuitBox<f64> {
    CircuitBox::Encoder {
        input: 1,
        outputs: vec![0],
        states: vec![re(&[1.0, 0.0], &[2]), re(&[0.0, 1.0], &[2])],
    }
}

#[test]
fn test_entry_cap_counts_every_classical_input_branch() {
    // An encoder on a two-valued classical input opens two branches before any box runs. Each
    // branch owns `d_total · d_in = 2 · 1` entries, four in all: a cap of three refuses the
    // evaluation with that count, a cap of four admits it.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![basis_encoder()],
        vec![],
        vec![0],
    )
    .unwrap();
    let three = NumericCaps {
        max_entries: 3,
        max_operators: 1 << 12,
    };
    let err = model.numeric_semantics(&three).unwrap_err();
    assert!(
        matches!(
            err.0,
            QuantumErrorEnum::NaturalityDimensionExceeded {
                n: 0,
                k: 1,
                entries: 4,
                cap: 3
            }
        ),
        "{err}"
    );
    let four = NumericCaps {
        max_entries: 4,
        max_operators: 1 << 12,
    };
    assert_eq!(model.numeric_semantics(&four).unwrap().blocks().len(), 2);
}

#[test]
fn test_entry_cap_counts_the_branches_a_measurement_opens() {
    // One qubit in, measured: two branches of `d_total · d_in = 2 · 2` entries, eight in all. A
    // cap of seven admits the initial branch and must refuse the measurement's second one.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    let seven = NumericCaps {
        max_entries: 7,
        max_operators: 1 << 12,
    };
    let err = model.numeric_semantics(&seven).unwrap_err();
    assert!(
        matches!(
            err.0,
            QuantumErrorEnum::NaturalityDimensionExceeded {
                n: 1,
                k: 0,
                entries: 8,
                cap: 7
            }
        ),
        "{err}"
    );
    let eight = NumericCaps {
        max_entries: 8,
        max_operators: 1 << 12,
    };
    assert_eq!(model.numeric_semantics(&eight).unwrap().blocks().len(), 2);
    // The same count through an instrument with two one-operator families.
    let p0 = re(&[1.0, 0.0, 0.0, 0.0], &[2, 2]);
    let p1 = re(&[0.0, 0.0, 0.0, 1.0], &[2, 2]);
    let instrument = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Instrument {
            wires: vec![0],
            outcome: 1,
            kraus: vec![vec![p0], vec![p1]],
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    assert!(matches!(
        instrument.numeric_semantics(&seven).unwrap_err().0,
        QuantumErrorEnum::NaturalityDimensionExceeded { entries: 8, .. }
    ));
    assert!(instrument.numeric_semantics(&eight).is_ok());
}

#[test]
fn test_operator_cap_counts_the_traced_basis_states() {
    // Two qubits in, none kept: the trace is four Kraus operators from one branch. A cap of three
    // refuses the family with that count, a cap of four admits it.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![],
        vec![0, 1],
        vec![],
    )
    .unwrap();
    let three = NumericCaps {
        max_entries: 1 << 24,
        max_operators: 3,
    };
    let err = model.numeric_semantics(&three).unwrap_err();
    assert!(
        matches!(
            err.0,
            QuantumErrorEnum::KrausFamilyExceeded {
                operators: 4,
                cap: 3
            }
        ),
        "{err}"
    );
    let four = NumericCaps {
        max_entries: 1 << 24,
        max_operators: 4,
    };
    assert_eq!(model.numeric_semantics(&four).unwrap().operator_count(), 4);
    // A measurement doubles the branches after the initial check: two branches times two traced
    // basis states is four operators, above a cap of three that the one initial branch passed.
    let measured = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    let err = measured.numeric_semantics(&three).unwrap_err();
    assert!(
        matches!(
            err.0,
            QuantumErrorEnum::KrausFamilyExceeded {
                operators: 4,
                cap: 3
            }
        ),
        "{err}"
    );
    // The cap counts the four operators before they are formed; the measured wire sits in |0⟩
    // afterwards, so the traced state |1⟩ carries the zero operator, which the assembly drops:
    // two operators remain, one per outcome.
    assert_eq!(
        measured.numeric_semantics(&four).unwrap().operator_count(),
        2
    );
}
