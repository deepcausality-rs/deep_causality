/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gate matrices against their textbook forms (Nielsen & Chuang §4.2): every literal below is the
//! published matrix entry, not a reading of the code.

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{GateOp, QuantumErrorEnum, gate_unitary};

type C = Complex<f64>;

fn close(a: C, b: C) -> bool {
    (a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12
}

#[test]
fn test_hadamard_and_paulis() {
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let (q, h) = gate_unitary::<f64>(&GateOp::H(3)).unwrap();
    assert_eq!(q, vec![3]);
    let hs = h.as_slice();
    assert!(close(hs[0], C::new(s, 0.0)) && close(hs[3], C::new(-s, 0.0)));
    let (_, y) = gate_unitary::<f64>(&GateOp::Y(0)).unwrap();
    // Y = [[0, −i], [i, 0]].
    assert!(close(y.as_slice()[1], C::new(0.0, -1.0)) && close(y.as_slice()[2], C::new(0.0, 1.0)));
    let (_, x) = gate_unitary::<f64>(&GateOp::X(0)).unwrap();
    assert!(close(x.as_slice()[1], C::new(1.0, 0.0)) && close(x.as_slice()[0], C::new(0.0, 0.0)));
    let (_, z) = gate_unitary::<f64>(&GateOp::Z(0)).unwrap();
    assert!(close(z.as_slice()[3], C::new(-1.0, 0.0)));
}

#[test]
fn test_phase_gates() {
    let t = std::f64::consts::FRAC_PI_4;
    for (op, phase) in [
        (GateOp::S(0), C::new(0.0, 1.0)),
        (GateOp::Sdg(0), C::new(0.0, -1.0)),
        (GateOp::T(0), C::new(t.cos(), t.sin())),
        (GateOp::Tdg(0), C::new(t.cos(), -t.sin())),
    ] {
        let (_, m) = gate_unitary::<f64>(&op).unwrap();
        assert!(close(m.as_slice()[3], phase), "{op:?}");
        assert!(close(m.as_slice()[0], C::new(1.0, 0.0)));
    }
}

#[test]
fn test_cnot_respects_ascending_order_whichever_qubit_controls() {
    // Control 0 (most significant), target 1: |10⟩ ↦ |11⟩, so column 2 has its one in row 3.
    let (q, m) = gate_unitary::<f64>(&GateOp::Cnot {
        control: 0,
        target: 1,
    })
    .unwrap();
    assert_eq!(q, vec![0, 1]);
    let ms = m.as_slice();
    assert!(close(ms[3 * 4 + 2], C::new(1.0, 0.0)) && close(ms[2 * 4 + 3], C::new(1.0, 0.0)));
    assert!(close(ms[0], C::new(1.0, 0.0)) && close(ms[4 + 1], C::new(1.0, 0.0)));
    // Control 1 (least significant), target 0: |01⟩ ↦ |11⟩, column 1 has its one in row 3.
    let (q, m) = gate_unitary::<f64>(&GateOp::Cnot {
        control: 1,
        target: 0,
    })
    .unwrap();
    assert_eq!(q, vec![0, 1]);
    let ms = m.as_slice();
    assert!(close(ms[3 * 4 + 1], C::new(1.0, 0.0)) && close(ms[4 + 3], C::new(1.0, 0.0)));
    assert!(close(ms[2 * 4 + 2], C::new(1.0, 0.0)));
}

#[test]
fn test_diagonal_family_phases_the_all_ones_state_only() {
    let (q, cz) = gate_unitary::<f64>(&GateOp::Cz {
        control: 2,
        target: 0,
    })
    .unwrap();
    assert_eq!(q, vec![0, 2]);
    let s = cz.as_slice();
    assert!(close(s[15], C::new(-1.0, 0.0)) && close(s[10], C::new(1.0, 0.0)));
    let (_, csdg) = gate_unitary::<f64>(&GateOp::Csdg {
        control: 0,
        target: 1,
    })
    .unwrap();
    assert!(close(csdg.as_slice()[15], C::new(0.0, -1.0)));
    let (_, ccz) = gate_unitary::<f64>(&GateOp::Ccz {
        q0: 0,
        q1: 1,
        q2: 2,
    })
    .unwrap();
    assert_eq!(ccz.shape(), &[8, 8]);
    assert!(
        close(ccz.as_slice()[63], C::new(-1.0, 0.0)) && close(ccz.as_slice()[54], C::new(1.0, 0.0))
    );
    let (q, cmz) = gate_unitary::<f64>(&GateOp::Cmz {
        qubits: vec![3, 1, 0, 2],
    })
    .unwrap();
    assert_eq!(q, vec![0, 1, 2, 3]);
    assert_eq!(cmz.shape(), &[16, 16]);
    assert!(close(cmz.as_slice()[255], C::new(-1.0, 0.0)));
}

#[test]
fn test_repeated_qubit_is_refused() {
    let err = gate_unitary::<f64>(&GateOp::Cz {
        control: 1,
        target: 1,
    })
    .unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_cmz_wider_than_the_gate_limit_is_refused_before_allocating() {
    use deep_causality_quantum::MAX_GATE_QUBITS;
    assert_eq!(
        MAX_GATE_QUBITS, 12,
        "a 2^12 × 2^12 matrix is the default entry cap"
    );
    // One past the limit: refused by name, no matrix formed.
    let wide: Vec<usize> = (0..=MAX_GATE_QUBITS).collect();
    let err = gate_unitary::<f64>(&GateOp::Cmz { qubits: wide }).unwrap_err();
    match err.0 {
        QuantumErrorEnum::DimensionMismatch(msg) => {
            assert!(msg.contains("13") && msg.contains("12"), "{msg}")
        }
        other => panic!("{other:?}"),
    }
    // A repeated qubit past the limit is still the repeat error: the count is of distinct qubits.
    let mut repeated: Vec<usize> = (0..MAX_GATE_QUBITS).collect();
    repeated.push(0);
    let err = gate_unitary::<f64>(&GateOp::Cmz { qubits: repeated }).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("more than once"))
    );
}
