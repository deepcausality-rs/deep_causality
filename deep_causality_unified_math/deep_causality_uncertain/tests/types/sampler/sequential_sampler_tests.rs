/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;
use deep_causality_num::Float106;
use deep_causality_uncertain::{
    ArithmeticOperator, BernoulliParams, ComparisonOperator, DistributionEnum, LogicalOperator,
    Node, NormalDistributionParams, Sample, Sampler, SequentialSampler, UncertainError,
    UniformDistributionParams,
};
use std::sync::Arc;

// Corrected helper functions
type UncertainNode<R> = ConstTree<Node<R>>;

// Helper to create a root node for a graph
fn create_root<R>(content: Node<R>) -> UncertainNode<R> {
    ConstTree::new(content)
}

// Helper to create a non-root node
fn create_node<R>(content: Node<R>) -> UncertainNode<R> {
    ConstTree::new(content)
}

#[test]
fn test_sample_value() {
    let sampler = SequentialSampler;

    // Test with f64 value
    let node_f64 = create_root(Node::Value(Sample::Real(42.0f64)));
    let result_f64 = Sampler::<f64>::sample(&sampler, &node_f64, 0).unwrap();
    assert_eq!(result_f64, Sample::Real(42.0));

    // Test with bool value
    let node_bool = create_root(Node::Value(Sample::Bool(true)));
    let result_bool = Sampler::<f64>::sample(&sampler, &node_bool, 0).unwrap();
    assert_eq!(result_bool, Sample::Bool(true));
}

#[test]
fn test_distribution_f64() {
    let sampler = SequentialSampler;

    // Point
    let node = create_root(Node::Distribution(DistributionEnum::Point(42.0)));
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(42.0)
    );

    // Normal
    let data = NormalDistributionParams::new(0.0, 1.0);
    let node = create_root(Node::Distribution(DistributionEnum::Normal(data)));
    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert!(matches!(result, Sample::Real(_)));

    // Uniform
    let data = UniformDistributionParams::new(0.0, 1.0);
    let node = create_root(Node::Distribution(DistributionEnum::Uniform(data)));
    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert!(matches!(result, Sample::Real(_)));
}

#[test]
fn test_distribution_bool() {
    let sampler = SequentialSampler;

    // A certain truth value is a `Value` node, not a point *distribution*. `Point` carries the
    // graph's scalar, and a Boolean is not one — which is the whole reason the Boolean case is a
    // branch of the sample rather than an instantiation of the distribution.
    let node: UncertainNode<f64> = create_root(Node::Value(Sample::Bool(true)));
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // Bernoulli
    let data = BernoulliParams::new(0.5);
    let node = create_root(Node::Distribution(DistributionEnum::Bernoulli(data)));
    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert!(matches!(result, Sample::Bool(_)));
}

