/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Barrett–Lorenz–Oreshkov dilation against the circuit's own probabilities.
//!
//! The oracle is the Born rule computed two ways that share no code: the process-operator trace
//! `Re Tr(σ · ⊗ τ)` over the dilation, and the closed form of a two-rotation chain,
//! `p(1) = sin²((θ₁ + θ₂)/2)` for `R_y(θ₁)` then `R_y(θ₂)` on `|0⟩`. The angles `0.7` and `0.9` are
//! asymmetric and give `sin²(0.8) = 0.51464…`, which no coincidence of halves or signs reproduces.
//! Corner cases: (A) a fresh line against a declared input, (B) one node, (D) a two-wire node whose
//! parent hands it one wire, (F) a cyclic grouping, (I) a classical box.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    Axis, Channel, CircuitBox, CircuitModel, CommutatorTolerance, Factorization, GateOp,
    NumericCaps, QuantumErrorEnum, QubitOperator, WireSource, WireType, choi_from_kraus,
    markov_certificate, quantum_markov_check_report,
};
use deep_causality_tensor::{CausalTensor, Tensor};
use std::collections::BTreeMap;

type C = Complex<f64>;

fn ry(theta: f64) -> CircuitBox<f64> {
    CircuitBox::Channel {
        wires: vec![0],
        channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, theta).unwrap()).unwrap(),
    }
}

/// `R_y(0.7)` at node 0, then `R_y(0.9)` at node 1, on one qubit.
fn chain() -> CircuitModel<f64> {
    CircuitModel::ungrouped(
        vec![WireType::qubit()],
        vec![ry(0.7), ry(0.9)],
        vec![],
        vec![0],
    )
    .unwrap()
}

#[test]
fn test_chain_dilation_is_markov_with_the_leg_convention() {
    let d = chain().dilation().unwrap();
    assert_eq!(d.legs().len(), 2);
    assert_eq!(d.legs()[1].d(), 2);
    assert_eq!(
        d.supports().declared_leg_dim(0),
        Some(4),
        "d_in · d_out for a qubit node"
    );
    assert_eq!(d.supports().declared_leg_dim(1), Some(4));
    assert_eq!(d.supports().support(0), Some(&[0][..]));
    assert_eq!(
        d.supports().support(1),
        Some(&[0, 1][..]),
        "node 1's parent is node 0"
    );
    assert_eq!(d.source(0, 0), Some(WireSource::Fresh));
    assert_eq!(d.source(1, 0), Some(WireSource::Parent(0)));
    let report =
        quantum_markov_check_report(d.factors(), d.supports(), &CommutatorTolerance::new())
            .unwrap();
    assert_eq!(report.examined(), 1, "one intersecting pair");
    assert!(report.accepted());
    assert_eq!(report.factorization(), Factorization::Rederived);
    assert!(markov_certificate(&report).is_ok());
    // The root factor is |0⟩⟨0| ⊗ I on (in, out): trace 2, entry ((0,0),(0,0)) = 1, ((1,0),(1,0)) = 0.
    let rho = d.factors().get(0).unwrap();
    assert_eq!(rho.shape(), &[4, 4]);
    let s = rho.as_slice();
    assert!((s[0].re - 1.0).abs() < 1e-15 && (s[5].re - 1.0).abs() < 1e-15);
    assert!(s[10].re.abs() < 1e-15 && s[15].re.abs() < 1e-15);
}

