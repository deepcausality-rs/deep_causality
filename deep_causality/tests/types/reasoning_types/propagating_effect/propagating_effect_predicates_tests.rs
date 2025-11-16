/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalityError, PropagatingEffect};

#[test]
fn test_is_ok() {
    let effect_ok = PropagatingEffect::from_deterministic(true);
    assert!(effect_ok.is_ok());

    let effect_err = PropagatingEffect::from_error(CausalityError::new("Error".into()));
    assert!(!effect_err.is_ok());
}

#[test]
fn test_is_err() {
    let effect_ok = PropagatingEffect::from_deterministic(true);
    assert!(!effect_ok.is_err());

    let effect_err = PropagatingEffect::from_error(CausalityError::new("Error".into()));
    assert!(effect_err.is_err());
}

#[test]
fn test_is_error() {
    let effect_ok = PropagatingEffect::from_deterministic(true);
    assert!(!effect_ok.is_error());

    let effect_err = PropagatingEffect::from_error(CausalityError::new("Error".into()));
    assert!(effect_err.is_error());
}

#[test]
fn test_has_log() {
    let effect_no_log = PropagatingEffect::from_deterministic(true);
    assert!(!effect_no_log.has_log());

    let mut effect_with_log = PropagatingEffect::from_deterministic(true);
    effect_with_log.logs.add_entry("Log entry");
    assert!(effect_with_log.has_log());
}
