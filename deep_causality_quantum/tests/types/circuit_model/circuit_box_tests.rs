/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_quantum::{Channel, CircuitBox, GateOp, QubitOperator};
use deep_causality_tensor::CausalTensor;

type C = Complex<f64>;

fn ket(amps: &[f64]) -> CausalTensor<C> {
    CausalTensor::from_slice(
        &amps.iter().map(|&a| C::new(a, 0.0)).collect::<Vec<_>>(),
        &[amps.len()],
    )
}

#[test]
fn test_wire_accessors_per_kind() {
    let enc = CircuitBox::<f64>::Encoder {
        input: 5,
        outputs: vec![1, 2],
        states: vec![ket(&[1.0, 0.0, 0.0, 0.0]), ket(&[0.0, 0.0, 0.0, 1.0])],
    };
    assert_eq!(enc.quantum_wires(), &[1, 2]);
    assert_eq!(enc.classical_read(), Some(5));
    assert_eq!(enc.classical_write(), None);
    assert_eq!(enc.kind(), "encoder");

    let uni = CircuitBox::<f64>::Unitary {
        wires: vec![0],
        program: vec![GateOp::H(0)],
    };
    assert_eq!(uni.quantum_wires(), &[0]);
    assert_eq!(uni.classical_read(), None);
    assert_eq!(uni.kind(), "unitary");

    let ch = CircuitBox::<f64>::Channel {
        wires: vec![3],
        channel: Channel::unitary(&QubitOperator::hadamard()).unwrap(),
    };
    assert_eq!(ch.quantum_wires(), &[3]);
    assert_eq!(ch.kind(), "channel");

    let ins = CircuitBox::<f64>::Instrument {
        wires: vec![0],
        outcome: 4,
        kraus: vec![vec![], vec![]],
    };
    assert_eq!(ins.classical_write(), Some(4));
    assert_eq!(ins.kind(), "instrument");

    let meas = CircuitBox::<f64>::Measurement {
        wires: vec![0, 1],
        outcome: 6,
    };
    assert_eq!(meas.classical_write(), Some(6));
    assert_eq!(meas.quantum_wires(), &[0, 1]);
    assert_eq!(meas.kind(), "measurement");
}