#[test]
fn test_process_operator_born_rule_matches_the_closed_form_and_the_kraus_semantics() {
    let model = chain();
    let d = model.dilation().unwrap();
    // Replace node 1's instrument with the Choi of P₁ ∘ R_y(0.9).
    let u = QubitOperator::rotation(Axis::Y, 0.9).unwrap();
    let p1 = CausalTensor::from_slice(
        &[
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    let k = p1.matmul(u.matrix()).unwrap();
    let mut overrides = BTreeMap::new();
    overrides.insert(1usize, choi_from_kraus(&[k]).unwrap());
    let tau = d.joint_instrument(&overrides).unwrap();
    let p = d.predict(&tau).unwrap();
    let expected = (0.8f64).sin().powi(2);
    assert!(
        (p - expected).abs() < 1e-12,
        "{p} vs sin²(0.8) = {expected}"
    );
    // The default instruments give total probability one.
    let total = d
        .predict(&d.joint_instrument(&BTreeMap::new()).unwrap())
        .unwrap();
    assert!((total - 1.0).abs() < 1e-12);
    // The Kraus-level semantics with a measurement box gives the same number.
    let measured = CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![
            ry(0.7),
            ry(0.9),
            CircuitBox::Measurement {
                wires: vec![0],
                outcome: 1,
            },
        ],
        vec![],
        vec![1],
    )
    .unwrap();
    let (blocks, _) = measured
        .numeric_semantics(&NumericCaps::default())
        .unwrap()
        .choi_blocks(&NumericCaps::default())
        .unwrap();
    let p_numeric = blocks.get(&(vec![], vec![1])).unwrap().as_slice()[0].re;
    assert!((p_numeric - expected).abs() < 1e-12);
}

#[test]
fn test_two_wire_node_fed_one_wire_by_its_parent() {
    // Node 0: X on wire 0. Node 1: CNOT on wires 0 (control) and 1 (target), wire 1 fresh.
    let model = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            CircuitBox::Unitary {
                wires: vec![0],
                program: vec![GateOp::X(0)],
            },
            CircuitBox::Unitary {
                wires: vec![0, 1],
                program: vec![GateOp::Cnot {
                    control: 0,
                    target: 1,
                }],
            },
        ],
        vec![],
        vec![0, 1],
    )
    .unwrap();
    let d = model.dilation().unwrap();
    assert_eq!(d.legs()[1].wires(), &[0, 1]);
    assert_eq!(d.supports().declared_leg_dim(1), Some(16));
    assert_eq!(d.source(1, 0), Some(WireSource::Parent(0)));
    assert_eq!(d.source(1, 1), Some(WireSource::Fresh));
    let report =
        quantum_markov_check_report(d.factors(), d.supports(), &CommutatorTolerance::new())
            .unwrap();
    assert!(report.accepted() && report.examined() == 1);
    // Probability of reading |11⟩ at the end is one: X then CNOT on |00⟩.
    let p11 = CausalTensor::from_slice(
        &(0..16)
            .map(|i| {
                if i == 15 {
                    C::new(1.0, 0.0)
                } else {
                    C::new(0.0, 0.0)
                }
            })
            .collect::<Vec<_>>(),
        &[4, 4],
    );
    let cnot = deep_causality_quantum::gate_unitary::<f64>(&GateOp::Cnot {
        control: 0,
        target: 1,
    })
    .unwrap()
    .1;
    let mut overrides = BTreeMap::new();
    overrides.insert(
        1usize,
        choi_from_kraus(&[p11.matmul(&cnot).unwrap()]).unwrap(),
    );
    let p = d.predict(&d.joint_instrument(&overrides).unwrap()).unwrap();
    assert!((p - 1.0).abs() < 1e-12, "{p}");
    // And |10⟩ has probability zero.
    let p10 = CausalTensor::from_slice(
        &(0..16)
            .map(|i| {
                if i == 10 {
                    C::new(1.0, 0.0)
                } else {
                    C::new(0.0, 0.0)
                }
            })
            .collect::<Vec<_>>(),
        &[4, 4],
    );
    overrides.insert(
        1usize,
        choi_from_kraus(&[p10.matmul(&cnot).unwrap()]).unwrap(),
    );
    let p = d.predict(&d.joint_instrument(&overrides).unwrap()).unwrap();
    assert!(p.abs() < 1e-12);
}

#[test]
fn test_declared_input_is_carried_as_the_normalised_identity() {
    let model =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![ry(0.7)], vec![0], vec![0])
            .unwrap();
    let d = model.dilation().unwrap();
    assert_eq!(d.source(0, 0), Some(WireSource::Input));
    // I/2 ⊗ I on (in, out): diagonal entries 1/2, trace 2.
    let s = d.factors().get(0).unwrap().as_slice();
    assert!(
        (s[0].re - 0.5).abs() < 1e-15 && (s[15].re - 0.5).abs() < 1e-15 && s[1].re.abs() < 1e-15
    );
    // The default instrument alone integrates to one over the maximally mixed input.
    let total = d
        .predict(&d.joint_instrument(&BTreeMap::new()).unwrap())
        .unwrap();
    assert!((total - 1.0).abs() < 1e-12);
}

