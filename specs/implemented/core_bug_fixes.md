ExecutableGraph::execute re-executes nodes with multiple incoming edges; final result is order-dependent

# Summary
- **Context**: The `ExecutableGraph::execute` method performs graph traversal to execute nodes in a causal/control flow graph, returning a final result.
- **Bug**: Nodes with multiple incoming edges are executed multiple times (once per incoming edge), and the final result is the output of whichever node was processed last in BFS order, not based on semantic meaning.
- **Actual vs. expected**: The method is documented as using "an iterative BFS approach," but unlike standard BFS, it doesn't track visited nodes. Standard BFS visits each node exactly once; this implementation visits nodes once per incoming edge.
- **Impact**: Users receive ambiguous, order-dependent results that may not match their expectations. In graphs with multiple paths or terminal nodes, the returned value depends on arbitrary traversal ordering rather than the logical structure of the computation.

# Code with bug

```rust
while let Some((node_idx, node_input)) = queue.pop_front() {
    if steps >= max_steps {
        return Err(GraphError::MaxStepsExceeded(max_steps));
    }

    // Execute the current node's logic
    let node = &self.nodes[node_idx];
    let output = (node.func)(node_input); // <-- BUG 🔴 Node executed every time it's dequeued

    // Store result (cloned because we might need it for multiple neighbors)
    last_result = Some(output.clone()); // <-- BUG 🔴 Last processed node wins

    // Propagate to neighbors
    if let Some(neighbors) = self.adjacency.get(node_idx) {
        for &neighbor_idx in neighbors {
            queue.push_back((neighbor_idx, output.clone())); // <-- BUG 🔴 No visited tracking
        }
    }

    steps += 1;
}

last_result.ok_or(GraphError::GraphExecutionProducedNoResult)
```

The bug is that:
1. There is no tracking of which nodes have been visited
2. Each time a node index is popped from the queue, it's executed again
3. The "last_result" is simply the output of whichever node was processed last

# Evidence

## Example

Consider a diamond graph:
```
      0
     / \
    1   2
     \ /
      3
```

With this execution:
- Node 0: input=1, output=1
- Node 1: input=1, output=101 (adds 100)
- Node 2: input=1, output=201 (adds 200)
- Node 3: input=101, output=1010 (first execution, multiplies by 10)
- Node 3: input=201, output=2010 (second execution, multiplies by 10)

**Result: 2010**

Node 3 was executed **twice** with different inputs (101 and 201), violating the BFS property that each node is visited once. The final result (2010) is from whichever execution of node 3 happened to be processed last in BFS order.

## Inconsistency with own spec / docstring

### Reference spec / comment

From `deep_causality_core/src/types/builder/executable_graph.rs:22`:
```rust
/// Executes the graph with an initial input using an iterative BFS approach.
```

From `specs/implemented/core_control_flow_builder.md`:
```markdown
## 2. Design Philosophy

1.  **Correctness by Construction:** Invalid graph topologies (type mismatches) are unrepresentable states in the compiler.
2.  **Zero-Cost Abstraction:** The type checks occur solely during the build phase. The runtime artifact uses simple index-based iteration.
3.  **Protocol Agnostic:** The builder operates on a generic `CausalProtocol`, allowing users to define their own domain (e.g., `QuantumProtocol`, `AvionicsProtocol`) without modifying the core.
```

The specification emphasizes "Correctness by Construction" and describes the system as "deterministic."

### Current code

From `deep_causality_core/src/types/builder/executable_graph.rs:51-73`:
```rust
while let Some((node_idx, node_input)) = queue.pop_front() {
    if steps >= max_steps {
        return Err(GraphError::MaxStepsExceeded(max_steps));
    }

    // Execute the current node's logic
    let node = &self.nodes[node_idx];
    let output = (node.func)(node_input);

    // Store result (cloned because we might need it for multiple neighbors)
    last_result = Some(output.clone());

    // Propagate to neighbors
    if let Some(neighbors) = self.adjacency.get(node_idx) {
        for &neighbor_idx in neighbors {
            queue.push_back((neighbor_idx, output.clone()));
        }
    }

    steps += 1;
}

last_result.ok_or(GraphError::GraphExecutionProducedNoResult)
```

