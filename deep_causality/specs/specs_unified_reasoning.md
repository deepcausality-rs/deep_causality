# Specification & Implementation Plan: Unified Reasoning Engine


This document outlines the specification and step-by-step plan to
refactor the deep_causality reasoning engine. The goal is to move
from a purely deterministic, boolean-based system to a unified,
expressive engine capable of handling deterministic, probabilistic,
and contextual reasoning.

##  Overview & Goal


The current engine is limited to boolean (true/false) causal
evaluation, which restricts its ability to model nuanced, real-world
systems. This project will refactor the core reasoning
infrastructure to use a more expressive PropagatingEffect enum,
enabling hybrid models that combine multiple reasoning modes within
a single, consistent, and dyn-free framework.

## Core Architectural Design


The new architecture is based on a fundamentally new mechanism: a
causal link is defined by a stateless function and its static
configuration data.


1. The `Evidence` Enum's Dual Role: The existing Evidence enum,
   defined in
   @deep_causality/src/types/reasoning_types/unified_evidence.rs,
   will be used for two purposes:
    * Runtime Evidence: Representing the dynamic data that flows
      through the causal graph during reasoning.
    * Static Configuration: Representing the static, unchanging
      configuration of a single Causaloid.


2. Stateless `CausalFn`: The core of the new design is a simple,
   stateless function pointer, CausalFn. It accepts both the runtime
   evidence and its own static configuration to produce an effect.
   This eliminates the need for dyn Trait objects and complex state
   management within the causaloid itself.


3. Simplified `Causaloid` Struct: For Singleton types, the Causaloid
   will directly store the CausalFn pointer and an Evidence instance
   that serves as its static configuration. The causaloid becomes a
   simple container for the function and its parameters.

3. Key Data Structures


