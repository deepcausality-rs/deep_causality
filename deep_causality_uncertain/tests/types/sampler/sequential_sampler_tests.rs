/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;
use deep_causality_uncertain::{
    ArithmeticOperator, BernoulliParams, ComparisonOperator, DistributionEnum, LogicalOperator,
    NormalDistributionParams, SampledValue, Sampler, SequentialSampler, UncertainError,
    UncertainNodeContent, UniformDistributionParams,
};
use std::sync::Arc;

// Corrected helper functions
type UncertainNode = ConstTree<UncertainNodeContent>;

// Helper to create a root node for a graph
fn create_root(content: UncertainNodeContent) -> UncertainNode {
    ConstTree::new(content)
}

// Helper to create a non-root node
fn create_node(content: UncertainNodeContent) -> UncertainNode {
    ConstTree::new(content)
}

#[test]
fn test_sample_value() {
    let sampler = SequentialSampler;

    // Test with f64 value
    let node_f64 = create_root(UncertainNodeContent::Value(SampledValue::Float(42.0f64)));
    let result_f64 = Sampler::<f64>::sample(&sampler, &node_f64).unwrap();
    assert_eq!(result_f64, SampledValue::Float(42.0));

    // Test with bool value
    let node_bool = create_root(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let result_bool = Sampler::<bool>::sample(&sampler, &node_bool).unwrap();
    assert_eq!(result_bool, SampledValue::Bool(true));
}

#[test]
fn test_distribution_f64() {
    let sampler = SequentialSampler;

    // Point
    let node = create_root(UncertainNodeContent::DistributionF64(
        DistributionEnum::Point(42.0),
    ));
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(42.0)
    );

    // Normal
    let data = NormalDistributionParams::new(0.0, 1.0);
    let node = create_root(UncertainNodeContent::DistributionF64(
        DistributionEnum::Normal(data),
    ));
    let result = Sampler::<f64>::sample(&sampler, &node).unwrap();
    assert!(matches!(result, SampledValue::Float(_)));

    // Uniform
    let data = UniformDistributionParams::new(0.0, 1.0);
    let node = create_root(UncertainNodeContent::DistributionF64(
        DistributionEnum::Uniform(data),
    ));
    let result = Sampler::<f64>::sample(&sampler, &node).unwrap();
    assert!(matches!(result, SampledValue::Float(_)));
}

#[test]
fn test_distribution_bool() {
    let sampler = SequentialSampler;

    // Point
    let node = create_root(UncertainNodeContent::DistributionBool(
        DistributionEnum::Point(true),
    ));
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // Bernoulli
    let data = BernoulliParams::new(0.5);
    let node = create_root(UncertainNodeContent::DistributionBool(
        DistributionEnum::Bernoulli(data),
    ));
    let result = Sampler::<bool>::sample(&sampler, &node).unwrap();
    assert!(matches!(result, SampledValue::Bool(_)));
}

#[test]
fn test_pure_op() {
    let sampler = SequentialSampler;
    let node = create_root(UncertainNodeContent::PureOp {
        value: SampledValue::Float(42.0),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(42.0)
    );
}

#[test]
fn test_fmap_op() {
    let sampler = SequentialSampler;
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));
    let func = Arc::new(|val: SampledValue| match val {
        SampledValue::Float(v) => SampledValue::Float(v * 2.0),
        _ => panic!("unexpected type"),
    });

    let node = create_root(UncertainNodeContent::FmapOp { func, operand });

    let result = Sampler::<f64>::sample(&sampler, &node).unwrap();
    assert_eq!(result, SampledValue::Float(20.0));
}

#[test]
fn test_apply_op() {
    let sampler = SequentialSampler;
    let arg = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));
    let func = Arc::new(|val: SampledValue| match val {
        SampledValue::Float(v) => SampledValue::Float(v + 5.0),
        _ => panic!("unexpected type"),
    });

    let node = create_root(UncertainNodeContent::ApplyOp { func, arg });

    let result = Sampler::<f64>::sample(&sampler, &node).unwrap();
    assert_eq!(result, SampledValue::Float(15.0));
}

#[test]
fn test_bind_op() {
    let sampler = SequentialSampler;
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));
    let func = Arc::new(|val: SampledValue| {
        let inner_val = match val {
            SampledValue::Float(v) => v,
            _ => panic!("unexpected type"),
        };
        create_node(UncertainNodeContent::Value(SampledValue::Float(
            inner_val * 2.0,
        )))
    });

    let node = create_root(UncertainNodeContent::BindOp { func, operand });

    let result = Sampler::<f64>::sample(&sampler, &node).unwrap();
    assert_eq!(result, SampledValue::Float(20.0));
}

