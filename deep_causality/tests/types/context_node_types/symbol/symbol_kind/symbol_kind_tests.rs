/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::{BaseSymbol, Identifiable, SymbolKind, Symbolic, SymbolicRepresentation};

#[test]
fn test_symbol_kind_id_and_repr() {
    let base = BaseSymbol::new(42, SymbolicRepresentation::new_atom("X".to_string()));
    let kind = SymbolKind::BaseSymbol(base.clone());

    assert_eq!(kind.id(), 42);
    assert_eq!(
        kind.symbol(),
        &SymbolicRepresentation::Atom("X".to_string())
    );
    assert_eq!(kind.to_string(), base.to_string());
}

#[test]
fn test_symbol_kind_display_with_all_variants() {
    // All symbol representation variants should display successfully via Debug.
    let a = SymbolicRepresentation::new_atom("A".into());
    let b = SymbolicRepresentation::new_atom("B".into());

    let cases = vec![
        SymbolicRepresentation::new_and(Box::new(a.clone()), Box::new(b.clone())),
        SymbolicRepresentation::new_or(Box::new(a.clone()), Box::new(b.clone())),
        SymbolicRepresentation::new_not(Box::new(a.clone())),
        SymbolicRepresentation::new_implies(Box::new(a.clone()), Box::new(b.clone())),
        SymbolicRepresentation::new_iff(Box::new(a.clone()), Box::new(b.clone())),
        SymbolicRepresentation::new_expr("A && !B".into()),
    ];

    for (i, repr) in cases.into_iter().enumerate() {
        let symbol = BaseSymbol::new(i as u64, repr.clone());
        let kind = SymbolKind::BaseSymbol(symbol.clone());

        // Test Display formatting
        let display_output = format!("{kind}");
        let debug_output = format!("{repr:?}");

        // Since Display is implemented via Debug, this should match
        assert_eq!(display_output, debug_output);

        // Also test equality to make sure SymbolKind is matching as expected
        assert_eq!(kind.symbol(), &repr);
    }
}

#[test]
fn test_symbol_kind_default_base_symbol() {
    let default = BaseSymbol::default();
    let kind = SymbolKind::BaseSymbol(default.clone());

    assert_eq!(kind.id(), 0);
    assert_eq!(
        kind.symbol(),
        &SymbolicRepresentation::Atom("Default".to_string())
    );

    // Display should format using Debug of SymbolicRepresentation
    assert_eq!(kind.to_string(), format!("{:?}", default.symbol()));
}