### Contradiction

1. **BFS claim is misleading**: Standard BFS traversal visits each node exactly once by tracking visited nodes. This implementation doesn't track visited nodes, allowing a node to be enqueued and executed multiple times if it has multiple incoming edges.

2. **Determinism is violated**: The specification claims "Correctness by Construction" and the system is meant to be deterministic. However, the final result depends on:
    - The order edges were added to the graph (affects adjacency list ordering)
    - Which path through the graph happens to be processed last in BFS order
    - In graphs with multiple terminal nodes or convergent paths, there's no clear semantic meaning for "the" final result

3. **Unexpected behavior for users**: A user creating a diamond graph would reasonably expect either:
    - Node 3 to execute once with some combined input from nodes 1 and 2, OR
    - An error indicating that the graph structure is ambiguous, OR
    - A clearly documented behavior about handling multiple inputs

   Instead, node 3 silently executes twice, and only the last execution's result is returned.

## Failing test

### Test script

```rust
/*
 * Test to demonstrate that nodes with multiple incoming edges are executed multiple times
 */
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use deep_causality_core::ControlFlowBuilder;
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

static NODE_3_EXECUTIONS: AtomicUsize = AtomicUsize::new(0);

fn main() {
    // Create a diamond graph:
    //     0
    //    / \
    //   1   2
    //    \ /
    //     3
    //
    // Node 3 has TWO incoming edges (from nodes 1 and 2)
    // In standard BFS, node 3 should be visited exactly once
    // Bug: node 3 is executed twice (once per incoming edge)

    println!("Test: Node with multiple incoming edges");
    println!("Diamond graph: 0 -> 1 -> 3, 0 -> 2 -> 3\n");

    let mut builder = ControlFlowBuilder::<TestProtocol>::new();

    let n0 = builder.add_node(|x: i32| {
        println!("  Node 0: input={}, output={}", x, x * 2);
        x * 2
    });

    let n1 = builder.add_node(|x: i32| {
        println!("  Node 1: input={}, output={}", x, x + 10);
        x + 10
    });

    let n2 = builder.add_node(|x: i32| {
        println!("  Node 2: input={}, output={}", x, x + 100);
        x + 100
    });

    let n3 = builder.add_node(|x: i32| {
        let count = NODE_3_EXECUTIONS.fetch_add(1, Ordering::SeqCst) + 1;
        println!("  Node 3 (execution #{}): input={}, output={}", count, x, x + 1000);
        x + 1000
    });

    // Create diamond: 0 -> 1, 0 -> 2, 1 -> 3, 2 -> 3
    builder.connect(n0, n1);
    builder.connect(n0, n2);
    builder.connect(n1, n3);
    builder.connect(n2, n3);

    let graph = builder.build();
    let mut queue = VecDeque::new();

    println!("Starting execution with input=5:\n");
    let result = graph.execute(TestProtocol::Int(5), 0, 20, &mut queue);

    let n3_count = NODE_3_EXECUTIONS.load(Ordering::SeqCst);
    println!("\nNode 3 was executed {} time(s)", n3_count);

    match result {
        Ok(TestProtocol::Int(val)) => {
            println!("Final result: {}", val);

            // Expected execution order (BFS):
            // 1. Node 0: 5 -> 10
            // 2. Node 1: 10 -> 20
            // 3. Node 2: 10 -> 110
            // 4. Node 3: 20 -> 1020 (first execution)
            // 5. Node 3: 110 -> 1110 (second execution)
            //
            // Last result should be 1110

            assert_eq!(n3_count, 1, "Expected node 3 to execute once (standard BFS), but it executed {} times", n3_count);

            // This assertion will fail, demonstrating the bug
        },
        Ok(other) => panic!("Unexpected protocol variant: {:?}", other),
        Err(e) => panic!("Error: {:?}", e),
    }
}
```

