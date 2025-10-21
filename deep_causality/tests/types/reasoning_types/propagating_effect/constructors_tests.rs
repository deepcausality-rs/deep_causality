/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{IdentificationValue, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::{GraphView, UltraGraph};

#[test]
fn test_none_variant() {
    let effect = PropagatingEffect::None;
    assert!(effect.is_none());
    assert_eq!(effect, PropagatingEffect::None);
    assert_ne!(effect, PropagatingEffect::Deterministic(true));
    assert_eq!(format!("{effect:?}"), "PropagatingEffect::None");
}

#[test]
fn test_default_variant() {
    let effect: PropagatingEffect = Default::default();
    assert!(effect.is_none());
    assert_eq!(effect, PropagatingEffect::None);
}

#[test]
fn test_from_deterministic() {
    let effect = PropagatingEffect::from_deterministic(true);
    assert!(matches!(effect, PropagatingEffect::Deterministic(true)));

    let effect = PropagatingEffect::from_deterministic(false);
    assert!(matches!(effect, PropagatingEffect::Deterministic(false)));
}

#[test]
fn test_from_numerical() {
    let effect = PropagatingEffect::from_numerical(1.0);
    assert!(matches!(effect, PropagatingEffect::Numerical(1.0)));

    let effect = PropagatingEffect::from_numerical(-10.5);
    assert!(matches!(effect, PropagatingEffect::Numerical(-10.5)));
}

#[test]
fn test_from_probabilistic() {
    let effect = PropagatingEffect::from_probabilistic(0.5);
    assert!(matches!(effect, PropagatingEffect::Probabilistic(0.5)));

    let effect = PropagatingEffect::from_probabilistic(0.99);
    assert!(matches!(effect, PropagatingEffect::Probabilistic(0.99)));
}

#[test]
fn test_from_tensor() -> Result<(), Box<dyn std::error::Error>> {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3])?;
    let effect = PropagatingEffect::from_tensor(tensor.clone());
    assert!(matches!(effect, PropagatingEffect::Tensor(_)));
    if let PropagatingEffect::Tensor(t) = effect {
        assert_eq!(t.data(), tensor.data());
    }
    Ok(())
}

#[test]
fn test_from_uncertain_bool() {
    let uncertain_bool = UncertainBool::point(true);
    let effect = PropagatingEffect::from_uncertain_bool(uncertain_bool.clone());
    assert!(matches!(effect, PropagatingEffect::UncertainBool(_)));
    if let PropagatingEffect::UncertainBool(ub) = effect {
        assert_eq!(ub, uncertain_bool);
    }
}

#[test]
fn test_from_uncertain_float() {
    let uncertain_float = UncertainF64::point(1.0);
    let effect = PropagatingEffect::from_uncertain_float(uncertain_float.clone());
    assert!(matches!(effect, PropagatingEffect::UncertainFloat(_)));
    if let PropagatingEffect::UncertainFloat(uf) = effect {
        assert_eq!(uf, uncertain_float);
    }
}

#[test]
fn test_from_maybe_uncertain_bool() {
    let maybe_uncertain_bool = MaybeUncertainBool::from_value(true);
    let effect = PropagatingEffect::from_maybe_uncertain_bool(maybe_uncertain_bool.clone());
    assert!(matches!(effect, PropagatingEffect::MaybeUncertainBool(_)));
    if let PropagatingEffect::MaybeUncertainBool(mub) = effect {
        assert_eq!(mub, maybe_uncertain_bool);
    }
}

#[test]
fn test_from_maybe_uncertain_float() {
    let maybe_uncertain_float = MaybeUncertainF64::from_value(1.0);
    let effect = PropagatingEffect::from_maybe_uncertain_float(maybe_uncertain_float.clone());
    assert!(matches!(effect, PropagatingEffect::MaybeUncertainFloat(_)));
    if let PropagatingEffect::MaybeUncertainFloat(muf) = effect {
        assert_eq!(muf, maybe_uncertain_float);
    }
}

#[test]
fn test_from_contextual_link() {
    let context_id = 1u64;
    let contextoid_id = 2u64;
    let effect = PropagatingEffect::from_contextual_link(context_id, contextoid_id);
    assert!(matches!(effect, PropagatingEffect::ContextualLink(1, 2)));
}

#[test]
fn test_new_map() {
    let effect = PropagatingEffect::new_map();
    assert!(matches!(effect, PropagatingEffect::Map(_)));
    if let PropagatingEffect::Map(map) = effect {
        assert!(map.is_empty());
    }
}

#[test]
fn test_from_map() {
    let mut initial_map = HashMap::new();
    initial_map.insert(
        IdentificationValue::from(1u64),
        Box::new(PropagatingEffect::Numerical(1.0)),
    );
    let effect = PropagatingEffect::from_map(initial_map.clone());
    assert!(matches!(effect, PropagatingEffect::Map(_)));
    if let PropagatingEffect::Map(map) = effect {
        assert_eq!(map.len(), 1);
        assert!(map.contains_key(&IdentificationValue::from(1u64)));
    }
}



#[test]
fn test_new_graph() {
    let effect = PropagatingEffect::new_graph();
    assert!(matches!(effect, PropagatingEffect::Graph(_)));
    assert!(effect.is_graph());

    let graph1 = Arc::new(UltraGraph::new());
    let graph2 = Arc::new(UltraGraph::new());
    let graph3 = Arc::new(UltraGraph::new());

    let effect1 = PropagatingEffect::Graph(Arc::clone(&graph1));
    let effect2 = PropagatingEffect::Graph(Arc::clone(&graph1)); // Same Arc
    let effect3 = PropagatingEffect::Graph(Arc::clone(&graph2)); // Different Arc, same content
    let effect4 = PropagatingEffect::Graph(Arc::clone(&graph3)); // Different Arc, different content

    assert!(effect1.is_graph());
    assert_eq!(effect1, effect2); // Should be equal due to Arc::ptr_eq
    assert_ne!(effect1, effect3); // Should be not equal due to Arc::ptr_eq
    assert_ne!(effect1, effect4);
    assert_eq!(
        format!("{effect1:?}"),
        format!(
            "PropagatingEffect::Graph(nodes: {}, edges: {})",
            graph1.number_nodes(),
            graph1.number_edges()
        )
    );
}

#[test]
fn test_from_graph() {
    let graph1 = Arc::new(UltraGraph::new());

    let effect1 = PropagatingEffect::from_graph(graph1);
    assert!(matches!(effect1, PropagatingEffect::Graph(_)));
}

#[test]
fn test_from_relay_to() {
    let effect_to_relay = Box::new(PropagatingEffect::Numerical(10.0));
    let effect = PropagatingEffect::from_relay_to(5, effect_to_relay.clone());
    assert!(matches!(effect, PropagatingEffect::RelayTo(5, _)));
    if let PropagatingEffect::RelayTo(id, inner_effect) = effect {
        assert_eq!(id, 5);
        assert_eq!(inner_effect, effect_to_relay);
    }
}

#[test]
fn test_relay_to_variant() {
    let effect1 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(true)));
    let effect2 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(true)));
    let effect3 = PropagatingEffect::RelayTo(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect4 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(false)));

    assert!(effect1.is_relay_to());
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);
    assert_ne!(effect1, effect4);
    assert_eq!(
        format!("{effect1:?}"),
        "PropagatingEffect::RelayTo(1, PropagatingEffect::Deterministic(true))"
    );
}
