---
title: Quantum
description: Quantum partial-trace, Choi, and channel laws built from first principles in Lean, including the partial-trace preservation counterexample, bound to Rust witnesses.
sidebar:
  order: 7
draft: true
---

:::caution[Good first issue: this page is a template to fill in]
This page is intentionally incomplete and is a **[good first issue](https://github.com/deepcausality-rs/deep_causality/issues)** for new contributors. It is a `draft`, so it does not appear on the public site until finished.

**Your task:** transcribe the `quantum.*` rows from the `## Quantum` section of [`lean/THEOREM_MAP.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/THEOREM_MAP.md) into the table below, using the completed **[Num](/formalization/num/)** and **[Core](/formalization/core/)** pages as your template.

- Keep the column shape `| id | statement | Lean proof | Rust witness | Test |`. This layer has no `Kani` column.
- Lean files sit under `Quantum/` (`PartialTrace.lean`, `Choi.lean`, `PartialTraceCounterexample.lean`). Rust witnesses live in `deep_causality_quantum/tests/kernels/` (`operator_linalg_tests.rs`, `channel_tests.rs`).
- Read the `## Quantum` prose in `THEOREM_MAP.md` first. The headline is that the unconditional `partial_trace_preservation` is `false` (`partial_trace_nonpreservation`), while the conditional boundary version holds. Keep that framing on the page.
- **When done:** delete this notice, remove `draft: true` from the frontmatter above, and add a link to this page in the [Formalization index](/formalization/) under "The layers".

See [CONTRIBUTING.md](https://github.com/deepcausality-rs/deep_causality/blob/main/CONTRIBUTING.md) for the workflow.
:::

Partial-trace linearity and product laws, the Choi correspondence, and the B1 preservation result, built from first principles because the pinned Mathlib carries neither partial trace nor a Choi layer. The `Quantum/` tree is exempt from the `sorry` CI gate while this foundation grows.

Every row below is `proved` in Lean. Witnesses live in `deep_causality_quantum/tests/kernels/`.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| | | | | |
