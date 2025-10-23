use deep_causality::IdentificationValue;
use deep_causality::PropagatingEffect;
use deep_causality::PropagatingEffect::{
    MaybeUncertainBool, MaybeUncertainFloat, UncertainBool, UncertainFloat,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{MaybeUncertain, Uncertain};
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::UltraGraph;

#[test]
fn test_partial_eq_none() {
    let effect1 = PropagatingEffect::None;
    let effect2 = PropagatingEffect::None;
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);
}

#[test]
fn test_partial_eq_deterministic() {
    let effect1 = PropagatingEffect::Deterministic(true);
    let effect2 = PropagatingEffect::Deterministic(true);
    let effect3 = PropagatingEffect::Deterministic(false);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);
}

#[test]
fn test_partial_eq_numerical() {
    let effect1 = PropagatingEffect::Numerical(1.0);
    let effect2 = PropagatingEffect::Numerical(1.0);
    let effect3 = PropagatingEffect::Numerical(23.0);
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);
}

#[test]
fn test_partial_eq_probabilistic() {
    let effect4 = PropagatingEffect::Probabilistic(0.5);
    let effect5 = PropagatingEffect::Probabilistic(0.5);
    let effect6 = PropagatingEffect::Probabilistic(0.6);
    assert_eq!(effect4, effect5);
    assert_ne!(effect4, effect6);
}

#[test]
fn test_partial_eq_tensor() {
    let res = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]);
    assert!(res.is_ok());
    let tensor = res.unwrap();

    let effect1 = PropagatingEffect::Tensor(tensor.clone());
    let effect2 = PropagatingEffect::Tensor(tensor);

    assert_eq!(effect1, effect2);
}

#[test]
fn test_partial_eq_complex_tensor() {
    use deep_causality_num::Complex;
    let res = CausalTensor::new(
        vec![Complex::new(1.0, 2.0), Complex::new(3.0, 4.0)],
        vec![2],
    );
    assert!(res.is_ok());
    let complex_tensor = res.unwrap();

    let effect1 = PropagatingEffect::ComplexTensor(complex_tensor.clone());
    let effect2 = PropagatingEffect::ComplexTensor(complex_tensor);

    assert_eq!(effect1, effect2);
}

#[test]
fn test_partial_eq_contextual_link() {
    let effect7 = PropagatingEffect::ContextualLink(1, 2);
    let effect8 = PropagatingEffect::ContextualLink(1, 2);
    let effect9 = PropagatingEffect::ContextualLink(2, 1);
    assert_eq!(effect7, effect8);
    assert_ne!(effect7, effect9);
}

#[test]
fn test_partial_eq_uncertain_bool() {
    let effect10 = UncertainBool(Uncertain::<bool>::point(false));
    let effect11 = UncertainBool(Uncertain::<bool>::point(false));
    let effect12 = UncertainBool(Uncertain::<bool>::point(true));
    assert_eq!(effect10, effect11);
    assert_ne!(effect10, effect12);
}

#[test]
fn test_partial_eq_uncertain_float() {
    let effect13 = UncertainFloat(Uncertain::<f64>::point(1.0f64));
    let effect14 = UncertainFloat(Uncertain::<f64>::point(1.0f64));
    let effect15 = UncertainFloat(Uncertain::<f64>::point(0.0f64));
    assert_eq!(effect13, effect14);
    assert_ne!(effect13, effect15);
}

#[test]
fn test_partial_eq_maybe_uncertain_bool() {
    let point = MaybeUncertain::<bool>::from_value(true);

    let effect = MaybeUncertainBool(point.clone());
    assert!(effect.is_maybe_uncertain_bool());

    let effect1 = MaybeUncertainBool(point.clone());
    assert!(effect1.is_maybe_uncertain_bool());

    let point = MaybeUncertain::<bool>::always_none();
    let effect2 = MaybeUncertainBool(point);
    assert!(effect2.is_maybe_uncertain_bool());

    assert_eq!(effect, effect1);
    assert_ne!(effect1, effect2);
}

#[test]
fn test_partial_eq_maybe_uncertain_float() {
    let point = MaybeUncertain::<f64>::from_value(4.0f64);

    let effect = MaybeUncertainFloat(point.clone());
    assert!(effect.is_maybe_uncertain_float());

    let effect1 = MaybeUncertainFloat(point);
    assert!(effect1.is_maybe_uncertain_float());

    let point = MaybeUncertain::<f64>::always_none();
    let effect2 = MaybeUncertainFloat(point);
    assert!(effect2.is_maybe_uncertain_float());

    assert_eq!(effect, effect1);
    assert_ne!(effect1, effect2);
}

#[test]
fn test_partial_eq_map() {
    let mut map1 = HashMap::new();
    map1.insert(
        IdentificationValue::from(1u64),
        Box::new(PropagatingEffect::Numerical(1.0)),
    );
    map1.insert(
        IdentificationValue::from(2u64),
        Box::new(PropagatingEffect::Deterministic(true)),
    );
    let effect10 = PropagatingEffect::Map(map1.clone());

    let mut map2 = HashMap::new();
    map2.insert(
        IdentificationValue::from(1u64),
        Box::new(PropagatingEffect::Numerical(1.0)),
    );
    map2.insert(
        IdentificationValue::from(2u64),
        Box::new(PropagatingEffect::Deterministic(true)),
    );
    let effect11 = PropagatingEffect::Map(map2.clone());

    let mut map3 = HashMap::new();
    map3.insert(
        IdentificationValue::from(1u64),
        Box::new(PropagatingEffect::Numerical(1.0)),
    );
    map3.insert(
        IdentificationValue::from(3u64),
        Box::new(PropagatingEffect::Deterministic(false)),
    ); // Different key and value
    let effect12 = PropagatingEffect::Map(map3.clone());

    assert_eq!(effect10, effect11);
    assert_ne!(effect10, effect12);
}

#[test]
fn test_partial_eq_graph() {
    let graph1 = Arc::new(UltraGraph::new());
    let graph2 = Arc::new(UltraGraph::new());

    let effect13 = PropagatingEffect::Graph(Arc::clone(&graph1));
    let effect14 = PropagatingEffect::Graph(Arc::clone(&graph1)); // Same Arc
    let effect15 = PropagatingEffect::Graph(Arc::clone(&graph2)); // Different Arc, same content
    assert_eq!(effect13, effect14);
    assert_ne!(effect13, effect15);
}

#[test]
fn test_partial_eq_relay_to() {
    let effect19 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(true)));
    let effect20 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(true)));
    let effect21 = PropagatingEffect::RelayTo(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect22 = PropagatingEffect::RelayTo(1, Box::new(PropagatingEffect::Deterministic(false)));
    assert_eq!(effect19, effect20);
    assert_ne!(effect19, effect21);
    assert_ne!(effect19, effect22);
}
