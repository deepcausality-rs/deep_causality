<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Building DeepCausality without `std`

Twenty-three of the thirty-three library crates build on bare metal. That covers all maths crtes,
the causal monad, linear algebra, statistics, tensors, multivectors, homology, FFT, the
physics kernels, SURD and mRMR, the quantum layer, and the `ultragraph` graph store. What stays
behind is uncertainty, topology, the context hypergraph and its store, the reasoning engine in
`deep_causality` with the ethos layer on top of it, and everything that touches files.

CI checks this on every pull request. `.github/workflows/rust_no_std.yml` builds each covered crate
for `thumbv7em-none-eabihf`, a 32-bit Cortex-M4F with no `std`, so nothing here is inferred from a
host build with `std` switched off. [Verification](#verification) has the commands.

## Quick start

```bash
rustup target add thumbv7em-none-eabihf

cargo build -p deep_causality_core \
  --no-default-features --features no-std \
  --target thumbv7em-none-eabihf
```

Five crates need only `core`. The other eighteen need an allocator, which rules out very little;
[Allocators](#allocators) explains why.

## Support by crate

`core` is bare metal with no heap. `alloc` is bare metal once the application provides a
`#[global_allocator]`. `std` is a hosted target. CI builds every crate with a mark under `core` or
`alloc` for `thumbv7em-none-eabihf` with `--no-default-features --features no-std`. The notes name
the features that stay host-only, and for the std-only crates, what ties them to `std`;
[Not covered](#not-covered) has the detail.

| Crate | `core` | `alloc` | `std` | Notes |
|---|:-:|:-:|:-:|---|
| `deep_causality_num` | ✓ | ✓ | ✓ | Chooses the float backend: intrinsics under `std`, `libm` under `no-std` |
| `deep_causality_algebra` | ✓ | ✓ | ✓ | |
| `deep_causality_num_complex` | ✓ | ✓ | ✓ | |
| `deep_causality_num_dual` | ✓ | ✓ | ✓ | |
| `deep_causality_num_rational` | ✓ | ✓ | ✓ | |
| `deep_causality_ast` | – | ✓ | ✓ | Builds with `alloc` alone |
| `deep_causality_metric` | – | ✓ | ✓ | Builds with `alloc` alone |
| `deep_causality_data_structures` | – | ✓ | ✓ | Builds with `alloc` alone |
| `ultragraph` | – | ✓ | ✓ | Builds with `alloc` alone |
| `deep_causality_par` | – | ✓ | ✓ | Builds with `alloc` alone; `parallel` is host-only |
| `deep_causality_haft` | – | ✓ | ✓ | |
| `deep_causality_calculus` | – | ✓ | ✓ | The allocator comes from `deep_causality_haft` |
| `deep_causality_core` | – | ✓ | ✓ | `EffectLog` allocates per entry |
| `deep_causality_linear` | – | ✓ | ✓ | |
| `deep_causality_rand` | – | ✓ | ✓ | `os-random` is host-only |
| `deep_causality_stats` | – | ✓ | ✓ | `os-random` is host-only |
| `deep_causality_tensor` | – | ✓ | ✓ | |
| `deep_causality_multivector` | – | ✓ | ✓ | |
| `deep_causality_homology` | – | ✓ | ✓ | |
| `deep_causality_fft` | – | ✓ | ✓ | `parallel` is host-only |
| `deep_causality_physics` | – | ✓ | ✓ | `topology`, `parallel` and `os-random` are host-only |
| `deep_causality_algorithms` | – | ✓ | ✓ | `topology` (BRCD) and `parallel` are host-only |
| `deep_causality_quantum` | – | ✓ | ✓ | `qcm`, `dem` and `qpu` are host-only |
| `deep_causality_uncertain` | – | – | ✓ | `HashMap`, `HashSet` |
| `deep_causality_topology` | – | – | ✓ | `HashMap`/`HashSet`, `std::sync`, Rayon loops |
| `deep_causality_context_store` | – | – | ✓ | `std::sync::Mutex` in the in-memory test backend |
| `deep_causality_context` | – | – | ✓ | Depends on `deep_causality_context_store` and `deep_causality_uncertain` |
| `deep_causality` | – | – | ✓ | Depends on `deep_causality_context` and `deep_causality_uncertain` |
| `deep_causality_ethos` | – | – | ✓ | Depends on `deep_causality` and `deep_causality_context` |
| `deep_causality_file` | – | – | ✓ | Filesystem loaders; out of scope |
| `deep_causality_tempfile` | – | – | ✓ | Scratch files for the test suites; out of scope |
| `deep_causality_discovery` | – | – | ✓ | Reads CSV and Parquet files; out of scope |
| `deep_causality_cfd` | – | – | ✓ | Writes output files; out of scope |

Five crates reach `core`, eighteen more reach `alloc`, and ten are std-only: twenty-three of the
thirty-three build on bare metal.

## The feature levels

The eighteen allocator-required crates declare the same three levels:

```toml
[features]
default = ["std"]
std     = ["alloc", "<dep>/std", ...]
alloc   = []
no-std  = ["alloc", "<dep>/no-std", ...]
```

`std` implies `alloc`; `no-std` selects `alloc` without `std`. Where a dependency gates heap code
behind its own `alloc`, the crate's `alloc` forwards to it, as `deep_causality_multivector` does
with `alloc = ["deep_causality_metric/alloc"]`. Three crates add a feature to
`default` that carries a std-only dependency, and each of those features enables `std`: `topology`
in `deep_causality_physics` and `deep_causality_algorithms`, `qcm` in `deep_causality_quantum`.
Turning them off is what brings those crates to bare metal.

Dependencies are declared with `default-features = false` in the workspace table, which every
member inherits, so that `--no-default-features` actually reaches them. Without that, Cargo hands a
dependency its own defaults and `std` returns through the back door, which compiles fine on a host
and fails only when you cross-compile. A member cannot opt out on its own: when the workspace entry
omits `default-features = false`, Cargo ignores the member's.

The five `core`-only crates declare no `alloc` feature at all, because nothing in them touches the
heap. They carry `default`, `std` and `no-std`, and `no-std` does not name `alloc`.
`deep_causality_num` is where the float-math backend is chosen:

```toml
# deep_causality_num
[features]
default   = ["std"]
std       = []
no-std    = ["libm_math"]
libm_math = ["dep:libm"]
```

`std` takes the intrinsics; `no-std` routes through `libm`. The other four core-only crates,
`deep_causality_algebra`, `deep_causality_num_complex`, `deep_causality_num_dual` and
`deep_causality_num_rational`, only forward that choice, for example
`no-std = ["deep_causality_num/no-std"]`.

### `alloc` on its own is not a configuration

`alloc` says a heap is available; it says nothing about where float math comes from.
`--no-default-features --features alloc` therefore selects neither `std` nor `no-std`, and
`deep_causality_num` gets no backend. It stops the build with a `compile_error!` that names the
fix:

```
error: deep_causality_num has no float-math backend: enable `std`, or `no-std` for bare metal
(libm). The `alloc` feature of a dependent crate selects the allocator level only and does not
choose a backend, so it cannot be enabled on its own. ...
```

`rustc` still reports the missing float bodies after that line; the first error is the one to read.
The guard sits in `deep_causality_num` because that crate compiles before everything that depends
on it. The guards in `deep_causality_core` and `deep_causality_calculus` state the same rule but
never get the chance to fire.

The crates that never reach `deep_causality_num` (`deep_causality_ast`, `deep_causality_metric`,
`deep_causality_par`, `deep_causality_data_structures` and `ultragraph`) do build with `alloc`
alone. For the rest, pick a platform level, `std` or `no-std`, and add `alloc` only through one of
them.

## Covered crates

### Allocator-free (`core` only)

These need no `#[global_allocator]`, so they can be used inside a deadline-bound loop without
reasoning about allocator behaviour.

| Crate | What it gives you |
|---|---|
| `deep_causality_num` | Float, integer and `Float106` extended precision; the libm routing |
| `deep_causality_algebra` | The trait tower: `Magma` through `Field`, `Real`, `RealField`, `Prob` |
| `deep_causality_num_complex` | Complex scalars over the tower |
| `deep_causality_num_dual` | Dual numbers for forward-mode differentiation |
| `deep_causality_num_rational` | Exact rationals over the integers |

These five are exactly the crates that declare no `alloc` feature.

### Allocator required

| Crate | What it gives you |
|---|---|
| `deep_causality_haft` | HKT witnesses, `Functor`/`Monad`/`Arrow`, `SymMonoidal` |
| `deep_causality_calculus` | Euler and RK4 integrators as causal arrows; see the note below |
| `deep_causality_core` | The causal monad, `CausalFlow`, `EffectLog`, `alternate_value` |
| `deep_causality_metric` | Metric signatures shared by tensor, multivector and physics |
| `deep_causality_ast` | `ConstTree`, the persistent tree behind the HKT impls |
| `deep_causality_data_structures` | Array grids, ring buffers, sliding windows |
| `deep_causality_linear` | Sparse (CSR), dense and bit-packed 𝔽₂ matrices; eliminations, decompositions, conjugate gradient |
| `deep_causality_stats` | Distributions, densities, moments and estimators |
| `deep_causality_tensor` | `CausalTensor`, einsum, SVD, QR, eigen, tensor trains |
| `deep_causality_multivector` | Geometric algebra, `HilbertState`, `CausalMultiVector` |
| `deep_causality_homology` | Chain complexes, boundary operators, homology over a chosen field |
| `deep_causality_fft` | 1-D and N-D FFT, real transforms |
| `deep_causality_rand` | Xoshiro256, Sobol sequences, range sampling |
| `deep_causality_par` | The `MaybeParallel` marker and `scoped_map` |
| `deep_causality_physics` | Physics kernels and quantities, without the `topology` slice |
| `deep_causality_algorithms` | SURD causal decomposition and mRMR feature selection, without BRCD |
| `deep_causality_quantum` | Density matrices, quantum gates, channels, Born read-out |
| `ultragraph` | `CsmGraph`, `DynamicGraph`, traversal, centrality, biconnectivity |

`deep_causality_calculus` is the borderline case. Its own operators allocate nothing: the
integrators step over stack values and `gradient` seeds one coordinate per pass. It is listed here
because it depends on `deep_causality_haft`, and both its `std` and its `no-std` feature enable
`alloc = ["deep_causality_haft/alloc"]`. Differentiation and integration therefore carry the
same allocator requirement as the rest of this table.

## Allocators

A `#[global_allocator]` is a software choice. Any target with RAM can have one, and with
[`embedded-alloc`](https://github.com/rust-embedded/embedded-alloc) it takes about ten lines:

```rust
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// once, at init
unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
```

The attribute has been stable since Rust 1.28. A Cortex-M0+ with 8 KB of RAM can carry a heap, so
the eighteen allocator-dependent crates are not gated on device class.

**The constraint that matters is timing.** Allocation is not bounded-time, so the question is
whether a crate allocates in the deadline path or only at init. A system with a working heap may
still be unable to afford a `Vec::push` inside a 235 µs syndrome round.

Two things soften that. `embedded-alloc` ships `TlsfHeap` as well as `LlffHeap`, and Two-Level
Segregated Fit allocates in bounded O(1), which answers the timing objection though not
fragmentation. And the usual embedded pattern is to allocate during init and never again, which
makes the heap a startup convenience rather than a runtime hazard.

Policy is what rules out a heap. Safety-certified work under DO-178C, IEC 61508 or MISRA
commonly forbids dynamic allocation after init whatever the RAM budget. That is the case where the
five allocator-free crates carry weight: the scalar tower, `Float106` extended precision, complex
numbers, dual numbers and exact rationals, usable with no heap at all. The Euler and RK4
integrators allocate nothing either, but they arrive through `deep_causality_calculus`, which links
`deep_causality_haft` and brings the allocator with it.

`deep_causality_core` allocates per stage. `EffectLog::add_entry` pushes an owned
`String` on every entry, so a hard-deadline loop wants a bounded log rather than the default one.

## What you give up

### Threads

`deep_causality_par` exposes `scoped_map`, which fans out over `std::thread::scope` under its
`parallel` feature. On bare metal the feature is unavailable: `parallel = ["std"]`, and on a target
without an operating system a `compile_error!` in `deep_causality_par` rejects it with the reason.
`scoped_map` still works and still returns results in input order; it runs the map inline.

Everything downstream inherits this. `deep_causality_fft`, `deep_causality_topology`,
`deep_causality_physics` and `deep_causality_algorithms` forward their own `parallel` features to
the same definition, so their parallel paths are host-only.

### Ambient entropy, not randomness

`deep_causality_rand` builds, and the generators work. Two things change.

`ThreadRng` is gone, because there is no thread to be local to. `rng()` returns an owned
`Xoshiro256` instead of a handle to a process-wide one, so the caller holds it.

`Xoshiro256::new()` still exists, but its seed comes from a different place. On `std` it mixes a
fresh `RandomState` with the thread id. On bare metal there is no ambient entropy and no thread
identity, so it mixes a per-call counter into a fixed base: successive calls within one run differ,
and **the sequence repeats identically after every reset**.

When the stream has to differ per boot, seed it yourself:

```rust
let rng = Xoshiro256::from_seed(seed_from_hardware_rng());
```

On an embedded target the entropy belongs to the board, whether that is an RNG peripheral, ADC
noise, or a timer capture. The crate cannot know what the board offers, so it does not pretend to.
Targets without atomic compare-and-swap must also use `from_seed` directly.

### The causal graph

`deep_causality_core` gives you the causal monad: `PropagatingEffect`, `PropagatingProcess`,
`bind`, `CausalFlow`, the `EffectLog`, and `alternate_value` for counterfactual substitution. The
five closed-loop programs in `examples/causal_correction_examples` are built from exactly these.

`ultragraph` is covered too, so the graph store underneath is available: `CsmGraph`,
`DynamicGraph`, traversal, centrality, biconnectivity. `deep_causality` itself, which holds
`CausableGraph` and the reasoning engine, is not covered; it waits on `deep_causality_context` and
`deep_causality_uncertain`. So you get the monad and the graph store, not the causal-graph engine.
For a control loop that monitors, tests an envelope, and intervenes, the monad is the part you
need.

### The topology slice

`deep_causality_physics` and `deep_causality_algorithms` reach bare metal with `topology` off. That
feature carries everything built on `deep_causality_topology`, which is std-only.

In `deep_causality_physics` it holds the gauge theories, the DEC kernels (Maxwell gradient, Lorenz
gauge, Proca, ideal induction, resistive diffusion, relativistic current, Klein–Gordon, heat
diffusion, Föppl–von Kármán strain) and the fluid-dynamics forms such as `VelocityOneForm` and
`SolenoidalField`. The remaining kernels and quantities are unaffected. In
`deep_causality_algorithms` it holds BRCD and the DAG sampler it draws from; SURD and mRMR are
unaffected.

```bash
# host: everything
cargo build -p deep_causality_physics

# bare metal: kernels and quantities, no DEC operators or gauge theories
cargo build -p deep_causality_physics \
  --no-default-features --features no-std --target thumbv7em-none-eabihf
```

### The quantum causal-model slice

`deep_causality_quantum` reaches bare metal with `qcm` off. That feature carries `CausalStructure`,
the Markov freeze check and the C₃-exclusion faithfulness check, and it pulls in `deep_causality`
for the graph. Density matrices, gates, channels and Born read-out are unaffected. The `qpu`
feature (`QpuSampler`, `ShotHistogram`, the shots-to-`Uncertain` bridges, `SimQpu`) pulls in
`deep_causality_uncertain` and so stays on the host as well.

```bash
# host: everything
cargo build -p deep_causality_quantum

# bare metal: gates and states, no causal-structure validation
cargo build -p deep_causality_quantum \
  --no-default-features --features no-std --target thumbv7em-none-eabihf
```

### Files, and everything shaped like a file

`deep_causality_file`, `deep_causality_discovery` and `deep_causality_cfd` read and write files, and
`deep_causality_tempfile` creates them for the test suites. Nothing to reclaim there.

## Not covered

Two things block a crate: its own use of `std`, and any dependency that is itself uncovered. The
second matters more, because it cannot be worked around locally.

**Out of scope by choice.** A controller does not open files, so crates built on `std::fs` and
`std::io` are not candidates and are not counted as gaps.

| Crate | Uncovered dependencies | Own blockers |
|---|---|---|
| `deep_causality_uncertain` | none | `HashMap`, `HashSet` |
| `deep_causality_topology` | none | `HashMap`/`HashSet`, `Arc` and `OnceLock` from `std::sync`, Rayon loops |
| `deep_causality_context_store` | none | `std::sync::Mutex` in the in-memory reference backend; the contract itself needs only `core` |
| `deep_causality_context` | `deep_causality_context_store`, `deep_causality_uncertain` | `HashMap`, `HashSet` |
| `deep_causality` | `deep_causality_context`, `deep_causality_uncertain` | `HashMap`, `std::sync`, `std::time` |
| `deep_causality_ethos` | `deep_causality`, `deep_causality_context` | `HashMap`, `HashSet` |
| `deep_causality_file` | none | `std::fs`, `std::io`; out of scope |
| `deep_causality_tempfile` | none | `std::fs`, `std::io`; out of scope |
| `deep_causality_discovery` | `deep_causality_topology`, through BRCD as well as directly | `std::fs`, `std::io`; out of scope |
| `deep_causality_cfd` | `deep_causality_file`, `deep_causality_topology`, `deep_causality_uncertain` | `std::fs`, `std::io`; out of scope |

Five crates have no uncovered dependency. Leave out the two that handle files, and three can be
worked on today: `deep_causality_uncertain`, `deep_causality_topology` and
`deep_causality_context_store`. Everything else waits on one of them.

`deep_causality_context_store` is the cheapest: its records and traits already need only `core`,
and the `Mutex` sits in the test backend under `utils_test`. With it and `deep_causality_uncertain`
covered, `deep_causality_context` is next, then `deep_causality`, then `deep_causality_ethos`.
`deep_causality_topology` unlocks the topology slices of `deep_causality_physics` and
`deep_causality_algorithms`.

### The `HashMap` question

`std::collections::HashMap` has wrapped [hashbrown](https://github.com/rust-lang/hashbrown) since
Rust 1.36, and hashbrown itself is `no_std` with `alloc`. The map is not the problem. The default
hasher is: `RandomState` seeds SipHash from OS entropy, and that is what bare metal cannot supply.

Where the keys are ordered and the maps are small, `alloc::collections::BTreeMap` needs no hasher at
all. SURD took that route: its maps key on `Vec<usize>` variable sets, and sorted iteration makes
its output order deterministic. `ultragraph` did the same, deduplicating neighbours by
sort-and-dedup over a `Vec` and counting edge multiplicity in a `BTreeMap`.

Replacing sites one by one stops paying off at the forty or so `HashMap` and `HashSet` sites in
`deep_causality_topology`. There the remedy is to name a hasher rather than switch container:

```rust
use hashbrown::{HashMap, HashSet};
use rustc_hash::FxBuildHasher;

pub type FxMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxSet<T> = HashSet<T, FxBuildHasher>;
```

`rustc_hash::FxHashMap` and `FxHashSet` are aliases over *std's* containers and are
therefore std-only. Take `FxBuildHasher` from `rustc-hash` and the container from `hashbrown`.

Unlike the libm split, this needs no `cfg`. std hands you hashbrown anyway, so using it directly on
both paths costs nothing on the host and removes a divergence that would otherwise go untested.

Dropping SipHash is not a security regression in these crates. HashDoS resistance guards against an
adversary choosing keys to force collisions, and the keys here are internal: lattice cells, simplex
and edge indices, node ids, `Vec<usize>` paths. On integer keys FxHash is faster than SipHash, so
the `no_std` path would likely be the quicker one. The exception is
`deep_causality_ethos`, which keys on `String` and `TeloidTag`; if those arrive from configuration,
that is the one place where the default hasher earns its cost.

## Verification

```bash
rustup target add thumbv7em-none-eabihf

make check_no_std   # every crate builds for the target, or is listed as std-only
make check_alloc    # every alloc-only build builds, or stops at the deep_causality_num guard
```

Both checks walk every workspace crate. A crate that declares `no-std` is built for the target in
its own `cargo` call, so a sibling's features cannot mask it. A crate that does not must appear in
`scripts/bare_metal.sh` with the reason it needs `std`; one that is neither fails the check, and so
does a listed crate that has since gained `no-std`. The default target is `thumbv7em-none-eabihf`.
Pass another as the first argument, for example `bash scripts/check_no_std.sh aarch64-unknown-none`.

The alloc check builds with `--keep-going` and passes a failed build only when
`deep_causality_num` is the one crate that failed and the guard message is present. An `alloc`
feature that does not reach a dependency's `alloc` shows up there as a second failing crate.

CI runs both checks in `.github/workflows/rust_no_std.yml` on every pull request and every push to
`main`.
