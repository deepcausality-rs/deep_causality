/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::TeloidRelation;

#[test]
fn test_teloid_relation_inherits() {
    let relation = TeloidRelation::Inherits;
    assert_eq!(relation, TeloidRelation::Inherits);
    assert_ne!(relation, TeloidRelation::Defeats);
    assert_eq!(format!("{}", relation), "Inherits");
    assert_eq!(format!("{:?}", relation), "Inherits");
}

#[test]
fn test_teloid_relation_defeats() {
    let relation = TeloidRelation::Defeats;
    assert_eq!(relation, TeloidRelation::Defeats);
    assert_ne!(relation, TeloidRelation::Inherits);
    assert_eq!(format!("{}", relation), "Defeats");
    assert_eq!(format!("{:?}", relation), "Defeats");
}

#[test]
fn test_teloid_relation_default() {
    let default_relation: TeloidRelation = Default::default();
    assert_eq!(default_relation, TeloidRelation::Inherits);
}

#[test]
fn test_teloid_relation_clone() {
    let original = TeloidRelation::Inherits;
    let cloned = original.clone();
    assert_eq!(original, cloned);

    let original = TeloidRelation::Defeats;
    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_teloid_relation_copy() {
    let original = TeloidRelation::Inherits;
    let copied = original; // Implicit copy
    assert_eq!(original, copied);
}
