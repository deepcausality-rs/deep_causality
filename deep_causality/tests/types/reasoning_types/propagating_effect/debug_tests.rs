/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::PropagatingEffect;
use deep_causality::PropagatingEffect::{
    MaybeUncertainBool, MaybeUncertainFloat, UncertainBool, UncertainFloat,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};
use std::collections::HashMap;

#[test]
fn test_debug_trait() {
    let effect = PropagatingEffect::None;
    assert_eq!(format!("{:?}", effect), "PropagatingEffect::None");

    let effect = PropagatingEffect::Deterministic(true);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::Deterministic(true)"
    );

    let effect = PropagatingEffect::Numerical(42.42);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::Numerical(42.42)"
    );

    let effect = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::Probabilistic(0.5)"
    );

    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let effect = PropagatingEffect::Tensor(tensor.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::Tensor({:?})", tensor)
    );

    let effect = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::ContextualLink(1, 2)"
    );

    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    let effect = PropagatingEffect::Map(map.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::Map({:?})", map)
    );

    let inner_effect = Box::new(PropagatingEffect::Deterministic(true));
    let effect = PropagatingEffect::RelayTo(1, inner_effect.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::RelayTo(1, {:?})", inner_effect)
    );

    let uncertain_bool = Uncertain::<bool>::point(true);
    let effect = UncertainBool(uncertain_bool.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::UncertainBool({:?})", uncertain_bool)
    );

    let uncertain_float = Uncertain::<f64>::point(3.0);
    let effect = UncertainFloat(uncertain_float.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::UncertainFloat({:?})", uncertain_float)
    );

    let maybe_uncertain_bool = MaybeUncertain::<bool>::from_value(true);
    let effect = MaybeUncertainBool(maybe_uncertain_bool.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!(
            "PropagatingEffect::MaybeUncertainBool({:?})",
            maybe_uncertain_bool
        )
    );

    let maybe_uncertain_float = MaybeUncertain::<f64>::from_value(2.71);
    let effect = MaybeUncertainFloat(maybe_uncertain_float.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!(
            "PropagatingEffect::MaybeUncertainFloat({:?})",
            maybe_uncertain_float
        )
    );
}
