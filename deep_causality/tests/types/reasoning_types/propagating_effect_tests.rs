/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::PropagatingEffect::{UncertainBool, UncertainFloat};
use deep_causality::*;
use deep_causality_uncertain::Uncertain;
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::{GraphView, UltraGraph};

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
fn test_is_uncertain_bool() {
    let point = Uncertain::<bool>::point(true);
    let effect1 = UncertainBool(point);
    assert!(effect1.is_uncertain_bool());

    // Ensure its not float
    assert!(!effect1.is_uncertain_float());
}

#[test]
fn test_uncertain_float() {
    let point = Uncertain::<f64>::point(4.0f64);
    let effect1 = UncertainFloat(point);
    assert!(effect1.is_uncertain_float());

    // Ensure its not bool
    assert!(!effect1.is_uncertain_bool());
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

    let point = Uncertain::<bool>::point(true);
    let effect4 = UncertainBool(point);
    assert_eq!(effect4.as_bool(), None);

    let point = Uncertain::<f64>::point(4.0f64);
    let effect5 = UncertainFloat(point);
    assert_eq!(effect5.as_bool(), None);

    let effect6 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect6.as_bool(), None);
}

#[test]
fn test_as_probability() {
    let effect1 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect1.as_probability(), Some(0.5));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_probability(), None);

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_probability(), None);

    let point = Uncertain::<bool>::point(true);
    let effect4 = UncertainBool(point);
    assert_eq!(effect4.as_probability(), None);

    let point = Uncertain::<f64>::point(4.0f64);
    let effect5 = UncertainFloat(point);
    assert_eq!(effect5.as_probability(), None);
}

#[test]
fn test_as_uncertain_bool() {
    let point = Uncertain::<bool>::point(true);
    let effect1 = UncertainBool(point.clone());
    assert_eq!(effect1.as_uncertain_bool(), Some(point));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_uncertain_bool(), None);

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_uncertain_bool(), None);
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
    assert_eq!(
        format!("{effect4}"),
        "PropagatingEffect::UncertainBool(Uncertain { id: 21, root_node: LeafBool { node_id: NodeId(21), dist: Point(true) }, _phantom: PhantomData<bool> })"
    );

    let point = Uncertain::<f64>::point(4.0f64);
    let effect5 = UncertainFloat(point);
    assert_eq!(
        format!("{effect5}"),
        "PropagatingEffect::UncertainFloat(Uncertain { id: 22, root_node: LeafF64 { node_id: NodeId(22), dist: Point(4.0) }, _phantom: PhantomData<f64> })"
    );
}

#[test]
fn test_debug() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(
        format!("{effect1:?}"),
        "PropagatingEffect::Deterministic(true)"
    );

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(
        format!("{effect2:?}"),
        "PropagatingEffect::Probabilistic(0.5)"
    );

    let effect3 = PropagatingEffect::ContextualLink(1, 2);
    assert_eq!(
        format!("{effect3:?}"),
        "PropagatingEffect::ContextualLink(1, 2)"
    );
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
fn test_partial_eq() {
    // None variant
    let effect1 = PropagatingEffect::None;
    let effect2 = PropagatingEffect::None;
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    // Deterministic variant
    let effect1 = PropagatingEffect::Deterministic(true);
    let effect2 = PropagatingEffect::Deterministic(true);
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    // Numerical variant
    let effect1 = PropagatingEffect::Numerical(1.0);
    let effect2 = PropagatingEffect::Numerical(1.0);
    let effect3 = PropagatingEffect::Numerical(23.0);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    // Probabilistic variant
    let effect4 = PropagatingEffect::Probabilistic(0.5);
    let effect5 = PropagatingEffect::Probabilistic(0.5);
    let effect6 = PropagatingEffect::Probabilistic(0.6);
    assert_eq!(effect4, effect5);
    assert_ne!(effect4, effect6);

    // ContextualLink variant
    let effect7 = PropagatingEffect::ContextualLink(1, 2);
    let effect8 = PropagatingEffect::ContextualLink(1, 2);
    let effect9 = PropagatingEffect::ContextualLink(2, 1);
    assert_eq!(effect7, effect8);
    assert_ne!(effect7, effect9);

    // Map variant
    let mut map1 = HashMap::new();
    map1.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map1.insert(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect10 = PropagatingEffect::Map(map1.clone());

    let mut map2 = HashMap::new();
    map2.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map2.insert(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect11 = PropagatingEffect::Map(map2.clone());

    let mut map3 = HashMap::new();
    map3.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map3.insert(3, Box::new(PropagatingEffect::Deterministic(false))); // Different key and value
    let effect12 = PropagatingEffect::Map(map3.clone());

    assert_eq!(effect10, effect11);
    assert_ne!(effect10, effect12);

    // Graph variant
    let graph1 = Arc::new(UltraGraph::new());
    let graph2 = Arc::new(UltraGraph::new());
    // let graph3 = Arc::new(UltraGraph::new());

    let effect13 = PropagatingEffect::Graph(Arc::clone(&graph1));
    let effect14 = PropagatingEffect::Graph(Arc::clone(&graph1)); // Same Arc
    let effect15 = PropagatingEffect::Graph(Arc::clone(&graph2)); // Different Arc, same content
    assert_eq!(effect13, effect14); // Should be equal due to Arc::ptr_eq
    assert_ne!(effect13, effect15); // Should be not equal due to Arc::ptr_eq

    // RelayTo variant
    let effect19 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(true)));
    let effect20 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(true)));
    let effect21 = PropagatingEffect::RelayTo(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect22 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(false)));
    assert_eq!(effect19, effect20);
    assert_ne!(effect19, effect21);
    assert_ne!(effect19, effect22);
}

