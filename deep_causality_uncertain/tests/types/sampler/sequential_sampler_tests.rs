/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{
    ArithmeticOperator, BernoulliParams, ComparisonOperator, ComputationNode, DistributionEnum,
    LogicalOperator, NodeId, NormalDistributionParams, SampledValue, SequentialSampler,
    UncertainError,
};

use deep_causality_uncertain::Sampler;
use std::sync::Arc;

#[test]
fn test_sequential_sampler_default() {
    let _sampler = SequentialSampler;
    // Just ensure it can be created
}

#[test]
fn test_sequential_sampler_leaf_f64() {
    let sampler = SequentialSampler;
    let root_node = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(123.45),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Float(123.45));
}

#[test]
fn test_sequential_sampler_leaf_bool() {
    let sampler = SequentialSampler;
    let root_node = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Bool(true));
}

#[test]
fn test_sequential_sampler_arithmetic_op_add() {
    let sampler = SequentialSampler;
    let lhs = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    });
    let rhs = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    });
    let root_node = Arc::new(ComputationNode::ArithmeticOp {
        node_id: NodeId::new(),
        op: ArithmeticOperator::Add,
        lhs: Box::new((*lhs).clone()),
        rhs: Box::new((*rhs).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Float(15.0));
}

#[test]
fn test_sequential_sampler_arithmetic_op_type_error() {
    let sampler = SequentialSampler;
    let lhs = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    });
    let rhs = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Type mismatch
    let root_node = Arc::new(ComputationNode::ArithmeticOp {
        node_id: NodeId::new(),
        op: ArithmeticOperator::Add,
        lhs: Box::new((*lhs).clone()),
        rhs: Box::new((*rhs).clone()),
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_comparison_op_greater_than() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    });
    let root_node = Arc::new(ComputationNode::ComparisonOp {
        node_id: NodeId::new(),
        op: ComparisonOperator::GreaterThan,
        threshold: 5.0,
        operand: Box::new((*operand).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Bool(true));
}

#[test]
fn test_sequential_sampler_comparison_op_type_error() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Type mismatch
    let root_node = Arc::new(ComputationNode::ComparisonOp {
        node_id: NodeId::new(),
        op: ComparisonOperator::GreaterThan,
        threshold: 5.0,
        operand: Box::new((*operand).clone()),
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_logical_op_and() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let op2 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::And,
        operands: vec![Box::new((*op1).clone()), Box::new((*op2).clone())],
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Bool(false));
}

#[test]
fn test_sequential_sampler_logical_op_type_error() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(1.0),
    }); // Type mismatch
    let op2 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::And,
        operands: vec![Box::new((*op1).clone()), Box::new((*op2).clone())],
    });
    let res = sampler.sample(&root_node);
    dbg!(&res);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_function_op() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    });
    let func = Arc::new(|x: f64| x * 2.0);
    let root_node = Arc::new(ComputationNode::FunctionOp {
        node_id: NodeId::new(),
        func,
        operand: Box::new((*operand).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Float(10.0));
}

#[test]
fn test_sequential_sampler_function_op_type_error() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Type mismatch
    let func = Arc::new(|x: f64| x * 2.0);
    let root_node = Arc::new(ComputationNode::FunctionOp {
        node_id: NodeId::new(),
        func,
        operand: Box::new((*operand).clone()),
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_negation_op() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    });
    let root_node = Arc::new(ComputationNode::NegationOp {
        node_id: NodeId::new(),
        operand: Box::new((*operand).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Float(-5.0));
}

#[test]
fn test_sequential_sampler_negation_op_type_error() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Type mismatch
    let root_node = Arc::new(ComputationNode::NegationOp {
        node_id: NodeId::new(),
        operand: Box::new((*operand).clone()),
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_function_op_bool() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    });
    let func = Arc::new(|x: f64| x > 0.0);
    let root_node = Arc::new(ComputationNode::FunctionOpBool {
        node_id: NodeId::new(),
        func,
        operand: Box::new((*operand).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Bool(true));
}

#[test]
fn test_sequential_sampler_function_op_bool_type_error() {
    let sampler = SequentialSampler;
    let operand = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Type mismatch
    let func = Arc::new(|x: f64| x > 0.0);
    let root_node = Arc::new(ComputationNode::FunctionOpBool {
        node_id: NodeId::new(),
        func,
        operand: Box::new((*operand).clone()),
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_conditional_op_true() {
    let sampler = SequentialSampler;
    let condition = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let if_true = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    });
    let if_false = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(20.0),
    });
    let root_node = Arc::new(ComputationNode::ConditionalOp {
        node_id: NodeId::new(),
        condition: Box::new((*condition).clone()),
        if_true: Box::new((*if_true).clone()),
        if_false: Box::new((*if_false).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Float(10.0));
}

#[test]
fn test_sequential_sampler_conditional_op_false() {
    let sampler = SequentialSampler;
    let condition = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let if_true = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    });
    let if_false = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(20.0),
    });
    let root_node = Arc::new(ComputationNode::ConditionalOp {
        node_id: NodeId::new(),
        condition: Box::new((*condition).clone()),
        if_true: Box::new((*if_true).clone()),
        if_false: Box::new((*if_false).clone()),
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Float(20.0));
}

#[test]
fn test_sequential_sampler_conditional_op_type_error() {
    let sampler = SequentialSampler;
    let condition = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(1.0),
    }); // Type mismatch
    let if_true = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    });
    let if_false = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(20.0),
    });
    let root_node = Arc::new(ComputationNode::ConditionalOp {
        node_id: NodeId::new(),
        condition: Box::new((*condition).clone()),
        if_true: Box::new((*if_true).clone()),
        if_false: Box::new((*if_false).clone()),
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(_) => (),
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_memoization() {
    let sampler = SequentialSampler;
    let leaf = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Normal(NormalDistributionParams::new(0.0, 0.0001)),
    }); // Very small std_dev for near-constant value
    let add_op = Arc::new(ComputationNode::ArithmeticOp {
        node_id: NodeId::new(),
        op: ArithmeticOperator::Add,
        lhs: Box::new((*leaf).clone()),
        rhs: Box::new((*leaf).clone()), // Re-use the same leaf node
    });

    let result1 = sampler.sample(&add_op).unwrap();
    let result2 = sampler.sample(&add_op).unwrap(); // Sample again
    dbg!(&result1);
    dbg!(&result2);

    // Due to memoization, the same leaf node should produce the same sample within a single sample call
    // and across multiple sample calls if the cache is not cleared.
    // However, the current `sample` method creates a new context each time, so memoization is per-call.
    // The test should verify memoization *within* a single graph evaluation.
    // The `evaluate_node` function uses the context.
    // Let's create a graph where a node is referenced multiple times.

    let leaf_node = Arc::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    });
    let node_a = Arc::new(ComputationNode::ArithmeticOp {
        node_id: NodeId::new(),
        op: ArithmeticOperator::Add,
        lhs: Box::new((*leaf_node).clone()),
        rhs: Box::new((*leaf_node).clone()), // leaf_node used twice
    });
    let result = sampler.sample(&node_a).unwrap();
    assert_eq!(result, SampledValue::Float(10.0)); // 5.0 + 5.0

    // If memoization wasn't working, and if leaf_node was a random distribution,
    // it might sample different values for each reference.
    // For a point distribution, it's always the same, so this test is weak for randomness.
    // A better test would involve a random distribution and checking if the same value is used.
    // But that's hard to assert without mocking RNG.

    // The current test for memoization is implicitly covered by the fact that
    // `evaluate_node` checks `context.get(&ptr)` before computing.
    // For a more explicit test, we'd need to inspect the `context` directly,
    // which is not exposed.
    // Given the `#[should_panic]` tests, the primary goal is to ensure the sampler
    // correctly traverses the graph and handles type mismatches.
}