### Test output

```
Test: Node with multiple incoming edges
Diamond graph: 0 -> 1 -> 3, 0 -> 2 -> 3

Starting execution with input=5:

  Node 0: input=5, output=10
  Node 1: input=10, output=20
  Node 2: input=10, output=110
  Node 3 (execution #1): input=20, output=1020
  Node 3 (execution #2): input=110, output=1110

Node 3 was executed 2 time(s)
Final result: 1110
thread 'main' panicked at test_multiple_visits.rs:97:13:
Expected node 3 to execute once (standard BFS), but it executed 2 times
```

The test clearly shows that node 3 is executed twice (once with input 20, once with input 110), violating the standard BFS property where each node is visited exactly once.

# Full context

The `ExecutableGraph` is the runtime artifact produced by the `ControlFlowBuilder`. It represents a directed graph of nodes (functions) connected by edges (data flow). The `execute` method is the primary way to run the graph:

1. **Builder phase** (`ControlFlowBuilder`): Users add typed nodes (functions) and connect them. Type checking ensures output types match input types at compile time.

2. **Build phase** (`build()` method): The builder is consumed and produces an `ExecutableGraph` with:
    - A vector of `ExecutableNode`s (type-erased functions wrapped in adapters)
    - An adjacency list representing the directed edges

3. **Execution phase** (`execute()` method): Starting from a specified node, the graph is traversed using a queue (claimed to be BFS), executing each node's function and propagating outputs to neighbor nodes.

The execute method is called from:
- Example code in `examples/core_examples/examples/control_flow_builder.rs`
- Tests in `deep_causality_core/tests/types/builder/executable_graph_tests.rs`
- Potentially from user applications using the deep_causality_core library

The bug affects any graph with:
- **Convergent paths** (multiple paths leading to the same node, like a diamond graph)
- **Multiple terminal nodes** (nodes with no outgoing edges)

In both cases, the returned result is ambiguous and depends on execution order rather than logical structure.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Test coverage focuses on simple linear graphs**: The existing tests (`test_execute_graph`, `test_execute_graph_max_steps`, `test_execute_graph_out_of_bounds`) use simple linear chains or cycles. None test diamond graphs or graphs with multiple terminal nodes.

2. **Example code uses linear pipelines**: The example in `control_flow_builder.rs` shows a simple pipeline: `read_sensor -> analyze_field -> check_safety`, which is a linear chain with no convergent paths.

3. **The bug's symptoms are subtle**: The graph doesn't crash or obviously malfunction. It executes successfully and returns *a* result, just not necessarily the *expected* result. Users might not realize the issue unless they:
    - Carefully track node execution counts
    - Have side effects in their node functions that reveal multiple executions
    - Compare results against expected behavior in complex graphs

4. **Ambiguous documentation**: The docstring says "BFS approach" but doesn't specify behavior for:
    - Nodes with multiple incoming edges
    - Graphs with multiple terminal nodes
    - What "the final result" means in complex graphs

   Users might assume the current behavior is intentional, even if surprising.

5. **The use case might be rare**: If most users build simple linear pipelines or trees (where each node has at most one incoming edge), they wouldn't encounter this issue. The bug only manifests in graphs with convergent paths (DAGs where nodes have multiple incoming edges).

6. **Possible confusion with data flow semantics**: If users think of this as a "data flow" graph where nodes *should* process multiple inputs, they might not question the behavior. However, the single-input function signature `Fn(I) -> O` suggests each node processes one input at a time, not a merged/joined input.

# Recommended fix

The fix depends on the intended semantics:

**Option 1: Standard BFS (each node visited once)**

If nodes should execute exactly once:

