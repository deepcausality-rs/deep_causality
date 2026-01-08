/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::collections::VecDeque;

use deep_causality_core::ControlFlowBuilder;
use deep_causality_core::GraphError;
use deep_causality_core::{ControlFlowProtocol, FromProtocol, ToProtocol};

#[derive(Debug, Clone, PartialEq)]
pub enum TestProtocol {
    Int(i32),
    Error(String),
}

impl ControlFlowProtocol for TestProtocol {
    fn error<E: core::fmt::Debug>(msg: E) -> Self {
        TestProtocol::Error(format!("{:?}", msg))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TestError;

impl FromProtocol<TestProtocol> for i32 {
    type Error = TestError;
    fn from_protocol(proto: TestProtocol) -> Result<Self, Self::Error> {
        match proto {
            TestProtocol::Int(val) => Ok(val),
            TestProtocol::Error(_) => Err(TestError),
        }
    }
}

impl ToProtocol<TestProtocol> for i32 {
    fn to_protocol(self) -> TestProtocol {
        TestProtocol::Int(self)
    }
}

#[test]
fn test_execute_graph() {
    let mut builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();

    fn inc(x: i32) -> i32 {
        x + 1
    }

    let n1 = builder.add_node(inc); // 0
    let n2 = builder.add_node(inc); // 1

    builder.connect(n1, n2);

    let graph = builder.build();
    let mut queue = VecDeque::new();

    let result = graph.execute(TestProtocol::Int(0), 0, 10, &mut queue);

    match result {
        Ok(TestProtocol::Int(val)) => assert_eq!(val, 2), // 0 -> 1 -> 2
        _ => panic!("Unexpected result: {:?}", result),
    }
}

#[test]
fn test_execute_graph_max_steps() {
    let mut builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();

    fn inc(x: i32) -> i32 {
        x + 1
    }

    let n1 = builder.add_node(inc);
    let n2 = builder.add_node(inc);

    builder.connect(n1, n2);
    builder.connect(n2, n1); // Cycle

    let graph = builder.build();
    let mut queue = VecDeque::new();

    let result = graph.execute(TestProtocol::Int(0), 0, 5, &mut queue);

    match result {
        Err(GraphError::MaxStepsExceeded(steps)) => assert_eq!(steps, 5),
        _ => panic!("Expected MaxStepsExceeded, got: {:?}", result),
    }
}

#[test]
fn test_execute_graph_out_of_bounds() {
    let builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();
    let graph = builder.build();
    let mut queue = VecDeque::new();

    let result = graph.execute(TestProtocol::Int(0), 0, 10, &mut queue);

    match result {
        Err(GraphError::StartNodeOutOfBounds(idx)) => assert_eq!(idx, 0),
        _ => panic!("Expected StartNodeOutOfBounds, got: {:?}", result),
    }
}
