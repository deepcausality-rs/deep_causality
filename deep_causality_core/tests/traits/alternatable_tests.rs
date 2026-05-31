/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::*;

#[derive(Clone, Debug, Default, PartialEq)]
struct St {
    counter: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct Cfg {
    label: &'static str,
}

// Generic helper that requires the full triple capability. If the blanket
// `impl<T, V, C, S> Alternatable<V, C, S> for T where T: AlternatableValue<V>
// + AlternatableContext<C> + AlternatableState<S>` is wired correctly, any
// carrier supporting all three sub-traits will satisfy this bound without an
// explicit impl.
fn triple_alternate<T, V, C, S>(carrier: T, v: V, c: C, s: S) -> T
where
    T: Alternatable<V, C, S>,
{
    carrier
        .alternate_value(v)
        .alternate_context(c)
        .alternate_state(s)
}

#[test]
fn test_propagating_process_satisfies_full_alternatable_via_blanket_impl() {
    let initial = PropagatingProcess::pure(1_i32);
    let process = PropagatingProcess::with_state(initial, St::default(), Some(Cfg { label: "a" }));

    let new_state = St { counter: 7 };
    let new_cfg = Cfg { label: "b" };
    let alternated = triple_alternate(process, 99_i32, new_cfg.clone(), new_state.clone());

    if let EffectValue::Value(v) = alternated.value {
        assert_eq!(v, 99);
    } else {
        panic!("Expected Value(99)");
    }
    assert_eq!(alternated.context.clone().unwrap(), new_cfg);
    assert_eq!(alternated.state, new_state);
    assert!(alternated.error.is_none());
}

#[test]
fn test_propagating_effect_also_satisfies_full_alternatable_via_unit_channels() {
    // The unit-channel impls mean PropagatingEffect satisfies the super-trait
    // too; calling alternate_context(()) and alternate_state(()) is well-typed
    // and observable only in the audit log.
    let effect = PropagatingEffect::pure(1_i32);
    let alternated = triple_alternate(effect, 42_i32, (), ());

    if let EffectValue::Value(v) = alternated.value {
        assert_eq!(v, 42);
    } else {
        panic!("Expected Value(42)");
    }
}
