/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::PropagatingEffect;
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::{GraphView, UltraGraph};

#[test]
fn test_debug_none() {
    let effect = PropagatingEffect::None;
    assert_eq!(format!("{:?}", effect), "PropagatingEffect::None");
}

#[test]
fn test_debug_deterministic() {
    let effect = PropagatingEffect::Deterministic(true);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::Deterministic(true)"
    );
}

#[test]
fn test_debug_numerical() {
    let effect = PropagatingEffect::Numerical(42.42);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::Numerical(42.42)"
    );
}

#[test]
fn test_debug_probabilistic() {
    let effect = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::Probabilistic(0.5)"
    );
}

#[test]
fn test_debug_tensor() {
    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let effect = PropagatingEffect::Tensor(tensor.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::Tensor({:?})", tensor)
    );
}

#[test]
fn test_debug_complex_tensor() {
    let complex_tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1]).unwrap();
    let effect = PropagatingEffect::ComplexTensor(complex_tensor.clone());
    assert_eq!(format!("{:?}", effect), "PropagatingEffect::ComplexTensor");
}

#[test]
fn test_debug_contextual_link() {
    let effect = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(
        format!("{:?}", effect),
        "PropagatingEffect::ContextualLink(1, 2)"
    );
}

#[test]
fn test_debug_map() {
    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    let effect = PropagatingEffect::Map(map.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::Map({:?})", map)
    );
}

#[test]
fn test_debug_graph() {
    let graph = Arc::new(UltraGraph::new());
    let effect = PropagatingEffect::Graph(graph.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!(
            "PropagatingEffect::Graph(nodes: {}, edges: {})",
            graph.number_nodes(),
            graph.number_edges()
        )
    );
}

#[test]
fn test_debug_relay_to() {
    let inner_effect = Box::new(PropagatingEffect::Deterministic(true));
    let effect = PropagatingEffect::RelayTo(1, inner_effect.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::RelayTo(1, {:?})", inner_effect)
    );
}

#[test]
fn test_debug_uncertain_bool() {
    let uncertain_bool = Uncertain::<bool>::point(true);
    let effect = PropagatingEffect::UncertainBool(uncertain_bool.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::UncertainBool({:?})", uncertain_bool)
    );
}

#[test]
fn test_debug_uncertain_float() {
    let uncertain_float = Uncertain::<f64>::point(3.0);
    let effect = PropagatingEffect::UncertainFloat(uncertain_float.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!("PropagatingEffect::UncertainFloat({:?})", uncertain_float)
    );
}

#[test]
fn test_debug_maybe_uncertain_bool() {
    let maybe_uncertain_bool = MaybeUncertain::<bool>::from_value(true);
    let effect = PropagatingEffect::MaybeUncertainBool(maybe_uncertain_bool.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!(
            "PropagatingEffect::MaybeUncertainBool({:?})",
            maybe_uncertain_bool
        )
    );
}

#[test]
fn test_debug_maybe_uncertain_float() {
    let maybe_uncertain_float = MaybeUncertain::<f64>::from_value(2.71);
    let effect = PropagatingEffect::MaybeUncertainFloat(maybe_uncertain_float.clone());
    assert_eq!(
        format!("{:?}", effect),
        format!(
            "PropagatingEffect::MaybeUncertainFloat({:?})",
            maybe_uncertain_float
        )
    );
}
