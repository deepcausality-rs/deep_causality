# DeepCausality, distilled: the core monad and the causaloid crate

Source: every file in `deep_causality/` (0.18.2) and `deep_causality_core/` (0.13.2), read in full on
2026-09-30: `src/`, `tests/`, `benches/`, `Cargo.toml`, `BUILD.bazel`, the READMEs and the two
`LEAN_*.md` notes.

## The finding

DeepCausality models a cause as a function that returns an effect inside one monad. A **causaloid**
wraps `fn(I) -> PropagatingEffect<O>`; the effect carries a value, a threaded state, a read-only
context, an append-only log, and an error channel that is exclusive with the value. Three shapes
exist and no fourth can be added: a single function, a bag of causaloids folded by a lattice
operation, and a graph of causaloids folded in topological order. Everything else in the two
crates (the fluent `CausalFlow` DSL, the reusable `CausalArrow`, the causal state machine, the
generative interpreter, the Lean witnesses) is built on that monad.

One structural gap matters more than the rest. The causaloid *type* is recursive, but its
*evaluation* is not: `Causaloid::evaluate` on a `Collection` or `Graph` causaloid returns an error
("Collection evaluation is not available in this build"). A composite is evaluated by calling
`evaluate_collection` or `evaluate_subgraph_from_cause` on the inner `Vec` or graph directly. The
formal claim `Causaloid ≅ μX.F(X)` with `evaluate` as its catamorphism is witnessed that way too
(§5).

## 1. The idea

The lib.rs statement is "Causality is a spacetime-agnostic functional dependency." In code:

- A cause is a pure function `I -> PropagatingEffect<O>` (`CausalFn`), or a function that also
  receives state and context: `fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>`
  (`ContextualCausalFn`, alias `StatefulContextualCausalFn`). `deep_causality/src/alias/alias_function.rs`.
- Time and space never appear in the signature. They enter only through the context value `C`,
  which the context crate supplies (see `dc-context.md`).
- Composition is monadic bind. Errors short-circuit, logs accumulate, state threads forward.

## 2. `deep_causality_core`: the causal monad

`no_std` + `alloc` capable, one dependency (`deep_causality_haft`). It holds the carrier, its laws,
two facades over it, and the `Identifiable` trait shared by the context and causal crates.

### 2.1 The carrier

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    outcome: Result<CausalEffect<Value>, Error>, // value-or-command XOR error
    state:   State,                              // Markovian state, threaded by bind
    context: Option<Context>,                    // read context, threaded by bind
    logs:    Log,                                // append-only audit trail
}
```

Two aliases fix `Error = CausalityError`, `Log = EffectLog`:

| Alias | State | Context | Use |
|---|---|---|---|
| `PropagatingEffect<T>` | `()` | `()` | stateless causal chains; what a singleton causaloid returns |
| `PropagatingProcess<T, S, C>` | `S` | `C` | Markovian chains that read and write state |

`CausalEffect<V>` is the success channel, a newtype over the free monad
`Free<CausalCommandWitness, Option<V>>`:

| Shape | Meaning |
|---|---|
| `Pure(Some(v))` | a value |
| `Pure(None)` | absence of evidence (not an error) |
| `Suspend(RelayTo(target, sub))` | a command: jump to the causaloid at index `target`, feed it `sub` |

So the outcome type is the transformer stack `Except E (Free CausalCommand (Maybe V))`. Value and
error share one `Result`, so "value and error at once" cannot be constructed. The code calls this
the W-invariant; it is what lets right identity `bind(m, pure) = m` hold on errored carriers.
`CausalCommand` has exactly one operation, `RelayTo(usize, K)`. `CausalEffect::fold` is the
interpreter for that command layer; `and_then` / `try_and_then` are its binds, with `None` a local
zero and `Err` a global zero.

### 2.2 Bind

There is one bind, and it threads state (`src/types/causal_effect_propagation_process/mod.rs`):

```rust
pub fn bind<F, NewValue>(self, f: F) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>
where F: FnOnce(CausalEffect<Value>, State, Option<Context>) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>
```

On `Err` the continuation does not run and the carrier comes back verbatim. On `Ok` the
continuation receives the whole effect (value, `None`, or command) plus state and context; its
returned state and context win; logs concatenate old-then-new. `bind_or_error` unwraps the value and
turns `None` into `CausalityErrorEnum::Custom(msg)`.

The `CausalMonad` trait restates `pure`/`bind` so generic code can bind against the contract. The
HKT witnesses (`PropagatingEffectWitness`, `PropagatingProcessWitness`,
`CausalEffectPropagationProcessWitness`) implement haft's `Functor`/`Applicative`; only the
stateless witness implements haft's value-only `Monad`, because that `bind` cannot thread state.
All `fmap`s route through the total `CausalEffect::map`, which preserves commands.

### 2.3 Counterfactual substitution

`AlternatableValue`, `AlternatableState`, `AlternatableContext` (and the blanket `Alternatable`)
replace one channel of an in-flight carrier, append a `!!ValueAlternation!!`-style log entry, and do
nothing on an errored carrier. `clear_context` sets the context to `None`. The crate docs state
plainly that this is value substitution; Pearl's `do()` (graph surgery) is assigned to the
causaloid/hypergraph layer. No `do()` exists in either crate.

### 2.4 Facades

`CausalFlow<V, S = (), C = ()>` wraps a `PropagatingProcess` and hides the witnesses:

```rust
let total = CausalFlow::value(0_i64)
    .iterate_n(5, |tick| tick.branch(|n| n % 2 == 0, |even| even.map(|n| n + 10), |odd| odd.map(|n| n + 1)))
    .finish();
