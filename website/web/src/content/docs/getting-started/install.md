---
title: Install
description: Add DeepCausality to a Rust project.
section: getting-started
order: 1
---

DeepCausality is a workspace of twenty independently published crates. Pick what you need.

## Prerequisites

DeepCausality targets the Rust 2024 edition:

```bash
rustup update stable
rustc --version
```

## The umbrella crate

Most users start here:

```bash
cargo add deep_causality
```

The [`deep_causality`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) crate re-exports the user-facing types: `Causaloid`, `CausaloidGraph`, `Context`, the propagating-effect machinery, and the surrounding aliases. It pulls in [`deep_causality_core`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core), [`deep_causality_haft`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_haft), [`deep_causality_uncertain`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain), [`deep_causality_ast`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_ast), [`deep_causality_data_structures`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_data_structures), and [`ultragraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph) transitively, so a single `cargo add` is enough for a first project.

## Specialized crates

Reach for one of these when the umbrella is broader than you need:

- [`deep_causality_algorithms`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_algorithms): MRMR feature selection, SURD information decomposition.
- [`deep_causality_data_structures`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_data_structures): sliding-window and grid-array containers, useful on stream workloads.
- [`deep_causality_discovery`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_discovery): the Causal Discovery Language (CDL) pipeline.
- [`deep_causality_ethos`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_ethos): the Effect Ethos and its Teloids.
- [`deep_causality_topology`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_topology), [`deep_causality_physics`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_physics), [`deep_causality_multivector`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_multivector): math and physics primitives.
- [`deep_causality_tensor`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_tensor), [`deep_causality_sparse`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_sparse): numerical containers.
- [`deep_causality_uncertain`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_uncertain): a first-order type for uncertain values.

Each crate stands on its own and links back to the others through clear seams. The full API reference is on [docs.rs](https://docs.rs/deep_causality), one page per crate.

## Verify the install

A tiny program that lifts a value into a `PropagatingEffect` is enough to prove the install is wired:

```rust
use deep_causality::PropagatingEffect;

fn main() {
    let effect: PropagatingEffect<f64> = PropagatingEffect::pure(42.0);
    println!("ok: {:?}", effect.value);
}
```

```bash
cargo run
```

If you see `ok: Value(42.0)`, the install is good. The [next page](/docs/getting-started/hello-causal-monad/) walks through what `pure` and `bind` actually do.

## A note on the docs.rs reference

Every crate ships rustdoc on [docs.rs](https://docs.rs). The pages in this site's reference section are short overviews: what the crate is for and when you should reach for it. The exhaustive API stays on docs.rs.
