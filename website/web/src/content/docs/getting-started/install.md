---
title: Install
description: Add DeepCausality to a Rust project.
section: getting-started
order: 1
---

DeepCausality is a workspace of independently published crates. Pick the ones you need; you do not have to take all twenty.

## Prerequisites

You need a recent stable Rust toolchain. Anything from 1.78 onward will work:

```bash
rustup update stable
rustc --version
```

A 2024-edition Rust is recommended but not required.

## The umbrella crate

Most users start here:

```bash
cargo add deep_causality
```

`deep_causality` re-exports the user-facing types: `Causaloid`, `Context`, `CausaloidGraph`, the propagating-effect machinery, and the surrounding aliases. It depends on `deep_causality_core`, `deep_causality_haft`, and `ultragraph` underneath, so a single `cargo add` is enough for a first project.

## Specialized crates

Reach for one of these when the umbrella is broader than you need:

- `deep_causality_algorithms` — MRMR feature selection, SURD information decomposition.
- `deep_causality_data_structures` — sliding-window and grid-array containers, useful on stream workloads.
- `deep_causality_discovery` — the Causal Discovery Language (CDL) pipeline.
- `deep_causality_ethos` — the Effect Ethos and its Teloids.
- `deep_causality_topology`, `deep_causality_physics`, `deep_causality_multivector` — math and physics primitives.
- `deep_causality_tensor`, `deep_causality_sparse` — numerical containers.
- `deep_causality_uncertain` — a first-order type for uncertain values.

Each crate stands on its own and links back to the others through clear seams. The [reference section](/docs/reference/deep_causality/) carries one page per crate.

## Verify the install

A two-line program is enough to prove the install is wired:

```rust
fn main() {
    println!(
        "deep_causality version: {}",
        deep_causality::VERSION,
    );
}
```

```bash
cargo run --release
```

If the version prints, you are ready to write a Causaloid. The [next page](/docs/getting-started/hello-causaloid/) does exactly that.

## A note on the docs.rs reference

Every crate ships rustdoc on [docs.rs](https://docs.rs). The pages in this site's reference section are short overviews: what the crate is for and when you should reach for it. The exhaustive API stays on docs.rs.