assert_eq!(total, Ok(50));
```

Its surface: constructors (`value`, `effect`, `fail`, `process`, `context`, `from_parts`,
`source`), steps (`and_then`, `next`, `try_step`, `map`, `guard`, `recover`, `try_step_with`,
`step_mut`, `update_value|state|context`, `update_value_state_context`), control
(`branch`, `branch_with`, `either`, `iterate_n`, `iterate_until`, `iterate_to_fixpoint` with
`MaxStepsExceeded` on budget exhaustion), IO (`commit`, `read_text_from`, `read_csv_from`,
`write_text_to`, `write_csv_to`), and terminals (`finish`, `run`, `into_process`).

`causal_arrow(f).next(g).build()` reifies a pipeline as a reusable Kleisli arrow over the flow
(`CausalLift`, `KleisliCompose`, bound `CausalArrow<A, B, S, C>`). It implements haft's `Arrow`, so
`run` takes `&self` and applies to many inputs.

### 2.5 Log, errors, IO

- `EffectLog` is a `Vec<LogEntry>` with a microsecond timestamp per entry (0 under `no_std`).
  Equality compares messages only; `eq_with_timestamps` compares both. It is the free monoid the
  Writer laws are proved on.
- `CausalityError(CausalityErrorEnum)` with variants `Unspecified`, `InternalLogicError`,
  `TypeConversionError`, `ValueNotAvailable`, `MaxStepsExceeded`, `IoError`, `Custom`,
  `ActionError`, `DeonticError`, `ModelError`.
- `ReadText`, `ReadCsv`, `WriteText`, `WriteCsv` are deferred `IoAction`s (std only). The CSV codec
  does no quoting; round-trip holds only when no field contains `,` or `\n`.

## 3. `deep_causality`: causaloids and reasoning

### 3.1 The causaloid

`Causaloid<I, O, STATE, CTX>` (`src/types/causal_types/causaloid/mod.rs`) holds an id, a
`CausaloidType`, and optional slots: `causal_fn`, `context_causal_fn` + `context`, `causal_coll`
(`Arc<Vec<Self>>`) with aggregate logic and threshold, `causal_graph` (`Arc<CausaloidGraph<Self>>`)
with optional `LambdaEdges`. Equality is id equality. Constructors: `new`, `new_with_context`,
`from_causal_collection[_with_context]`, `from_causal_graph[_with_context|_with_lambda_edges]`.

`Causable`, `MonadicCausable`, and `StatefulMonadicCausable` are sealed: `Causaloid` is the only
implementor, and the three `CausaloidType` variants are the whole causal form set.

Evaluating a singleton is a three-step bind chain: log the input, run the function, log the output.
Every causaloid therefore writes `Causaloid {id}: Incoming effect: …` and `… Outgoing effect: …`
into the log, which is what `explain()` prints. A `RelayTo` on the input is rejected with a
command-specific error; a `RelayTo` on the output passes through for the engine; a `None` output is
an error.

Statefulness is chosen by the call, not the constructor. `evaluate` on a contextual singleton calls
the function with `PS::default()` and the causaloid's stored context, then drops state.
`evaluate_stateful` passes the incoming state and context (incoming context first, stored context as
fallback) and returns the function's state.

### 3.2 Collections: a bag folded by a verdict algebra

`MonadicCausableCollection` is implemented for slices, `Vec`, `VecDeque`, `HashMap`, `BTreeMap`.
`evaluate_collection(effect, logic, threshold)` feeds the **same** input to every member, collects
the outputs, and aggregates them with `AggregateLogic::{All, Any, None, Some(k)}`. The first member
error short-circuits. The carrier must be `Aggregatable: Verdict` (a bounded lattice with
complement from `deep_causality_algebra`); shipped carriers are `bool`, `f64`, `UncertainBool`,
and `UncertainF64` (which errors: "convert to UncertainBool first").

| Carrier | All | Any | None | Some(k) |
|---|---|---|---|---|
| `bool` | AND | OR | NOT OR | count(true) ≥ k |
| `f64` | product | 1 − Π(1 − p) | Π(1 − p) | 1.0 if count(p > 0.5) ≥ k, else 0.0 |
| `UncertainBool` | `&` | `\|` | `!(\|)` | point(count ≥ k), sampled at `threshold` |

The stateful variant threads state member to member, so member order affects state but not the
aggregated value.

### 3.3 Graphs: a Kahn fold with a join at reconvergence

`CausaloidGraph<T>` wraps `ultragraph::UltraGraphWeighted<T, u64>` (default capacity 500). It must
be frozen before reasoning. `freeze_dag` rejects cycles and rolls back; `freeze_verified(writers)`
also enforces the single-writer rule (at most one branch into a join may write state);
`freeze_verified_with_check` adds a caller hook.

`evaluate_subgraph_from_cause[_with_lambda_edges]`
(`src/traits/causable_graph/graph_reasoning/mod.rs`) works in rounds:

1. Reject a cyclic graph. Mark the start node and its descendants reachable.
2. Pop the lowest-index ready node, build its input from its fired parents, evaluate it.
   - One parent: pass the effect through, applying that edge's Λ if decorated.
   - Two or more: apply each edge's Λ, then fold with `∇ = Verdict::join` (`bool`: OR, `f64`: max).
     Parent logs concatenate in ascending parent index. `None` parents drop out.
3. An error returns immediately. A `RelayTo(t, sub)` ends the round and starts a new one at `t`
   seeded with `sub`, carrying the log. `MAX_RELAY_ROUNDS = 1024` cuts relay cycles with an error.
4. Return the effect of the **last node processed**, i.e. the highest-index node reached.

```rust
// diamond root(0) -> A(+1), B(+10) -> C(identity); ∇ = max
let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
assert_eq!(res.value(), Some(&10.0));
```

`evaluate_single_cause` evaluates one node; `evaluate_shortest_path_between_causes` chains values
along the ultragraph shortest path and returns on the first relay without following it. The stateful
twins thread state and context node to node and **error at any multi-parent join** ("the
reconvergence merge (∇) is not yet defined"), consistent with the single-writer rule having no merge
semantics for state.

### 3.4 Causal state machine

`CSM<I, O, C>` maps state ids to `(CausalState, CausalAction)` behind an `Arc<RwLock<…>>`. A
`CausalState` holds a causaloid, stored input data, and an optional `UncertainParameter`
(threshold, confidence, epsilon, max samples). `eval_single_state` / `eval_all_states` evaluate the
causaloid; if the output's `CsmEvaluable::is_active` is true the `CausalAction` (`fn() ->
Result<(), ActionError>`) fires. `bool` is active when true; `UncertainBool` runs a sequential
probability test; `UncertainF64` requires the parameter and tests `P(x > threshold)`. Relay and
errored outputs are inactive. `ProposedAction` and `ActionParameterValue` are the vocabulary the
ethos crate consumes; ethos checks were moved out of the CSM (`fire_action_with_ethos_check` now
only fires).

### 3.5 Models, assumptions, inference

- `Model<I, O, C>`: id, author, description, optional `Arc<Vec<Assumption>>`, a root causaloid, an
  optional `Arc<RwLock<C>>` context. `Transferable::verify_assumptions` runs every assumption
  (`EvalFn = fn(&[PropagatingEffect<f64>]) -> Result<bool, AssumptionError>`) and fails on the first
  false. `Assumption` records tested/valid flags in `Arc<RwLock<bool>>`.
- `Inference` and `Observation` with the `Inferable`/`Observable` traits and their collection
  "reasoning" extensions compute counts, percentages, and a conjoint delta over threshold tests.
  They predate the monad and do not use it.

### 3.6 Generative interpreter

`Operation<I, O, C, N>` is an AST of model edits (create/update/delete causaloid, context,
contextoid; `Sequence`; `NoOp`) stored in `OpTree = ConstTree<Operation>`. `Interpreter::execute`
walks it inside its own arity-3 effect (`GraphGeneratableEffect { value, error, logs:
ModificationLog }`). `Model::evolve(&op_tree)` loads the model's causaloid and context into a
`CausalSystemState`, runs the tree, and rebuilds a model from the same causaloid id and context id.

## 4. How the two crates fit

```
deep_causality_core  PropagatingEffect / PropagatingProcess, CausalEffect (value | None | RelayTo),
                     bind, CausalFlow, CausalArrow, EffectLog, Identifiable
        ▲
