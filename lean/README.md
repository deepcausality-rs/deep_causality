<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# DeepCausality — Causal Algebra Formalization (Lean 4)

Machine-checked proofs of the DeepCausality core laws, plus the traceability bridge that links
each proof to a Rust witness. This is the **L1** layer of the four-layer verification architecture
in [`../openspec/notes/causal-algebra/Formalization.md`](../openspec/notes/causal-algebra/Formalization.md).

**Separate toolchain.** This is a Lean/`lake` project, *not* part of the Rust workspace. It does
not affect `cargo build`/`cargo test`. Keep it self-contained here under `lean/`.

## Layout

```
lean/
  lean-toolchain                       # pinned Lean version (must match Mathlib rev)
  lakefile.toml                        # project + Mathlib dependency
  DeepCausalityFormal.lean             # root import aggregator
  DeepCausalityFormal/
    Algebra/Monoid.lean                    # Exemplar 1: AddMonoid assoc + identity
    Core/CausalMonad.lean              # Exemplar 2: bind left-identity
  THEOREM_MAP.md                       # Lean theorem id  ↔  Rust witness
  README.md
```

The module tree mirrors the Rust crate tiers: `Algebra` → `Haft` → `Core` → `Topology`.

## Build

```bash
cd lean
elan toolchain install "$(cat lean-toolchain)"   # if not already installed
lake exe cache get                               # download prebuilt Mathlib (fast; optional)
lake build                                       # compile & check all proofs
```

A broken law fails `lake build`. That is the CI gate (`.github/workflows/formalization.yml`).

> **Version note.** `lean-toolchain` and the Mathlib `rev` in `lakefile.toml` are pinned together
> (`v4.15.0`). If `lake build` reports a toolchain/Mathlib mismatch, bump **both** to a matching
> released tag from <https://github.com/leanprover-community/mathlib4/tags>.

## The Lean ↔ Rust bridge

No tool turns a Lean proof into a Rust test. Each property is stated once in Lean (proved) and once
in Rust (checked), sharing an **id** recorded in [`THEOREM_MAP.md`](THEOREM_MAP.md). Rust witnesses:

- `num` / `haft`: law-tests (`cargo test -p <crate>`) and the law-carrying traits themselves.
- `core`: Kani harnesses — `cargo kani --tests -p deep_causality_core`.

## Scope of this skeleton

Exactly two exemplar theorems are proved end-to-end (`algebra.add_monoid.*`, `core.causal_monad.left_id`)
to establish the Lean → Rust → CI pipeline on real theorems. The exemplars are chosen to be
**independent of preconditions P1/P2**. The right-identity / associativity / `LawfulMonad` theorems
depend on the W-invariant (P2) and control-free (P1) fixes and come next — see the work plan in
`../openspec/notes/causal-algebra/Formalization.md` and the "Not yet on the map" section of
`THEOREM_MAP.md`.
