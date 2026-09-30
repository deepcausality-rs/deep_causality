[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

[![Crates.io][crates-badge]][crates-url]
[![Docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]
[![CodeFactor][codefactor-badge]][codefactor-url]
[![OpenSSF Best Practices][ossf-badge]][ossf-url]
[![codecov][codecov-badge]][codecov-url]

[//]: # ([![Miri][miri-badge]][miri-url])

[codefactor-badge]: https://www.codefactor.io/repository/github/deepcausality-rs/deep_causality/badge

[codefactor-url]: https://www.codefactor.io/repository/github/deepcausality-rs/deep_causalityl

[codecov-badge]: https://codecov.io/gh/deepcausality-rs/deep_causality/branch/main/graph/badge.svg?token=W9TA1VVJ7O

[codecov-url]: https://codecov.io/gh/deepcausality-rs/deep_causality

[ossf-badge]: https://bestpractices.coreinfrastructure.org/projects/7568/badge

[ossf-url]:https://bestpractices.coreinfrastructure.org/projects/7568

[crates-badge]: https://img.shields.io/badge/Crates.io-Latest-blue

[crates-url]: https://crates.io/crates/deep_causality

[docs-badge]: https://img.shields.io/badge/Docs.rs-Latest-blue

[docs-url]: https://docs.rs/deep_causality/latest/deep_causality/

[mit-badge]: https://img.shields.io/badge/License-MIT-blue.svg

[mit-url]: https://github.com/deepcausality-rs/deep_causality/blob/main/LICENSE

[miri-badge]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust_miri.yml/badge.svg

[miri-url]: https://github.com/deepcausality-rs/deep_causality/actions/workflows/rust_miri.yml

---

<div align="center">

[<img src="https://raw.githubusercontent.com/deepcausality-rs/deep_causality/main/img/logo_background.jpg" width="900">](https://deepcausality.com)

</div>

---

<div style="display: flex; flex-wrap: wrap; justify-content: center; align-items: center; text-align: center;">

[Website](https://deepcausality.com) | [Getting started](https://www.deepcausality.com/docs/getting-started/install/) | [Blog](https://deepcausality.com/blog/) | [Discord](https://discord.gg/Bxj9P7JXSj) | [Crates](https://crates.io/crates/deep_causality)

</div>

# DeepCausality

**Dynamic causal reasoning: cause and effect in systems whose state, environment and rules change dynamically.**

DeepCausality is a Rust library for dynamic causal reasoning. A model is written as a chain of causal steps that 
natively supports dynamic change via:

- **State carries forward.** Each step receives the state the previous step left behind, so the next effect can
  depend on the history of prior state.
- **The environment changes.** A context of data, time and space can be updated between evaluations, or kept current
  from an event stream e.g. from a message bus.
- **The applicable law changes.** A step can choose its law from the state it receives: the
  [event horizon probe](examples/physics_examples/event_horizon_probe/) switches from Newtonian to relativistic gravity
  as the probe nears a black hole. A causaloid graph can be unfrozen, edited and frozen again, and a context model can be
  evolved by programmable edits.
- **"What if?" runs mid-process.** A value, the state or the context can be replaced in the middle of a run; the rest
  runs on the replacement, and an append-only log records the change.

## See how it works

```bash
cargo add deep_causality_core
```

A lithium-ion cell is fast-charged for 30 minutes. Each minute, the charger reads the cell temperature and chooses its
current to protect the cell: 50 A below 45 °C, 25 A up to 55 °C, and none above. The current adds charge and heats the
cell, and the coolant carries heat away in proportion to the cell's rise above ambient. Each minute is one causal step:
the temperature is the value it hands to the next minute, and the charge is the state it carries forward.

The question is "what if?": what if the cell had been at 52 °C after ten minutes? The first run charges as observed.
The second run replaces the temperature at minute ten and lets the remaining twenty minutes run from there.

```rust
use deep_causality_core::{CausalEffect, CausalFlow, EffectLog};

type FloatType = f64;

/// A charging run: the value is the cell temperature (°C), the state is the charge (%).
type Charge = CausalFlow<FloatType, FloatType, ()>;

const CELL_CAPACITY_AH: FloatType = 50.0;
const AMBIENT_C: FloatType = 25.0;

fn main() {
    let observed = plug_in().iterate_n(30, one_minute).into_process();

    let what_if = plug_in()
        .iterate_n(10, one_minute)
        .alternate_value(52.0) // what if the cell had been at 52 °C after ten minutes?
        .iterate_n(20, one_minute)
        .into_process();

    for (name, run) in [("Observed", &observed), ("What if ", &what_if)] {
        if let Some(cell_c) = run.value() {
            println!("{name}: {cell_c:.1} °C, {:.1} % charged after 30 minutes", run.state());
        }
    }
    println!("{}", what_if.logs());
}

/// The charger's law depends on the temperature it reads: full current below 45 °C,
/// half current up to 55 °C, and no current above.
fn charge_current_a(cell_c: FloatType) -> FloatType {
  if cell_c < 45.0 {
    50.0
  } else if cell_c < 55.0 {
    25.0
  } else {
    0.0
  }
}

/// One minute of fast charging. The current adds charge and heats the cell;
/// the coolant removes heat in proportion to the cell's rise above ambient.
fn one_minute(run: Charge) -> Charge {
  run.step_mut(|cell_c, charge_pct, _| {
    let amps = charge_current_a(cell_c);
    *charge_pct = (*charge_pct + amps / CELL_CAPACITY_AH * 100.0 / 60.0).min(100.0);
    Ok(cell_c + 0.00048 * amps * amps - 0.08 * (cell_c - AMBIENT_C))
  })
}

/// The cell starts at ambient temperature, 20 % charged.
fn plug_in() -> Charge {
  CausalFlow::from_parts(Ok(CausalEffect::value(AMBIENT_C)), 20.0, None, EffectLog::new())
}
```

```text
Observed: 38.8 °C, 70.0 % charged after 30 minutes
What if : 41.2 °C, 65.8 % charged after 30 minutes
EffectLog (1 entries):
[[1790749984339484] !!ValueAlternation!!: Value(33.48417318664552) replaced with Value(52.0)
```

At 52 °C the charger switches to half current. The cell cools below 45 °C within five minutes and full current
resumes, but the charge lost in those minutes stays lost: the second run ends 4.2 percentage points lower. The log
names the temperature the model had computed at minute ten, 33.5 °C, and the 52 °C that replaced it. Replacing the
value is Pearl's intervention `do(T₁₀ = 52 °C)`: the first run *observes*, `alternate_value` *intervenes*, and
comparing the two runs answers the *counterfactual* question.

## What you get

| You want to                                   | DeepCausality gives you                                                                                          |
|-----------------------------------------------|------------------------------------------------------------------------------------------------------------------|
| Ask "what if?" of a running process           | `alternate_value`, `alternate_state` and `alternate_context` substitute a value, the state, or the environment mid-run |
| Explain a result                              | An append-only log that interventions and causaloid evaluations write to; it survives an error that stops the pipeline |
| Put physics inside a causal order             | The body of each causal step is unconstrained, so a time-reversible solver can run inside a one-directional causal chain |
| Reason over structure that changes            | Causaloids as a singleton, a collection or a graph; a `Context` hypergraph of sensor data, time and space that updates while reasoning runs |
| Check an action before it executes            | Effect Ethos returns a `Verdict` (`Obligatory`, `Impermissible`, `Optional`) that cites the norms behind it       |
| Discover causes from data                     | SURD, mRMR and BRCD behind a typed pipeline: load → clean → select → discover → analyse                           |
| Carry the same idea into physics              | Tensors, geometric algebra, manifolds, fluid dynamics and quantum models on one categorical interface            |

## Run larger examples

The same three moves (observe, intervene, compare) scale to avionics, fluid dynamics and quantum models.

```bash
# Causal structure evolves as the system crosses a physical threshold
cargo run -p physics_examples --example event_horizon_probe

# Five sensor checks, a state estimator and six envelope protections in one pipeline
cargo run -p avionics_examples --example flight_envelope_monitor

# Fork a running plasma-blackout simulation into counterfactual worlds; commit the best bank command
cargo run --release -p avionics_examples --example plasma_blackout_corridor

# Reject a quantum causal model whose operators fail to commute, naming the offending pair
cargo run -p quantum_examples --example qcm_freeze_check

# Find causes in data: load → clean → mRMR → SURD → analyse
cargo run -p causal_discovery_examples --example example_surd_discovery
```

* [examples/README.md](examples/README.md) lists every example. 
* [cfd.deepcausality.com](https://cfd.deepcausality.com) covers counterfactual fluid-dynamics. 
* [quantum.deepcausality.com](https://quantum.deepcausality.com) covers quantim causal models.

## Pick a crate

| Start with                                                                               | When you want                                                                    |
|------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| [`deep_causality_core`](deep_causality_core/README.md)                                   | Effect pipelines, interventions and audit logs (`PropagatingEffect`, `PropagatingProcess`, `CausalFlow`) |
| [`deep_causality`](deep_causality/README.md)                                             | Causaloids (singleton, collection, graph) and the Causal State Machine           |
| [`deep_causality_context`](deep_causality_context/README.md)                             | A `Context` of data, time, space and spacetime that causal functions read, and its persistence |
| [`deep_causality_ethos`](deep_causality_ethos/README.md)                                 | Norms that verify a proposed action                                              |
| [`deep_causality_discovery`](deep_causality_discovery/README.md)                         | Causal discovery from tabular data                                               |
| [`deep_causality_uncertain`](deep_causality_unified_math/deep_causality_uncertain/README.md) | Uncertain values (`Uncertain<T>`) inside a pipeline                              |
| [`deep_causality_physics`](deep_causality_physics/README.md)                             | Physics formulas for astrophysics, fluids, EM, relativity, thermodynamics and more |
| [`deep_causality_cfd`](deep_causality_cfd/README.md)                                     | Fluid dynamics with counterfactual forks, coupled to chemistry, navigation and control |
| [`deep_causality_quantum`](deep_causality_quantum/README.md)                             | Quantum causal models, density matrices, channels and gates                       |

<details>
<summary>All crates</summary>

| Crate                                                                                             | Role                                                                                        |
|---------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------|
| [`deep_causality`](deep_causality/README.md)                                                      | Causaloid (Singleton, Collection, Graph), CausaloidGraph reasoning, CSM                     |
| [`deep_causality_core`](deep_causality_core/README.md)                                            | `PropagatingEffect`, `PropagatingProcess`, `CausalMonad`, `CausalArrow`, `CausalFlow`       |
| [`deep_causality_context`](deep_causality_context/README.md)                                      | `Context` hypergraph: contextoids and data, space, time and spacetime nodes                 |
| [`deep_causality_context_store`](deep_causality_context_store/README.md)                          | Persistence contract for `Context`: the records it projects onto and the `ContextStorage` trait a backend implements |
| [`deep_causality_ethos`](deep_causality_ethos/README.md)                                          | `EffectEthos` and `Teloid` for defeasible deontic reasoning                                 |
| [`deep_causality_discovery`](deep_causality_discovery/README.md)                                  | Causal Discovery Language (typestate DSL)                                                   |
| [`deep_causality_algorithms`](deep_causality_algorithms/README.md)                                | SURD, mRMR, BRCD and feature-selection primitives                                           |
| [`deep_causality_physics`](deep_causality_physics/README.md)                                      | Physics formulas, generic over the float type                                               |
| [`deep_causality_cfd`](deep_causality_cfd/README.md)                                              | DEC and tensor-train flow solvers behind the `CfdFlow` DSL                                  |
| [`deep_causality_quantum`](deep_causality_quantum/README.md)                                      | Quantum causal models, density matrices, channels, gates                                    |
| [`deep_causality_uncertain`](deep_causality_unified_math/deep_causality_uncertain/README.md)      | `Uncertain<T>` and `MaybeUncertain<T>` (after Bornholt et al.)                              |
| [`deep_causality_tensor`](deep_causality_unified_math/deep_causality_tensor/README.md)            | N-dimensional tensors, broadcasting, Einstein summation, tensor trains                      |
| [`deep_causality_multivector`](deep_causality_unified_math/deep_causality_multivector/README.md)  | Clifford algebras: Pauli, spacetime, conformal, projective, Dixon, Spin(10)                 |
| [`deep_causality_topology`](deep_causality_unified_math/deep_causality_topology/README.md)        | Graphs, simplicial complexes, manifolds, exterior calculus, lattice gauge fields            |
| [`deep_causality_linear`](deep_causality_unified_math/deep_causality_linear/README.md)            | Sparse, dense and bit-packed 𝔽₂ matrices, decompositions, conjugate gradient               |
| [`deep_causality_num`](deep_causality_unified_math/deep_causality_num/README.md)                  | Numerical traits (casts, identity, float, integer) and `Float106`                           |
| [`deep_causality_haft`](deep_causality_unified_math/deep_causality_haft/README.md)                | Higher-kinded types by witness; Functor, Applicative, Monad, CoMonad                        |
| [`deep_causality_metric`](deep_causality_unified_math/deep_causality_metric/README.md)            | Metric signatures: East Coast, West Coast, Cl(p,q,r)                                        |
| [`ultragraph`](ultragraph/README.md)                                                              | Two-phase hypergraph backend for `CausaloidGraph` and `Context`                             |
| [`deep_causality_data_structures`](deep_causality_data_structures/README.md)                      | Sliding window, grid array and other specialised structures                                 |
| [`deep_causality_rand`](deep_causality_unified_math/deep_causality_rand/README.md)                | Generators, uniform range sampling, Sobol sequences                                         |
| [`deep_causality_ast`](deep_causality_utils/deep_causality_ast/README.md)                         | Generic abstract syntax tree                                                                |

The remaining mathematics crates (algebra, calculus, fft, homology, num_complex, num_dual, num_rational, stats) are
described in [deep_causality_unified_math/README.md](deep_causality_unified_math/README.md).

</details>

## How it works

DeepCausality implements the **Effect Propagation Process (EPP)**, a theory of dynamic causality built on Whitehead's
process metaphysics. The EPP rests on one axiom, **`m₂ = m₁ >>= f`**. The causal monad in `deep_causality_core`
implements it as follows:

> Effects propagate by monadic dependency. Each step is computed from exactly three channels of the previous step:
> effect, state and context.

### Two layers of composition

The classical formulation of causality, "if A then B, and if not A then not B", composes on one layer: the causal
direction from A to B is also the direction of time, so a time-symmetric relation cannot be written as a cause.
This is a problem for physics: its fundamental equations of motion, from Newton's through Maxwell's to Schrödinger's,
are time-symmetric, so classical causality cannot express them.

DeepCausality composes on two layers.

- **The outer layer is the causal order.** Steps compose by Kleisli composition, `bind`. The order is linear and
  one-directional: a step runs after the one before it and receives its effect, state and context. A causal graph
  arranges steps into a directed acyclic graph and refuses a cycle when it is frozen, so no step can feed back into an
  earlier one.
- **The inner layer is the step itself.** The body of a step is an unconstrained function, so it can hold a
  time-symmetric law of physics as readily as any other computation. The function carries no assumption of space or
  time: location and time reach it only as data, through the context or as values such as the `dt` below.

The example below moves a satellite 600 s forward along its orbit and then 600 s back, using the exact two-body
propagator from `deep_causality_physics`:

```rust
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_physics::{EARTH_GM, TwoBodyPropagator};

type FloatType = f64;
type Orbit = ([FloatType; 2], [FloatType; 2]); // (position, velocity)

fn main() {
    let epoch: Orbit = ([7.0e6, 0.0], [0.0, 7.8e3]);
    let result = PropagatingEffect::pure(epoch)
        .bind_or_error(|s, _, _| kepler_step(s, 600.0), "no state") // forward in time
        .bind_or_error(|s, _, _| kepler_step(s, -600.0), "no state"); // next in causal order, backward in time

    let (p, _) = result.into_value().expect("both steps succeed");
    let miss = ((p[0] - 7.0e6).powi(2) + p[1].powi(2)).sqrt();
    println!("returned to epoch within {miss:.2e} m");
}

/// One Kepler step. The propagator is exact and accepts a negative `dt`.
fn kepler_step(state: Orbit, dt: FloatType) -> PropagatingEffect<Orbit> {
    let (p, v) = state;
    match TwoBodyPropagator::from_state(p, v, EARTH_GM).and_then(|orbit| orbit.propagate(dt)) {
        Ok(next) => PropagatingEffect::pure(next),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
```

```text
returned to epoch within 5.67e-10 m
```

**What it computes.** Orbit determination routinely propagates a satellite's measured position and velocity forward
and backward in time. Here the satellite starts 7,000 km from Earth's centre at 7.8 km/s, which puts it on an orbit
between 629 km and 1,658 km altitude, once around every 108 minutes. The first step carries it 4,585 km along that
orbit. The second step takes the position and velocity the first step produced, derives a fresh orbit from them, and
runs the clock back 600 s. The satellite lands 0.57 nanometres from where it started. At 7,000 km, neighbouring `f64`
values lie 0.93 nanometres apart, so the round trip is exact to the resolution of the number type.

**Why the round trip is exact.** Kepler's problem has a closed-form solution. Measured by the angle around the
ellipse, called the eccentric anomaly, the orbit is simple harmonic motion, and one formula advances it by any
interval of time. The only iterative part is Kepler's equation, which converts that angle into clock time; the kernel
solves it by Newton's method to the precision of the number type. No step size enters, so no truncation error
accumulates, and a negative `dt` runs the same formula backward. A step-by-step integrator such as RK4 lands slightly
off its starting point on the same round trip, by an error that grows with its step size.

**What it shows about causality.** The second step follows the first in causal order. It cannot run until the first
has produced its state, and an error in the first would stop it. In physical time, it runs the other way. Classical
causality, with one arrow for both cause and time, cannot write that chain. Here `bind` carries the causal order and
the propagator carries the physics, and a time-symmetric problem runs inside a one-directional causal chain in under
thirty lines of Rust.

### Three channels and a log

| Channel | Type                                                         | What it gives a model                                                  |
|---------|--------------------------------------------------------------|------------------------------------------------------------------------|
| Effect  | a value, `None` (no evidence), or a `RelayTo` command        | the result of the previous step                                        |
| State   | any type; `()` when unused                                   | memory carried from step to step                                       |
| Context | any type, passed as `Option<C>`                              | the environment: data, time and space, with fixed or changing relations |

Without state, a step depends only on the effect it receives. With state, a step carries whatever history the state
type holds.

The log is not a channel. `bind` appends each step's entries after the earlier ones and never passes the log to a
step, so no step can read it. `EffectLog` offers no way to remove or edit an entry, and under `std` each entry carries a
wall-clock timestamp. The log records what was computed and when: provenance for debugging, and evidence where an auditor asks
for it.

Two properties are left to the model author. A `bind` step is any `FnOnce`, so the monad cannot guarantee that it is
pure; a causaloid narrows this by storing a function pointer, which cannot capture its environment. And `bind` accepts
any function of the right type, so which dependencies count as causal is the author's decision.

### 1. Causaloid and CausalMonad

- **Causaloid.** A container for the causal function `f` (after Hardy), in three forms: a **Singleton** runs one
  function; a **Collection** feeds one input to every member and aggregates the results with `All`, `Any`, `None` or
  `Some(k)`; a **Graph** evaluates a directed acyclic graph of causaloids in topological order and joins effects where
  branches reconverge. `Causaloid::evaluate` evaluates a singleton. A collection or a graph is evaluated by its own
  engine, `evaluate_collection` or `evaluate_subgraph_from_cause`, which can run inside a `bind` step.
- **CausalMonad.** The outer layer: `bind` sequences steps, short-circuits on error and appends the log.
  `alternate_value`, `alternate_state` and `alternate_context` substitute a channel mid-run and log the substitution.

Both use one carrier:

| Type                          | Purpose                      | Channels                              |
|-------------------------------|------------------------------|---------------------------------------|
| `PropagatingEffect<T>`        | Stateless effect propagation | Effect · Error · Log                  |
| `PropagatingProcess<T, S, C>` | Stateful effect propagation  | Effect · State · Context · Error · Log |

Pick the shape that fits each stage:

* **Sequential steps** belong in a bind chain.
* **Many causes of one input, aggregated,** belong in a Causaloid collection.
* **Dependencies that branch and reconverge** belong in a Causaloid graph.

The [flight envelope monitor](examples/avionics_examples/control/flight_envelope_monitor/) uses all three: a Causaloid
collection over five sensor-health checks, a three-step bind-chain for state estimation, and a Causaloid hypergraph of
six envelope protections, all inside one `PropagatingProcess<T, FlightState, AircraftConfig>`.

### 2. Context

A graph of typed nodes that holds the operating environment: data, time, space and spacetime. It lives in
`deep_causality_context`, a separate crate that `deep_causality` depends on; a model built on `deep_causality_core`
alone does not link it. Space covers Euclidean,
ECEF, geodetic and NED positions; time covers Euclidean, Lorentzian, discrete and entropic clocks; spacetime covers
Euclidean, Lorentzian and tangent frames, each carrying its own metric. The causal engine passes the context to a
causal function and reads nothing from it, so the context is the route by which time, space and external data reach a
cause. Causal functions query the Context while they run, and the Context updates as the system evolves.
`deep_causality_context_store` defines how a context is persisted: snapshot and restore, applying a stream of store
events, and storing a modified branch as a new container.

### 3. Causal State Machine

The CSM connects inference to action. Each state holds a causaloid; when its output is active (true, or an uncertain
value that passes a sequential probability test), the CSM fires the paired action. The CSM does not consult the Effect
Ethos: to check an action first, evaluate the ethos on a `ProposedAction` and fire only on a permitting verdict.

### 4. Effect Ethos

An optional norm layer. A norm (`Teloid`) is a predicate over the context and a `ProposedAction`, with a modality and
three conflict-resolution values: specificity, timestamp and priority. Norms link by `Inherits` and `Defeats` edges.
`evaluate_action` selects norms by tag, runs their predicates, removes defeated norms, and returns a `Verdict`
(`Impermissible`, `Obligatory` or `Optional(cost)`) that lists the norms behind it. Its vocabulary follows the
Defeasible Deontic Inheritance Calculus (DDIC).

### Unified mathematics

Seventeen crates under `deep_causality_unified_math/` supply the mathematics: numeric traits and two software
scalars, abstract algebra, linear algebra, statistics and distributions, FFT, calculus and automatic differentiation,
tensors and tensor trains, geometric algebra, chain complexes and homology, topology with discrete exterior calculus
and gauge fields, and uncertain values. None of them has a required external dependency. Two design decisions make
them work as one library.

**One categorical interface.** `deep_causality_haft` provides higher-kinded types in stable Rust. A crate that owns a
container declares a zero-sized *witness* type and implements `Functor`, `Applicative`, `Monad`, `CoMonad`,
`Foldable` or `Traversable` against it; forty witnesses across seven crates do so. A witness accepts any element type,
so containers from different crates nest, and one `fmap` rotates every cell of a tensor of multivectors:

```rust
let rotated: CausalTensor<CausalMultiVector<FloatType>> =
    CausalTensorWitness::fmap(field, |v| rotor.geometric_product(&v).geometric_product(&rotor_rev));
```

The comonadic `extend` hands a closure a cursor into a graph, a mesh or a manifold, which is how a stencil, a
diffusion step or a graph convolution is written. Each of the monads (tensor, dense vector, manifold) comes with a
Kleisli category, and its laws are machine-checked in Lean. A value that *is* a computation, held now and run later,
implements `Arrow`: the lazy sampling graph behind `Uncertain<T>` and the differentiation and integration operators of
`deep_causality_calculus` both do. The causal monad is built on the same traits.

| Domain      | Type                   | Categorical role                                   |
|-------------|------------------------|----------------------------------------------------|
| Mechanics   | `CausalTensor<T>`      | Functor, Monad, CoMonad (map, chain, stencil)      |
| Algebra     | `CausalMultiVector<T>` | Functor, Applicative, CoMonad                      |
| Topology    | `Manifold<T>`          | Monad, CoMonad (neighbourhood analysis)            |
| Uncertainty | `Uncertain<T>`         | Arrow (a lazy computation, sampled on demand)      |
| Causality   | `PropagatingEffect<T>` | Monad (sequencing + logs)                          |

**Precision as a parameter.** Every crate above `deep_causality_num` is generic in its scalar. A program names its
working type once, `type FloatType = f64;`, and changing that line moves the whole program between `BFloat16`, `f32`,
`f64` and `Float106`, a double-double with about 31 decimal digits. The unified-math README sums a million terms of a
series whose value is known in closed form, at each precision:

| `FloatType` | Error after a million terms | Correct digits |
|-------------|-----------------------------|----------------|
| `BFloat16`  | `3.1e-2`                    | 2              |
| `f32`       | `1.5e-4`                    | 4              |
| `f64`       | `4.8e-14`                   | 13             |
| `Float106`  | `9.8e-31`                   | 30             |

The alias holds across crates. A field carried from a tensor onto a manifold, differentiated by comonadic extension
and rotated a thousand times as multivectors stays at one precision end to end, with nothing converted between
crates. Parts of one program can also run at different precisions, each chosen against its own error budget, and
meet at a wider one.

The [unified math README](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_unified_math/README.md)
covers the dependency tiers, the full witness table, the rules for crossing into and out of the working type, and the
programs behind these numbers.


## Built for assurance

* **Memory safety.** `unsafe_code = "forbid"` is a workspace lint enforced by the Rust compiler.
* **Small supply chain.** 26 of the 33 library crates have zero external dependencies.
* **Bare metal.** `deep_causality_core` and other crates build with `no-std`; CI builds them for
  `thumbv7em-none-eabihf`. [README_NO_STD.md](README_NO_STD.md) lists the configuration of every crate.
* **Machine-checked laws.** [lean/](lean/README.md) holds Lean 4 proofs of the core laws, and CI fails when a proof breaks.
* **Precision as a parameter.** Numeric code is generic over the scalar type; `Float106` gives about 31 decimal digits
  on stable Rust.

## Support

Dynamic causality can be daunting at first. For a larger or commercial project, contact the
[Center for Dynamic Causality](https://www.causalcenter.com/contact/), which backs DeepCausality. To build with a coding
assistant, start from [SKILLS.md](SKILLS.md).

DeepCausality is a member project of the LF AI & Data Foundation, part of the Linux Foundation.

## Build and test

```bash
cargo build -p deep_causality_core   # build one crate
cargo test  -p deep_causality_core   # test one crate
make build                           # build the whole workspace
make test                            # test the whole workspace
make check                           # security audit
```

The repository also builds with Bazel (`bazel build //...`, `bazel test //...`); see [Bazel.md](Bazel.md) and
[BUILD.md](BUILD.md).

## Contributing

Contributions are welcome. Please read:

* [AI Coding Assistants](AiCodingAssistants.md)
* [Contributing Guide](CONTRIBUTING.md)
* [Code of Conduct](CODE_OF_CONDUCT.md)

Before you submit a pull request, run `make test` and `make check`.

## Acknowledgement

Inspired by research from:

* [Judea Pearl](http://bayes.cs.ucla.edu/jp_home.html): Structural Causal Models
* [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy): Causaloid framework
* [Elias Bareinboim](https://causalai.net/): Transportability and data fusion
* "[Root Cause Analysis of Failures in Microservices via Bayesian Root Cause Discovery](https://icml.cc/virtual/2026/poster/65359)"
* [Maximum Relevance and Minimum Redundancy Feature Selection](deep_causality_algorithms/papers/mrmr_feature_selector.pdf)
* ["Observational causality by states and interaction type for scientific discovery"](deep_causality_algorithms/papers/surd-state.pdf)
* ["A Defeasible Deontic Calculus for Resolving Norm Conflicts"](deep_causality_ethos/papers/ddic.pdf), Olson, Salas-Damian & Forbus

Many crates contains a "papers" folder where you find copies of research
that was used for during the imppementation. 

---

## Community

* [Discord](https://discord.gg/Bxj9P7JXSj)
* [GitHub Discussions](https://github.com/orgs/deepcausality-rs/discussions)
* [LF Email Lists](https://deepcausality.com/community/)

---

## 👮 Security

The DeepCausality project follows the Linux Foundation CRA stewardship framework to comply with
the EU Cyber Resilience Act (CRA).

See [SECURITY.md](SECURITY.md) for security policies and details.

---

## Sponsors

[![JetBrains logo.](https://resources.jetbrains.com/storage/products/company/brand/logos/jetbrains.svg)](https://jb.gg/OpenSource)

[JetBrains](https://www.jetbrains.com/) provides project core maintainers with an all-product license.

<a href="https://www.causalcenter.com">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/deepcausality-rs/deep_causality/main/img/causal_center_logo_dark.svg">
    <img src="https://raw.githubusercontent.com/deepcausality-rs/deep_causality/main/img/causal_center_logo.svg" alt="Center for Dynamic Causality" width="140">
  </picture>
</a>

The [Center for Dynamic Causality](https://www.causalcenter.com) contributes ongoing research and resources to the DeepCausality project.

---

## License

This project is licensed under the [MIT license](LICENSE).
