/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::ControlFlowBuilder;
use deep_causality_core::{ControlFlowProtocol, FromProtocol, ToProtocol};

#[derive(Debug, Clone, PartialEq)]
pub enum TestProtocol {
    Int(i32),
    Error(String),
}

impl ControlFlowProtocol for TestProtocol {
    fn error<E: std::fmt::Debug>(msg: E) -> Self {
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
fn test_new_builder() {
    let builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();
    let graph = builder.build();
    assert!(graph.nodes().is_empty());
    assert!(graph.adjacency().is_empty());
}

#[test]
fn test_add_node() {
    let mut builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();

    fn inc(x: i32) -> i32 {
        x + 1
    }

    let node_id = builder.add_node(inc);
    assert_eq!(node_id.id(), 0);

    let graph = builder.build();
    assert_eq!(graph.nodes().len(), 1);
}

#[test]
fn test_connect_nodes() {
    let mut builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();

    fn inc(x: i32) -> i32 {
        x + 1
    }

    let n1 = builder.add_node(inc);
    let n2 = builder.add_node(inc);

    builder.connect(n1, n2);

    let graph = builder.build();
    assert_eq!(graph.adjacency().len(), 2);
    assert_eq!(graph.adjacency()[0], vec![1]);
    assert!(graph.adjacency()[1].is_empty());
}
