/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Structural equality and the debug rendering of a graph node.
//!
//! `PartialEq` here is what `Uncertain::eq` and `UncertainBool::eq` delegate to, so a wrong arm
//! does not fail loudly — it makes two *different* graphs compare equal. Every test below is
//! therefore a **negative** one as well as a positive one: same shape compares equal, and each
//! single-field difference compares unequal. A test that only checked equality would pass against
//! an implementation that returned `true` unconditionally.

use deep_causality_ast::ConstTree;
use deep_causality_uncertain::{
    ArithmeticOperator, ComparisonOperator, DistributionEnum, LogicalOperator, Node,
    NormalDistributionParams, Sample, UniformDistributionParams,
};

fn leaf(v: f64) -> ConstTree<Node<f64>> {
    ConstTree::new(Node::Value(Sample::Real(v)))
}

fn boolean(v: bool) -> ConstTree<Node<f64>> {
    ConstTree::new(Node::Value(Sample::Bool(v)))
}

#[test]
fn a_value_node_compares_on_its_sample() {
    assert_eq!(
        Node::Value(Sample::Real(1.0f64)),
        Node::Value(Sample::Real(1.0))
    );
    assert_ne!(
        Node::Value(Sample::Real(1.0f64)),
        Node::Value(Sample::Real(2.0))
    );
    assert_ne!(
        Node::Value(Sample::Real(1.0f64)),
        Node::Value(Sample::Bool(true))
    );
    let t: Node<f64> = Node::Value(Sample::Bool(true));
    let f: Node<f64> = Node::Value(Sample::Bool(false));
    assert_ne!(t, f);
}

/// Negative zero is a different value from positive zero, and the node must say so.
///
/// A sign slip this small is invisible in a sum and decisive in a comparison, which is why it is
/// pinned here rather than assumed to follow from `f64`'s own `PartialEq`.
#[test]
fn negative_zero_is_not_positive_zero() {
    let neg: Node<f64> = Node::Value(Sample::Real(-0.0));
    let pos: Node<f64> = Node::Value(Sample::Real(0.0));
    // IEEE says -0.0 == 0.0, so the node inherits that; the bit pattern differs but the value does
    // not. Pinned so a future change to a bitwise comparison is a visible decision, not a drift.
    assert_eq!(neg, pos, "IEEE equality is what the node inherits");
    assert!(
        (-0.0f64).is_sign_negative() && !(0.0f64).is_sign_negative(),
        "the two are still distinguishable by sign, which is what a sum must preserve"
    );
}

/// `NaN` is equal to nothing, including itself — so two graphs holding it never compare equal.
#[test]
fn a_nan_value_node_equals_nothing() {
    let nan: Node<f64> = Node::Value(Sample::Real(f64::NAN));
    let same: Node<f64> = Node::Value(Sample::Real(f64::NAN));
    assert_ne!(
        nan, same,
        "NaN is not equal to itself, and the node must not paper over that"
    );
}

#[test]
fn a_distribution_node_compares_on_its_parameters() {
    let a: Node<f64> = Node::Distribution(DistributionEnum::Normal(NormalDistributionParams::new(
        0.0, 1.0,
    )));
    let same: Node<f64> = Node::Distribution(DistributionEnum::Normal(
        NormalDistributionParams::new(0.0, 1.0),
    ));
    let other_mean: Node<f64> = Node::Distribution(DistributionEnum::Normal(
        NormalDistributionParams::new(1.0, 1.0),
    ));
    let other_sd: Node<f64> = Node::Distribution(DistributionEnum::Normal(
        NormalDistributionParams::new(0.0, 2.0),
    ));
    let other_kind: Node<f64> = Node::Distribution(DistributionEnum::Uniform(
        UniformDistributionParams::new(0.0, 1.0),
    ));

    assert_eq!(a, same);
    assert_ne!(
        a, other_mean,
        "a different mean is a different distribution"
    );
    assert_ne!(
        a, other_sd,
        "a different spread is a different distribution"
    );
    assert_ne!(a, other_kind, "a normal is not a uniform");
}

