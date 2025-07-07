/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

#[test]
fn test_is_deterministic() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert!(effect1.is_deterministic());

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(!effect2.is_deterministic());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(!effect3.is_deterministic());
}

#[test]
fn test_is_probabilistic() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert!(!effect1.is_probabilistic());

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(effect2.is_probabilistic());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(!effect3.is_probabilistic());
}

#[test]
fn test_is_contextual_link() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert!(!effect1.is_contextual_link());

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(!effect2.is_contextual_link());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(effect3.is_contextual_link());
}

#[test]
fn test_as_bool() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect1.as_bool(), Some(true));

    let effect2 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect2.as_bool(), Some(false));

    let effect3 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect3.as_bool(), None);

    let effect4 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect4.as_bool(), None);
}

#[test]
fn test_as_probability() {
    let effect1 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect1.as_probability(), Some(0.5));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_probability(), None);

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_probability(), None);
}

#[test]
fn test_as_contextual_link() {
    let effect1 = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(effect1.as_contextual_link(), Some((1, 2)));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_contextual_link(), None);

    let effect3 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect3.as_contextual_link(), None);
}

#[test]
fn test_display() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(format!("{effect1}"), "Deterministic: true");

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(format!("{effect2}"), "Probabilistic: 0.5");

    let effect3 = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(format!("{effect3}"), "ContextualLink: 1 Contextoid: 2");
}

#[test]
fn test_clone() {
    let effect1 = PropagatingEffect::Deterministic(true);
    let clone1 = effect1.clone();
    assert_eq!(effect1, clone1);

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    let clone2 = effect2.clone();
    assert_eq!(effect2, clone2);

    let effect3 = PropagatingEffect::ContextualLink(1, 2);
    let clone3 = effect3.clone();
    assert_eq!(effect3, clone3);
}

#[test]
fn test_debug() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(format!("{effect1:?}"), "Deterministic(true)");

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(format!("{effect2:?}"), "Probabilistic(0.5)");

    let effect3 = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(format!("{effect3:?}"), "ContextualLink(1, 2)");
}

#[test]
fn test_partial_eq() {
    let effect1 = PropagatingEffect::Deterministic(true);
    let effect2 = PropagatingEffect::Deterministic(true);
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let effect4 = PropagatingEffect::Probabilistic(0.5);
    let effect5 = PropagatingEffect::Probabilistic(0.5);
    let effect6 = PropagatingEffect::Probabilistic(0.6);
    assert_eq!(effect4, effect5);
    assert_ne!(effect4, effect6);

    let effect7 = PropagatingEffect::ContextualLink(1, 2);
    let effect8 = PropagatingEffect::ContextualLink(1, 2);
    let effect9 = PropagatingEffect::ContextualLink(2, 1);
    assert_eq!(effect7, effect8);
    assert_ne!(effect7, effect9);

    assert_ne!(effect1, effect4);
    assert_ne!(effect1, effect7);
    assert_ne!(effect4, effect7);
}
