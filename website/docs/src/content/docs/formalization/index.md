---
title: Formalization
description: The DeepCausality core laws, machine-checked in Lean 4 and each bound to a Rust witness that checks the same statement.
sidebar:
  order: 0
---

The algebraic laws DeepCausality rests on are machine-checked in [Lean 4](https://lean-lang.org/) against [Mathlib](https://github.com/leanprover-community/mathlib4). Each proof is bound to a **Rust witness** that checks the same statement in the codebase. This section publishes that map.

It is the **L1** layer of a four-layer verification architecture. The Lean project lives under [`lean/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean) on a separate toolchain, so it never touches `cargo build`. A broken law fails `lake build`, which is a CI gate.

## The Lean ↔ Rust bridge

No tool turns a Lean proof into a Rust test. Each property is stated twice and linked by a shared **id**:

- **Lean proves it.** Deductive, unbounded, higher-order.
- **A Rust witness checks it.** A law-test for the `num`, `algebra`, and `haft` layers, or a [Kani](https://model-checking.github.io/kani/) bounded-model-checking harness for the `core` layer.

CI (`formalization.yml`) fails if an id is missing either side. The authoritative source is [`lean/THEOREM_MAP.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/THEOREM_MAP.md); the pages here render it for the web.

## How to read these tables

Each row is one property.

- **id**: the shared identifier, present in the Lean file (the `THEOREM_MAP:` tag) and in the Rust witness.
- **statement**: the law, in math notation.
- **Lean proof**: the file and theorem name under [`lean/DeepCausalityFormal/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal). Every row on these pages is `proved`, closed with no `sorry`.
- **Rust witness**: the test or trait contract that checks the same statement.
- **Test**: `✓` present and passing.
- **Kani**: a bounded model check is present. Only the **Core** layer carries this column; the other layers are checked by law-tests.

## The layers

- **[Num](/formalization/num/)**: identity, integer ring laws, cast round-trips, and the `Float106` real-field model.
- **[Algebra](/formalization/algebra/)**: the trait-tower laws from monoid through group, ring, field, module, and division algebra, plus conjugation, norm, and the verdict lattice.
- **[Haft](/formalization/haft/)**: the higher-kinded functional laws — functor, applicative, monad, arrow, free monad, monoidal, traversable — behind [Higher-Kinded Types](/concepts/hkt/).
- **[Core](/formalization/core/)**: the causal-monad, causal-arrow, causaloid-fixpoint, verdict, and graph-fold laws behind the [Effect Propagation Process](/concepts/effect-propagation-process/).
- **[Complex & Dual](/formalization/complex-dual/)**: `ℂ` field/conjugation/norm laws, `ℍ` division-ring laws with a non-commutativity witness, and the dual numbers with the forward-mode Leibniz rule.
- **[Topology](/formalization/topology/)**: Riemann curvature symmetries.
- **[Quantum](/formalization/quantum/)**: the partial-trace / Choi foundation, headlined by the B1 counterexample — unconditional partial-trace preservation is proved false, the conditional boundary version holds.

## Scope

L1 is deliberately bounded, and the map is honest about its edges.

- The `Float106` double-double **bit-exact error bounds** are `[open]`. The Lean model proves the real-field laws; the empirical bounds are covered by Rust tests.
- **Octonions** sit outside L1, because they are not in Mathlib. Rust tests cover them.
- The **Quantum** layer builds partial-trace and Choi structure from first principles, because the pinned Mathlib has neither. Its headline result: the unconditional `partial_trace_preservation` is *false*, with a witnessed counterexample. A conditional boundary version does hold. That tree is exempt from the `sorry` CI gate while the foundation grows.

## Related reading

- [`lean/README.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/README.md): how to build and check the proofs locally.
- [The Axiom](/concepts/axiom/) and [Uniform Math](/concepts/uniform-math/): the concepts these proofs stand under.