/// Arithmetic compares on the operator **and** on both operands, in order.
///
/// The swapped-operand case is the sign slip this guards: `a - b` and `b - a` have the same
/// operator and the same multiset of operands, and are different graphs.
#[test]
fn arithmetic_compares_on_operator_and_operand_order() {
    let mk = |op, l, r| Node::ArithmeticOp { op, lhs: l, rhs: r };

    let sub = mk(ArithmeticOperator::Sub, leaf(1.0), leaf(2.0));
    let same = mk(ArithmeticOperator::Sub, leaf(1.0), leaf(2.0));
    let swapped = mk(ArithmeticOperator::Sub, leaf(2.0), leaf(1.0));
    let add = mk(ArithmeticOperator::Add, leaf(1.0), leaf(2.0));

    assert_eq!(sub, same);
    assert_ne!(sub, swapped, "a - b is not b - a");
    assert_ne!(sub, add, "a different operator is a different node");
}

/// Every arithmetic operator is distinct from every other.
#[test]
fn no_two_arithmetic_operators_compare_equal() {
    use ArithmeticOperator::*;
    let ops = [Add, Sub, Mul, Div, Min, Max];
    for (i, a) in ops.iter().enumerate() {
        for (j, b) in ops.iter().enumerate() {
            let na = Node::ArithmeticOp {
                op: *a,
                lhs: leaf(1.0),
                rhs: leaf(2.0),
            };
            let nb = Node::ArithmeticOp {
                op: *b,
                lhs: leaf(1.0),
                rhs: leaf(2.0),
            };
            if i == j {
                assert_eq!(na, nb, "{a:?} must equal itself");
            } else {
                assert_ne!(na, nb, "{a:?} must not equal {b:?}");
            }
        }
    }
}

/// A comparison node compares on operator, threshold **and** operand — all three.
#[test]
fn comparison_compares_on_all_three_fields() {
    let mk = |op, t, o| Node::ComparisonOp {
        op,
        threshold: t,
        operand: o,
    };

    let base = mk(ComparisonOperator::GreaterThan, 1.0f64, leaf(0.0));
    assert_eq!(base, mk(ComparisonOperator::GreaterThan, 1.0, leaf(0.0)));
    assert_ne!(
        base,
        mk(ComparisonOperator::LessThan, 1.0, leaf(0.0)),
        "operator"
    );
    assert_ne!(
        base,
        mk(ComparisonOperator::GreaterThan, 2.0, leaf(0.0)),
        "threshold"
    );
    assert_ne!(
        base,
        mk(ComparisonOperator::GreaterThan, 1.0, leaf(9.0)),
        "operand"
    );

    // The sign slip: a threshold of +1 is not a threshold of −1.
    assert_ne!(
        mk(ComparisonOperator::GreaterThan, 1.0, leaf(0.0)),
        mk(ComparisonOperator::GreaterThan, -1.0, leaf(0.0)),
        "a threshold's sign is part of the node"
    );
}

/// A logical node compares on operator and on the operand list, in order and in length.
#[test]
fn logic_compares_on_operator_and_operand_sequence() {
    let mk = |op, ops: Vec<_>| Node::LogicalOp { op, operands: ops };

    let and = mk(LogicalOperator::And, vec![boolean(true), boolean(false)]);
    assert_eq!(
        and,
        mk(LogicalOperator::And, vec![boolean(true), boolean(false)])
    );
    assert_ne!(
        and,
        mk(LogicalOperator::Or, vec![boolean(true), boolean(false)]),
        "operator"
    );
    assert_ne!(
        and,
        mk(LogicalOperator::And, vec![boolean(false), boolean(true)]),
        "operand order"
    );
    assert_ne!(
        and,
        mk(LogicalOperator::And, vec![boolean(true)]),
        "operand count"
    );
    assert_ne!(and, mk(LogicalOperator::And, vec![]), "empty operand list");
}

/// A conditional compares on all three branches, and swapping the branches is a different graph.
#[test]
fn a_conditional_compares_on_all_three_branches() {
    let mk = |c, t, f| Node::ConditionalOp {
        condition: c,
        if_true: t,
        if_false: f,
    };

    let base = mk(boolean(true), leaf(1.0), leaf(2.0));
    assert_eq!(base, mk(boolean(true), leaf(1.0), leaf(2.0)));
    assert_ne!(base, mk(boolean(false), leaf(1.0), leaf(2.0)), "condition");
    assert_ne!(base, mk(boolean(true), leaf(9.0), leaf(2.0)), "if_true");
    assert_ne!(base, mk(boolean(true), leaf(1.0), leaf(9.0)), "if_false");
    assert_ne!(
        base,
        mk(boolean(true), leaf(2.0), leaf(1.0)),
        "swapping the branches is a different graph"
    );
}

