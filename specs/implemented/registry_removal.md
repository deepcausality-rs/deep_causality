
Migration Plan: Removing CausaloidRegistry and Centralizing Causaloid Ownership

Problem Statement

The CausaloidRegistry introduces an layer of indirection and global state that complicates the ownership
model and the direct application of functional programming patterns (like monadic collection evaluation). The
goal is to remove this registry, allowing Causaloid instances to directly contain their sub-Causaloids or
sub-graphs, thereby simplifying evaluation logic and improving type safety.

High-Level Vision

* Causaloid of type Collection will directly own a Vec<Causaloid>.
* Causaloid of type Graph will directly own a CausaloidGraph<Causaloid>.
* The Causaloid::evaluate method will no longer take a registry parameter.
* The MonadicCausableCollection trait's evaluate_collection method will be directly utilized for
  CausaloidType::Collection.
* CausaloidGraph evaluation will also be simplified, with nodes directly accessible.

Detailed Migration Steps

Phase 1: Structural Changes to Causaloid

1. Remove `causal_registry` field from `Causaloid` struct:

* Location: deep_causality/src/types/causal_types/causaloid/mod.rs
* Change: Delete the line causal_registry: Option<Arc<CausaloidRegistry>>.

2. Update `causal_coll` and `causal_graph` fields to hold `Causaloid` instances directly:

* Location: deep_causality/src/types/causal_types/causaloid/mod.rs
* Change `causal_coll`: Modify causal_coll: Option<Arc<Vec<CausaloidId>>> to causal_coll:
  Option<Arc<Vec<Self>>>. (Self refers to Causaloid<I, O, D, S, T, ST, SYM, VS, VT>).
* Change `causal_graph`: Modify causal_graph: Option<Arc<CausaloidGraph<CausaloidId>>> to causal_graph:
  Option<Arc<CausaloidGraph<Self>>>.

Phase 2: Refactoring Causaloid Constructors

1. Update constructors using `CausaloidRegistry` and `CausaloidId`s (for Collections):

* `Causaloid::from_causal_collection`:
    * Location: deep_causality/src/types/causal_types/causaloid/mod.rs
    * Change:
        * Remove the internal registry = CausaloidRegistry::new() and the loop
          causal_coll.as_slice().iter().for_each(...) that registers causaloids.
        * The parameter causal_coll: Arc<Vec<Causaloid<>>> should now be directly assigned to
          self.causal_coll, without involving CausaloidIds or a registry.
        * Remove the causal_registry: Some(Arc::new(registry)) assignment.
* `Causaloid::from_causal_collection_with_registry`:
    * Location: deep_causality/src/types/causal_types/causaloid/mod.rs
    * Change: This constructor is now redundant as it relies on external IDs and a registry. Its signature
      and implementation should be refactored to take causal_coll: Arc<Vec<Self>> directly and then stored.
* `Causaloid::from_causal_collection_with_context`:
    * Location: deep_causality/src/types/causal_types/causaloid/mod.rs
    * Change: Similar to from_causal_collection, remove internal registry and directly store the provided
      Arc<Vec<Self>>.
* `Causaloid::from_causal_collection_with_context_and_registry`:
    * Location: deep_causality/src/types/causal_types/causaloid/mod.rs
    * Change: Similar to from_causal_collection_with_registry, refactor to take and store Arc<Vec<Self>>
      directly.

2. Update constructors using `CausaloidRegistry` and `CausaloidId`s (for Graphs):

* `Causaloid::from_causal_graph_with_registry`:
    * Location: deep_causality/src/types/causal_types/causaloid/mod.rs
    * Change: Modify the causal_graph parameter to causal_graph: Arc<CausaloidGraph<Self>>. The caller will
      now be responsible for constructing a CausaloidGraph that directly contains Causaloid instances.
      Remove the causal_registry parameter and assignment.
* `Causaloid::from_causal_graph_with_context_and_registry`:
    * Location: deep_causality/src/types/causal_types/causaloid/mod.rs
    * Change: Similar to the above, update causal_graph parameter to Arc<CausaloidGraph<Self>> and remove
      registry parameter/assignment.

Phase 3: Updating Causaloid::evaluate Logic

1. Modify `MonadicCausable::evaluate` signature:

* Location: deep_causality/src/types/causal_types/causaloid/causable.rs
* Change: Remove the registry: &CausaloidRegistry parameter. The new signature should be fn evaluate(&self,
  incoming_effect: &PropagatingEffect) -> PropagatingEffect.
* Impact: All call sites to this evaluate method will need to be updated to no longer pass a registry. This
  includes external calls and internal calls within CausaloidType::Graph or CausaloidType::Collection if
  Causaloids call each other's evaluate method.

2. Refactor `CausaloidType::Collection` branch in `evaluate`:

