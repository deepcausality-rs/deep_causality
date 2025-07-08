/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::SymbolicResult;

#[test]
fn test_symbolic_result_variants_equality() {
    assert_eq!(SymbolicResult::Proven, SymbolicResult::Proven);
    assert_eq!(SymbolicResult::Disproven, SymbolicResult::Disproven);
    assert_eq!(SymbolicResult::Undetermined, SymbolicResult::Undetermined);

    assert_ne!(SymbolicResult::Proven, SymbolicResult::Disproven);
    assert_ne!(SymbolicResult::Proven, SymbolicResult::Undetermined);
    assert_ne!(SymbolicResult::Disproven, SymbolicResult::Undetermined);
}

#[test]
fn test_symbolic_result_display_output() {
    let proven = SymbolicResult::Proven.to_string();
    let disproven = SymbolicResult::Disproven.to_string();
    let undetermined = SymbolicResult::Undetermined.to_string();

    assert_eq!(proven, "Proven");
    assert_eq!(disproven, "Disproven");
    assert_eq!(undetermined, "Undetermined");
}