The following types, to be defined in
@deep_causality/src/types/reasoning_types/**, will form the
foundation of the new engine.



    1 // The effect that a causaloid can produce and propagate.
    2 // To be defined in 
      @deep_causality/src/types/reasoning_types/propagating_eff
      ect.rs
    3 pub enum PropagatingEffect {
    4     // Halts further reasoning along this path.
    5     Halting,
    6     // Represents a classical true/false outcome.
    7     Deterministic(bool),
    8     // Represents a probabilistic outcome (e.g., a 
      probability value).
    9     Probabilistic(NumericalValue),
    10     // Represents a link to another context for
    contextual reasoning.
    11     ContextualLink(ContextId, ContextoidId),
    12 }
    13
    14 // The unified function signature for all singleton
    causaloids.
    15 // To be updated in
    @deep_causality/src/types/alias_types/alias_function.rs
    16 pub type CausalFn = fn(
    17     runtime_evidence: &Evidence,
    18     static_config: &Evidence, // The Causaloid's own
    configuration
    19     context: &Option<Context<T>>, // Assuming a generic
    context
    20 ) -> Result<PropagatingEffect, CausalityError>;
    21
    22 // The updated Causaloid struct fields for Singleton
    types.
    23 struct Causaloid {
    24     // ... other fields
    25     causal_fn: Option<CausalFn>,
    26     static_config: Option<Evidence>,
    27     // ... other fields
    28 }


  ---


## Phase 1: Foundational Type Refactoring


Goal: Update the core function aliases and data structures. This is
a foundational step that will cause widespread compilation errors,
which will be fixed in subsequent phases.


* Implementation Steps:
    1. Navigate to
       @deep_causality/src/types/alias_types/alias_function.rs.
    2. Modify the CausalFn, ContextualCausalDataFn, and any related
       function type aliases to return Result<PropagatingEffect,
       CausalityError> instead of Result<bool, CausalityError>.
    3. Define the PropagatingEffect enum as specified above in @deep_
       causality/src/types/reasoning_types/propagating_effect.rs,
       ensuring it's accessible to the necessary modules.

* Testing Strategy:
    * No new functional tests are needed in this phase. The primary
      goal is to get the types right.


* Verification:
    1. Run cargo check to identify all locations in the codebase that
       are now broken due to the signature change. This list of errors
       will serve as a to-do list for the next phase.


### Design Note: Static fn Pointers for Causal Functions


A core design principle of the deep_causality library is that all
causal functions are defined as static function pointers (e.g., pub
type CausalFn = fn(...) -> ...), not as dynamically-dispatched trait
objects (e.g., Box<dyn Fn(...)>). This is a deliberate
architectural decision with significant benefits for performance,
predictability, and robust engineering.

The Core Principle: Explicit Inputs Only

By design, a causal function's behavior must be determined only by
its explicit inputs:


1. The runtime_evidence passed to it during graph traversal.
2. The static_config provided when the Causaloid is created.
3. The optional Context object it has access to.


This design intentionally disallows the use of closures that capture
variables from their surrounding environment. Supporting such
closures would require Box<dyn Fn(...)>, which introduces heap
allocation and the overhead of dynamic dispatch for every function
call. This would violate the library's commitment to a
high-performance, dyn-free architecture where the call-graph can be
heavily optimized by the compiler.

Practical Benefits: Separation of Concerns


While this might seem like a constraint, it is a practical
application of the separation of concerns principle, which is a
cornerstone of good software engineering. It enforces a clean and
healthy boundary between stateless logic and the state it operates
on.


* Reasoning Logic (The `CausalFn`): This is a pure, stateless, and
  highly reusable piece of code. For example, a single
  fn_check_threshold function can be used by thousands of different
  Causaloid instances across the application.
* Contextual Data (The `static_config` and `Context`): This is the
  data and state that the logic operates on. By keeping it separate,
  we gain several advantages:
    * Clarity & Predictability: The behavior of a function is
      perfectly predictable based on its inputs. There are no hidden
      side-effects or dependencies on captured state.
    * Testability: Causal functions are trivial to unit test. You can
      test the logic in complete isolation by simply providing mock
      Evidence and Context objects.
    * Reusability: The same function can be reused in countless
      scenarios just by pairing it with different configuration data.
    * Performance: We avoid the runtime overhead of vtable lookups
      and ensure that function calls can be inlined by the compiler
      where possible.



In summary, the use of static fn pointers is a foundational design
choice that ensures the reasoning engine is not only fast but also
predictable, testable, and architecturally sound. It guides
developers to cleanly separate their reasoning algorithms from the
data those algorithms act upon.


## Phase 2: Causaloid Struct and Causable Trait Implementation


Goal: Rework the Causaloid to use the new stateless function model
and update the core Causable trait.


* Implementation Steps:
    1. In deep_causality/src/types/causaloid.rs, modify the Causaloid
       struct:
        * Add the fields: causal_fn: Option<CausalFn> and
          static_config: Option<Evidence>.
        * Remove the previous fields related to the old causal
          function mechanism.
    2. Implement the new private method
       Causaloid::reason_singleton(&self, runtime_evidence: &Evidence)
       -> Result<PropagatingEffect, CausalityError>.
    3. In @deep_causality/src/traits/causable/mod.rs, refactor the
       Causable trait. Replace the verify_single_cause and
       verify_all_causes methods with a new primary evaluation method
       (e.g., fn evaluate(&self, evidence: &Evidence) ->
       Result<PropagatingEffect, CausalityError>) that calls
       reason_singleton for the base case.


* Testing Strategy:
    1. Create new unit tests in
       deep_causality/tests/types/causaloid_tests.rs specifically for
       the reason_singleton method.
    2. These tests should define simple, stateless CausalFn
       implementations, construct a Causaloid, and assert that the
       correct PropagatingEffect is returned.

* Verification:
    1. Run cargo test --lib to execute the new unit tests.
    2. Run cargo clippy to ensure the new code adheres to Rust best
       practices.

  3. 

  
### Design Note: Centralized Graph Logic via Extension Traits

A key architectural pattern in the deep_causality library is the
separation of the core causal graph data structure from the complex
algorithms that operate on it. This is achieved by using a set of
traits with default implementations, a pattern often referred to as
"extension traits" in Rust.

The core of this design can be seen in the
@deep_causality/src/traits/causable_graph/** module.

The Architectural Pattern


1. The Core Interface (`CausableGraph`): The CausableGraph<T> trait
   defines the fundamental contract for what a causal graph data
   structure must be able to do. It specifies the essential,
   low-level API for manipulating the graph: adding/removing nodes
   (add_causaloid), managing edges (add_edge), and retrieving data
   (get_causaloid). It makes no assumptions about how these
   operations are implemented.


2. The Algorithmic Extensions (`CausableGraphReasoning`,
   `CausableGraphExplaining`): These traits are the workhorses of the
   library. They are defined with a where Self: CausableGraph<T> bound
   and provide default implementations for all their methods. These
   methods contain the most complex logic in the entire codebase:

    * Graph traversal algorithms (e.g., the Breadth-First Search in
      reason_subgraph_from_cause).
    * Pathfinding (e.g., get_shortest_path).
    * Recursive explanation generation (e.g., the Depth-First Search
      in explain_from_to_cause).


Any struct that implements the basic CausableGraph contract
automatically inherits this rich, high-level functionality for
free.

Rationale: Centralize, Simplify, and Evolve

This design was chosen for several critical long-term benefits:


* Centralization of Complexity: All of the intricate, error-prone,
  and performance-sensitive graph traversal logic is located in one
  single, authoritative place. There is only one BFS implementation
  and one DFS implementation to worry about.


* Simplified Maintenance: This centralization is a massive advantage
  for maintainability. If a bug is found in the graph reasoning
  logic, or if a more efficient traversal algorithm is developed, the
  fix or improvement only needs to be applied in one location—the
  default implementation within the trait. Every part of the library
  that uses the trait will immediately benefit from the update
  without requiring any further changes.


* Facilitating Code Evolution: The architecture makes the library
  significantly easier to evolve. If a new type of graph-wide
  analysis or reasoning is needed in the future, it can be added as a
  new method with a default implementation to the
  CausableGraphReasoning trait. This new functionality will instantly
  become available on all existing and future causal graph types,
  dramatically reducing the effort required to extend the library's
  capabilities.
