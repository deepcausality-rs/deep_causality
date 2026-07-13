# Proposal: transcribe-lean-algebra-verification

## Why

The website's formalization section publishes the Lean ↔ Rust theorem map, but the Algebra page (`website/docs/src/content/docs/formalization/algebra.md`) is still a `draft: true` template stub with an empty table. The 33 `algebra.*` laws are already proved in Lean and witnessed by Rust law-tests; the documentation just hasn't been transcribed, so the Algebra layer is invisible on the public site.

## What Changes

- Fill the `formalization/algebra.md` table with all 33 `algebra.*` rows from `lean/THEOREM_MAP.md` (monoid, commutative-monoid, semilattice, verdict, group, ring, field, module, algebra-over-a-ring, division-algebra, conjugation, norm).
- Rewrite the page intro prose to follow the completed `num.md` pattern (law count, GitHub link to `lean/DeepCausalityFormal/Algebra/`, witness directory reference).
- Remove the "good first issue" caution block and the `draft: true` frontmatter flag so the page publishes.
- Update the formalization index (`formalization/index.md`): add an Algebra bullet under "The layers" and remove Algebra from the "being documented" sentence.

No Rust, Lean, or CI changes — this is a documentation-only transcription from an authoritative, CI-enforced source.

## Capabilities

### New Capabilities

- `algebra-formalization-docs`: the website formalization page for the Algebra layer — a complete, accurate rendering of the `algebra.*` rows of `lean/THEOREM_MAP.md`, published (non-draft) and linked from the formalization index.

### Modified Capabilities

None. No spec-level behavior of existing capabilities changes; the Lean proofs and Rust witnesses are untouched.

## Impact

- `website/docs/src/content/docs/formalization/algebra.md` — table filled, prose completed, draft flag removed.
- `website/docs/src/content/docs/formalization/index.md` — "The layers" list gains an Algebra entry; "being documented" sentence updated.
- Source of truth read (not modified): `lean/THEOREM_MAP.md`, `lean/DeepCausalityFormal/Algebra/*.lean`, `deep_causality_algebra/tests/formalization_lean/*.rs`.
- No code, dependency, or CI impact.
