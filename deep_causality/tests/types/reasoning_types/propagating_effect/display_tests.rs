/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::PropagatingEffect;
use deep_causality::PropagatingEffect::{UncertainBool, UncertainFloat};
use deep_causality_uncertain::Uncertain;

#[test]
fn test_display() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(
        format!("{effect1}"),
        "PropagatingEffect::Deterministic(true)"
    );

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(
        format!("{effect2}"),
        "PropagatingEffect::Probabilistic(0.5)"
    );

    let effect3 = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(
        format!("{effect3}"),
        "PropagatingEffect::ContextualLink(1, 2)"
    );

    let point = Uncertain::<bool>::point(true);
    let effect4 = UncertainBool(point);
    assert!(format!("{effect4}").contains("Point(true)"));

    let point = Uncertain::<f64>::point(4.0f64);
    let effect5 = UncertainFloat(point);
    assert!(format!("{effect5}").contains("Point(4.0)"));
}
