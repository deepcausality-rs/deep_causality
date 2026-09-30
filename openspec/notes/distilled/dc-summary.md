# DeepCausality: an overview for Rust programmers

DeepCausality is a Rust workspace of 33 library crates for computational causality. It composes a
causal model on two separate layers: an outer layer that fixes the causal order, and an inner layer,
the body of each step, that is free to contain any computation, including time-symmetric physics.
Both live in one monad, and larger models fold that monad over a collection or a graph. Around that
core sit a context (the world a cause is evaluated in), a norm checker for proposed actions, causal
discovery algorithms, and a stack of seventeen mathematics crates that let the whole program choose
its floating-point precision in one line.

This overview is assembled from five notes in this folder, each written from a full read of the
crates it covers, plus `AGENTS.md` and `deep_causality_unified_math/README.md`:

| Note | Covers |
|---|---|
| `dc.md` | `deep_causality_core` (the monad) and `deep_causality` (causaloids and reasoning) |
| `dc-context.md` | `deep_causality_context` and `deep_causality_context_store` |
| `dc-ethos.md` | `deep_causality_ethos` |
| `dc-discovery.md` | `deep_causality_algorithms` and `deep_causality_discovery` |

The physics, quantum and CFD crates are described here at the level of `AGENTS.md`; no note in
this folder reads them. The one exception is the physics code behind the example in section 1.2,
which was read and run for this overview.

## 1. The idea: two layers of composition

The premise, as the causal monad in `deep_causality_core` implements it:

> Effects propagate by monadic dependency. Each step is computed from exactly three channels of the
> previous step, effect, state and context, and from nothing else, and no channel assumes a
> spacetime.

The classical formulation of causality, "if A then B, and if not A then not B", composes on one
layer: the causal direction from A to B is also the temporal direction from earlier to later. A
model written that way cannot contain a time-symmetric relation, because the relation itself is the
arrow of time. The causal monad splits that one layer into two.

### 1.1 The outer layer: causal order

Steps compose by Kleisli composition, which is monadic `bind`
(`deep_causality_core/src/types/causal_effect_propagation_process/mod.rs`):

```rust
pub fn bind<F, NewValue>(self, f: F) -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>
where
    F: FnOnce(CausalEffect<Value>, State, Option<Context>)
        -> CausalEffectPropagationProcess<NewValue, State, Context, Error, Log>,
{
    match self.outcome {
        Err(error) => /* the carrier comes back unchanged; f never runs */,
        Ok(effect) => {
            let mut next_process = f(effect, self.state, self.context);
            // logs: old entries first, then the new step's
            …
        }
    }
}
```

This layer is strictly linear and one-directional. Step two runs after step one and receives its
effect, state and context; an error in step one means step two never runs. A causal graph arranges
many such chains into a directed acyclic graph, and freezing the graph refuses a cycle, so the
causal order stays one-directional at every scale.

### 1.2 The inner layer: the step itself

The body of a step, `f`, is any `FnOnce` of the right type. Nothing constrains what it computes, so
it can integrate an equation forward or backward in time, solve a boundary-value problem, or run a
time-symmetric law. The causal order comes from the outer layer, not from the equation inside.

Here the second step runs later in causal order and earlier in physical time. Both steps use the
exact two-body propagator from `deep_causality_physics`, which accepts a negative time step:

```rust
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_physics::{EARTH_GM, TwoBodyPropagator};

type FloatType = f64;
type Orbit = ([FloatType; 2], [FloatType; 2]); // (position, velocity)

/// Inner layer: an exact, time-reversible Kepler step. `dt` may be negative.
fn kepler_step(state: Orbit, dt: FloatType) -> PropagatingEffect<Orbit> {
    let (p, v) = state;
    match TwoBodyPropagator::from_state(p, v, EARTH_GM).and_then(|orbit| orbit.propagate(dt)) {
        Ok(next) => PropagatingEffect::pure(next),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

let epoch: Orbit = ([7.0e6, 0.0], [1.0e3, 7.5e3]);
// Outer layer: step 2 depends on step 1, although its physical time runs backwards.
let result = PropagatingEffect::pure(epoch)
    .bind_or_error(|s, _, _| kepler_step(s, 600.0), "no state")
    .bind_or_error(|s, _, _| kepler_step(s, -600.0), "no state");
```

Run against the workspace, the chain returns to the epoch position within `8.15e-10` m.

### 1.3 Three channels, and a log

Each step sees exactly three channels of the step before it:

| Channel | Type | What it gives a model |
|---|---|---|
| Effect | `CausalEffect<V>`: a value, `None` (no evidence), or a `RelayTo` command | the result of the previous step |
| State | any type `S`; `()` when unused | memory carried from step to step |
| Context | any type `C`, passed as `Option<C>` | the environment: data, time, space, with static or dynamic relations between its parts |

The channels make the model's memory explicit. Without state, a step depends only on the effect it
receives. With state, a step can carry whatever history the state type holds, so processes whose
next step depends on more than the last effect can be written down. The context can be as simple as
`()` or as rich as the context hypergraph of section 4, and its relations can be fixed or change
between evaluations.

The fourth field, the log, is not a channel. `bind` appends the new step's entries after the old ones
and never passes the log to `f`, so the log cannot influence any step. `EffectLog` is append-only: it
offers `add_entry` and `append` and no way to remove or edit an entry. Under `std` every entry is
stamped with `SystemTime::now()` (zero under `no_std`). The log is therefore sequential and tied to
wall-clock time, which is what provenance needs: it records what was computed and when, for
debugging and as evidence where a regulator asks for it. Equality on the log compares messages and
ignores timestamps, so two runs of the same computation compare equal.

### 1.4 What the monad leaves to the model

Two properties are the model author's responsibility:

- **Purity.** `f` is an `FnOnce`, which may capture variables, read a clock, do IO or mutate shared
  state. The monad guarantees what `f` receives; it cannot guarantee that `f` depends on nothing
  else. A causaloid narrows this: it stores an `fn` pointer, which cannot capture its environment.
- **What counts as causal.** `bind` accepts any function of the right type. Which dependencies a
  model treats as causal is the author's choice; the monad fixes their order and records their
  provenance.

A Rust programmer can read the monad as "a `Result`-like carrier with an `and_then` that also
threads state and a context and appends a log". Everything else in the project is built on that
carrier or sits beside it.

## 2. The map

The crates form a strict dependency order: a crate depends only on crates in lower tiers. Grouped by
role rather than by tier:

```text
Causal reasoning      deep_causality_core      the causal monad, CausalFlow DSL, CausalArrow
                      deep_causality           Causaloid, collections, graphs, state machine, models
Environment           deep_causality_context   Context hypergraph: data, time, space, spacetime nodes
                      deep_causality_context_store   persistence contract, zero dependencies
Norms                 deep_causality_ethos     norms (Teloids) and verdicts on proposed actions
Discovery             deep_causality_algorithms     SURD, MRMR, BRCD, BOSS, MEC sampling
                      deep_causality_discovery      the Causal Discovery Language (CDL)
Domains               deep_causality_physics, _quantum, _cfd
Mathematics           17 crates under deep_causality_unified_math/ (num … topology)
Utilities             ultragraph, deep_causality_data_structures, and 4 crates under
                      deep_causality_utils/ (ast, file, par, tempfile)
```

The tiers that matter for orientation, simplified from `AGENTS.md`:

```text
tier 0   num, metric, ultragraph, context_store, ast, par, data_structures, tempfile
tier 1   algebra
tier 2   haft (higher-kinded traits), rand, num_rational
tier 3   core (the causal monad), linear, num_complex, num_dual, file
tier 4-5 stats, calculus, fft, homology, tensor, uncertain
tier 6   context, multivector
tier 7   deep_causality, topology
tier 8   ethos, algorithms, physics, quantum
tier 9   discovery, cfd
```

Three things follow from that order. The causal monad (`core`, tier 3) depends only on the
functional-traits crate, so it builds without the mathematics above it. The context (tier 6) is a
separate crate that `deep_causality` depends on unconditionally; a program that uses only
`deep_causality_core` links no context types. The ethos and discovery crates sit above the engine;
the engine cannot call them.

External dependencies are few. Twenty-six of the 33 crates have none at runtime. `discovery` needs
`csv` and `parquet`, `file` needs `chrono`, and `libm`, `getrandom` and `rayon` appear only behind
optional features.

## 3. The causal core

### 3.1 The carrier

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    outcome: Result<CausalEffect<Value>, Error>, // a value (or a command) XOR an error
    state:   State,                              // threaded by bind
    context: Option<Context>,                    // threaded by bind
    logs:    Log,                                // append-only provenance, never read by a step
}
```

Section 1 shows `bind` and what each field is for. Two aliases cover almost every use: `PropagatingEffect<T>` has no state and no context, and
`PropagatingProcess<T, S, C>` has both. The success channel `CausalEffect<V>` holds either a value,
`None` (absence of evidence, which is not an error), or one command, `RelayTo(target, input)`, which
asks the graph engine to jump to another node. Value and error share one `Result`, so a carrier can
never hold both.

`deep_causality_core` also ships a fluent DSL over the carrier:

```rust
let total = CausalFlow::value(0_i64)
    .iterate_n(5, |tick| tick.branch(|n| n % 2 == 0, |even| even.map(|n| n + 10), |odd| odd.map(|n| n + 1)))
    .finish();