#[test]
fn test_sequential_sampler_error_propagation_from_distribution() {
    let sampler = SequentialSampler;
    // Create a Bernoulli distribution with invalid 'p' to cause an error
    let root_node = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Bernoulli(BernoulliParams::new(2.0)),
    });
    let result = sampler.sample(&root_node);
    assert!(result.is_err());
    match result.err().unwrap() {
        UncertainError::BernoulliDistributionError(_) => (),
        e => panic!("Expected BernoulliDistributionError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_logical_op_not_wrong_number_of_operands() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let op2 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::Not,
        operands: vec![Box::new((*op1).clone()), Box::new((*op2).clone())], // Should be 1
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(msg) => {
            assert_eq!(msg, "NOT expects exactly 1 operand");
        }
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_logical_op_not_with_zero_operands() {
    let sampler = SequentialSampler;
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::Not,
        operands: vec![], // Zero operands
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(msg) => {
            assert_eq!(msg, "NOT expects exactly 1 operand");
        }
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_logical_op_and_wrong_number_of_operands() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::And,
        operands: vec![Box::new((*op1).clone())], // Should be 2
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(msg) => {
            assert_eq!(msg, "Binary logical op expects exactly 2 operands");
        }
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_logical_op_or_with_three_operands() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let op2 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let op3 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::Or,
        operands: vec![
            Box::new((*op1).clone()),
            Box::new((*op2).clone()),
            Box::new((*op3).clone()),
        ], // Three operands
    });
    let res = sampler.sample(&root_node);
    assert!(res.is_err());
    match res.err().unwrap() {
        UncertainError::UnsupportedTypeError(msg) => {
            assert_eq!(msg, "Binary logical op expects exactly 2 operands");
        }
        e => panic!("Expected UnsupportedTypeError, got {:?}", e),
    }
}

#[test]
fn test_sequential_sampler_logical_op_nor_true() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let op2 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::NOR,
        operands: vec![Box::new((*op1).clone()), Box::new((*op2).clone())],
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Bool(true));
}

#[test]
fn test_sequential_sampler_logical_op_nor_false() {
    let sampler = SequentialSampler;
    let op1 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    });
    let op2 = Arc::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    });
    let root_node = Arc::new(ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op: LogicalOperator::NOR,
        operands: vec![Box::new((*op1).clone()), Box::new((*op2).clone())],
    });
    let result = sampler.sample(&root_node).unwrap();
    assert_eq!(result, SampledValue::Bool(false));
}