#[test]
fn test_arithmetic_op() {
    let sampler = SequentialSampler;
    let lhs = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));
    let rhs = create_node(UncertainNodeContent::Value(SampledValue::Float(5.0f64)));

    // Add
    let node = create_root(UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(15.0)
    );

    // Sub
    let node = create_root(UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Sub,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(5.0)
    );

    // Mul
    let node = create_root(UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Mul,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(50.0)
    );

    // Div
    let node = create_root(UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Div,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(2.0)
    );

    // Error case
    let bool_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let node = create_root(UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: bool_node,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_comparison_op() {
    let sampler = SequentialSampler;
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));

    // EqualTo
    let node = create_root(UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 10.0,
        operand: operand.clone(),
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // GreaterThan
    let node = create_root(UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::GreaterThan,
        threshold: 5.0,
        operand: operand.clone(),
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // LessThan
    let node = create_root(UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::LessThan,
        threshold: 15.0,
        operand: operand.clone(),
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // Error case
    let bool_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let node = create_root(UncertainNodeContent::ComparisonOp {
        op: ComparisonOperator::EqualTo,
        threshold: 0.0, // dummy
        operand: bool_node,
    });
    assert!(matches!(
        Sampler::<bool>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_logical_op() {
    let sampler = SequentialSampler;
    let true_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let false_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(false)));

    // Not
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: vec![true_node.clone()],
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(false)
    );

    // And
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(false)
    );

    // Or
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Or,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // XOR
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::XOR,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // NOR
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::NOR,
        operands: vec![true_node.clone(), false_node.clone()],
    });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(false)
    );
}

#[test]
fn test_logical_op_errors() {
    let sampler = SequentialSampler;
    let true_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let float_node = create_node(UncertainNodeContent::Value(SampledValue::Float(42.0f64)));

    // NOT with 2 operands
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::Not,
        operands: vec![true_node.clone(), true_node.clone()],
    });
    assert!(matches!(
        Sampler::<bool>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    // AND with 1 operand
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![true_node.clone()],
    });
    assert!(matches!(
        Sampler::<bool>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    // Wrong input type
    let node = create_root(UncertainNodeContent::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![true_node.clone(), float_node],
    });
    assert!(matches!(
        Sampler::<bool>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_function_op_f64() {
    let sampler = SequentialSampler;
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));
    let func = Arc::new(|x: f64| x.powi(2));

    let node = create_root(UncertainNodeContent::FunctionOpF64 { func, operand });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(100.0)
    );

    // Error case
    let bool_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let func = Arc::new(|x: f64| x.powi(2));
    let node = create_root(UncertainNodeContent::FunctionOpF64 {
        func,
        operand: bool_node,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_negation_op() {
    let sampler = SequentialSampler;
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));

    let node = create_root(UncertainNodeContent::NegationOp { operand });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(-10.0)
    );

    // Error case
    let bool_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let node = create_root(UncertainNodeContent::NegationOp { operand: bool_node });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_function_op_bool() {
    let sampler = SequentialSampler;
    let operand = create_node(UncertainNodeContent::Value(SampledValue::Float(10.0f64)));
    let func = Arc::new(|x: f64| x > 5.0);

    let node = create_root(UncertainNodeContent::FunctionOpBool { func, operand });
    assert_eq!(
        Sampler::<bool>::sample(&sampler, &node).unwrap(),
        SampledValue::Bool(true)
    );

    // Error case
    let bool_node = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let func = Arc::new(|x: f64| x > 5.0);
    let node = create_root(UncertainNodeContent::FunctionOpBool {
        func,
        operand: bool_node,
    });
    assert!(matches!(
        Sampler::<bool>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_conditional_op() {
    let sampler = SequentialSampler;
    let condition_true = create_node(UncertainNodeContent::Value(SampledValue::Bool(true)));
    let condition_false = create_node(UncertainNodeContent::Value(SampledValue::Bool(false)));
    let if_true = create_node(UncertainNodeContent::Value(SampledValue::Float(1.0f64)));
    let if_false = create_node(UncertainNodeContent::Value(SampledValue::Float(0.0f64)));

    // True branch
    let node = create_root(UncertainNodeContent::ConditionalOp {
        condition: condition_true,
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(1.0)
    );

    // False branch
    let node = create_root(UncertainNodeContent::ConditionalOp {
        condition: condition_false,
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    });
    assert_eq!(
        Sampler::<f64>::sample(&sampler, &node).unwrap(),
        SampledValue::Float(0.0)
    );

    // Error case: non-boolean condition
    let float_condition = create_node(UncertainNodeContent::Value(SampledValue::Float(42.0f64)));
    let node = create_root(UncertainNodeContent::ConditionalOp {
        condition: float_condition,
        if_true,
        if_false,
    });
    assert!(matches!(
        Sampler::<f64>::sample(&sampler, &node),
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_memoization() {
    let sampler = SequentialSampler;

    // Create a distribution node. If it's evaluated once, then subtracting it from itself should be 0.
    // Without memoization, two different random numbers would be generated, and their difference would not be 0.
    let data = NormalDistributionParams::new(0.0, 1.0);
    let dist_node = create_node(UncertainNodeContent::DistributionF64(
        DistributionEnum::Normal(data),
    ));

    let node = create_root(UncertainNodeContent::ArithmeticOp {
        op: ArithmeticOperator::Sub,
        lhs: dist_node.clone(),
        rhs: dist_node,
    });

    let result = Sampler::<f64>::sample(&sampler, &node).unwrap();
    assert_eq!(result, SampledValue::Float(0.0));
}