```rust
pub fn execute(
    &self,
    input: P,
    start_node: usize,
    max_steps: usize,
    queue: &mut VecDeque<(usize, P)>,
) -> Result<P, GraphError> {
    if start_node >= self.nodes.len() {
        return Err(GraphError::StartNodeOutOfBounds(start_node));
    }

    queue.clear();
    queue.push_back((start_node, input));

    let mut steps = 0;
    let mut last_result = None;
    let mut visited = vec![false; self.nodes.len()]; // <-- FIX 🟢 Track visited nodes

    while let Some((node_idx, node_input)) = queue.pop_front() {
        if steps >= max_steps {
            return Err(GraphError::MaxStepsExceeded(max_steps));
        }

        if visited[node_idx] { // <-- FIX 🟢 Skip already visited nodes
            continue;
        }
        visited[node_idx] = true; // <-- FIX 🟢 Mark as visited

        // Execute the current node's logic
        let node = &self.nodes[node_idx];
        let output = (node.func)(node_input);

        // Store result (cloned because we might need it for multiple neighbors)
        last_result = Some(output.clone());

        // Propagate to neighbors
        if let Some(neighbors) = self.adjacency.get(node_idx) {
            for &neighbor_idx in neighbors {
                if !visited[neighbor_idx] { // <-- FIX 🟢 Only enqueue if not visited
                    queue.push_back((neighbor_idx, output.clone()));
                }
            }
        }

        steps += 1;
    }

    last_result.ok_or(GraphError::GraphExecutionProducedNoResult)
}
```

However, this introduces a new issue: if a node has multiple incoming edges, which input does it receive? With the above fix, it would receive the input from whichever path reaches it first (still order-dependent).

**Option 2: Document and enforce single-path graphs**

Add validation in the `build()` method to detect and reject graphs where any node has multiple incoming edges. Update documentation to clarify that each node must have at most one incoming edge.

**Option 3: Require explicit terminal node specification**

Change the API to require users to specify which node(s) are terminal. Return only the results from those nodes, either as a single value (if one terminal) or as a collection (if multiple terminals).

**Recommendation**: Option 1 with additional API changes to handle multi-input nodes properly, or Option 2 if the current single-input node model should be preserved. Option 3 is the most user-friendly but requires the most API changes.


--

ExecutableGraph::execute() returns arbitrary "last processed" output for fan‑out graphs

# Summary
- **Context**: The `ExecutableGraph::execute()` method performs breadth-first traversal of a causal graph and returns a final result after processing all reachable nodes.
- **Bug**: In graphs with multiple terminal nodes (fan-out structures), the method returns the output of whichever node happened to be processed last in BFS order, not a semantically meaningful "final result".
- **Actual vs. expected**: The return value is non-deterministic and depends on edge insertion order rather than graph semantics or any notion of which node represents the "final" computation.
- **Impact**: Graphs with fan-out structures produce unpredictable results that vary based on implementation details (edge ordering) rather than the logical structure of the computation, violating the stated goal of "deterministic control flow".

# Code with bug

```rust
pub fn execute(
    &self,
    input: P,
    start_node: usize,
    max_steps: usize,
    queue: &mut VecDeque<(usize, P)>,
) -> Result<P, GraphError> {
    if start_node >= self.nodes.len() {
        return Err(GraphError::StartNodeOutOfBounds(start_node));
    }

    queue.clear();
    queue.push_back((start_node, input));

    let mut steps = 0;
    let mut last_result = None;

    while let Some((node_idx, node_input)) = queue.pop_front() {
        if steps >= max_steps {
            return Err(GraphError::MaxStepsExceeded(max_steps));
        }

        // Execute the current node's logic
        let node = &self.nodes[node_idx];
        let output = (node.func)(node_input);

        // Store result (cloned because we might need it for multiple neighbors)
        last_result = Some(output.clone());  // <-- BUG 🔴 blindly overwrites with each node's output

        // Propagate to neighbors
        if let Some(neighbors) = self.adjacency.get(node_idx) {
            for &neighbor_idx in neighbors {
                queue.push_back((neighbor_idx, output.clone()));
            }
        }

        steps += 1;
    }

    last_result.ok_or(GraphError::GraphExecutionProducedNoResult)  // <-- BUG 🔴 returns arbitrary last value
}
```

