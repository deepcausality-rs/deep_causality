/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unit tests for the `LinearJoin` kernel: `bias + Σ weights[p]·v_p`.

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, LinearJoin, MonadicCausableGraphReasoning,
    ParentEffects, PropagatingEffect, linear_join,
};
use deep_causality_core::CausalEffect;
use deep_causality_num::Dual;
use std::collections::BTreeMap;

fn parents_f64(entries: &[(usize, f64)]) -> ParentEffects<f64> {
    let mut m: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    for (k, v) in entries {
        m.insert(*k, PropagatingEffect::from_value(*v));
    }
    ParentEffects::new(m)
}

fn config_f64(weights: &[(usize, f64)], bias: f64) -> LinearJoin<f64> {
    LinearJoin::new(weights.iter().copied().collect(), bias)
}

#[test]
fn linear_join_weighted_sum() {
    // 1.0 + 2.0·x1 + 3.0·x2 with x1 = 1.0, x2 = 10.0  ->  1 + 2 + 30 = 33.
    let parents = parents_f64(&[(1, 1.0), (2, 10.0)]);
    let config = config_f64(&[(1, 2.0), (2, 3.0)], 1.0);
    let res = linear_join(&parents, Some(&config));
    assert_eq!(res.value(), Some(&33.0));
}

#[test]
fn linear_join_surgery_locality() {
    // Removing parent 1's wire drops exactly its term `weights[1]·v1` from the result.
    let config = config_f64(&[(1, 2.0), (2, 3.0)], 1.0);

    let full = linear_join(&parents_f64(&[(1, 1.0), (2, 10.0)]), Some(&config));
    let cut = linear_join(&parents_f64(&[(2, 10.0)]), Some(&config));

    let full_v = *full.value().unwrap();
    let cut_v = *cut.value().unwrap();
    // full − cut == weights[1]·v1 == 2.0·1.0 == 2.0.
    assert_eq!(full_v - cut_v, 2.0);
}

#[test]
fn linear_join_missing_weight_contributes_nothing() {
    // Parent 2 has no weight entry -> contributes nothing. Result = 1 + 2·1 = 3.
    let parents = parents_f64(&[(1, 1.0), (2, 10.0)]);
    let config = config_f64(&[(1, 2.0)], 1.0);
    let res = linear_join(&parents, Some(&config));
    assert_eq!(res.value(), Some(&3.0));
}

#[test]
fn linear_join_none_value_contributes_nothing() {
    // A `Pure(None)` parent contributes nothing. Result = 0 + 5·... only from parent 1.
    let mut m: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    m.insert(1, PropagatingEffect::from_value(2.0));
    m.insert(2, PropagatingEffect::from_effect(CausalEffect::none()));
    let parents = ParentEffects::new(m);
    let config = config_f64(&[(1, 5.0), (2, 7.0)], 0.0);
    let res = linear_join(&parents, Some(&config));
    // 0 + 5·2 + (nothing for parent 2) = 10.
    assert_eq!(res.value(), Some(&10.0));
}

#[test]
fn linear_join_command_parent_errors() {
    let mut m: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    m.insert(1, PropagatingEffect::from_value(1.0));
    m.insert(
        2,
        PropagatingEffect::from_effect(CausalEffect::relay_to(9, CausalEffect::value(0.0))),
    );
    let parents = ParentEffects::new(m);
    let config = config_f64(&[(1, 1.0), (2, 1.0)], 0.0);
    let res = linear_join(&parents, Some(&config));
    assert!(res.is_err());
    assert!(res.error().unwrap().to_string().contains("command"));
}

#[test]
fn linear_join_missing_config_errors() {
    let parents = parents_f64(&[(1, 1.0)]);
    let res = linear_join(&parents, None);
    assert!(res.is_err());
    assert!(
        res.error()
            .unwrap()
            .to_string()
            .contains("missing LinearJoin configuration")
    );
}

#[test]
fn linear_join_dual_sensitivity() {
    // Seed parent 1 as the differentiation variable; the ε channel of the output is the
    // sensitivity ∂output/∂v1 = weights[1].
    let mut m: BTreeMap<usize, PropagatingEffect<Dual<f64>>> = BTreeMap::new();
    m.insert(1, PropagatingEffect::from_value(Dual::variable(1.0)));
    m.insert(2, PropagatingEffect::from_value(Dual::constant(10.0)));
    let parents = ParentEffects::new(m);

    let mut weights: BTreeMap<usize, Dual<f64>> = BTreeMap::new();
    weights.insert(1, Dual::constant(2.0));
    weights.insert(2, Dual::constant(3.0));
    let config = LinearJoin::new(weights, Dual::constant(1.0));

    let res = linear_join(&parents, Some(&config));
    let out = res.value().unwrap();
    // value = 1 + 2·1 + 3·10 = 33; derivative w.r.t. parent 1 = weights[1] = 2.
    assert_eq!(out.value(), 33.0);
    assert_eq!(out.derivative(), 2.0);
}

#[test]
fn linear_join_diamond_via_engine() {
    // The kernel as a configured join on a real diamond: root(0)->A(1),B(2); A,B->C(3),
    // C = new_with_context_join(identity, linear_join, LinearJoin{1:2, 2:3, bias:1}).
    fn identity(x: f64) -> PropagatingEffect<f64> {
        PropagatingEffect::from_value(x)
    }
    fn add_one(x: f64) -> PropagatingEffect<f64> {
        PropagatingEffect::from_value(x + 1.0)
    }
    fn add_ten(x: f64) -> PropagatingEffect<f64> {
        PropagatingEffect::from_value(x + 10.0)
    }

    let config = config_f64(&[(1, 2.0), (2, 3.0)], 1.0);
    let mut g: CausaloidGraph<Causaloid<f64, f64, (), LinearJoin<f64>>> = CausaloidGraph::new(0);
    let root = g
        .add_causaloid(Causaloid::new(0, identity, "root"))
        .unwrap();
    let a = g.add_causaloid(Causaloid::new(1, add_one, "A")).unwrap();
    let b = g.add_causaloid(Causaloid::new(2, add_ten, "B")).unwrap();
    let c = g
        .add_causaloid(Causaloid::new_with_context_join(
            3,
            identity,
            linear_join,
            config,
            "C",
        ))
        .unwrap();
    g.add_edge(root, a).unwrap();
    g.add_edge(root, b).unwrap();
    g.add_edge(a, c).unwrap();
    g.add_edge(b, c).unwrap();
    g.freeze();

    // A(+1)=1, B(+10)=10; C = 1 + 2·1 + 3·10 = 33.
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok());
    assert_eq!(res.value(), Some(&33.0));
}