#[test]
fn test_pure_op() {
    let sampler = SequentialSampler;
    let node = create_root(Node::PureOp {
        value: Sample::Real(42.0),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(42.0)
    );
}

#[test]
fn test_fmap_op() {
    let sampler = SequentialSampler;
    let operand = create_node(Node::Value(Sample::Real(10.0f64)));
    let func = Arc::new(|val: Sample<f64>| match val {
        Sample::Real(v) => Sample::Real(v * 2.0),
        _ => panic!("unexpected type"),
    });

    let node = create_root(Node::FmapOp { func, operand });

    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert_eq!(result, Sample::Real(20.0));
}

#[test]
fn test_apply_op() {
    let sampler = SequentialSampler;
    let arg = create_node(Node::Value(Sample::Real(10.0f64)));
    let func = Arc::new(|val: Sample<f64>| match val {
        Sample::Real(v) => Sample::Real(v + 5.0),
        _ => panic!("unexpected type"),
    });

    let node = create_root(Node::ApplyOp { func, arg });

    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert_eq!(result, Sample::Real(15.0));
}

#[test]
fn test_bind_op() {
    let sampler = SequentialSampler;
    let operand = create_node(Node::Value(Sample::Real(10.0f64)));
    let func = Arc::new(|val: Sample<f64>| {
        let inner_val = match val {
            Sample::Real(v) => v,
            _ => panic!("unexpected type"),
        };
        create_node(Node::Value(Sample::Real(inner_val * 2.0)))
    });

    let node = create_root(Node::BindOp { func, operand });

    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert_eq!(result, Sample::Real(20.0));
}

#[test]
fn test_arithmetic_op() {
    let sampler = SequentialSampler;
    let lhs = create_node(Node::Value(Sample::Real(10.0f64)));
    let rhs = create_node(Node::Value(Sample::Real(5.0f64)));

    // Add
    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(15.0)
    );

    // Sub
    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Sub,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(5.0)
    );

    // Mul
    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Mul,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(50.0)
    );

    // Div
    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Div,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(2.0)
    );

    // Error case
    let bool_node = create_node(Node::Value(Sample::Bool(true)));
    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: bool_node,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_comparison_op() {
    let sampler = SequentialSampler;
    let operand = create_node(Node::Value(Sample::Real(10.0f64)));

    // EqualTo
    let node = create_root(Node::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 10.0,
        operand: operand.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // GreaterThan
    let node = create_root(Node::ComparisonOp {
        op: ComparisonOperator::GreaterThan,
        threshold: 5.0,
        operand: operand.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // LessThan
    let node = create_root(Node::ComparisonOp {
        op: ComparisonOperator::LessThan,
        threshold: 15.0,
        operand: operand.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // Error case
    let bool_node = create_node(Node::Value(Sample::Bool(true)));
    let node = create_root(Node::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.0, // dummy
        operand: bool_node,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_logical_op() {
    let sampler = SequentialSampler;
    let true_node = create_node(Node::Value(Sample::Bool(true)));
    let false_node = create_node(Node::Value(Sample::Bool(false)));

    // Not
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::Not,
        operands: vec![true_node.clone()],
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(false)
    );

    // And
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(false)
    );

    // Or
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::Or,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // XOR
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::XOR,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // NOR
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::NOR,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(false)
    );
}

#[test]
fn test_logical_op_errors() {
    let sampler = SequentialSampler;
    let true_node = create_node(Node::Value(Sample::Bool(true)));
    let float_node = create_node(Node::Value(Sample::Real(42.0f64)));

    // NOT with 2 operands
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::Not,
        operands: vec![true_node.clone(), true_node.clone()],
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    // AND with 1 operand
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![true_node.clone()],
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    // Wrong input type
    let node = create_root(Node::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![true_node.clone(), float_node],
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_function_op_f64() {
    let sampler = SequentialSampler;
    let operand = create_node(Node::Value(Sample::Real(10.0f64)));
    let func: fn(_) -> _ = |x: f64| x.powi(2);

    let node = create_root(Node::FunctionOpReal { func, operand });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(100.0)
    );

    // Error case
    let bool_node = create_node(Node::Value(Sample::Bool(true)));
    let func: fn(_) -> _ = |x: f64| x.powi(2);
    let node = create_root(Node::FunctionOpReal {
        func,
        operand: bool_node,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_negation_op() {
    let sampler = SequentialSampler;
    let operand = create_node(Node::Value(Sample::Real(10.0f64)));

    let node = create_root(Node::NegationOp { operand });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(-10.0)
    );

    // Error case
    let bool_node = create_node(Node::Value(Sample::Bool(true)));
    let node = create_root(Node::NegationOp { operand: bool_node });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_function_op_bool() {
    let sampler = SequentialSampler;
    let operand = create_node(Node::Value(Sample::Real(10.0f64)));
    let func: fn(_) -> _ = |x: f64| x > 5.0;

    let node = create_root(Node::FunctionOpBool { func, operand });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // Error case
    let bool_node = create_node(Node::Value(Sample::Bool(true)));
    let func: fn(_) -> _ = |x: f64| x > 5.0;
    let node = create_root(Node::FunctionOpBool {
        func,
        operand: bool_node,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_conditional_op() {
    let sampler = SequentialSampler;
    let condition_true = create_node(Node::Value(Sample::Bool(true)));
    let condition_false = create_node(Node::Value(Sample::Bool(false)));
    let if_true = create_node(Node::Value(Sample::Real(1.0f64)));
    let if_false = create_node(Node::Value(Sample::Real(0.0f64)));

    // True branch
    let node = create_root(Node::ConditionalOp {
        condition: condition_true,
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(1.0)
    );

    // False branch
    let node = create_root(Node::ConditionalOp {
        condition: condition_false,
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(0.0)
    );

    // Error case: non-boolean condition
    let float_condition = create_node(Node::Value(Sample::Real(42.0f64)));
    let node = create_root(Node::ConditionalOp {
        condition: float_condition,
        if_true,
        if_false,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

// =============================================================================
// Float106 (double-double) precision paths
// =============================================================================

fn f106(x: f64) -> Float106 {
    Float106::from(x)
}

fn dfloat(x: f64) -> UncertainNode<Float106> {
    create_node(Node::Value(Sample::Real(f106(x))))
}

#[test]
fn test_distribution_f106_point_normal_uniform() {
    let sampler = SequentialSampler;

    let node = create_root(Node::Distribution(DistributionEnum::Point(f106(42.0))));
    assert_eq!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(f106(42.0))
    );

    let node = create_root(Node::Distribution(DistributionEnum::Normal(
        NormalDistributionParams::new(f106(0.0), f106(1.0)),
    )));
    assert!(matches!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(_)
    ));

    let node = create_root(Node::Distribution(DistributionEnum::Uniform(
        UniformDistributionParams::new(f106(0.0), f106(1.0)),
    )));
    assert!(matches!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(_)
    ));
}

#[test]
fn test_double_float_arithmetic_and_negation() {
    let sampler = SequentialSampler;

    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: dfloat(10.0),
        rhs: dfloat(5.0),
    });
    assert_eq!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(f106(15.0))
    );

    let node = create_root(Node::NegationOp {
        operand: dfloat(7.0),
    });
    assert_eq!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(f106(-7.0))
    );
}

#[test]
fn test_double_float_comparison_and_function_ops() {
    let sampler = SequentialSampler;

    // A comparison over a double-double operand. The threshold is `Float106` too: it is compared
    // against a draw of that precision, and stating it at a narrower one would narrow the
    // comparison rather than the threshold.
    let node = create_root(Node::ComparisonOp {
        op: ComparisonOperator::GreaterThan,
        threshold: f106(5.0),
        operand: dfloat(10.0),
    });
    assert_eq!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );

    // A mapped function over a double-double operand is typed in the graph's scalar, so it adds
    // at `Float106` rather than narrowing to `f64` and widening back. That round trip was the
    // old boundary, and its absence is what makes the result exact here.
    let func: fn(_) -> _ = |x: Float106| x + f106(1.0);
    let node = create_root(Node::FunctionOpReal {
        func,
        operand: dfloat(10.0),
    });
    assert_eq!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Real(f106(11.0))
    );

    // The same for a predicate: it sees the graph's scalar.
    let func: fn(_) -> _ = |x: Float106| x > f106(0.0);
    let node = create_root(Node::FunctionOpBool {
        func,
        operand: dfloat(10.0),
    });
    assert_eq!(
        Sampler::<Float106>::sample(&sampler, &node, 0).unwrap(),
        Sample::Bool(true)
    );
}

#[test]
fn test_sample_kind_mismatch_errors() {
    // The old "this distribution does not produce that type" refusals are gone: with one arm per
    // distribution rather than three per scalar, there is no mismatched pair left to report. A
    // Bernoulli in a graph of reals is no longer an error — it is simply the Boolean leaf.
    let sampler = SequentialSampler;
    let bernoulli: UncertainNode<f64> = create_root(Node::Distribution(
        DistributionEnum::Bernoulli(BernoulliParams::new(0.5)),
    ));
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &bernoulli, 0).unwrap(),
        Sample::Bool(_)
    ));

    // What can still go wrong is a kind mismatch: an operator handed the wrong sort of sample.
    // Arithmetic over a Boolean operand is the clearest case, and it reports rather than coercing.
    let node: UncertainNode<f64> = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: create_node(Node::Value(Sample::Bool(true))),
        rhs: create_node(Node::Value(Sample::Real(1.0))),
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    // And the mirror: a logical operator handed a real.
    let node: UncertainNode<f64> = create_root(Node::LogicalOp {
        op: LogicalOperator::Not,
        operands: vec![create_node(Node::Value(Sample::Real(1.0)))],
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    // A conditional whose condition is a real, likewise.
    let node: UncertainNode<f64> = create_root(Node::ConditionalOp {
        condition: create_node(Node::Value(Sample::Real(1.0))),
        if_true: create_node(Node::Value(Sample::Real(2.0))),
        if_false: create_node(Node::Value(Sample::Real(3.0))),
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node, 0),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_memoization() {
    let sampler = SequentialSampler;

    // Create a distribution node. If it's evaluated once, then subtracting it from itself should be 0.
    // Without memoization, two different random numbers would be generated, and their difference would not be 0.
    let data = NormalDistributionParams::new(0.0, 1.0);
    let dist_node = create_node(Node::Distribution(DistributionEnum::Normal(data)));

    let node = create_root(Node::ArithmeticOp {
        op: ArithmeticOperator::Sub,
        lhs: dist_node.clone(),
        rhs: dist_node,
    });

    let result = Sampler::<f64>::sample(&sampler, &node, 0).unwrap();
    assert_eq!(result, Sample::Real(0.0));
}