# Evidence

## Failing test

### Test script

```rust
/*
 * Test to demonstrate the bug in ExecutableGraph::execute()
 *
 * The bug: In graphs with multiple terminal nodes (fan-out),
 * the execute() method returns the output of whichever node
 * happened to be processed last in BFS order, not a deterministic
 * "final result" of the graph.
 */

use std::collections::VecDeque;
use deep_causality_core::{ControlFlowBuilder, ControlFlowProtocol, FromProtocol, ToProtocol};

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

fn main() {
    // Build a graph with fan-out: Node0 -> Node1, Node0 -> Node2
    // Node1 adds 10, Node2 adds 100
    let mut builder: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();

    fn identity(x: i32) -> i32 { x }
    fn add_ten(x: i32) -> i32 { x + 10 }
    fn add_hundred(x: i32) -> i32 { x + 100 }

    let n0 = builder.add_node(identity);  // Node 0
    let n1 = builder.add_node(add_ten);    // Node 1
    let n2 = builder.add_node(add_hundred); // Node 2

    // Create fan-out: 0 -> 1, 0 -> 2
    builder.connect(n0, n1);
    builder.connect(n0, n2);

    let graph = builder.build();
    let mut queue = VecDeque::new();

    // Execute with input 5
    let result = graph.execute(TestProtocol::Int(5), 0, 10, &mut queue);

    match result {
        Ok(TestProtocol::Int(val)) => {
            println!("Result: {}", val);
            // BFS order: Node0 (returns 5), then Node1 (returns 15), then Node2 (returns 105)
            // The function returns whichever was processed last
            // Since edges are added in order [0->1, 0->2], queue will be [(1,5), (2,5)]
            // Pop (1,5): output=15, last_result=15
            // Pop (2,5): output=105, last_result=105
            // Returns 105

            if val == 105 {
                println!("BUG CONFIRMED: Result is {}, which is the output of Node2 (last processed).", val);
                println!("This demonstrates that the return value depends on BFS traversal order,");
                println!("not on any semantic notion of 'final result'.");
            } else if val == 15 {
                println!("Result is {}, which is the output of Node1.", val);
            } else {
                println!("Unexpected result: {}", val);
            }
        }
        Ok(TestProtocol::Error(e)) => {
            println!("Got error in protocol: {}", e);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    // Test with different edge ordering
    println!("\n--- Test 2: Reverse edge order ---");
    let mut builder2: ControlFlowBuilder<TestProtocol> = ControlFlowBuilder::new();

    let n0 = builder2.add_node(identity);  // Node 0
    let n1 = builder2.add_node(add_ten);    // Node 1
    let n2 = builder2.add_node(add_hundred); // Node 2

    // Create fan-out with REVERSED order: 0 -> 2, 0 -> 1
    builder2.connect(n0, n2);
    builder2.connect(n0, n1);

    let graph2 = builder2.build();
    let mut queue2 = VecDeque::new();

    let result2 = graph2.execute(TestProtocol::Int(5), 0, 10, &mut queue2);

    match result2 {
        Ok(TestProtocol::Int(val)) => {
            println!("Result: {}", val);
            // Now edges are [0->2, 0->1], so queue will be [(2,5), (1,5)]
            // Pop (2,5): output=105, last_result=105
            // Pop (1,5): output=15, last_result=15
            // Returns 15

            if val == 15 {
                println!("BUG CONFIRMED: Result is {} (Node1's output) when edges added in reverse order.", val);
                println!("The same graph structure returns different results based on edge insertion order!");
            }
        }
        Ok(TestProtocol::Error(e)) => {
            println!("Got error in protocol: {}", e);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}
```

