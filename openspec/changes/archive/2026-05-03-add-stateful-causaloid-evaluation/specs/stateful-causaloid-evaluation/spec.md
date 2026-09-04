## ADDED Requirements

### Requirement: StatefulMonadicCausable trait

The `deep_causality` crate SHALL expose a public trait
`StatefulMonadicCausable<I, O, S, C>` declaring exactly one method
`evaluate_stateful(&self, incoming: &PropagatingProcess<I, S, C>) -> PropagatingProcess<O, S, C>`,
re-exported at the crate root via `deep_causality::StatefulMonadicCausable`.

#### Scenario: Trait is reachable from a downstream crate

- **WHEN** a downstream crate writes
  `use deep_causality::StatefulMonadicCausable;`
- **THEN** the import resolves and the trait, with its one method
  `evaluate_stateful`, is in scope

#### Scenario: Existing MonadicCausable trait is unchanged

- **WHEN** the source of `traits/causable/mod.rs` and the existing impl in
  `types/causal_types/causaloid/causable.rs` are inspected
- **THEN** the public signature of `MonadicCausable<I, O>::evaluate` is
  unchanged and its body is unchanged

### Requirement: Stateful singleton evaluation threads state and context

For a `Causaloid<I, O, S, C>` of type `Singleton` whose `context_causal_fn` is
set, `evaluate_stateful` SHALL invoke the stored closure with the `state` and
`context` carried by the `incoming` process (not with `S::default()` and not
with `None`), and SHALL return a `PropagatingProcess<O, S, C>` whose `state`
is the `state` produced by the closure (not a freshly defaulted `S`) and
whose `context` is the `context` carried by the closure's output.

#### Scenario: Closure observes the incoming state and context

- **WHEN** a caller invokes `evaluate_stateful` on a singleton
  `Causaloid<I, O, S, C>` with a context-aware closure that records the
  values of its `state` and `context` arguments into the outgoing process's
  log
- **THEN** the recorded values match the `state` and `context` that the
  caller placed on the incoming process (not `S::default()`, not `None`)

#### Scenario: Returned state is the state produced by the closure

- **WHEN** the closure mutates the state (e.g. increments a counter field
  on `S`) and returns the mutated state on its outgoing process
- **THEN** the `state` on the process returned by `evaluate_stateful` is
  the mutated state, byte-for-byte equal to what the closure produced

### Requirement: Stateful singleton evaluation supports stateless closures

For a `Causaloid<I, O, S, C>` of type `Singleton` whose `causal_fn` (the
stateless closure variant) is set rather than `context_causal_fn`,
`evaluate_stateful` SHALL invoke the stateless closure on the value, and
SHALL return a `PropagatingProcess<O, S, C>` whose `state` and `context`
fields are pass-through copies of the `state` and `context` carried by the
incoming process.

#### Scenario: Stateless closure does not perturb caller state

- **WHEN** a caller invokes `evaluate_stateful` on a singleton causaloid
  built with `Causaloid::new(...)` (stateless variant) and a non-trivial
  incoming `state`
- **THEN** the `state` on the returned process equals the incoming `state`
  by `PartialEq`

### Requirement: Singleton error short-circuit preserves state and logs

When the singleton causaloid produces an error during stateful evaluation,
`evaluate_stateful` SHALL return a process whose `error` is `Some(err)`,
whose `value` is `EffectValue::default()`, whose `state` is the `state`
carried by the incoming process at the moment of evaluation (not
`S::default()`), and whose `logs` contain every log entry the implementation
would have emitted up to and including the failing step.

#### Scenario: Error path preserves caller state byte-for-byte

- **WHEN** the closure returns an error
- **THEN** `state` on the returned process equals the `state` of the
  incoming process by `PartialEq`, and `logs` contain at minimum the input
  log entry written before the closure was invoked

### Requirement: StatefulMonadicCausableCollection trait and Causaloid impl

