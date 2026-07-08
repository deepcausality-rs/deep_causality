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

## Scope

The numeric layers are formalized in full against Mathlib carriers, each theorem bound to a Rust
witness (see `THEOREM_MAP.md`):

- `Num` — identity (`Zero`/`One`), integer ring laws, cast round-trips, and the `Float106`
  real-field model. The bit-exact double-double error bounds are **[open]** (out of L1 scope; the
  Rust double-double tests cover them empirically).
- `Algebra` — the trait tower: monoid / commutative-monoid / semilattice, group / abelian-group,
  ring / commutative-ring, field / real-field, module / algebra, division algebra, conjugation
  (`star`), and norm multiplicativity.
- `Complex` — `ℂ` is a field with involutive conjugation and multiplicative norm; `ℍ` is a division
  ring with multiplicative norm and a non-commutativity witness. Octonions are out of L1 scope (not
  in Mathlib) and remain covered by the Rust tests.
- `Dual` — `R[ε]` is a commutative ring, `ε² = 0`, the real projection is a ring map, and the
  tangent part satisfies the Leibniz product rule (forward-mode AD).

The `Core` layer proves the causal-monad laws under preconditions P1/P2 (`core.causal_monad.*`); the
remaining `LawfulMonad` / parity work is tracked in
`../openspec/notes/causal-algebra/Formalization.md` and the "Not yet on the map" section of
`THEOREM_MAP.md`.