### Test output

```
Result: 105
BUG CONFIRMED: Result is 105, which is the output of Node2 (last processed).
This demonstrates that the return value depends on BFS traversal order,
not on any semantic notion of 'final result'.

--- Test 2: Reverse edge order ---
Result: 15
BUG CONFIRMED: Result is 15 (Node1's output) when edges added in reverse order.
The same graph structure returns different results based on edge insertion order!
```

## Example

Consider a simple graph with one source node that fans out to two terminal nodes:

```
    Node0 (identity)
      /        \
   Node1      Node2
  (x+10)     (x+100)
```

**Scenario 1: Edges added as [0→1, 0→2]**

1. Queue starts: `[(0, 5)]`
2. Process Node0 with input 5, output = 5, `last_result = Some(5)`
3. Add neighbors to queue: `[(1, 5), (2, 5)]`
4. Process Node1 with input 5, output = 15, `last_result = Some(15)`
5. Process Node2 with input 5, output = 105, `last_result = Some(105)`
6. **Return value: 105** (output of Node2, the last node processed)

**Scenario 2: Edges added as [0→2, 0→1]**

1. Queue starts: `[(0, 5)]`
2. Process Node0 with input 5, output = 5, `last_result = Some(5)`
3. Add neighbors to queue: `[(2, 5), (1, 5)]` (different order!)
4. Process Node2 with input 5, output = 105, `last_result = Some(105)`
5. Process Node1 with input 5, output = 15, `last_result = Some(15)`
6. **Return value: 15** (output of Node1, the last node processed)

**The same graph structure returns different results (105 vs 15) based solely on the order edges were inserted.**

## Inconsistency with own spec / docstring

### Reference spec

From `specs/implemented/core_control_flow_builder.md` line 7:
```markdown
# Specification: Deterministic Control Flow Builder

**Version:** 1.0
**Status:** Active
**Module:** `deep_causality_core::builder`

## 1. Abstract

The **Control Flow Builder** is a compile-time architectural pattern designed to construct deterministic, type-safe causal graphs.
```

From `specs/implemented/core_control_flow_builder.md` line 22:
```markdown
## 2. Design Philosophy

1.  **Correctness by Construction:** Invalid graph topologies (type mismatches) are unrepresentable states in the compiler.
2.  **Zero-Cost Abstraction:** The type checks occur solely during the build phase. The runtime artifact uses simple index-based iteration.
```

### Current code

`deep_causality_core/src/types/builder/executable_graph.rs`:
```rust
/// # Returns
/// * `Result<P, GraphError>` - The final result of the execution or an error.
pub fn execute(
    &self,
    input: P,
    start_node: usize,
    max_steps: usize,
    queue: &mut VecDeque<(usize, P)>,
) -> Result<P, GraphError> {
    // ... BFS traversal ...
    let mut last_result = None;

    while let Some((node_idx, node_input)) = queue.pop_front() {
        // ...
        last_result = Some(output.clone());
        // ...
    }

    last_result.ok_or(GraphError::GraphExecutionProducedNoResult)
}
```

### Contradiction

The specification explicitly states the system is designed for "**deterministic** control flow" as a core design goal. However, the current implementation violates this guarantee:

1. **Non-determinism**: The "final result" varies based on edge insertion order, which is an implementation detail that should not affect the logical result of a computation.

2. **Semantic ambiguity**: The docstring promises "the final result of the execution", but in graphs with multiple terminal nodes, there is no clear semantic definition of what "final" means. The implementation arbitrarily chooses the last node processed in BFS order.

3. **Hidden dependency**: The result depends on the order that `builder.connect()` calls were made, which is not documented and contradicts the "correctness by construction" philosophy.

# Full context