The `deep_causality` crate SHALL expose a public trait
`StatefulMonadicCausableCollection<I, O, S, C, T>` declaring exactly one
method
`evaluate_collection_stateful(&self, incoming: &PropagatingProcess<I, S, C>, logic: &AggregateLogic, threshold_value: Option<NumericalValue>) -> PropagatingProcess<O, S, C>`,
re-exported at the crate root.

#### Scenario: Trait is reachable from a downstream crate

- **WHEN** a downstream crate writes
  `use deep_causality::StatefulMonadicCausableCollection;`
- **THEN** the import resolves and the method `evaluate_collection_stateful`
  is callable on the existing collection accessor types

#### Scenario: Stateful collection aggregates per AggregateLogic and preserves state

- **WHEN** a caller invokes `evaluate_collection_stateful` on a collection
  of three singleton causaloids configured with `AggregateLogic` having a
  threshold of 2-of-3, with an incoming process carrying non-trivial
  `state` and `context`
- **THEN** the aggregation outcome equals what the stateless
  `evaluate_collection` would compute on the same logical input AND the
  returned process's `state` and `context` are not defaulted (they are
  threaded through and reflect any per-item closure mutations)

### Requirement: Stateful collection error short-circuits

When any item in the collection returns an error during stateful evaluation,
`evaluate_collection_stateful` SHALL halt aggregation, return a process
carrying that error, preserve all logs accumulated up to and including the
failing item, and preserve the state at the moment the failing item began
evaluation.

#### Scenario: Failing item does not advance state past the failure point

- **WHEN** the second item in a three-item collection returns an error
  while incrementing a counter field on `S`
- **THEN** the `state` on the returned process reflects the counter value
  carried into the second item (i.e. the result of the first item's
  successful step), not the counter value the third item would have
  produced

### Requirement: StatefulMonadicCausableGraphReasoning trait and impl

The `deep_causality` crate SHALL expose a public trait
`StatefulMonadicCausableGraphReasoning<V, S, C>` declaring at minimum these
three methods:

- `evaluate_single_cause_stateful(&self, index: usize, effect: &PropagatingProcess<V, S, C>) -> PropagatingProcess<V, S, C>`
- `evaluate_subgraph_from_cause_stateful(&self, start_index: usize, initial_effect: &PropagatingProcess<V, S, C>) -> PropagatingProcess<V, S, C>`
- `evaluate_shortest_path_between_causes_stateful(&self, start_index: usize, stop_index: usize, initial_effect: &PropagatingProcess<V, S, C>) -> PropagatingProcess<V, S, C>`

The trait SHALL be implemented by every existing graph type that implements
`MonadicCausableGraphReasoning<V, PS, C>` so the stateful methods are
available wherever the stateless methods are. The trait SHALL be
re-exported at the crate root.

#### Scenario: All three methods are callable on an existing graph type

- **WHEN** a caller has an instance of a graph type that already implements
  `MonadicCausableGraphReasoning<V, PS, C>` and writes
  `use deep_causality::StatefulMonadicCausableGraphReasoning;`
- **THEN** all three methods listed above are callable on that instance
  with non-`()` `S` and `C` type arguments

#### Scenario: BFS subgraph traversal threads state across nodes

- **WHEN** a caller invokes `evaluate_subgraph_from_cause_stateful` on a
  graph of three nodes wired in a path, with each node's closure
  incrementing a counter field on `S`
- **THEN** the `state` on the returned process reflects three counter
  increments AND the `logs` contain entries from all three nodes in
  traversal order

#### Scenario: RelayTo adaptive jump preserves state

- **WHEN** a node in the BFS traversal emits a `PropagatingEffect::RelayTo`
  pointing to a different node, while the incoming process carries
  non-trivial `state`
- **THEN** the relayed-to node observes the same `state` that the relaying
  node carried at the moment of relay, and the final returned process
  reflects the relayed-to node's state mutation (not a defaulted state)

### Requirement: Stateful graph error short-circuits across BFS traversal

When any node in the graph traversal returns an error during stateful
evaluation, `evaluate_subgraph_from_cause_stateful` SHALL halt traversal,
return a process carrying that error, preserve logs accumulated up to and
including the failing node, and preserve the state carried at the moment
the failing node began evaluation.