assert_eq!(total, Ok(50));
```

and `causal_arrow(f).next(g).build()`, which turns a pipeline into a reusable value that can be run
on many inputs.

### 3.2 The causaloid

`Causaloid<I, O, STATE, CTX>` is the unit of causality. It comes in exactly three shapes, and the
traits that evaluate it are sealed, so no fourth can be added:

| Shape | What evaluation does |
|---|---|
| Singleton | runs the wrapped function, logging input and output |
| Collection | feeds the same input to every member, then aggregates with `All`, `Any`, `None` or `Some(k)` |
| Graph | walks a DAG of causaloids in topological order, joining effects where branches reconverge |

A singleton with a context, taken from the ethos example:

```rust
fn context_causal_fn(
    effect: CausalEffect<f64>,
    _state: (),
    _context: Option<Arc<RwLock<BaseContext>>>,
) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
    let obs = effect.into_value().unwrap_or(0.0);
    PropagatingProcess::pure(obs >= 0.55)
}

let causaloid = Causaloid::new_with_context(1, context_causal_fn, context, "reading exceeds 0.55");
```

The aggregation algebra is a bounded lattice (`Verdict`) from `deep_causality_algebra`. For `bool`,
`All` is AND and `Any` is OR; for probabilities it is product and noisy-OR; for `UncertainBool` it is
the uncertain logical operators.

A graph is an `ultragraph` hypergraph that must be frozen and acyclic before evaluation. The engine
repeatedly takes the lowest-index ready node, builds its input from its parents (a single parent
passes through; several are joined, `bool` by OR, `f64` by max), and evaluates it. An error ends the
walk. A `RelayTo` command starts a new round at the named node, with a cap of 1024 rounds. The
result is the effect of the last node processed:

```rust
// diamond root(0) -> A(+1), B(+10) -> C(identity); the join takes max
let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
assert_eq!(res.value(), Some(&10.0));
```

### 3.3 Around the causaloid

- **Causal state machine (`CSM`).** Maps states to actions. Each state holds a causaloid; when its
  output is "active" (true, or an uncertain value that passes a sequential probability test), the
  paired action fires.
- **Models and assumptions.** A `Model` bundles a root causaloid, an optional context and a list of
  assumptions that can be checked against data.
- **Generative interpreter.** An AST of model edits (create, update or delete causaloids, contexts and
  contextoids) that `Model::evolve` executes.
- **Counterfactuals.** The carrier can have its value, state or context replaced mid-flight, with a
  log entry recording the substitution. The crate documents this as value substitution; Pearl's
  `do()` operator is not implemented.

### 3.4 Formal backing

`deep_causality_core` and `deep_causality` carry Lean 4 proofs under `lean/DeepCausalityFormal/`
(no Mathlib, no `sorry`): the monad laws, the transformer stack, relay termination, verdict closure,
order invariance of the graph join, and more. Each theorem has one Rust test that pins the
implementation to the same statement at representative inputs; `lean/THEOREM_MAP.md` links them.

## 4. The context

A context is a directed graph of typed, identified nodes: data, time, space, spacetime and a root.
It is generic over four parameters, one per node kind, and every coordinate type is generic in its
scalar. `BaseContext` fixes all four to Euclidean types at `f64`.

```text
Causaloid<I, O, PS, C>                     C = Arc<RwLock<Context<D, S, T, ST>>>
  └─ context_causal_fn(effect, state, Option<C>) -> PropagatingProcess<O, PS, C>