The `ExecutableGraph::execute()` method is the core runtime execution engine for the Control Flow Builder system in `deep_causality_core`. This system is designed for safety-critical applications (avionics, quantum control) where deterministic behavior is essential.

The method is called by user code after building a graph with `ControlFlowBuilder`. Users call:
1. `builder.add_node()` to register computation nodes
2. `builder.connect()` to link nodes (with compile-time type checking)
3. `builder.build()` to create an `ExecutableGraph`
4. `graph.execute()` to run the computation

The existing tests (`deep_causality_core/tests/types/builder/executable_graph_tests.rs`) only test linear pipelines (sequential chains of nodes), where this bug doesn't manifest. All example code (`examples/core_examples/examples/control_flow_builder.rs`, `examples/core_examples/examples/control_flow_strict_zst.rs`) also uses linear pipelines.

However, the system allows arbitrary directed graphs through the `connect()` API. Nothing prevents users from creating fan-out structures where a single node connects to multiple downstream nodes. When such graphs are executed, the return value becomes unpredictable.

The system is used in:
- Physics simulations (sensor → analysis → safety check pipelines)
- Control systems (feedback loops and multi-path decision trees)
- Causal reasoning (modeling complex cause-effect relationships)

In these domains, non-deterministic behavior could lead to:
- **Safety violations**: Different results from the same logical computation could cause incorrect safety decisions
- **Debugging nightmares**: Results that vary based on code order (not data) are extremely difficult to diagnose
- **Certification failures**: Safety-critical systems require deterministic behavior for DO-178C/ISO 26262 certification

# Why has this bug gone undetected?

This bug has remained undetected because:

1. **Limited test coverage**: All existing tests use linear pipelines (chains) where each node has exactly one outgoing edge. The bug only manifests in graphs with fan-out (one node with multiple outgoing edges).

2. **Example code patterns**: All example code follows the same linear pattern (sensor → analysis → check), which represents the primary intended use case but doesn't exercise the full graph capabilities.

3. **Implicit use case assumption**: The system's design appears to assume linear pipelines as the primary pattern, even though the API allows arbitrary directed graphs. The specification document's example (lines 279-318) only shows linear chains.

4. **Subtle manifestation**: The bug doesn't cause crashes or obvious errors. It produces valid output—just the wrong value. In complex systems, this could easily be mistaken for a logic error in the user's computation rather than a framework bug.

5. **Edge case rarity**: Most causal reasoning naturally follows linear or tree-merge patterns (multiple inputs converging to one output), not tree-split patterns (one input diverging to multiple outputs). Fan-out structures are less common in typical causal workflows.

6. **Recent addition**: The feature was added in commit fb6b35a4 (Nov 30, 2025), making it relatively new code that hasn't been extensively battle-tested in production scenarios.

# Recommended fix

The bug exists because the concept of "final result" is semantically unclear for graphs with multiple terminal nodes. Several fix approaches are possible:

**Option 1: Restrict to linear pipelines only**
- Add validation in `build()` to reject graphs where any node has multiple outgoing edges
- Document that only linear pipelines are supported
- This matches current usage patterns but limits expressiveness

**Option 2: Return all terminal node results**
- Change return type to `Result<Vec<P>, GraphError>`
- Track all terminal nodes (nodes with no outgoing edges)
- Return the outputs of all terminal nodes in a predictable order (e.g., sorted by node ID)
- This is semantically clearer but breaks the existing API

**Option 3: Require explicit result node**
- Add a `result_node: usize` parameter to `execute()`
- Return the output of that specific node only
- Provides explicit control but adds API complexity

**Option 4: Return aggregated result**
- Add a user-provided aggregation function to combine multiple terminal outputs
- Most flexible but most complex to implement

Given the system's design goals and current usage patterns, **Option 1** (restrict to linear pipelines) or **Option 3** (explicit result node) seem most appropriate. Option 1 maintains simplicity, while Option 3 provides flexibility without API ambiguity.