deep_causality       Causaloid = Kleisli arrow I → PropagatingEffect<O>
                     Collection = verdict fold;  Graph = Kahn fold + ∇ join + RelayTo handler
                     CSM, Model/Assumption, Interpreter
        │ uses
        ├── deep_causality_context (Context, the C in Causaloid<…, Arc<RwLock<Context>>>)
        ├── deep_causality_algebra (Verdict), deep_causality_uncertain (UncertainBool/F64)
        ├── ultragraph (graph storage, has_cycle, shortest_path), deep_causality_ast (ConstTree)
```

The graph engine is the handler for the core's command layer: `RelayTo` is emitted by a causaloid
and interpreted by the `'rounds` loop. The loop re-implements the fold; it does not call
`CausalEffect::fold`.

## 5. The formal layer

Both crates carry Lean 4 proofs under `lean/DeepCausalityFormal/Core/` (no Mathlib, zero `sorry`)
and one Rust witness test per theorem id (`tests/formalization_lean/`, ids in `lean/THEOREM_MAP.md`).
Core: 26 ids (monad laws, transformer stack, fold universality, relay termination, lens laws,
free-monoid log, functor agreement, CSV round-trip). Main crate: 13 ids (fixpoint, Hardy
inversion, verdict closure/carriers/permutation invariance, graph-fold order invariance,
catamorphism uniqueness, encapsulation flat, arrow fragment, command input, relay round
composition, context threading, acyclicity separable). Kani harnesses in
`deep_causality_core/tests/kani_proofs.rs` are gated `#![cfg(kani)]` and not run in CI.

