/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fault sets: the count `C(n, t) · 3^t` is the oracle for every enumeration, checked by hand at
//! `(8, 1) = 24`, `(4, 2) = 54` and `(10, 3) = 3240`; the cap refuses by count before any fault
//! exists; declared and DEM sets keep order and drop duplicates.

use deep_causality_quantum::{
    FAULT_SET_CAP, Fault, FaultOrigin, FaultSet, PauliKind, QuantumErrorEnum, Query,
};
use std::collections::BTreeSet;

type W = u64;

#[test]
fn test_weight_one_on_eight_locations_has_24_distinct_faults() {
    let set =
        FaultSet::pauli_weight(&(0..8).collect::<Vec<_>>(), Some(0), 1, FAULT_SET_CAP).unwrap();
    assert_eq!(set.len(), 24);
    assert_eq!(set.origin(), FaultOrigin::PauliWeight(1));
    assert_eq!(format!("{set}"), "pauli_weight(1) (24 faults)");
    let distinct: BTreeSet<&Fault> = set.faults().iter().collect();
    assert_eq!(distinct.len(), 24);
    assert!(
        set.faults()
            .iter()
            .all(|f| f.weight() == 1 && f.after() == Some(0))
    );
    let per_wire = |w: usize| set.faults().iter().filter(|f| f.wires() == vec![w]).count();
    assert!((0..8).all(|w| per_wire(w) == 3));
    let queries = set.queries();
    assert_eq!(queries.len(), 24);
    assert!(matches!(&queries[0], Query::Fault(f) if f == &set.faults()[0]));
    assert_eq!(queries[0].kind(), "fault");
    // The first fault is X on the lowest wire; the pattern digit runs X, Y, Z.
    assert_eq!(set.faults()[0].errors(), &[(0, PauliKind::X)]);
    assert_eq!(set.faults()[1].errors(), &[(0, PauliKind::Y)]);
    assert_eq!(set.faults()[2].errors(), &[(0, PauliKind::Z)]);
}

#[test]
fn test_counts_follow_the_binomial_times_three_to_the_weight() {
    assert_eq!(FaultSet::count_pauli_weight(8, 1), Some(24));
    assert_eq!(FaultSet::count_pauli_weight(4, 2), Some(54));
    assert_eq!(FaultSet::count_pauli_weight(10, 3), Some(3240));
    assert_eq!(FaultSet::count_pauli_weight(18, 1), Some(54));
    assert_eq!(FaultSet::count_pauli_weight(3, 5), Some(0));
    assert_eq!(FaultSet::count_pauli_weight(0, 1), Some(0));
    assert_eq!(
        FaultSet::count_pauli_weight(200, 100),
        None,
        "overflows a u64"
    );
    // Locations need not be contiguous or sorted; the faults come out sorted by wire.
    let set = FaultSet::pauli_weight(&[7, 2, 5, 0], None, 2, FAULT_SET_CAP).unwrap();
    assert_eq!(set.len(), 54);
    assert!(
        set.faults()
            .iter()
            .all(|f| f.weight() == 2 && f.after().is_none())
    );
    assert_eq!(set.faults()[0].wires(), vec![0, 2]);
    assert_eq!(set.faults()[53].wires(), vec![5, 7]);
    assert_eq!(
        set.faults()[53].errors(),
        &[(5, PauliKind::Z), (7, PauliKind::Z)]
    );
}

#[test]
fn test_above_the_cap_is_refused_naming_count_and_cap() {
    let err = FaultSet::pauli_weight(&(0..10).collect::<Vec<_>>(), None, 3, 1000).unwrap_err();
    match err.0 {
        QuantumErrorEnum::CalculationError(msg) => {
            assert!(msg.contains("3240") && msg.contains("1000"), "{msg}")
        }
        other => panic!("{other:?}"),
    }
    // Exactly at the cap is admitted.
    assert_eq!(
        FaultSet::pauli_weight(&(0..10).collect::<Vec<_>>(), None, 3, 3240)
            .unwrap()
            .len(),
        3240
    );
    let overflow =
        FaultSet::pauli_weight(&(0..200).collect::<Vec<_>>(), None, 100, u64::MAX).unwrap_err();
    assert!(
        matches!(overflow.0, QuantumErrorEnum::CalculationError(ref m) if m.contains("overflows"))
    );
}

