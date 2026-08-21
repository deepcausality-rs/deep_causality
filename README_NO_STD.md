<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Building DeepCausality without `std`

Seventeen of the twenty-seven active crates build on bare metal. That covers the whole numeric
tower, the causal monad, tensors, multivectors, sparse matrices, FFT, and the quantum layer. What
stays behind is the graph engine, uncertainty, topology, and everything that touches files.

Verified against `aarch64-unknown-none` on 2026-08-21. Every crate listed as covered was compiled
for that target; nothing here is inferred from a host build with `std` switched off.

## Quick start

```bash
rustup target add aarch64-unknown-none

cargo build -p deep_causality_core \
  --no-default-features --features no-std \
  --target aarch64-unknown-none
```

Five crates need only `core`. The other twelve need an allocator. See
[Allocators](#allocators) for what that does and does not rule out; the short answer is that it
rules out very little.

## The three feature levels

Every covered crate declares the same three levels:

```toml
[features]
default = ["std"]
std     = ["alloc", "<dep>/std", ...]
alloc   = []
no-std  = ["alloc", "<dep>/no-std", ...]
```

`std` implies `alloc`; `no-std` selects `alloc` without `std`. Dependencies are declared with
`default-features = false` so that `--no-default-features` actually reaches them. Without that,
Cargo hands a dependency its own defaults and `std` returns through the back door, which compiles
fine on a host and fails only when you cross-compile.

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
| `deep_causality_calculus` | Euler and RK4 integrators as causal arrows |

### Allocator required

| Crate | What it gives you |
|---|---|
| `deep_causality_haft` | HKT witnesses, `Functor`/`Monad`/`Arrow`, `SymMonoidal` |
| `deep_causality_core` | The causal monad, `CausalFlow`, `EffectLog`, `alternate_value` |
| `deep_causality_metric` | Metric signatures shared by tensor, multivector and physics |
| `deep_causality_ast` | `ConstTree`, the persistent tree behind the HKT impls |
| `deep_causality_data_structures` | Array grids, ring buffers, sliding windows |
| `deep_causality_tensor` | `CausalTensor`, einsum, SVD, QR, eigen, tensor trains |
| `deep_causality_multivector` | Geometric algebra, `HilbertState`, `CausalMultiVector` |
| `deep_causality_sparse` | CSR matrices, conjugate-gradient solver |
| `deep_causality_fft` | 1-D and N-D FFT, real transforms |
| `deep_causality_rand` | Xoshiro256, Sobol sequences, normal and uniform distributions |
| `deep_causality_par` | The `MaybeParallel` marker and `scoped_map` |
| `deep_causality_quantum` | Density matrices, quantum gates, channels, Born read-out |

## Allocators

A `#[global_allocator]` is a software choice, not a hardware capability. Any target with RAM can
have one, and with [`embedded-alloc`](https://github.com/rust-embedded/embedded-alloc) it takes
about ten lines:

```rust
use embedded_alloc::LlffHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

// once, at init
unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
```

The attribute has been stable since Rust 1.28. A Cortex-M0+ with 8 KB of RAM can carry a heap, so
the twelve allocator-dependent crates are not gated on device class.

**The constraint that matters is timing, not availability.** Allocation is not bounded-time, so the
question is whether a given crate allocates in the deadline path or only at init. A system can have
a perfectly good heap and still be unable to afford a `Vec::push` inside a 235 µs syndrome round.

Two things soften that. `embedded-alloc` ships `TlsfHeap` as well as `LlffHeap`, and Two-Level
Segregated Fit allocates in bounded O(1), which answers the timing objection though not
fragmentation. And the usual embedded pattern is to allocate during init and never again, which
makes the heap a startup convenience rather than a runtime hazard.

What genuinely rules out a heap is policy. Safety-certified work under DO-178C, IEC 61508 or MISRA
commonly forbids dynamic allocation after init whatever the RAM budget. That is the case where the
five allocator-free crates carry weight: the scalar tower and the RK4 and Euler integrators, usable
with no heap at all.

Note that `deep_causality_core` allocates per stage. `EffectLog::add_entry` pushes an owned
`String` on every entry, so a hard-deadline loop wants a bounded log rather than the default one.

## What you give up

### Threads

`deep_causality_par` exposes `scoped_map`, which fans out over `std::thread::scope` under its
`parallel` feature. On bare metal the feature is unavailable: `parallel = ["std"]`, so `no-std`
combined with `parallel` will not resolve. `scoped_map` still works and still returns results in
input order; it runs the map inline.

Everything downstream inherits this. `deep_causality_fft` and `deep_causality_topology` forward
their own `parallel` features to the same definition, so their parallel paths are host-only.

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

`deep_causality` itself, which holds `CausableGraph` and the hypergraph reasoning engine, is not
covered. So you get the monad and not the graph. For a control loop that monitors, tests an
envelope, and intervenes, the monad is the part you need.

### The quantum causal-model slice

`deep_causality_quantum` reaches bare metal with `qcm` off. That feature carries `CausalStructure`,
the Markov freeze check and the C₃-exclusion faithfulness check, and it pulls in `deep_causality`
for the graph. Density matrices, gates, channels, Born read-out and the QPU seam are all
unaffected.

```bash
# host: everything
cargo build -p deep_causality_quantum

# bare metal: gates and states, no causal-structure validation
cargo build -p deep_causality_quantum \
  --no-default-features --features no-std --target aarch64-unknown-none
```

### Files, and everything shaped like a file

`deep_causality_file`, `deep_causality_discovery` and `deep_causality_cfd` read and write files.
Nothing to reclaim there.

## Not covered

Two things block a crate: its own use of `std`, and any dependency that is itself uncovered. The
second matters more, because it cannot be worked around locally.

**Out of scope by choice.** A controller does not open files, so crates built on `std::fs` and
`std::io` are not candidates and are not counted as gaps.

| Crate | Uncovered dependencies | Own blockers |
|---|---|---|
| `ultragraph` | none | `HashMap`, `HashSet`, `VecDeque`; 5 sites |
| `deep_causality_uncertain` | none | `HashMap`, `Arc`, `std::sync::atomic`; 19 sites |
| `deep_causality_topology` | none | `HashMap`/`HashSet` 67 sites, `OnceLock` (one lazy field) |
| `deep_causality_physics` | `deep_causality_topology` | one `HashMap` in the MHD kernel |
| `deep_causality_algorithms` | `deep_causality_topology` | `HashMap`/`HashSet`; 90 sites |
| `deep_causality` | `deep_causality_uncertain`, `ultragraph` | `HashMap`, `Arc`, `std::time`; 67 sites |
| `deep_causality_ethos` | `deep_causality`, `ultragraph` | `HashMap`/`HashSet`; 28 sites |
| `deep_causality_file` | none | `std::fs`, `std::io`; out of scope |
| `deep_causality_discovery` | `deep_causality_algorithms`, `deep_causality_topology` | `std::fs`, `std::io`; out of scope |
| `deep_causality_cfd` | `deep_causality_file`, `deep_causality_physics`, `deep_causality_topology`, `deep_causality_uncertain` | `std::fs`, `std::io`; out of scope |

Only four crates have no uncovered dependency, so only four can be worked on today: `ultragraph`,
`deep_causality_uncertain`, `deep_causality_topology`, and `deep_causality_file`. Everything else
waits on one of them.

`ultragraph` is the smallest at five sites. `deep_causality_topology` is the one that unblocks the
most: `deep_causality_physics` needs only it, and `deep_causality_algorithms` likewise.

### The `HashMap` question

`std::collections::HashMap` has wrapped [hashbrown](https://github.com/rust-lang/hashbrown) since
Rust 1.36, and hashbrown itself is `no_std` with `alloc`. The map is not the problem. The default
hasher is: `RandomState` seeds SipHash from OS entropy, and that is what bare metal cannot supply.

So the port names a hasher instead of switching container:

```rust
use hashbrown::{HashMap, HashSet};
use rustc_hash::FxBuildHasher;

pub type FxMap<K, V> = HashMap<K, V, FxBuildHasher>;
pub type FxSet<T> = HashSet<T, FxBuildHasher>;
```

Note that `rustc_hash::FxHashMap` and `FxHashSet` are aliases over *std's* containers and are
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
for c in num algebra haft num_complex num_dual metric ast core calculus \
         data_structures par fft tensor multivector sparse rand quantum; do
  cargo build -p deep_causality_$c \
    --no-default-features --features no-std \
    --target aarch64-unknown-none || echo "FAIL: $c"
done
```

All seventeen build clean. `cargo build --workspace` reports no warnings and no errors, and
`cargo clippy` is clean on both the `std` and `no-std` paths.