#### Scenario: Downstream nodes do not execute after a failing node

- **WHEN** the second of three sequentially-connected nodes returns an
  error
- **THEN** the third node's closure is not invoked AND the returned
  process's `state` reflects the counter value at entry to the second
  node, not at entry to a hypothetical third node

### Requirement: StatefulContextualCausalFn type alias

The `deep_causality` crate SHALL expose a public type alias
`StatefulContextualCausalFn<I, O, S, C>` whose underlying type is exactly
`fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>`, defined in
`deep_causality/src/alias/alias_function.rs` adjacent to the existing
`ContextualCausalFn` alias, and re-exported at the crate root.

#### Scenario: Alias is reachable from a downstream crate

- **WHEN** a downstream crate writes
  `use deep_causality::StatefulContextualCausalFn;` and declares a
  function pointer of that aliased type
- **THEN** the import resolves and the function-pointer declaration
  compiles

#### Scenario: Alias resolves to the same fn type as ContextualCausalFn

- **WHEN** a downstream crate assigns a value of type
  `StatefulContextualCausalFn<I, O, S, C>` to a variable of type
  `ContextualCausalFn<I, O, S, C>` (or vice versa) for the same `I`,
  `O`, `S`, `C`
- **THEN** the assignment compiles, confirming the two aliases resolve
  to the same fn pointer type

#### Scenario: Existing ContextualCausalFn alias is unchanged

- **WHEN** the diff for `deep_causality/src/alias/alias_function.rs` is
  inspected
- **THEN** the existing `ContextualCausalFn` alias has zero modifications
  to its body, signature, or visibility

### Requirement: No new Causaloid constructors

This change SHALL NOT add any new associated function on
`Causaloid<I, O, STATE, CTX>`. Stateful evaluation is determined
entirely by which trait method the caller invokes
(`StatefulMonadicCausable::evaluate_stateful`,
`StatefulMonadicCausableCollection::evaluate_collection_stateful`, and
the three `StatefulMonadicCausableGraphReasoning` methods); no existing
`Causaloid` constructor makes a state-threading decision, and no new
constructor SHALL be introduced to mark one.

#### Scenario: Existing constructors are unchanged

- **WHEN** the diff for `deep_causality/src/types/causal_types/causaloid/mod.rs`
  is inspected
- **THEN** the existing `new`, `new_with_context`,
  `from_causal_collection`, `from_causal_collection_with_context`,
  `from_causal_graph`, and `from_causal_graph_with_context`
  constructors have zero modifications to their bodies or signatures

#### Scenario: No new constructor symbols are introduced

- **WHEN** the diff for `deep_causality/src/types/causal_types/causaloid/mod.rs`
  is inspected
- **THEN** no new `pub fn` is added to any `impl` block on `Causaloid`
  by this change

#### Scenario: Trait rustdoc directs the reader to existing constructors

- **WHEN** the rustdoc for `StatefulMonadicCausable::evaluate_stateful`
  is rendered
- **THEN** it explicitly names the existing
  `Causaloid::new_with_context` constructor as the way to author a
  Causaloid intended for stateful evaluation, and references the
  `StatefulContextualCausalFn` alias as the recommended type for the
  closure argument

### Requirement: Stateful execute_causal_logic helper

The crate-internal helper module
`types/causal_types/causaloid/causable_utils.rs` SHALL gain a function
`execute_causal_logic_stateful<I, O, S, C>(input: I, state: S, context: Option<C>, causaloid: &Causaloid<I, O, S, C>) -> PropagatingProcess<O, S, C>`
that invokes the causaloid's closure (context-aware or stateless variant)
without defaulting state or discarding context, and returns the resulting
`PropagatingProcess` without conversion. The existing
`execute_causal_logic` SHALL remain in the same file with its current
signature and body unchanged.

#### Scenario: Stateful helper does not call S::default()

- **WHEN** the source of `execute_causal_logic_stateful` is inspected
- **THEN** the body does not call `S::default()` or
  `<S as Default>::default()` and threads the `state` parameter directly
  into the closure call

