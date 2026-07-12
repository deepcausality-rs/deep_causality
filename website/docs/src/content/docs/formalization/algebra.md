---
title: Algebra
description: Abstract-algebra trait-tower laws (group, ring, field, module, algebra, conjugation, norm), machine-checked in Lean and bound to Rust law-tests.
sidebar:
  order: 2
draft: true
---

:::caution[Good first issue: this page is a template to fill in]
This page is intentionally incomplete and is a **[good first issue](https://github.com/deepcausality-rs/deep_causality/issues)** for new contributors. It is a `draft`, so it does not appear on the public site until finished.

**Your task:** transcribe the `algebra.*` rows from [`lean/THEOREM_MAP.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/THEOREM_MAP.md) into the table below, using the completed **[Num](/formalization/num/)** and **[Core](/formalization/core/)** pages as your template.

- The `algebra.*` rows sit in **two** tables in `THEOREM_MAP.md`: the `## Map` table (monoid, commutative-monoid, semilattice, verdict) and the `### Num / Algebra / Complex / Dual` table (group, ring, field, module, algebra, division-algebra, conjugation, norm).
- Keep the column shape `| id | statement | Lean proof | Rust witness | Test |`. Drop the source's `Lean` (proved) column and its `Kani` column, since this layer has no Kani harnesses.
- Rust witnesses live in `deep_causality_algebra/tests/formalization_lean/`. Show cells as `file.lean :: theorem` and `file_tests.rs :: test`, relative to that directory.
- **When done:** delete this notice, remove `draft: true` from the frontmatter above, and add a link to this page in the [Formalization index](/formalization/) under "The layers".

See [CONTRIBUTING.md](https://github.com/deepcausality-rs/deep_causality/blob/main/CONTRIBUTING.md) for the workflow.
:::

Abstract-algebra laws for the trait tower: monoid and commutative-monoid, group and abelian-group, ring and commutative-ring, field and real-field, module and algebra, division algebra, conjugation (`star`), and norm multiplicativity. These are the laws the [Uniform Math](/concepts/uniform-math/) surface relies on.

Every row is `proved` in Lean. Witnesses live in `deep_causality_algebra/tests/formalization_lean/`.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| | | | | |
