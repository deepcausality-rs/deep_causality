---
title: Haft
description: Higher-kinded functional laws (functor, applicative, monad, arrow, free monad, monoidal, traversable), machine-checked in Lean and bound to Rust law-tests.
sidebar:
  order: 3
draft: true
---

:::caution[Good first issue: this page is a template to fill in]
This page is intentionally incomplete and is a **[good first issue](https://github.com/deepcausality-rs/deep_causality/issues)** for new contributors. It is a `draft`, so it does not appear on the public site until finished.

**Your task:** transcribe the `haft.*` rows from the `### Haft layer` table in [`lean/THEOREM_MAP.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/THEOREM_MAP.md) into the table below, using the completed **[Topology](/formalization/topology/)** page as your template. Haft has the same shape, with no per-row Rust-witness column.

- Keep the column shape `| id | statement | Lean proof | Test |`. Drop the source's `Lean` (proved) and `Kani` columns.
- This is the largest layer, near 45 laws. Work through it in passes; a partial table is still useful.
- Lean files sit under `Haft/` (`Functor.lean`, `Applicative.lean`, `Arrow.lean`, and so on). Rust witnesses live in `deep_causality_haft/tests/formalization_lean/`, which mirrors the Lean tree one-to-one (`Haft/Functor.lean` maps to `functor_tests.rs`), with one `#[test]` per id named `test_<id>`.
- **When done:** delete this notice, remove `draft: true` from the frontmatter above, and add a link to this page in the [Formalization index](/formalization/) under "The layers".

See [CONTRIBUTING.md](https://github.com/deepcausality-rs/deep_causality/blob/main/CONTRIBUTING.md) for the workflow.
:::

The Higher-Order Abstract Functional Traits: functor, applicative, monad, comonad, bifunctor, profunctor, arrow, free monad, category, Kleisli, symmetric-monoidal, foldable, traversable, adjunction, and the effect system. These are the type-level laws behind [Higher-Kinded Types](/concepts/hkt/).

Every row is `proved` in Lean. Witnesses live in `deep_causality_haft/tests/formalization_lean/`.

| id | statement | Lean proof | Test |
|---|---|---|---|
| | | | |