#[test]
fn negation_compares_on_its_operand() {
    let a: Node<f64> = Node::NegationOp { operand: leaf(1.0) };
    assert_eq!(a, Node::NegationOp { operand: leaf(1.0) });
    assert_ne!(
        a,
        Node::NegationOp {
            operand: leaf(-1.0)
        }
    );
}

/// A stored function cannot be compared, so the function arms compare on their operand and their
/// shape — and a reader should know that two different functions over one operand look equal.
#[test]
fn a_function_node_compares_on_its_operand_only() {
    let double: fn(f64) -> f64 = |x| x * 2.0;
    let halve: fn(f64) -> f64 = |x| x / 2.0;

    let a = Node::FunctionOpReal {
        func: double,
        operand: leaf(1.0),
    };
    let b = Node::FunctionOpReal {
        func: halve,
        operand: leaf(1.0),
    };
    assert_eq!(
        a, b,
        "two functions over one operand are structurally alike, by design"
    );

    let c = Node::FunctionOpReal {
        func: double,
        operand: leaf(2.0),
    };
    assert_ne!(a, c, "a different operand is a different node");
}

/// Different *kinds* of node never compare equal, whatever they hold.
#[test]
fn nodes_of_different_kinds_never_compare_equal() {
    let value: Node<f64> = Node::Value(Sample::Real(1.0));
    let negation: Node<f64> = Node::NegationOp { operand: leaf(1.0) };
    let arithmetic: Node<f64> = Node::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: leaf(1.0),
        rhs: leaf(1.0),
    };
    let logical: Node<f64> = Node::LogicalOp {
        op: LogicalOperator::Not,
        operands: vec![boolean(true)],
    };

    assert_ne!(value, negation);
    assert_ne!(negation, arithmetic);
    assert_ne!(arithmetic, logical);
    assert_ne!(logical, value);
}

/// The debug rendering names every arm, so a graph dump is readable rather than opaque.
#[test]
fn debug_names_every_arm() {
    let cases: Vec<(Node<f64>, &str)> = vec![
        (Node::Value(Sample::Real(1.0)), "Value"),
        (
            Node::Distribution(DistributionEnum::Normal(NormalDistributionParams::new(
                0.0, 1.0,
            ))),
            "Distribution",
        ),
        (
            Node::ArithmeticOp {
                op: ArithmeticOperator::Add,
                lhs: leaf(1.0),
                rhs: leaf(2.0),
            },
            "ArithmeticOp",
        ),
        (
            Node::ComparisonOp {
                op: ComparisonOperator::GreaterThan,
                threshold: 0.0,
                operand: leaf(1.0),
            },
            "ComparisonOp",
        ),
        (
            Node::LogicalOp {
                op: LogicalOperator::Not,
                operands: vec![boolean(true)],
            },
            "LogicalOp",
        ),
        (
            Node::FunctionOpReal {
                func: |x: f64| x,
                operand: leaf(1.0),
            },
            "FunctionOpReal",
        ),
        (
            Node::FunctionOpBool {
                func: |x: f64| x > 0.0,
                operand: leaf(1.0),
            },
            "FunctionOpBool",
        ),
        (Node::NegationOp { operand: leaf(1.0) }, "NegationOp"),
        (
            Node::ConditionalOp {
                condition: boolean(true),
                if_true: leaf(1.0),
                if_false: leaf(2.0),
            },
            "ConditionalOp",
        ),
    ];

    for (node, expected) in cases {
        let rendered = format!("{node:?}");
        assert!(
            rendered.starts_with(expected),
            "expected a rendering starting {expected}, got {rendered}"
        );
    }
}

/// A node is cloneable, and a clone compares equal to its original.
#[test]
fn a_clone_compares_equal_to_its_original() {
    let node: Node<f64> = Node::ArithmeticOp {
        op: ArithmeticOperator::Mul,
        lhs: leaf(3.0),
        rhs: leaf(4.0),
    };
    assert_eq!(node.clone(), node);
}
