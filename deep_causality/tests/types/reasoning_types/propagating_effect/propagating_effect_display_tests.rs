/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{CausalEffectLog, CausalityError, PropagatingEffect};

#[test]
fn test_display_delegates_to_debug() {
    // 1. Test with a simple value and no error/log.
    let effect_simple = PropagatingEffect::from_boolean(true);
    let display_simple = format!("{}", effect_simple);
    let debug_simple = format!("{:?}", effect_simple);
    assert_eq!(display_simple, debug_simple);

    // 2. Test with a value and an error.
    let error = CausalityError::new("Test error".to_string());
    let effect_with_error = PropagatingEffect::from_error(error);
    let display_with_error = format!("{}", effect_with_error);
    let debug_with_error = format!("{:?}", effect_with_error);
    assert_eq!(display_with_error, debug_with_error);

    // 3. Test with a value and a log.
    let mut effect_with_log = PropagatingEffect::from_boolean(false);
    let mut log = CausalEffectLog::new();
    log.add_entry("Test log entry");
    effect_with_log.logs = log;
    let display_with_log = format!("{}", effect_with_log);
    let debug_with_log = format!("{:?}", effect_with_log);
    assert_eq!(display_with_log, debug_with_log);

    // 4. Test with a value, an error, and a log.
    let error = CausalityError::new("Another test error".to_string());
    let mut effect_with_all = PropagatingEffect::from_error(error);
    let mut log = CausalEffectLog::new();
    log.add_entry("Another test log entry");
    effect_with_all.logs = log;
    let display_with_all = format!("{}", effect_with_all);
    let debug_with_all = format!("{:?}", effect_with_all);
    assert_eq!(display_with_all, debug_with_all);

    // 5. Test with a different EffectValue
    let effect_numerical = PropagatingEffect::from_numerical(42.0);
    let display_numerical = format!("{}", effect_numerical);
    let debug_numerical = format!("{:?}", effect_numerical);
    assert_eq!(display_numerical, debug_numerical);
}
