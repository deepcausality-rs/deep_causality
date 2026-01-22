/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::ControlFlowBuilder;
use deep_causality_core::{ControlFlowProtocol, FromProtocol, ToProtocol};
use std::sync::atomic::{AtomicUsize, Ordering};

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

static NODE_3_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn test_diamond_graph_execution_bug() {
    // Create a diamond graph:
    //     0
    //    / \
    //   1   2
    //    \ /
    //     3
    //
    // Node 3 has TWO incoming edges (from nodes 1 and 2)
    // In standard BFS, node 3 should be visited exactly once
    // Reset counter
    NODE_3_EXECUTIONS.store(0, Ordering::SeqCst);

    let mut builder = ControlFlowBuilder::<TestProtocol>::new();

    let n0 = builder.add_node(|x: i32| {
        // Node 0: input -> input * 2
        x * 2
    });

    let n1 = builder.add_node(|x: i32| {
        // Node 1: input -> input + 10
        x + 10
    });

    let n2 = builder.add_node(|x: i32| {
        // Node 2: input -> input + 100
        x + 100
    });

    let n3 = builder.add_node(|x: i32| {
        // Node 3: input -> input + 1000
        let _ = NODE_3_EXECUTIONS.fetch_add(1, Ordering::SeqCst);
        x + 1000
    });

    // Create diamond: 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
    builder.connect(n0, n1);
    builder.connect(n0, n2);
    builder.connect(n1, n3);
    builder.connect(n2, n3);

    let graph = builder.build();
    let mut queue = std::collections::VecDeque::new();
    // Start at n0 (id 0) with input 1
    let _ = graph.execute(1.to_protocol(), n0.id(), 20, &mut queue);

    let n3_count = NODE_3_EXECUTIONS.load(Ordering::SeqCst);

    // We assert 1 here to FAIL if the bug is present (Regression Test logic)
    // Or we can assert > 0 to pass but log the issue?
    // Usually for reproduction, we want to SEE it fail or assert the faulty behavior exists if we are proving it.
    // But since I need to fix it, I will write the assertion for the CORRECT behavior.
    // So this test SHOULD FAIL now, and PASS after I fix the code.

    assert_eq!(
        n3_count, 1,
        "Expected node 3 to execute exactly once (BFS), but it executed {} times",
        n3_count
    );
}
