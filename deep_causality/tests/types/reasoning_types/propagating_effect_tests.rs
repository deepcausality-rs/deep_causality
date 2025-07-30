/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
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
fn test_is_contextual_link() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert!(!effect1.is_contextual_link());

    let effect2 = PropagatingEffect::Probabilistic(0.5);
    assert!(!effect2.is_contextual_link());

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert!(effect3.is_contextual_link());
}

#[test]
fn test_is_halting() {
    let effect = PropagatingEffect::Halting;
    assert!(effect.is_halting());
}

#[test]
fn test_as_bool() {
    let effect1 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect1.as_bool(), Some(true));

    let effect2 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect2.as_bool(), Some(false));

    let effect3 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect3.as_bool(), None);

    let effect4 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect4.as_bool(), None);
}

#[test]
fn test_as_probability() {
    let effect1 = PropagatingEffect::Probabilistic(0.5);
    assert_eq!(effect1.as_probability(), Some(0.5));

    let effect2 = PropagatingEffect::Deterministic(true);
    assert_eq!(effect2.as_probability(), None);

    let effect3 = PropagatingEffect::ContextualLink(1, 1);
    assert_eq!(effect3.as_probability(), None);
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

    let effect4 = PropagatingEffect::Halting;
    assert_eq!(format!("{effect4}"), "PropagatingEffect::Halting");
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

    let effect4 = PropagatingEffect::Halting;
    assert_eq!(format!("{effect4:?}"), "PropagatingEffect::Halting");
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
    let effect1 = PropagatingEffect::None;
    let effect2 = PropagatingEffect::None;
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let effect1 = PropagatingEffect::Deterministic(true);
    let effect2 = PropagatingEffect::Deterministic(true);
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let effect1 = PropagatingEffect::Numerical(1.0);
    let effect2 = PropagatingEffect::Numerical(1.0);
    let effect3 = PropagatingEffect::Numerical(23.0);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let effect4 = PropagatingEffect::Probabilistic(0.5);
    let effect5 = PropagatingEffect::Probabilistic(0.5);
    let effect6 = PropagatingEffect::Probabilistic(0.6);
    assert_eq!(effect4, effect5);
    assert_ne!(effect4, effect6);

    let effect7 = PropagatingEffect::ContextualLink(1, 2);
    let effect8 = PropagatingEffect::ContextualLink(1, 2);
    let effect9 = PropagatingEffect::ContextualLink(2, 1);
    assert_eq!(effect7, effect8);
    assert_ne!(effect7, effect9);

    assert_ne!(effect1, effect4);
    assert_ne!(effect1, effect7);
    assert_ne!(effect4, effect7);

    let map1 = HashMap::new();

    let effect1 = PropagatingEffect::Map(map1.clone());
    let effect2 = PropagatingEffect::Map(map1);
    let effect3 = PropagatingEffect::None;
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let graph = Arc::new(UltraGraph::new());
    let effect1 = PropagatingEffect::Graph(graph.clone());
    let effect2 = PropagatingEffect::Graph(graph.clone());
    let effect3 = PropagatingEffect::None;
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let effect1 = PropagatingEffect::Halting;
    let effect2 = PropagatingEffect::Halting;
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);

    let effect7 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::None));
    let effect8 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::None));
    let effect9 = PropagatingEffect::None;
    assert_eq!(effect7, effect8);
    assert_ne!(effect7, effect9);
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