#[test]
fn test_glued_dilation_is_the_induced_factorization() {
    let first =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![ry(0.7)], vec![], vec![0])
            .unwrap();
    let second =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![ry(0.9)], vec![0], vec![0])
            .unwrap();
    let glued = first.glue(&second, &[(0, 0)]).unwrap();
    assert_eq!(glued.boxes().len(), 2);
    assert_eq!(glued.wires().len(), 1, "the joined wire is one line");
    assert_eq!(glued.inputs(), &[] as &[usize]);
    assert_eq!(glued.outputs(), &[0]);
    let d = glued.dilation().unwrap();
    let report =
        quantum_markov_check_report(d.factors(), d.supports(), &CommutatorTolerance::new())
            .unwrap();
    assert!(report.accepted());
    assert_eq!(report.factorization(), Factorization::Rederived);
    assert!(
        markov_certificate(&report).is_ok(),
        "no CertificateNotInherited on a constructed factorization"
    );
    // The glued model predicts what the chain predicts.
    assert_eq!(
        d.factors().len(),
        chain().dilation().unwrap().factors().len()
    );
    let h = d
        .hypothesis("glued")
        .unwrap()
        .check_markov(&CommutatorTolerance::new())
        .unwrap();
    assert!(h.certificate().unwrap().accepted());
}

#[test]
fn test_glue_refuses_bad_joins() {
    let first = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![ry(0.7)],
        vec![],
        vec![0],
    )
    .unwrap();
    let second =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![ry(0.9)], vec![0], vec![0])
            .unwrap();
    assert!(
        matches!(first.glue(&second, &[(1, 0)]).unwrap_err().0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("not a quantum output"))
    );
    let not_input =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![ry(0.9)], vec![], vec![0])
            .unwrap();
    assert!(
        matches!(first.glue(&not_input, &[(0, 0)]).unwrap_err().0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("declared input"))
    );
    assert!(
        matches!(first.glue(&second, &[(0, 0), (0, 0)]).unwrap_err().0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("more than once"))
    );
    let qutrit = CircuitModel::<f64>::ungrouped(
        vec![WireType::Quantum { dim: 3 }],
        vec![],
        vec![0],
        vec![0],
    )
    .unwrap();
    assert!(
        matches!(first.glue(&qutrit, &[(0, 0)]).unwrap_err().0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("different types"))
    );
    // Unjoined wires of the second model are appended and renumbered.
    let wide = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Unitary {
            wires: vec![0, 1],
            program: vec![GateOp::Cz {
                control: 0,
                target: 1,
            }],
        }],
        vec![0],
        vec![0, 1],
    )
    .unwrap();
    let glued = first.glue(&wide, &[(0, 0)]).unwrap();
    assert_eq!(glued.wires().len(), 3);
    assert_eq!(glued.outputs(), &[0, 2]);
    assert_eq!(glued.boxes()[1].quantum_wires(), &[0, 2]);
}

