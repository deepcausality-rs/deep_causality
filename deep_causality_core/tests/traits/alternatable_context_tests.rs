/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::*;

#[derive(Clone, Debug, PartialEq)]
struct Cfg {
    threshold: i32,
}

#[test]
fn test_propagating_process_alternate_context_replaces_only_context() {
    let initial = PropagatingProcess::pure(42_i32);
    let process = PropagatingProcess::with_state(initial, "running", Some(Cfg { threshold: 10 }));

    assert_eq!(*process.state(), "running");
    assert_eq!(process.context().clone().unwrap(), Cfg { threshold: 10 });

    let new_cfg = Cfg { threshold: 99 };
    let alternated = process.alternate_context(new_cfg.clone());

    // Context must change.
    assert_eq!(alternated.context().clone().unwrap(), new_cfg);
    // Value must be preserved.
    if let Some(v) = alternated.value() {
        assert_eq!(*v, 42);
    } else {
        panic!("Expected Value(42)");
    }
    // State must be preserved.
    assert_eq!(*alternated.state(), "running");
    // No error must be introduced.
    assert!(alternated.is_ok());
}

#[test]
fn test_alternate_context_with_error_is_noop() {
    let err = CausalityError::new(CausalityErrorEnum::Custom("upstream failure".into()));
    let process = PropagatingProcess::<i32, &'static str, Cfg>::from_error(err);

    let alternated = process.alternate_context(Cfg { threshold: 99 });

    // Error must propagate, context must not change to the new value.
    assert!(alternated.is_err());
    assert!(alternated.context().is_none());
}

#[test]
fn test_alternate_context_appends_log_marker() {
    let initial = PropagatingProcess::pure(1_i32);
    let process = PropagatingProcess::with_state(initial, (), Some(Cfg { threshold: 0 }));
    let alternated = process.alternate_context(Cfg { threshold: 1 });
    assert!(
        alternated
            .logs()
            .to_string()
            .contains("!!ContextAlternation!!")
    );
}

#[test]
fn test_alternate_context_on_propagating_effect_is_unit_only() {
    // Documented behaviour: PropagatingEffect (Context = ()) accepts the
    // call but only the audit log changes.
    let effect = PropagatingEffect::pure(7_i32);
    let alternated = effect.alternate_context(());
    if let Some(v) = alternated.value() {
        assert_eq!(*v, 7);
    }
    assert!(
        alternated
            .logs()
            .to_string()
            .contains("!!ContextAlternation!!")
    );
}
