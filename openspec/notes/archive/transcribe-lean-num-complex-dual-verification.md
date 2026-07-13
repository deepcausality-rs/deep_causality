# Note: transcribe the Complex & Dual Lean verification to the website

## What

Fill in the draft website page `website/docs/src/content/docs/formalization/complex-dual.md` — the second of the "good first issue" formalization stubs — by transcribing the 15 `complex.*`, `quaternion.*`, and `dual.*` rows from `lean/THEOREM_MAP.md`, then publish it (remove `draft: true`) and link it from the formalization index. Follow-up to the completed `transcribe-lean-algebra-verification` change (archived 2026-07-12), using the same workflow and house style.

## Context

- The stub's `:::caution` block contains the transcription instructions: use `num.md` as the template, column shape `| id | statement | Lean proof | Rust witness | Test |`, drop the source's proved-status and Kani columns.
- All 15 rows sit in the `### Num / Algebra / Complex / Dual layers` table of `THEOREM_MAP.md`: 5 `complex.*` (field inverse, involutive conjugation, conjugation homomorphism, normSq and norm multiplicativity), 4 `quaternion.*` (division-ring inverse, normSq multiplicativity, anti-homomorphic `star`, non-commutativity witness), 6 `dual.*` (commutative ring, `ε² = 0`, real-projection homomorphism ×2, Leibniz product rule, nonzero zero-divisor / not-a-field).
- Verified present on both sides: Lean theorems in `lean/DeepCausalityFormal/Complex/Complex.lean`, `Complex/Quaternion.lean`, `Dual/Dual.lean`; Rust tests in `deep_causality_num_complex/tests/formalization_lean/` (complex_tests.rs, quaternion_tests.rs) and `deep_causality_num_dual/tests/formalization_lean/` (dual_tests.rs).
- Difference from the Algebra page: sources span **two** Lean directories and **two** witness crates, so the "relative to the directories above" convention needs a small adaptation (directory-qualified Lean cells).
- Two rows are deliberate negative results (`quaternion.noncomm`, `dual.not_field.zero_divisor`) — the stub's existing body prose already frames them; keep that framing.

## Constraints

- Documentation-only: no Rust, Lean, or CI changes. `THEOREM_MAP.md` is the CI-enforced source of truth and is read, not modified.
- Transcription must survive the same mechanical cross-check used for the Algebra page: page ids set-equal to the map, every theorem/test name verified in the exact named file.
- Site must build (`npm run build` in `website/docs/`) with the `/formalization/complex-dual/` route present.
- Index update: add the layer bullet and remove "Complex & Dual" from the "being documented" sentence (leaving Haft and Quantum).
