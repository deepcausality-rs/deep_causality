/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::SymbolicRepresentation;

#[test]
fn test_atom_constructor_and_display() {
    let atom = SymbolicRepresentation::new_atom("A".to_string());
    assert_eq!(format!("{}", atom), r#"Atom("A")"#);
    assert_eq!(atom, SymbolicRepresentation::Atom("A".to_string()));
}

#[test]
fn test_expr_constructor_and_display() {
    let expr = SymbolicRepresentation::new_expr("Temp > 100".to_string());
    assert_eq!(format!("{}", expr), r#"Expr("Temp > 100")"#);
    assert_eq!(expr, SymbolicRepresentation::Expr("Temp > 100".to_string()));
}

#[test]
fn test_and_constructor_and_display() {
    let left = Box::new(SymbolicRepresentation::new_atom("A".to_string()));
    let right = Box::new(SymbolicRepresentation::new_atom("B".to_string()));
    let and_expr = SymbolicRepresentation::new_and(left.clone(), right.clone());

    assert_eq!(
        format!("{}", and_expr),
        format!("And({:?}, {:?})", left, right)
    );
}

#[test]
fn test_or_constructor_and_display() {
    let left = Box::new(SymbolicRepresentation::new_atom("X".to_string()));
    let right = Box::new(SymbolicRepresentation::new_atom("Y".to_string()));
    let or_expr = SymbolicRepresentation::new_or(left.clone(), right.clone());

    assert_eq!(
        format!("{}", or_expr),
        format!("Or({:?}, {:?})", left, right)
    );
}

#[test]
fn test_not_constructor_and_display() {
    let inner = Box::new(SymbolicRepresentation::new_atom("Z".to_string()));
    let not_expr = SymbolicRepresentation::new_not(inner.clone());

    assert_eq!(format!("{}", not_expr), format!("Not({:?})", inner));
}

#[test]
fn test_implies_constructor_and_display() {
    let a = Box::new(SymbolicRepresentation::new_atom("A".to_string()));
    let b = Box::new(SymbolicRepresentation::new_atom("B".to_string()));
    let implies_expr = SymbolicRepresentation::new_implies(a.clone(), b.clone());

    assert_eq!(
        format!("{}", implies_expr),
        format!("Implies({:?}, {:?})", a, b)
    );
}

#[test]
fn test_iff_constructor_and_display() {
    let a = Box::new(SymbolicRepresentation::new_atom("C".to_string()));
    let b = Box::new(SymbolicRepresentation::new_atom("D".to_string()));
    let iff_expr = SymbolicRepresentation::new_iff(a.clone(), b.clone());

    assert_eq!(format!("{}", iff_expr), format!("Iff({:?}, {:?})", a, b));
}
