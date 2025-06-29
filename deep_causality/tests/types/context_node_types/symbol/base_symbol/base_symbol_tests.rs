/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;
use std::format;

#[test]
fn test_base_symbol_default() {
    let default_symbol = BaseSymbol::default();
    assert_eq!(default_symbol.id(), 0);
    assert_eq!(
        default_symbol.symbol(),
        &SymbolicRepresentation::Atom("Default".to_string())
    );
    assert_eq!(format!("{default_symbol}"), r#"Atom("Default")"#);
}

#[test]
fn test_base_symbol_custom_atom() {
    let symbol = BaseSymbol::new(42, SymbolicRepresentation::new_atom("A".to_string()));
    assert_eq!(symbol.id(), 42);
    assert_eq!(
        symbol.symbol(),
        &SymbolicRepresentation::Atom("A".to_string())
    );
    assert_eq!(format!("{symbol}"), r#"Atom("A")"#);
}

#[test]
fn test_base_symbol_complex_display() {
    let a = Box::new(SymbolicRepresentation::new_atom("A".to_string()));
    let b = Box::new(SymbolicRepresentation::new_atom("B".to_string()));
    let expr = SymbolicRepresentation::new_and(a.clone(), b.clone());

    let symbol = BaseSymbol::new(1, expr.clone());
    assert_eq!(symbol.id(), 1);
    assert_eq!(symbol.symbol(), &expr);
    assert_eq!(format!("{symbol}"), format!("{expr:?}"));
}
