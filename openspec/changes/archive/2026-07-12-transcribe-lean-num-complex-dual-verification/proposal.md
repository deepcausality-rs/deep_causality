# Proposal: transcribe-lean-num-complex-dual-verification

## Why

The website's formalization section still hides the Complex & Dual layer: `website/docs/src/content/docs/formalization/complex-dual.md` is a `draft: true` template stub with an empty table, while the 15 `complex.*`, `quaternion.*`, and `dual.*` laws are already proved in Lean and witnessed by Rust law-tests in `deep_causality_num_complex` and `deep_causality_num_dual`. This is the direct follow-up to the completed `transcribe-lean-algebra-verification` change (archived 2026-07-12), continuing the same documentation effort; the originating note is `openspec/notes/archive/transcribe-lean-num-complex-dual-verification.md`.

## What Changes

- Fill the `formalization/complex-dual.md` table with all 15 `complex.*`, `quaternion.*`, and `dual.*` rows from the `### Num / Algebra / Complex / Dual layers` table of `lean/THEOREM_MAP.md` (ℂ field/conjugation/norm laws, ℍ division-ring/conjugation/non-commutativity laws, `R[ε]` ring/projection/Leibniz/zero-divisor laws).
- Complete the page intro prose per the `num.md` pattern (law count, GitHub links to the Lean sources, witness crate directories), keeping the stub's existing framing of the two negative results (ℍ non-commutativity, `R[ε]` not a field).
- Remove the "good first issue" caution block and the `draft: true` frontmatter flag so the page publishes.
- Update the formalization index (`formalization/index.md`): add a Complex & Dual bullet under "The layers" and remove it from the "being documented" sentence (leaving Haft and Quantum).

No Rust, Lean, or CI changes — documentation-only transcription from the CI-enforced source of truth.

## Capabilities

### New Capabilities

- `complex-dual-formalization-docs`: the website formalization page for the Complex & Dual layer — a complete, accurate rendering of the `complex.*`, `quaternion.*`, and `dual.*` rows of `lean/THEOREM_MAP.md`, published (non-draft) and linked from the formalization index.

### Modified Capabilities

None. The Lean proofs, Rust witnesses, and existing capabilities are untouched.

## Impact

- `website/docs/src/content/docs/formalization/complex-dual.md` — table filled, prose completed, draft flag removed.
- `website/docs/src/content/docs/formalization/index.md` — "The layers" list gains a Complex & Dual entry; "being documented" sentence updated.
- Source of truth read (not modified): `lean/THEOREM_MAP.md`, `lean/DeepCausalityFormal/Complex/{Complex,Quaternion}.lean`, `lean/DeepCausalityFormal/Dual/Dual.lean`, `deep_causality_num_complex/tests/formalization_lean/*.rs`, `deep_causality_num_dual/tests/formalization_lean/*.rs`.
- No code, dependency, or CI impact.