The claim the LEAN notes make is "laws machine-checked in Lean; implementation pinned to the same
statements by tests at representative inputs". They state the scopes: value channel only,
stateless all-success path, per fixed carrier.

## 6. Where the code and the claims part

1. **Composite causaloids do not evaluate.** `Causaloid::evaluate` and `evaluate_stateful` return
   an error for `Collection` and `Graph`. The stored `coll_aggregate_logic` and
   `coll_threshold_value` are read by nothing. `test_encapsulation_flat` compares two direct graph
   evaluations; `test_fixpoint_…` checks getters. Nesting a graph causaloid inside a graph fails at
   runtime.
2. **`f64` aggregation is not the verdict algebra it is bounded by.** `Verdict` for `f64` is
   min/max/1 − p (pinned by `test_verdict_carriers`), and the graph join uses `max`. The collection
   `Aggregatable for f64` uses product and noisy-OR, and `Some(k)` hard-codes `p > 0.5`. The
   closure and permutation witnesses run on `bool`, where product and min agree.
3. **The `threshold` argument is ignored** for `bool` and `f64` collections; only `UncertainBool`
   `Some(k)` reads it.
4. **Graph results depend on node indices.** The returned effect is the highest-index node reached;
   in a branching tree without a join, the sibling with the larger index wins
   (`test_evaluate_subgraph_branching_tree_returns_highest_index_leaf`). Order invariance is proved
   for join values, not for this choice of output.
5. **Stateful graphs cannot join.** The stateless engine merges at reconvergence; the stateful one
   errors.
6. **The generative effect allows value and error together** (`GraphGeneratableEffect` has two
   `Option` fields, and `bind` propagates both). The core monad forbids that state by construction.
7. **`Model::evolve` drops everything except the original causaloid id and context id.**
   `CreateExtraContext` creates a second top-level context named `ExtraContext_{id}` with no link to
   its "parent"; it does not survive `evolve`.
8. **Panics remain on public paths:** `CausalState::context()` panics without a context; CSM and
   `Assumption` call `RwLock::unwrap`.
9. **Stale documents:** `deep_causality_core/Notes.md` describes `CausalEffectSystem`,
   `CausalSystem`, a `CausalMonad` struct and `EffectValue`, none of which exist. Several alias
   docstrings in `alias_base.rs` and `alias_uniform.rs` list generic parameters (`EuclideanSpace`,
   two `FloatType`s) that the aliases no longer have.

## 7. File map

| Concern | Path |
|---|---|
| Carrier, bind, constructors | `deep_causality_core/src/types/causal_effect_propagation_process/` |
| Value/None/Command | `deep_causality_core/src/types/causal_effect/mod.rs`, `causal_command/` |
| Fluent DSL | `deep_causality_core/src/types/causal_flow/` |
| Reusable arrow | `deep_causality_core/src/types/causal_arrow/` |
| Causaloid + evaluation | `deep_causality/src/types/causal_types/causaloid/` |
| Collection fold | `deep_causality/src/traits/causable_collection/`, `src/utils/monadic_collection_utils.rs` |
| Graph engine | `deep_causality/src/traits/causable_graph/` |
| Λ edges | `deep_causality/src/types/causal_types/causaloid_graph/lambda_edges.rs` |
| CSM | `deep_causality/src/types/csm_types/` |
| Interpreter | `deep_causality/src/types/generative_types/` |
| Lean witnesses | `deep_causality*/tests/formalization_lean/` |
