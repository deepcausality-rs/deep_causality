# DeepCausality Skills for Coding Agents

Use this file when helping a user build an application or library that consumes
DeepCausality crates. This is user-facing guidance, not contributor guidance for
editing this repository. For repository contribution rules, use `AGENTS.md`.

## Start Here

DeepCausality is a Rust framework for dynamic causality. It models causality as a
spacetime-agnostic monadic dependency: `m2 = m1 >>= f`. In application code,
choose the smallest crate that covers the user's task:

- Use `deep_causality` for Causaloids, Causaloid graphs, Context, CSM, and the
  integrated causal framework.
- Use `deep_causality_core` for monadic effect pipelines with
  `PropagatingEffect`, `PropagatingProcess`, `CausalMonad`, and `CausalFlow`.
- Use a domain crate directly when the user only needs that domain layer, such
  as `deep_causality_uncertain`, `deep_causality_tensor`,
  `deep_causality_multivector`, `deep_causality_topology`,
  `deep_causality_physics`, `deep_causality_discovery`, or `ultragraph`.

Minimal `Cargo.toml` for the full framework crate:

```toml
[dependencies]
deep_causality = "0.14"
```

For the monadic quickstart used by the repository examples:

```toml
[dependencies]
deep_causality_core = "0.11"
```

## First Example

Start with `PropagatingEffect` when the user wants a small deterministic causal
pipeline without graph or Context setup. This pattern is used in
[examples/starter_example/src/main.rs](examples/starter_example/src/main.rs)
and
[examples/core_examples/examples/propagating_effect.rs](examples/core_examples/examples/propagating_effect.rs).

```rust
use deep_causality_core::PropagatingEffect;

fn main() {
    let effect = PropagatingEffect::pure(10)
        .fmap(|x| x * 2)
        .bind(|x, _state, _context| {
            let value = x.into_value().unwrap_or_default();
            PropagatingEffect::pure(value + 5)
        });

    println!("{:?}", effect.value());
}
```

Use `alternate_value` from `deep_causality_core::AlternatableValue` when modeling
Pearl-style intervention or counterfactual examples; the starter example uses
this for forced mid-chain substitutions.

## Core Concepts

- Causaloid: a self-contained causal unit. It stores a causal function and can
  also wrap a collection or graph of other Causaloids. In user code, look at
  `Causaloid::new`, `Causaloid::new_with_context`,
  `Causaloid::from_causal_collection`, and graph constructors before inventing
  custom abstractions.
- CausaloidGraph: a graph of Causaloids backed by `ultragraph`. Use it when
  dependencies cross-influence each other or graph traversal matters. Examples
  build graphs with `CausaloidGraph::new`, add Causaloids, add edges, then
  freeze the graph before reasoning.
- Context: an explicit hypergraph for operational environment data: data-like,
  time-like, space-like, spacetime-like, and symbolic entities. Use Context
  when a causal function needs dynamic environment data instead of only an input
  value.
- Causal State Machine: the bridge from inference to action. `CSM` pairs
  `CausalState` values with `CausalAction` handlers, evaluates incoming
  `PropagatingEffect` evidence, and fires actions when state conditions are met.
- Effect Ethos: the optional deontic safety layer from `deep_causality_ethos`.
  It uses `Teloid` rules and `EffectEthos` reasoning to decide whether a
  proposed action is obligatory, impermissible, or optional.

## Common Patterns

- Simple causal chain: use `PropagatingEffect::pure`, `.fmap`, `.bind`, and
  `.value()` or `.into_value()`. See [examples/core_examples/](examples/core_examples/).
- Stateful chain: use `PropagatingProcess<T, State, Context>` when each stage
  must thread Markovian state or read configuration. See
  [examples/causal_correction_examples/](examples/causal_correction_examples/)
  and [examples/causal_uncertain_examples/](examples/causal_uncertain_examples/).
- Counterfactual or intervention: run the factual chain, then rerun with
  `alternate_value` at the intervention point. See
  [examples/starter_example/](examples/starter_example/) and
  [examples/causal_counterfactual_examples/](examples/causal_counterfactual_examples/).
- Corrective control loop: monitor a trajectory and use `alternate_value` to
  snap unsafe values back inside the safe envelope. See
  [examples/causal_correction_examples/](examples/causal_correction_examples/).
- Causaloid-backed CSM: define a `Causaloid` whose output is a boolean-like
  trigger, wrap it in `CausalState`, pair it with `CausalAction`, and evaluate
  through `CSM`. See [examples/csm_examples/csm_basic/](examples/csm_examples/csm_basic/).
