/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_quantum::{InducedDag, QuantumErrorEnum, Query, QuerySignature};

fn chain_dag() -> InducedDag {
    let mut g = InducedDag::new(3);
    g.add_edge(0, 1).add_edge(1, 2);
    g
}

#[test]
fn test_interchange_on_a_chain_is_refused_by_name() {
    let err = QuerySignature::new(&chain_dag(), vec![Query::Inc(vec![vec![0, 1]])]).unwrap_err();
    match err.0 {
        QuantumErrorEnum::NotParallelisable { set, from, to } => {
            assert_eq!((set, from, to), (0, 0, 1))
        }
        other => panic!("{other:?}"),
    }
    // Two parallelisable singletons are fine; a node in two sets is not.
    assert!(QuerySignature::new(&chain_dag(), vec![Query::Inc(vec![vec![0], vec![2]])]).is_ok());
    let err =
        QuerySignature::new(&chain_dag(), vec![Query::Inc(vec![vec![0], vec![0]])]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_out_of_range_nodes_and_accessors() {
    let err = QuerySignature::new(&chain_dag(), vec![Query::Open(vec![5])]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let err = QuerySignature::new(&chain_dag(), vec![Query::Inc(vec![vec![9]])]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
    let sig = QuerySignature::new(
        &chain_dag(),
        vec![Query::Io, Query::Observe(vec![0]), Query::Open(vec![2])],
    )
    .unwrap();
    assert_eq!(sig.len(), 3);
    assert!(!sig.is_empty());
    assert_eq!(sig.queries()[1].kind(), "observe");
    assert_eq!(Query::Io.kind(), "io");
    assert_eq!(Query::Open(vec![]).kind(), "open");
    assert_eq!(Query::Inc(vec![]).kind(), "interchange");
    assert!(
        QuerySignature::new(&chain_dag(), vec![])
            .unwrap()
            .is_empty()
    );
}