#### Scenario: Original execute_causal_logic body is unchanged

- **WHEN** the diff for `causable_utils.rs` is inspected
- **THEN** the existing function `execute_causal_logic` has zero
  modifications inside its body or signature

### Requirement: Backward compatibility of the stateless API

This change SHALL NOT modify the signature, body, visibility, or location
of any existing public symbol on `MonadicCausable`,
`MonadicCausableCollection`, or `MonadicCausableGraphReasoning`, nor any of
their existing implementations. All existing tests in the repository's
`tests/` tree SHALL continue to compile and pass without modification.

#### Scenario: Existing tests continue to pass

- **WHEN** the existing test files under
  `deep_causality/tests/traits/causable/`,
  `deep_causality/tests/traits/causable_collection/`, and
  `deep_causality/tests/traits/causable_graph/` are run after this change
  is implemented
- **THEN** every existing test passes with no modifications to its source
  required

#### Scenario: Public stateless symbols are unchanged

- **WHEN** `cargo doc -p deep_causality --no-deps` is built before and
  after this change is implemented
- **THEN** the documented signatures of `MonadicCausable::evaluate`,
  `MonadicCausableCollection::evaluate_collection`,
  `MonadicCausableGraphReasoning::evaluate_single_cause`,
  `MonadicCausableGraphReasoning::evaluate_subgraph_from_cause`, and
  `MonadicCausableGraphReasoning::evaluate_shortest_path_between_causes`
  are byte-for-byte identical

### Requirement: Test coverage for stateful APIs

This change SHALL add unit test files mirroring the source tree, covering
the stateful singleton, collection, and graph evaluation paths under
`deep_causality/tests/`. Each test file SHALL be registered in its
corresponding `mod.rs` per AGENTS.md test conventions, and SHALL include
scenarios for:

- happy-path state evolution across one and across multiple steps,
- error short-circuit with state preservation,
- log aggregation in chronological order,
- pass-through behavior for the stateless-closure variant of singletons,
- collection aggregation with non-trivial state,
- graph BFS with state-mutating closures,
- graph `RelayTo` with non-trivial state.

#### Scenario: Tests are wired into the module tree

- **WHEN** `cargo test -p deep_causality` is run
- **THEN** the new stateful tests execute (i.e. they appear in the test
  output) and pass

### Requirement: No new external dependencies

This change SHALL NOT add any external crate to
`deep_causality/Cargo.toml` or to any other workspace `Cargo.toml`. The
implementation SHALL use only crates already declared as workspace
dependencies of `deep_causality` (notably `deep_causality_core` and
`deep_causality_haft`).

#### Scenario: Manifest diffs reveal no new external dependencies

- **WHEN** the diff to all `Cargo.toml` files under the workspace is
  inspected
- **THEN** no new entry is introduced under `[dependencies]` referencing a
  crates.io crate not already declared by the affected manifest before
  this change

### Requirement: AGENTS.md conventions

This change SHALL adhere to the conventions in `AGENTS.md` regarding code
organization, safety, and visibility:

- One trait per file under `src/traits/.../`.
- One impl per file under `src/types/.../`.
- Test files mirror the source tree under `tests/...` and are registered
  in their `mod.rs` files with `#[cfg(test)]`.
- No `unsafe` is introduced.
- No macros are introduced in `src/`.
- No prelude file is introduced.
- Public types' fields remain private; access through constructors,
  getters, and setters as appropriate. (No new public types are introduced
  by this change; only new traits.)

#### Scenario: New source files conform to one-trait-per-file

- **WHEN** the new source files are inspected
- **THEN** each file under `src/traits/.../` defines at most one new
  trait, and each file under `src/types/.../` provides at most one new
  impl block per trait

#### Scenario: No unsafe and no macros in lib code

- **WHEN** every new and modified file under `deep_causality/src/` is
  searched for the tokens `unsafe` and `macro_rules!`
- **THEN** zero matches are found in any file added or modified by this
  change