```

The engine hands the context to the user's function and reads nothing from it; what the function
reads there is the only way time, space or outside data reach a cause. Swapping one context for
another and evaluating the same causaloid is the project's model of a counterfactual world.

The node vocabulary is broad: Euclidean, ECEF, geodetic and NED space; Euclidean, Lorentzian,
discrete and entropic time; Euclidean, Lorentzian and tangent spacetime, each reporting its own
metric, so one context can hold a Newtonian and a relativistic node side by side. Uncertain data
nodes carry `deep_causality_uncertain` values.

The newer half of the crate makes a context persistent. `deep_causality_context_store` has no
dependencies and states the contract: record types, a fourteen-operation async `ContextStorage`
trait, an optional event stream, and an optional `Substrate` that stores large values out of the
graph. On the context side, `snapshot`, `restore` and `apply(event)` project a context onto records
and back, and `ContextStore::store_branch` stores a modified copy as a new container that shares the
nodes it did not change. All of it is tested end to end against an in-memory backend.

## 5. The effect ethos

`deep_causality_ethos` checks a `ProposedAction` against a set of norms and returns a verdict:
`Impermissible`, `Obligatory` or `Optional(cost)`, with the IDs of the norms that justify it.

A norm (`Teloid`) is a predicate over `(Context, ProposedAction)`, deterministic or uncertain, plus a
modality and three conflict-resolution numbers: specificity, timestamp and priority. Norms are
indexed by tag and linked in a graph by `Inherits` and `Defeats` edges. Evaluation selects norms by
tag, runs their predicates, removes defeated norms, and ranks the survivors: any prohibition wins,
then any obligation, else the costs of the optional norms are summed.

```rust
let mut ethos = EffectEthos::new()
    .add_deterministic_norm(1, "takeoff", &["flight_safety"], battery_low,
                            TeloidModal::Impermissible, 0, 10, 100)?;
ethos.verify_graph()?;
let verdict = ethos.evaluate_action(&action, &context, &["flight_safety"])?;
```

The ethos reads the same `Context` type a causaloid reads, so a norm and a cause can depend on the
same data, space and time. The README cites the Defeasible Deontic Inheritance Calculus (DDIC) as
its formal basis; the crate uses DDIC's vocabulary and its three resolution heuristics.

## 6. Causal discovery

Discovery answers two different questions with two algorithms in `deep_causality_algorithms`.

- **SURD** (Martínez-Sánchez and Lozano-Durán) takes a joint probability table of a target and its
  sources and splits the information the sources carry about the target into redundant, unique and
  synergistic parts, plus a leak for what no source explains.
- **BRCD** (Lee, Zhou and Kocaoglu, ICML 2026) takes data from a normal and an anomalous regime and
  a causal graph, supplied or learned by the BOSS structure learner, and ranks every candidate root
  cause by posterior probability. It samples from Markov equivalence classes with a polynomial-time
  Clique-Picking port.

**MRMR** feature selection prepares data for SURD. The BRCD port reproduces the reference Python
ranking position for position on four real microservice incidents.

`deep_causality_discovery` wraps both in the Causal Discovery Language, a typestate pipeline inside
an error-and-warning monad. Each stage exists only on the state before it, so the compiler rejects a
pipeline run out of order:

```rust
CdlBuilder::build_surd(&config)
    .surd_load_input()
    .clean_data(OptionNoneDataCleaner)
    .feature_select()
    .surd_discover()
    .surd_analyze()
    .finalize()
    .print_results();
```

The BRCD pipeline has the same shape and adds a CPDAG cache keyed on a hash of the data, so a second
run reuses the learned structure.

## 7. The mathematics

Seventeen crates under `deep_causality_unified_math/` supply numbers, algebra, linear algebra,
statistics, tensors, geometric algebra, topology and uncertain values. Two design decisions hold
them together.

**Higher-kinded traits.** `deep_causality_haft` emulates higher-kinded types in stable Rust. A crate
that owns a container declares a zero-sized *witness* type, binds `type Type<T>` to the container,
and implements `Functor`, `Applicative`, `Monad`, `CoMonad`, `Foldable` and others against the
witness. Forty witnesses across seven crates exist. Because a witness accepts any element type,
containers from different crates nest, and one `fmap` can rotate every cell of a tensor of
multivectors:

```rust
let rotated: CausalTensor<CausalMultiVector<FloatType>> =
    CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v).geometric_product(&rotor_rev));
