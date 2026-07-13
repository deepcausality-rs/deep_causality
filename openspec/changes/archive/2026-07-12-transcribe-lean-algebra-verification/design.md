# Design: transcribe-lean-algebra-verification

## Context

The website formalization section renders `lean/THEOREM_MAP.md` for the web, one page per verification layer. `num.md`, `core.md`, and `topology.md` are complete and define the house style; `algebra.md` exists as a `draft: true` stub whose caution block contains the transcription instructions (it was staged as a "good first issue").

The 33 `algebra.*` rows sit in **two** tables inside `THEOREM_MAP.md`:

- the `## Map` table (lines ~29–39): `add_monoid` ×2, generic `monoid` ×3, `commutative_monoid` ×1, `semilattice` ×3, `verdict` ×2 — 11 rows;
- the `### Num / Algebra / Complex / Dual layers` table (lines ~98–119): `group`/`abelian_group` ×3, `ring`/`commutative_ring` ×4, `field`/`real_field` ×3, `module` ×4, `algebra` (over a ring) ×2, `division_algebra` ×1, `conjugate` ×3, `normed` ×2 — 22 rows.

Both sides of the bridge are verified present: all 33 ids carry `THEOREM_MAP:` tags in `lean/DeepCausalityFormal/Algebra/*.lean` (10 files), and every named test exists in `deep_causality_algebra/tests/formalization_lean/*.rs` (10 test files). CI (`formalization.yml`) enforces the id linkage, so `THEOREM_MAP.md` can be trusted as the single source of truth.

## Goals / Non-Goals

**Goals:**

- Publish a complete, accurate `formalization/algebra.md` matching the `num.md` house style.
- Link it from the formalization index so the page is discoverable.

**Non-Goals:**

- No changes to `THEOREM_MAP.md`, the Lean proofs, the Rust witnesses, or CI.
- No new laws, no re-statement or re-derivation of laws — transcription only.
- No changes to the other draft pages (`complex-dual.md`, `haft.md`, `quantum.md`).

## Decisions

1. **Column shape `| id | statement | Lean proof | Rust witness | Test |`** — as mandated by the stub's instructions and matching `num.md`. The source's `Lean` (proved-status) column is dropped (every row is `proved`; the prose says so once) and the `Kani` column is dropped (this layer has no Kani harnesses; `core.md` is the only page that carries it).

2. **Paths rendered relative, directories linked once in prose** — cells read `Monoid.lean :: add_monoid_assoc` and `monoid_tests.rs :: test_add_monoid_assoc`; the intro links `lean/DeepCausalityFormal/Algebra/` on GitHub and names `deep_causality_algebra/tests/formalization_lean/` as the witness directory. This is the exact `num.md` convention.

3. **Keep Mathlib parentheticals in statements** (e.g. "(Mathlib `one_mul`)") — they document which Mathlib lemma anchors the proof at zero cost. Alternative considered: stripping them for brevity, rejected because `num.md`/`core.md` keep source annotations like "(bit-exact bounds are `[open]`)".

4. **Shared witnesses transcribed as-is** — the three generic-monoid rows all cite `test_generic_monoid_laws`, and `semilattice.assoc`/`semilattice.comm` share `test_semilattice_assoc_and_comm`. The map is the source of truth; the page must not invent per-row test names. Multi-theorem Lean cells (`verdict_meet_comm / verdict_absorption`) use the ` / ` separator per `core.md`.

5. **Row order: monoid → group → ring → field → module/algebra → division algebra → conjugation → norm → semilattice → verdict** — i.e. the trait-tower order the crate README uses, merging the two source tables rather than preserving their split. Alternative considered: keeping the two source-table blocks verbatim; rejected because the split reflects THEOREM_MAP's history (extracted crates), not the tower structure a reader follows. Within each family, source order is preserved.

6. **Index update is minimal** — one bullet added between Num and Core in "The layers" (matching sidebar order 1, 2, 4), and "Algebra" removed from the "being documented" sentence. The stub's `sidebar: order: 2` is already correct and collision-free.

## Risks / Trade-offs

- [Transcription typo breaks the doc↔map correspondence] → Mechanical verification during apply: extract ids/theorems/tests from the new page and diff them against `THEOREM_MAP.md` and against `grep` output from the Lean and Rust trees (the same cross-check already performed during exploration).
- [Reordering rows away from the source tables makes future audits slightly harder] → Ids are globally unique; an audit greps by id, not by position. The count (33) is stated in the prose as a checksum.
- [Site build could reject the page] → Run the Astro build (`npm run build` in `website/docs/`) before finishing; the page is plain frontmatter + markdown table, identical in kind to `num.md`.

## Open Questions

None — the stub's embedded instructions resolve all format questions.