* Location: deep_causality/src/types/causal_types/causaloid/causable.rs
* Change:
    * After checking for coll_aggregate_logic existence, retrieve self.causal_coll.
    * It will now be of type Option<Arc<Vec<Self>>>.
    * Replace the entire manual loop (for &causaloid_id in coll_ids.iter() { ... }) with a call to the
      evaluate_collection method provided by the MonadicCausableCollection trait.
    * The code should look something like:

    1         let causal_collection = match self.causal_coll.as_ref() {
    2             Some(coll_arc) => coll_arc.as_ref(), // Get &Vec<Self>
    3             None => {
    4                 let err_msg = "Causaloid::evaluate: causal_collection is None".into();
    5                 return PropagatingEffect {
    6                     value: EffectValue::None,
    7                     error: Some(CausalityError(err_msg)),
    8                     logs: initial_monad.logs,
    9                 };
10             }
11         };
12
13         // Call the trait method. `collection_of_causaloids` now directly implements
MonadicCausableCollection.
14         causal_collection.evaluate_collection(
15             incoming_effect,
16             &aggregate_logic,
17             self.coll_threshold_value,
18         )
* Remove `causal_registry` checks: Lines like let registry = match &self.causal_registry { ... } will be
  removed.

3. Refactor `CausaloidType::Graph` branch in `evaluate`:

* Location: deep_causality/src/types/causal_types/causaloid/causable.rs
* Change:
    * Retrieve self.causal_graph: Option<Arc<CausaloidGraph<Self>>>.
    * The MonadicCausableGraphReasoning trait (from
      deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs) defines evaluate_subgraph_from_cause
      which currently takes a registry parameter.
    * The MonadicCausableGraphReasoning trait and its implementations will need to be updated to no longer
      require a CausaloidRegistry. The evaluate_subgraph_from_cause could then call
      node.evaluate(incoming_effect) directly on the Causaloid nodes contained within the
      CausaloidGraph<Self>. This will involve a deeper refactoring of CausaloidGraph and its reasoning
      methods to directly interact with Causaloid instances.

4. Refactor `MonadicCausableGraphReasoning` trait and implementations:

* Location: deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs and
  deep_causality/src/types/causal_types/causaloid_graph/causable_graph.rs
* Change:
    * Generalize the `MonadicCausableGraphReasoning` trait to be generic over the full `Causaloid` type
      (`Causaloid<I,O,D,S,T,ST,SYM,VS,VT>`) instead of `CausaloidId`.
    * Remove any `registry` parameters from the trait methods.
    * Update the implementation of this trait for `CausaloidGraph<Causaloid<...>>` to reflect the new
      trait signature.
    * Modify the method logic to directly retrieve and evaluate `Causaloid` objects from the graph using
      `self.get_causaloid(index).evaluate(...)` or similar direct access, removing any reliance on an
      external registry.

Phase 4: Levering HKT over Causal Collections (Verification)

This phase verifies that the structural changes enable the intended HKT benefits.

* Confirm `MonadicCausableCollection` usage: Ensure that once causal_coll is Arc<Vec<Causaloid>>, the
  evaluate_collection call is correctly resolved and functions as expected.
* Review `CausaloidGraph` evaluation: Verify that CausaloidGraph<Causaloid> can now directly traverse and
  evaluate its contained Causaloid nodes without manual lookups or an external registry. This might involve
  updating the evaluate method within CausaloidGraph to directly act on Causaloid objects.

Phase 5: Cleanup and Testing

1. Delete the `CausaloidRegistry` definition:

* Location: Identify and delete all files defining CausaloidRegistry and any related types, likely starting
  with deep_causality/src/types/causal_types/causaloid_registry/mod.rs (or similar path).

2. Update / Delete `CausaloidRegistry` tests:

* Location: deep_causality/tests/types/causaloid_registry/mod.rs
* Change: All tests in this file will become obsolete and should be removed.

3. Adjust `CausaloidGraph` structure and related traits:

* Ensure all references to CausaloidGraph<CausaloidId> are updated to CausaloidGraph<Self> (or
  CausaloidGraph<Causaloid<...>>).
* Review deep_causality/src/traits/causable_graph/graph/mod.rs and
  deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs to confirm they operate seamlessly with
  Causaloid objects directly.

4. Comprehensive Test Suite Update:

* Run make build and make test. All existing tests throughout the codebase will likely need updates due to
  the removal of CausaloidRegistry.
* Pay special attention to tests for CausaloidType::Collection and CausaloidType::Graph to ensure they
  function correctly with the direct ownership model.

  ---

This plan outlines a staged approach, starting with fundamental structural changes and moving towards
refactoring the logic that consumed the registry. The primary benefit will be a more idiomatic Rust design
with clearer ownership and better leverage of existing functional programming abstractions