```

The comonadic `extend` is the idiom for graphs, meshes and manifolds: it hands a cursor to a closure
that reads a neighbourhood and returns one value, which is how a stencil, a diffusion step or a graph
convolution is written. The causal monad and the CDL effect are built on the same traits.

**Precision as a parameter.** Every crate above `deep_causality_num` is generic in its scalar. A
program names its working type once:

```rust
type FloatType = f64; // or f32, BFloat16, Float106
```

and writes everything else against the alias. `Float106` is a software double-double with about 31
decimal digits; `BFloat16` matches the 16-bit format accelerators use. Literals, counts and display
values cross the boundary through `lift`, `lift_count` and `lower` in `deep_causality_num`, because
`as` and `From` fail on one or another of the four types. The unified-math README shows one program
carrying a field through four crates and landing at errors of `2e-7`, `7e-17` and `2e-32` at `f32`,
`f64` and `Float106`, with no conversion between crates.

## 8. The domain crates

From `AGENTS.md`, without a note in this folder behind them:

- `deep_causality_physics`: a standard library of physics formulas and engineering primitives.
- `deep_causality_quantum`: density matrices, channels and gates over the complex number tower.
- `deep_causality_cfd`: computational fluid dynamics solvers and a Flow DSL.

All three depend on `deep_causality_core`, `deep_causality_haft` and the mathematics crates. The
physics crate lifts kernels into the causal monad through wrappers that return a
`PropagatingEffect`: seventeen files across sixteen kernel domains, from astrodynamics to waves (for
example `deep_causality_physics/src/kernels/propulsion/wrappers.rs`). A wrapped kernel becomes the
inner layer of a causal step, as in section 1.2.

## 9. Engineering conventions

These hold across the workspace and shape how the code reads:

- **No `unsafe`.** `unsafe_code = "forbid"` is a workspace lint every member opts into.
- **Static dispatch.** Generics and sealed traits over trait objects; the causal and ethos APIs are
  generic over the four context parameters rather than boxed.
- **Private fields** on public types, with constructors and getters.
- **One type per module**, with each trait impl in its own file; tests mirror `src/` one to one, and
  shared test utilities live in `src/utils_test/` so Bazel can see them.
- **Two build systems.** Cargo for local work, Bazel for the full workspace; every example needs a
  target in both.
- **Tests beyond coverage.** Coverage is treated as a floor. `cargo mutants` runs on numeric kernels,
  and equivalent mutants are recorded with the measurement that justifies them.
- **`no_std`.** The core monad and several lower crates build with `alloc` only.

## 10. How the pieces connect today

The notes each end with a list of gaps. Read together, they show a pattern: the parts are carefully
built and individually tested, and several of the links between them exist only in documentation or
user code.

| Link | State in the code |
|---|---|
| Causaloid → nested causaloid | The type is recursive, but `Causaloid::evaluate` on a collection or graph causaloid returns an error. A composite is evaluated by calling the collection or graph engine directly, so a graph cannot contain a graph that is evaluated. |
| Causal engine → context | Works: the context reaches every contextual function. |
| Context → store | Complete and tested, but nothing outside the two crates calls `snapshot`, `restore`, `apply` or `ContextStore`. |
| State machine → ethos | The CSM's `fire_action_with_ethos_check` fires the action unconditionally. The one example runs the two side by side and gates nothing. |
| Discovery → causal model | Neither discovery crate depends on `deep_causality`. The README describes how a SURD result should guide the design of a causal graph; a person does that translation. |

Beyond the links, the notes record defects that a reader should know before relying on a result:

- **The CDL's SURD pipeline passes the wrong input to SURD.** It hands over the raw data matrix,
  without the target column, where SURD expects a joint probability table, so the numbers it reports
  have no meaning. The algorithm itself, called directly with a probability table, is sound.
- **Ethos verdicts are not reproducible.** Conflict resolution depends on `HashSet` iteration order;
  a chain of two defeats returned two different verdicts over 2000 identical calls. Norms are also
  selected by tag without checking the action's name, and inherited norms join the result without
  their own predicate being evaluated.
- **Graph results depend on node numbering.** The graph engine returns the effect of the
  highest-index node reached, so in a branching graph without a join, the leaf with the larger index
  wins.
- **Stateful graphs cannot join.** The stateless engine merges effects where branches reconverge;
  the stateful engine returns an error there.

The strongest parts are the causal monad with its Lean proofs, the context and its persistence
contract, the BRCD port, and the mathematics stack with precision as a parameter.

## 11. Where to start reading

| To understand | Read |
|---|---|
| The carrier and bind | `deep_causality_core/src/types/causal_effect_propagation_process/` |
| The fluent DSL | `deep_causality_core/src/types/causal_flow/` |
| The causaloid | `deep_causality/src/types/causal_types/causaloid/` |
| Graph reasoning | `deep_causality/src/traits/causable_graph/graph_reasoning/mod.rs` |
| The context | `deep_causality_context/src/types/context_types/context_graph/` |
| Persistence | `deep_causality_context_store/README.md`, then `src/traits/` |
| Norms | `deep_causality_ethos/src/types/effect_ethos/deontic_inference.rs` |
| Discovery algorithms | `deep_causality_algorithms/src/causal_discovery/{surd,brcd}/` |
| The CDL pipeline | `deep_causality_discovery/src/types/cdl/` |
| Higher-kinded traits and precision | `deep_causality_unified_math/README.md` |
| Proofs | `lean/THEOREM_MAP.md` |
| Workspace rules and dependency tiers | `AGENTS.md` |

For the detail behind any section, and the full list of gaps with file references, read the
matching note in this folder.
