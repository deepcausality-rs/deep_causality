# Design: transcribe-lean-num-complex-dual-verification

## Context

Second page in the formalization-docs series, after the archived `transcribe-lean-algebra-verification`. The `complex-dual.md` stub carries its own transcription instructions in a `:::caution` block and already has correct frontmatter (`sidebar: order: 5`) and body prose framing the layer: ℂ as a field with involutive conjugation and multiplicative norm, ℍ as a division ring with a non-commutativity witness, and the dual numbers `R[ε]` with `ε² = 0` and the forward-mode Leibniz product rule.

All 15 rows sit in one source table (`### Num / Algebra / Complex / Dual layers` in `lean/THEOREM_MAP.md`): 5 `complex.*`, 4 `quaternion.*`, 6 `dual.*`. Both sides verified present: theorems in `lean/DeepCausalityFormal/Complex/Complex.lean`, `Complex/Quaternion.lean`, and `Dual/Dual.lean`; tests in `deep_causality_num_complex/tests/formalization_lean/{complex,quaternion}_tests.rs` and `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs`.

The structural difference from the Algebra page: sources span **two Lean directories** and **two witness crates**, so "paths relative to the directory linked above" needs adaptation.

## Goals / Non-Goals

**Goals:**

- Publish a complete, accurate `formalization/complex-dual.md` in the established house style.
- Link it from the formalization index.

**Non-Goals:**

- No changes to `THEOREM_MAP.md`, the Lean proofs, the Rust witnesses, or CI.
- No changes to the remaining draft pages (`haft.md`, `quantum.md`).
- Octonions stay out of scope — they are outside L1 (not in Mathlib), as the index's Scope section already documents.

## Decisions

1. **Column shape `| id | statement | Lean proof | Rust witness | Test |`** — per the stub's instructions; source's proved-status and Kani columns dropped (no Kani harnesses in this layer; prose states every row is `proved`).

2. **Lean cells are directory-qualified relative to `lean/DeepCausalityFormal/`** — `Complex/Complex.lean :: complex_field_mul_inv`, `Dual/Dual.lean :: dual_leibniz`. Rationale: two directories feed one page, so bare filenames would leave `Complex.lean` vs `Dual.lean` ambiguous against a single linked directory; qualifying with one path segment keeps every cell self-locating. The intro links `lean/DeepCausalityFormal/` (Complex and Dual subdirectories named in prose). Alternative considered: bare filenames with two directory links in prose — rejected as harder to trace per-row.

3. **Rust cells are bare filenames** — `complex_tests.rs :: test_...`, `dual_tests.rs :: test_...`. The three test filenames are unique across the two crates, and the prose names which crate carries which file (the stub already does this). No ambiguity, matches the `num.md` cell shape.

4. **Row order: complex → quaternion → dual, preserving source order within each family** — this is already the source-table order and follows the construction ladder (ℂ, then ℍ, then `R[ε]`). No reordering needed, unlike the Algebra page.

5. **Keep the stub's body prose, extend per `num.md`** — the existing paragraph framing the negative results (`quaternion.noncomm`, `dual.not_field.zero_divisor`) is accurate and worth keeping; prepend the law count and source links per the house pattern. Note the witness-name asymmetry preserved as-is from the map: `dual.*` tests drop the `dual_` prefix (`test_mul_comm`, not `test_dual_mul_comm`) and `norm_sq` renders as `norm_sqr` in two test names — the map is the source of truth, the page must not "fix" these.

6. **Index update is minimal** — one bullet added under "The layers" after Core/Topology ordering considerations: sidebar order is 5 (num=1, algebra=2, haft=3, core=4, complex-dual=5, topology=6), so the bullet goes between Core and Topology; the "being documented" sentence shrinks to Haft and Quantum.

## Risks / Trade-offs

- [Transcription typo breaks the doc↔map correspondence] → Same mechanical verification as the Algebra page at apply time: page ids diffed against the map; every theorem and test name checked in the exact named file.
- [Directory-qualified Lean cells diverge from `num.md`/`algebra.md` cell style] → Accepted: consistency with per-row traceability beats cosmetic uniformity, and the prose states the convention. If the team prefers bare filenames later, it is a mechanical edit.
- [Site build could reject the page] → Run `npm run build` in `website/docs/` before finishing; content is plain frontmatter + markdown table.

## Open Questions

None — the stub's embedded instructions plus the Algebra-page precedent resolve all format questions.
