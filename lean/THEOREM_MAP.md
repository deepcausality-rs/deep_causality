<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# THEOREM_MAP — Lean ↔ Rust traceability

This is **the bridge**. There is no tool that converts a Lean proof into a Rust test
(`openspec/notes/causal-algebra/Formalization.md` §3). Instead, each **property statement** is
transcribed once per layer and linked here:

- **Lean** *proves* the statement (deductive, unbounded, higher-order).
- **Rust witness** *checks* the same statement independently:
  - `num` / `haft`: a law-test and/or the trait contract itself (the house style).
  - `core`: a **Kani** harness (bounded model checking — first-order, fixed continuations).
  - `core` (deferred): **Aeneas** extraction — "the code IS the model".

The `THEOREM_MAP:` tag in each Lean file and the matching comment in each Rust witness carry the
same **id**. CI (`.github/workflows/formalization.yml`) fails if an id lacks either side.

## Legend

- **Lean**: `proved` = closed, no `sorry`; `sorry` = stated but unproved; `—` = not yet stated.
- **Kani** / **Test** / **Aeneas**: `✓` present & passing · `partial` · `—` not started · `n/a`.

## Map

| id | statement | Lean | Lean location | Rust witness | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|---|
| `num.add_monoid.assoc` | `(a+b)+c = a+(b+c)` for `AddMonoid` | proved | `Num/Monoid.lean :: add_monoid_assoc` | `deep_causality_num/tests/algebra/monoid_tests.rs :: test_add_monoid_associativity` | ✓ | n/a | — |
| `num.add_monoid.identity` | `a+0 = a ∧ 0+a = a` for `AddMonoid` | proved | `Num/Monoid.lean :: add_monoid_identity` | `deep_causality_num/tests/algebra/monoid_tests.rs :: test_add_monoid_identity` | ✓ | n/a | — |
| `core.causal_monad.left_id` | `pure a >>= f = f a` | proved | `Core/CausalMonad.lean :: bind_left_id` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_left_identity` | n/a | ✓ | — |

## Not yet on the map (blocked / scaling — see Formalization.md work plan)

| id (planned) | statement | blocked on |
|---|---|---|
| `core.causal_monad.right_id` | `m >>= pure = m` | **P2** (W-invariant: `error = Some ⇒ value = None`) |
| `core.causal_monad.assoc` | `(m >>= f) >>= g = m >>= (λx. f x >>= g)` | P1 (remove `RelayTo`/`Map`) + P2 |
| `core.causal_monad.lawful` | `LawfulMonad` instance | P1 + P2 |
| `haft.functor.*`, `haft.monad.*` | functor/monad laws | scaling (Haft layer) |
