/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_quantum::WireType;

#[test]
fn test_qubit_and_bit_constructors() {
    assert_eq!(WireType::qubit(), WireType::Quantum { dim: 2 });
    assert_eq!(WireType::bit(), WireType::Classical { outcomes: 2 });
    assert!(WireType::qubit().is_quantum());
    assert!(!WireType::bit().is_quantum());
}

#[test]
fn test_cardinality_reads_both_kinds() {
    assert_eq!(WireType::Quantum { dim: 3 }.cardinality(), 3);
    assert_eq!(WireType::Classical { outcomes: 5 }.cardinality(), 5);
}