#[test]
fn test_dilation_refuses_classical_boxes_and_cycles() {
    let classical = CircuitModel::<f64>::ungrouped(
        vec![WireType::qubit(), WireType::bit()],
        vec![CircuitBox::Measurement {
            wires: vec![0],
            outcome: 1,
        }],
        vec![0],
        vec![1],
    )
    .unwrap();
    assert!(
        matches!(classical.dilation().unwrap_err().0, QuantumErrorEnum::CalculationError(ref m) if m.contains("measurement"))
    );
    let cyclic = CircuitModel::<f64>::new(
        vec![WireType::qubit()],
        vec![ry(0.1), ry(0.2), ry(0.3)],
        vec![vec![0, 2], vec![1]],
        vec![],
        vec![0],
    )
    .unwrap();
    assert!(matches!(
        cyclic.dilation().unwrap_err().0,
        QuantumErrorEnum::CyclicStructureUnsupported(_)
    ));
    let empty =
        CircuitModel::<f64>::ungrouped(vec![WireType::qubit()], vec![], vec![0], vec![0]).unwrap();
    assert!(matches!(
        empty.dilation().unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
    // An override of the wrong shape is refused.
    let d = chain().dilation().unwrap();
    let mut bad = BTreeMap::new();
    bad.insert(
        0usize,
        CausalTensor::from_slice(&[C::new(1.0, 0.0)], &[1, 1]),
    );
    assert!(matches!(
        d.joint_instrument(&bad).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    assert_eq!(d.instruments().len(), 2);
}

#[test]
fn test_opening_on_the_circuit_equals_mechanism_replacement_on_the_dilation() {
    // Open node 1 on the circuit: the fresh input passes straight to the output, so an input state
    // ρ = R_y(0.5)|0⟩⟨0|R_y† reads |1⟩ with probability sin²(0.25). On the dilation, the same
    // opening is `intervene_mechanism` at node 1 with the mechanism "prepare ρ on the input half,
    // identity elsewhere", evaluated with the identity instrument composed with P₁ at node 1. The
    // two sides share no code path: one runs the Kraus semantics, the other the process-operator
    // trace.
    let model = chain();
    let expected = (0.25f64).sin().powi(2);
    let r = QubitOperator::rotation(Axis::Y, 0.5).unwrap();
    let p0 = CausalTensor::from_slice(
        &[
            C::new(1.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
        ],
        &[2, 2],
    );
    let rho = r
        .matrix()
        .matmul(&p0)
        .and_then(|m| m.matmul(&r.matrix().dagger().unwrap()))
        .unwrap();
    // Circuit side.
    let opened = model.opened(&[1]).unwrap();
    let sem = opened.numeric_semantics(&NumericCaps::default()).unwrap();
    let out =
        deep_causality_quantum::apply_kraus(sem.blocks().get(&(vec![], vec![])).unwrap(), &rho)
            .unwrap();
    assert!((out.as_slice()[3].re - expected).abs() < 1e-12);
    // Dilation side: legs 0 and 1 of dimension 4 each, index = leg0 · 4 + (in · 2 + out).
    let d = model.dilation().unwrap();
    let mut data = vec![C::new(0.0, 0.0); 256];
    for l0 in 0..4 {
        for i in 0..2 {
            for ip in 0..2 {
                for o in 0..2 {
                    let row = l0 * 4 + i * 2 + o;
                    let col = l0 * 4 + ip * 2 + o;
                    data[row * 16 + col] = rho.as_slice()[i * 2 + ip];
                }
            }
        }
    }
    let mechanism = CausalTensor::from_slice(&data, &[16, 16]);
    let intervened = d
        .hypothesis("chain")
        .unwrap()
        .intervene_mechanism(1, mechanism)
        .unwrap();
    let p1 = CausalTensor::from_slice(
        &[
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    let mut overrides = BTreeMap::new();
    overrides.insert(1usize, choi_from_kraus(&[p1]).unwrap());
    let tau = d.joint_instrument(&overrides).unwrap();
    let p = intervened.evaluate(&tau).unwrap();
    assert!((p - expected).abs() < 1e-12, "{p} vs {expected}");
    // The intervened factorization is still Markov, with its own certificate.
    let report = quantum_markov_check_report(
        intervened.factors().unwrap(),
        intervened.supports().unwrap(),
        &CommutatorTolerance::new(),
    )
    .unwrap();
    assert!(report.accepted());
}

/// `CNOT` with the control on wire 1 and the target on wire 0, wire 0 most significant: the
/// hand-written permutation `|01⟩ ↔ |11⟩`, indices 1 and 3.
fn cnot_control_wire_one() -> CausalTensor<C> {
    let mut data = vec![C::new(0.0, 0.0); 16];
    data[0] = C::new(1.0, 0.0);
    data[2 * 4 + 2] = C::new(1.0, 0.0);
    data[3 * 4 + 1] = C::new(1.0, 0.0);
    data[4 + 3] = C::new(1.0, 0.0);
    CausalTensor::from_slice(&data, &[4, 4])
}

/// Node 0 sets wire 1; node 1 is `middle`, a box listing its wires as `[1, 0]`; node 2 is the
/// identity on wire 0, whose instrument the test replaces with the projector onto `|1⟩`.
fn descending_wire_model(middle: CircuitBox<f64>) -> CircuitModel<f64> {
    CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![
            CircuitBox::Unitary {
                wires: vec![1],
                program: vec![GateOp::X(0)],
            },
            middle,
            CircuitBox::Unitary {
                wires: vec![0],
                program: vec![],
            },
        ],
        vec![],
        vec![0, 1],
    )
    .unwrap()
}

#[test]
fn test_box_wires_listed_in_descending_order_keep_their_leg_order() {
    // The box's first wire is its most significant leg, whatever its number: `CNOT` with the
    // control on wire 1 flips wire 0 once node 0 has set wire 1, so wire 0 reads `|1⟩` with
    // probability one. Read in ascending order the control would be wire 0, which is never set,
    // and the probability would be zero.
    let p1 = CausalTensor::from_slice(
        &[
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(0.0, 0.0),
            C::new(1.0, 0.0),
        ],
        &[2, 2],
    );
    let mut overrides = BTreeMap::new();
    overrides.insert(2usize, choi_from_kraus(&[p1]).unwrap());
    let (_, forward) = deep_causality_quantum::gate_unitary::<f64>(&GateOp::Cnot {
        control: 0,
        target: 1,
    })
    .unwrap();
    let boxes = [
        CircuitBox::Unitary {
            wires: vec![1, 0],
            program: vec![GateOp::Cnot {
                control: 0,
                target: 1,
            }],
        },
        CircuitBox::Kraus {
            wires: vec![1, 0],
            kraus: vec![forward.clone()],
        },
        CircuitBox::Channel {
            wires: vec![1, 0],
            channel: Channel::from_kraus(&[forward]).unwrap(),
        },
    ];
    let expected = choi_from_kraus(&[cnot_control_wire_one()]).unwrap();
    for middle in boxes {
        let kind = middle.kind();
        let model = descending_wire_model(middle);
        let d = model.dilation().unwrap();
        assert_eq!(d.legs()[1].wires(), &[0, 1], "the leg itself is ascending");
        let instrument = d.instruments().get(&1).unwrap();
        for (a, b) in instrument.as_slice().iter().zip(expected.as_slice()) {
            assert!(
                (a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12,
                "{kind}: the instrument is not the Choi of CNOT with control wire 1"
            );
        }
        let p = d.predict(&d.joint_instrument(&overrides).unwrap()).unwrap();
        assert!((p - 1.0).abs() < 1e-12, "{kind}: {p}");
        // The Kraus semantics reads the same order: `|00⟩ ↦ |11⟩`.
        let sem = model.numeric_semantics(&NumericCaps::default()).unwrap();
        let k = &sem.blocks().get(&(vec![], vec![])).unwrap()[0];
        assert!((k.as_slice()[3].re - 1.0).abs() < 1e-12, "{kind}");
    }
}

#[test]
fn test_joint_instrument_refuses_a_union_above_the_entry_cap() {
    // Seven independent one-qubit nodes: every factor is 4 × 4 and the dilation forms, but the
    // union of the legs has dimension 4^7 = 16384 and the joint instrument would have 2^28
    // entries, above the cap of 2^24.
    let n = 7;
    let boxes: Vec<CircuitBox<f64>> = (0..n)
        .map(|w| CircuitBox::Channel {
            wires: vec![w],
            channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, 0.7).unwrap()).unwrap(),
        })
        .collect();
    let model =
        CircuitModel::ungrouped(vec![WireType::qubit(); n], boxes, vec![], (0..n).collect())
            .unwrap();
    let d = model.dilation().unwrap();
    assert_eq!(d.factors().len(), n);
    let err = d.joint_instrument(&BTreeMap::new()).unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            let entries = 1u64 << 28;
            assert!(
                msg.contains(&entries.to_string()) && msg.contains("dilation cap"),
                "{msg}"
            );
        }
        other => panic!("{other:?}"),
    }
}
