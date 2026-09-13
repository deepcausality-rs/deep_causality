/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The circuit model as a `QcModel`: the wire map of a query that adds no inputs is empty, and a
//! malformed query is the query's own error on every variant.

use deep_causality_quantum::{
    Axis, Channel, CircuitBox, CircuitModel, Fault, PauliKind, QcModel, QuantumErrorEnum,
    QubitOperator, Query, WireType,
};

fn model() -> CircuitModel<f64> {
    CircuitModel::ungrouped(
        vec![WireType::qubit(), WireType::qubit()],
        vec![CircuitBox::Channel {
            wires: vec![0],
            channel: Channel::unitary(&QubitOperator::rotation(Axis::Y, 0.3).unwrap()).unwrap(),
        }],
        vec![0],
        vec![0],
    )
    .unwrap()
}

#[test]
fn test_the_wire_map_validates_every_query_variant() {
    let m = model();
    assert!(m.query_wire_map(&Query::Io).unwrap().is_empty());
    assert!(
        m.query_wire_map(&Query::Observe(vec![0]))
            .unwrap()
            .is_empty()
    );
    let fault = Fault::new(Some(0), vec![(0, PauliKind::X)]).unwrap();
    assert!(m.query_wire_map(&Query::Fault(fault)).unwrap().is_empty());
    assert_eq!(
        m.query_wire_map(&Query::Open(vec![0])).unwrap(),
        vec![(0, 2)]
    );

    let observe = m.query_wire_map(&Query::Observe(vec![9])).unwrap_err();
    assert!(
        matches!(observe.0, QuantumErrorEnum::DimensionMismatch(ref s) if s.contains("wire 9")),
        "{observe:?}"
    );
    let after_missing = Fault::new(Some(7), vec![(0, PauliKind::Z)]).unwrap();
    let fault = m.query_wire_map(&Query::Fault(after_missing)).unwrap_err();
    assert!(
        matches!(fault.0, QuantumErrorEnum::DimensionMismatch(ref s) if s.contains("node 7")),
        "{fault:?}"
    );
    let open = m.query_wire_map(&Query::Open(vec![5])).unwrap_err();
    assert!(matches!(open.0, QuantumErrorEnum::DimensionMismatch(_)));
}
