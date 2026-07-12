---
title: Complex & Dual
description: Complex, quaternion, and dual-number laws (field, division ring, conjugation, norm, Leibniz product rule), machine-checked in Lean and bound to Rust law-tests.
sidebar:
  order: 5
draft: true
---

:::caution[Good first issue: this page is a template to fill in]
This page is intentionally incomplete and is a **[good first issue](https://github.com/deepcausality-rs/deep_causality/issues)** for new contributors. It is a `draft`, so it does not appear on the public site until finished.

**Your task:** transcribe the `complex.*`, `quaternion.*`, and `dual.*` rows from the `### Num / Algebra / Complex / Dual` table in [`lean/THEOREM_MAP.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/THEOREM_MAP.md) into the table below, using the completed **[Num](/formalization/num/)** page as your template.

- Keep the column shape `| id | statement | Lean proof | Rust witness | Test |`. Drop the source's `Lean` (proved) and `Kani` columns.
- Lean files: `Complex/Complex.lean`, `Complex/Quaternion.lean`, `Dual/Dual.lean`. Rust witnesses live in `deep_causality_num_complex/tests/formalization_lean/` (complex, quaternion) and `deep_causality_num_dual/tests/formalization_lean/` (dual).
- **When done:** delete this notice, remove `draft: true` from the frontmatter above, and add a link to this page in the [Formalization index](/formalization/) under "The layers".

See [CONTRIBUTING.md](https://github.com/deepcausality-rs/deep_causality/blob/main/CONTRIBUTING.md) for the workflow.
:::

Number-type laws layered on the algebra tower: `ℂ` as a field with involutive conjugation and multiplicative norm, `ℍ` as a division ring with a non-commutativity witness, and the dual numbers `R[ε]` with `ε² = 0` and the forward-mode Leibniz product rule.

Every row is `proved` in Lean. Witnesses live in `deep_causality_num_complex/` and `deep_causality_num_dual/`.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| | | | | |
