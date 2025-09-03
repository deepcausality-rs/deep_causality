/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{
    ArithmeticOperator, ComparisonOperator, ComputationNode, DistributionEnum, LogicalOperator,
    NodeId,
};
use std::sync::Arc;

#[test]
fn test_computation_node_leaf_f64_construction_and_clone() {
    let node = ComputationNode::LeafF64 {
        // Changed to struct variant
        node_id: NodeId::new(), // Added node_id
        dist: DistributionEnum::Point(1.0),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::LeafF64 { node_id: _, dist } => match dist {
            DistributionEnum::Point(val) => assert_eq!(val, 1.0),
            _ => panic!("Expected Point distribution"),
        },
        _ => panic!("Cloned node is not LeafF64"),
    }
}

#[test]
fn test_computation_node_leaf_bool_construction_and_clone() {
    let node = ComputationNode::LeafBool {
        // Changed to struct variant
        node_id: NodeId::new(), // Added node_id
        dist: DistributionEnum::Point(true),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::LeafBool { node_id: _, dist } => {
            assert!(matches!(dist, DistributionEnum::Point(true)))
        } // Updated pattern
        _ => panic!("Cloned node is not LeafBool(Point)"),
    }
}

#[test]
fn test_computation_node_arithmetic_op_construction_and_clone() {
    let lhs = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(1.0),
    }); // Updated
    let rhs = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(2.0),
    }); // Updated
    let node = ComputationNode::ArithmeticOp {
        node_id: NodeId::new(), // Added node_id
        op: ArithmeticOperator::Add,
        lhs: lhs.clone(),
        rhs: rhs.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::ArithmeticOp {
            node_id: _, // Added node_id
            op,
            lhs: cloned_lhs,
            rhs: cloned_rhs,
        } => {
            assert_eq!(op, ArithmeticOperator::Add);
            assert!(matches!(*cloned_lhs, ComputationNode::LeafF64 { .. })); // Updated matches!
            assert!(matches!(*cloned_rhs, ComputationNode::LeafF64 { .. })); // Updated matches!
        }
        _ => panic!("Cloned node is not ArithmeticOp"),
    }
}

#[test]
fn test_computation_node_comparison_op_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    }); // Updated
    let node = ComputationNode::ComparisonOp {
        node_id: NodeId::new(), // Added node_id
        op: ComparisonOperator::GreaterThan,
        threshold: 5.0,
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::ComparisonOp {
            node_id: _, // Added node_id
            op,
            threshold,
            operand: cloned_operand,
        } => {
            assert_eq!(op, ComparisonOperator::GreaterThan);
            assert_eq!(threshold, 5.0);
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64 { .. })); // Updated matches!
        }
        _ => panic!("Cloned node is not ComparisonOp"),
    }
}

#[test]
fn test_computation_node_logical_op_construction_and_clone() {
    let operand1 = Box::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Updated
    let operand2 = Box::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(false),
    }); // Updated
    let node = ComputationNode::LogicalOp {
        node_id: NodeId::new(), // Added node_id
        op: LogicalOperator::And,
        operands: vec![operand1.clone(), operand2.clone()],
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::LogicalOp {
            node_id: _, // Added node_id
            op,
            operands: cloned_operands,
        } => {
            assert_eq!(op, LogicalOperator::And);
            assert_eq!(cloned_operands.len(), 2);
            assert!(matches!(
                *cloned_operands[0],
                ComputationNode::LeafBool { .. }
            )); // Updated matches!
            assert!(matches!(
                *cloned_operands[1],
                ComputationNode::LeafBool { .. }
            )); // Updated matches!
        }
        _ => panic!("Cloned node is not LogicalOp"),
    }
}

#[test]
fn test_computation_node_function_op_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    }); // Updated
    let func = Arc::new(|x: f64| x * 2.0);
    let node = ComputationNode::FunctionOpF64 {
        node_id: NodeId::new(), // Added node_id
        func: func.clone(),
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::FunctionOpF64 {
            node_id: _, // Added node_id
            func: cloned_func,
            operand: cloned_operand,
        } => {
            assert_eq!(cloned_func(3.0), 6.0); // Test the function still works
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64 { .. })); // Updated matches!
        }
        _ => panic!("Cloned node is not FunctionOp"),
    }
}

#[test]
fn test_computation_node_function_op_bool_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(5.0),
    }); // Updated
    let func = Arc::new(|x: f64| x > 0.0);
    let node = ComputationNode::FunctionOpBool {
        node_id: NodeId::new(), // Added node_id
        func: func.clone(),
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::FunctionOpBool {
            node_id: _, // Added node_id
            func: cloned_func,
            operand: cloned_operand,
        } => {
            assert!(cloned_func(3.0)); // Test the function still works
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64 { .. })); // Updated matches!
        }
        _ => panic!("Cloned node is not FunctionOpBool"),
    }
}

#[test]
fn test_computation_node_negation_op_construction_and_clone() {
    let operand = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(10.0),
    }); // Updated
    let node = ComputationNode::NegationOp {
        node_id: NodeId::new(), // Added node_id
        operand: operand.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::NegationOp {
            node_id: _, // Added node_id
            operand: cloned_operand,
        } => {
            assert!(matches!(*cloned_operand, ComputationNode::LeafF64 { .. })); // Updated matches!
        }
        _ => panic!("Cloned node is not NegationOp"),
    }
}

#[test]
fn test_computation_node_conditional_op_construction_and_clone() {
    let condition = Box::new(ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(true),
    }); // Updated
    let if_true = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(1.0),
    }); // Updated
    let if_false = Box::new(ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(0.0),
    }); // Updated
    let node = ComputationNode::ConditionalOp {
        node_id: NodeId::new(), // Added node_id
        condition: condition.clone(),
        if_true: if_true.clone(),
        if_false: if_false.clone(),
    };
    let cloned_node = node.clone();

    match cloned_node {
        ComputationNode::ConditionalOp {
            node_id: _, // Added node_id
            condition: cloned_condition,
            if_true: cloned_if_true,
            if_false: cloned_if_false,
        } => {
            assert!(matches!(
                *cloned_condition,
                ComputationNode::LeafBool { .. }
            )); // Updated matches!
            assert!(matches!(*cloned_if_true, ComputationNode::LeafF64 { .. })); // Updated matches!
            assert!(matches!(*cloned_if_false, ComputationNode::LeafF64 { .. })); // Updated matches!
        }
        _ => panic!("Cloned node is not ConditionalOp"),
    }
}