- Context-aware CSM: share `BaseContext` through `Arc<RwLock<_>>` when causal
  logic must inspect mutable environment data. See
  [examples/csm_examples/csm_context/](examples/csm_examples/csm_context/).
- Effect Ethos check: define Teloid norms, evaluate a `ProposedAction` against a
  Context, and use the verdict before executing CSM-proposed actions. See
  [docs/ETHOS.md](docs/ETHOS.md) and
  [examples/csm_examples/csm_effect_ethos/](examples/csm_examples/csm_effect_ethos/).
- Causal discovery: use `deep_causality_discovery` when the user starts from
  observational data and needs the CDL pipeline: load, clean, select, discover,
  analyze, report. See [docs/DISCOVERY.md](docs/DISCOVERY.md) and
  [examples/causal_discovery_examples/](examples/causal_discovery_examples/).

## User-Facing Crate Map

- `deep_causality`: full causal framework: Causaloid, CausaloidGraph, Context,
  CSM, model types, and integration with Effect Ethos concepts.
- `deep_causality_core`: monadic carriers and fluent causal pipelines:
  `PropagatingEffect`, `PropagatingProcess`, `CausalMonad`, `CausalFlow`, logs,
  and causal errors.
- `deep_causality_ethos`: `EffectEthos`, `Teloid`, `TeloidModal`, and deontic
  conflict resolution for action safety.
- `deep_causality_uncertain`: `Uncertain<T>` and `MaybeUncertain<T>` for
  explicit probabilistic values and probabilistic presence.
- `deep_causality_discovery`: Causal Discovery Language (CDL), a typestate
  pipeline from data files to causal analysis reports.
- `deep_causality_algorithms`: SURD, MRMR, and feature-selection primitives used
  by discovery workflows.
- `deep_causality_tensor`: N-dimensional tensors, broadcasting, Einstein
  summation, and categorical interfaces.
- `deep_causality_multivector`: Clifford and geometric algebra types.
- `deep_causality_topology`: graphs, hypergraphs, simplicial complexes,
  manifolds, point clouds, exterior calculus, and lattice gauge structures.
- `deep_causality_sparse`: CSR sparse matrices.
- `deep_causality_physics`: physics formulas and engineering primitives across
  astrophysics, electromagnetics, fluids, MHD, quantum mechanics, relativity,
  thermodynamics, and waves.
- `deep_causality_num`, `deep_causality_num_complex`,
  `deep_causality_num_dual`, and `deep_causality_algebra`: numerical and
  algebraic foundations.
- `deep_causality_haft`: higher-kinded type witness patterns and categorical
  traits used across the stack.
- `ultragraph`: dynamic and frozen graph backend used by CausaloidGraph and
  Context.

## Internal Docs To Read By Name

- [docs/INTRO.md](docs/INTRO.md): conceptual introduction to dynamic causality, explicit
  Context, and the spacetime-agnostic dependency model.
- [docs/CORE.md](docs/CORE.md): monadic effect system, `PropagatingEffect`,
  `PropagatingProcess`, error channels, logs, and interventions.
- [docs/DEEP_DIVE.md](docs/DEEP_DIVE.md): Effect Propagation Process, Causaloids, Context,
  Causaloid collections, causal hypergraphs, CSM, and Effect Ethos.
- [docs/ETHOS.md](docs/ETHOS.md): Effect Ethos, Teloids, modalities, conflict resolution, and
  verdicts.
- [docs/DISCOVERY.md](docs/DISCOVERY.md): Causal Discovery Language, MRMR, SURD, and CDL pipeline
  stages.
- [examples/README.md](examples/README.md): runnable examples and the command catalogue.

## Agent Rules

- Do not invent DeepCausality APIs. Search the crate source, docs, and examples
  before writing code.
- Prefer repository examples over prose if an API name differs between docs.
- Import public types from crate roots when possible, for example
  `use deep_causality::{Causaloid, CausaloidGraph, PropagatingEffect};`.
- For user projects, start from `cargo add deep_causality` or the smallest
  domain crate. Add extra DeepCausality crates only when the user needs their
  APIs directly.
- When choosing an example to adapt, start with `examples/starter_example/` for
  basic pipelines, `examples/core_examples/` for monadic mechanics,
  `examples/csm_examples/` for inference-to-action, and
  `examples/causal_discovery_examples/` for data-to-graph workflows.
- Mention that this file should be reviewed with each release when API names,
  crate versions, or conceptual docs change.