#[test]
fn test_none_variant() {
    let effect = PropagatingEffect::None;
    assert!(effect.is_none());
    assert_eq!(effect, PropagatingEffect::None);
    assert_ne!(effect, PropagatingEffect::Deterministic(true));
    assert_eq!(format!("{effect:?}"), "PropagatingEffect::None");
}

#[test]
fn test_numerical_variant() {
    let effect = PropagatingEffect::Numerical(123.45);
    assert!(effect.is_numerical());
    assert_eq!(effect.as_numerical(), Some(123.45));
    assert_eq!(effect, PropagatingEffect::Numerical(123.45));
    assert_ne!(effect, PropagatingEffect::Numerical(543.21));
    assert_eq!(
        format!("{effect:?}"),
        "PropagatingEffect::Numerical(123.45)"
    );
}

#[test]
fn test_map_variant() {
    let mut map1 = HashMap::new();
    map1.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map1.insert(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect1 = PropagatingEffect::Map(map1.clone());

    let mut map2 = HashMap::new();
    map2.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map2.insert(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect2 = PropagatingEffect::Map(map2.clone());

    let mut map3 = HashMap::new();
    map3.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map3.insert(3, Box::new(PropagatingEffect::Deterministic(false))); // Different key and value
    let effect3 = PropagatingEffect::Map(map3.clone());

    assert!(effect1.is_map());
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);
    assert_eq!(
        format!("{effect1:?}"),
        format!("PropagatingEffect::Map({:?})", map1)
    );

    // Test map specific methods
    let mut new_map = PropagatingEffect::new_map();
    new_map.insert(10, PropagatingEffect::Numerical(100.0));
    assert_eq!(new_map.get_numerical_from_map(10).unwrap(), 100.0);
    assert!(new_map.get_numerical_from_map(99).is_err());

    new_map.insert(11, PropagatingEffect::Deterministic(true));
    assert!(new_map.get_deterministic_from_map(11).unwrap());
    assert!(new_map.get_deterministic_from_map(99).is_err());
}

#[test]
#[should_panic(expected = "Cannot insert into PropagatingEffect that is not a Map variant")]
fn test_insert_panic() {
    let mut effect = PropagatingEffect::None;
    effect.insert(1, PropagatingEffect::Numerical(1.0));
}

#[test]
fn test_get_numerical_from_map_error_not_map() {
    let effect = PropagatingEffect::None;
    let result = effect.get_numerical_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Cannot get value by key from PropagatingEffect that is not a Map variant"
    );
}

#[test]
fn test_get_numerical_from_map_error_wrong_type() {
    let mut map = PropagatingEffect::new_map();
    map.insert(1, PropagatingEffect::Deterministic(true));
    let result = map.get_numerical_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Effect for key '1' is not of type Numerical"
    );
}

#[test]
fn test_get_deterministic_from_map_error_not_map() {
    let effect = PropagatingEffect::None;
    let result = effect.get_deterministic_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Cannot get value by key from PropagatingEffect that is not a Map variant"
    );
}

#[test]
fn test_get_deterministic_from_map_error_wrong_type() {
    let mut map = PropagatingEffect::new_map();
    map.insert(1, PropagatingEffect::Numerical(1.0));
    let result = map.get_deterministic_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Effect for key '1' is not of type Deterministic"
    );
}

#[test]
fn test_graph_variant() {
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

#[test]
fn test_default_variant() {
    let effect: PropagatingEffect = Default::default();
    assert!(effect.is_none());
    assert_eq!(effect, PropagatingEffect::None);
}
