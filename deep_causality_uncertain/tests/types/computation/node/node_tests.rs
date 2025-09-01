/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::types::computation::{
    ArithmeticOperator, ComparisonOperator, ComputationNode, LogicalOperator,
};
use deep_causality_uncertain::types::distribution::DistributionEnum;
use std::sync::Arc;

#[test]
fn test_computation_node_leaf_f64_construction_and_clone() {
    let node = ComputationNode::LeafF64(DistributionEnum::Point(1.0));
    let cloned_node = node.clone();

    // We can't directly compare enums without PartialEq, but we can check the variant
    // and potentially the inner data if it's accessible.
    match cloned_node {
        ComputationNode::LeafF64(DistributionEnum::Point(val)) => assert_eq!(val, 1.0),
        _ => panic!("Cloned node is not LeafF64(Point)"),
    }
}

#[test]
fn test_computation_node_leaf_bool_construction_and_clone() {
    let node = ComputationNode::LeafBool(DistributionEnum::Point(true));
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::LeafBool(DistributionEnum::Point(val)) => assert_eq!(val, true),
        _ => panic!("Cloned node is not LeafBool(Point)"),
    }
}

#[test]
fn test_computation_node_arithmetic_op_construction_and_clone() {
    let lhs = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(1.0)));
    let rhs = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(2.0)));
    let node = ComputationNode::ArithmeticOp {
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::ArithmeticOp {
            op,
            lhs: cloned_lhs,
            rhs: cloned_rhs,
        } => {
            assert_eq!(op, ArithmeticOperator::Add);
            // Cannot directly compare Box<ComputationNode> without PartialEq,
            // but we can assert they are not null and represent some structure.
            // The clone trait itself is what we are primarily testing here.
            assert!(matches!(*cloned_lhs, ComputationNode::LeafF64(_)));
            assert!(matches!(*cloned_rhs, ComputationNode::LeafF64(_)));
        }
        _ => panic!("Cloned node is not ArithmeticOp"),
    }
}

#[test]
fn test_computation_node_comparison_op_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(10.0)));
    let node = ComputationNode::ComparisonOp {
        op: ComparisonOperator::GreaterThan,
        threshold: 5.0,
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::ComparisonOp {
            op,
            threshold,
            operand: cloned_operand,
        } => {
            assert_eq!(op, ComparisonOperator::GreaterThan);
            assert_eq!(threshold, 5.0);
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64(_)));
        }
        _ => panic!("Cloned node is not ComparisonOp"),
    }
}

#[test]
fn test_computation_node_logical_op_construction_and_clone() {
    let operand1 = Box::new(ComputationNode::LeafBool(DistributionEnum::Point(true)));
    let operand2 = Box::new(ComputationNode::LeafBool(DistributionEnum::Point(false)));
    let node = ComputationNode::LogicalOp {
        op: LogicalOperator::And,
        operands: vec![operand1.clone(), operand2.clone()],
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::LogicalOp {
            op,
            operands: cloned_operands,
        } => {
            assert_eq!(op, LogicalOperator::And);
            assert_eq!(cloned_operands.len(), 2);
            assert!(matches!(*cloned_operands[0], ComputationNode::LeafBool(_)));
            assert!(matches!(*cloned_operands[1], ComputationNode::LeafBool(_)));
        }
        _ => panic!("Cloned node is not LogicalOp"),
    }
}

#[test]
fn test_computation_node_function_op_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(5.0)));
    let func = Arc::new(|x: f64| x * 2.0);
    let node = ComputationNode::FunctionOp {
        func: func.clone(),
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::FunctionOp {
            func: cloned_func,
            operand: cloned_operand,
        } => {
            // Cannot directly compare Arc<dyn Fn>
            assert_eq!(cloned_func(3.0), 6.0); // Test the function still works
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64(_)));
        }
        _ => panic!("Cloned node is not FunctionOp"),
    }
}

#[test]
fn test_computation_node_function_op_bool_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(5.0)));
    let func = Arc::new(|x: f64| x > 0.0);
    let node = ComputationNode::FunctionOpBool {
        func: func.clone(),
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::FunctionOpBool {
            func: cloned_func,
            operand: cloned_operand,
        } => {
            // Cannot directly compare Arc<dyn Fn>
            assert_eq!(cloned_func(3.0), true); // Test the function still works
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64(_)));
        }
        _ => panic!("Cloned node is not FunctionOpBool"),
    }
}

#[test]
fn test_computation_node_negation_op_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(10.0)));
    let node = ComputationNode::NegationOp {
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::NegationOp {
            operand: cloned_operand,
        } => {
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64(_)));
        }
        _ => panic!("Cloned node is not NegationOp"),
    }
}

#[test]
fn test_computation_node_conditional_op_construction_and_clone() {
    let condition = Box::new(ComputationNode::LeafBool(DistributionEnum::Point(true)));
    let if_true = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(1.0)));
    let if_false = Box::new(ComputationNode::LeafF64(DistributionEnum::Point(0.0)));
    let node = ComputationNode::ConditionalOp {
        condition: condition.clone(),
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::ConditionalOp {
            condition: cloned_condition,
            if_true: cloned_if_true,
            if_false: cloned_if_false,
        } => {
            assert!(matches!(*cloned_condition, ComputationNode::LeafBool(_)));
            assert!(matches!(*cloned_if_true, ComputationNode::LeafF64(_)));
            assert!(matches!(*cloned_if_false, ComputationNode::LeafF64(_)));
        }
        _ => panic!("Cloned node is not ConditionalOp"),
    }
}