#[test]
fn test_construction_errors() {
    let zero = FaultSet::pauli_weight(&[0, 1], None, 0, FAULT_SET_CAP).unwrap_err();
    assert!(matches!(zero.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("weight")));
    let repeated = FaultSet::pauli_weight(&[0, 1, 0], None, 1, FAULT_SET_CAP).unwrap_err();
    assert!(
        matches!(repeated.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("repeat"))
    );
    let empty = Fault::new(None, vec![]).unwrap_err();
    assert!(matches!(empty.0, QuantumErrorEnum::DimensionMismatch(_)));
    let twice = Fault::new(None, vec![(3, PauliKind::X), (3, PauliKind::Z)]).unwrap_err();
    assert!(matches!(twice.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("wire 3")));
    let wide = Fault::new(None, vec![(4, PauliKind::X)]).unwrap();
    let err = wide.as_logical_pauli::<W>(4).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(ref m) if m.contains("wire 4")));
}

#[test]
fn test_declared_and_dem_sets_keep_order_and_drop_duplicates() {
    let a = Fault::new(Some(1), vec![(0, PauliKind::X)]).unwrap();
    let b = Fault::new(Some(1), vec![(2, PauliKind::Z), (1, PauliKind::Y)]).unwrap();
    let set = FaultSet::declared(&[a.clone(), b.clone(), a.clone()]);
    assert_eq!(set.faults(), &[a.clone(), b.clone()]);
    assert_eq!(set.origin(), FaultOrigin::Declared);
    assert_eq!(format!("{}", set.origin()), "declared");
    let empty = FaultSet::declared(&[]);
    assert!(empty.is_empty());
    assert_eq!(format!("{empty}"), "declared (0 faults)");

    let dem = FaultSet::from_dem(
        &[
            vec![(1, PauliKind::Y), (2, PauliKind::Z)],
            vec![],
            vec![(0, PauliKind::X)],
            vec![(2, PauliKind::Z), (1, PauliKind::Y)],
        ],
        Some(1),
    )
    .unwrap();
    assert_eq!(dem.origin(), FaultOrigin::Dem);
    assert_eq!(
        dem.faults(),
        &[b, a],
        "sorted within a fault, deduplicated across"
    );
    assert_eq!(format!("{dem}"), "from_dem (2 faults)");
    let bad = FaultSet::from_dem(&[vec![(0, PauliKind::X), (0, PauliKind::Z)]], None).unwrap_err();
    assert!(matches!(bad.0, QuantumErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_a_fault_reads_as_a_pauli_and_as_a_program() {
    let f = Fault::new(None, vec![(2, PauliKind::Z), (0, PauliKind::X)]).unwrap();
    assert_eq!(f.name(), "X0 Z2 at the inputs");
    assert_eq!(format!("{f}"), "X0 Z2 at the inputs");
    assert_eq!(
        f.program(),
        vec![
            deep_causality_quantum::GateOp::X(0),
            deep_causality_quantum::GateOp::Z(1)
        ]
    );
    let p = f.as_logical_pauli::<W>(4).unwrap();
    assert_eq!(p.x().support().collect::<Vec<_>>(), vec![0]);
    assert_eq!(p.z().support().collect::<Vec<_>>(), vec![2]);
    let y = Fault::new(Some(3), vec![(1, PauliKind::Y)]).unwrap();
    assert_eq!(y.name(), "Y1 after node 3");
    let p = y.as_logical_pauli::<W>(2).unwrap();
    assert_eq!(p.x().support().collect::<Vec<_>>(), vec![1]);
    assert_eq!(p.z().support().collect::<Vec<_>>(), vec![1]);
    assert!(PauliKind::Y.has_x() && PauliKind::Y.has_z());
    assert!(PauliKind::X.has_x() && !PauliKind::X.has_z());
    assert!(!PauliKind::Z.has_x() && PauliKind::Z.has_z());
}
