/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_graph;
use deep_causality::*;

#[test]
fn test_left_imbalanced_cause_graph() {
    //   root(0) -> A(1), B(2), C(3)
    //   A(1) -> D(4), E(5)
    let (g, _data) = test_utils_graph::get_left_imbalanced_cause_graph();

    // Single reasoning: Activate node C (index 3).
    let index_c = 3;
    let effect = PropagatingEffect::from_value(0.99);
    let res = g.evaluate_single_cause(index_c, &effect);
    dbg!(&res);
    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Value(1.0));

    // 3. Partial reasoning from A (index 1). This will also activate its descendants D and E.
    let res = g.evaluate_subgraph_from_cause(1, &effect);
    assert!(res.is_ok());
    dbg!(&res);
    assert_eq!(res.value, EffectValue::Value(1.0));

    // 4. Selective sub-graph reasoning from A(1) to D(4).
    // The path is [1, 4]. Re-evaluating them is idempotent.
    let res = g.evaluate_shortest_path_between_causes(1, 4, &effect);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Value(1.0));

    // 5. Single reasoning: Deactivate node A (index 1).
    let index_c = 1;
    let effect = PropagatingEffect::from_value(0.02);
    let res = g.evaluate_single_cause(index_c, &effect);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Value(0.0));
}

#[test]
fn test_right_imbalanced_cause_graph() {
    //   root(0) -> A(1), B(2), C(3)
    //   C(3) -> D(4), E(5)
    let (g, _data) = test_utils_graph::get_right_imbalanced_cause_graph();

    // Single reasoning: Activate node C (index 3).
    let index_c = 3;
    let effect = PropagatingEffect::from_value(0.99);
    let res = g.evaluate_single_cause(index_c, &effect);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Value(1.0));

    // 3. Partial reasoning from C (index 3). This includes descendants D and E.
    let res = g.evaluate_subgraph_from_cause(3, &effect);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Value(1.0));

    // 4. Single reasoning: Deactivate node A (index 1).
    let index_c = 1;
    let effect = PropagatingEffect::from_value(0.02);
    let res = g.evaluate_single_cause(index_c, &effect);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(res.value, EffectValue::Value(0.0));
}
