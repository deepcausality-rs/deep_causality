# Tasks: transcribe-lean-algebra-verification

## 1. Fill the Algebra page

- [x] 1.1 Transcribe the 11 `algebra.*` rows from the `## Map` table of `lean/THEOREM_MAP.md` (add_monoid, generic monoid, commutative_monoid, semilattice, verdict) into `website/docs/src/content/docs/formalization/algebra.md`, columns `| id | statement | Lean proof | Rust witness | Test |`, paths relative (`Monoid.lean :: add_monoid_assoc` / `monoid_tests.rs :: test_add_monoid_assoc`)
- [x] 1.2 Transcribe the 22 `algebra.*` rows from the `### Num / Algebra / Complex / Dual layers` table (group, ring, field, module, algebra-over-ring, division_algebra, conjugate, normed) in the same format
- [x] 1.3 Order the merged rows by trait-tower position (monoid → group → ring → field → module/algebra → division algebra → conjugation → norm → semilattice → verdict), preserving source order within each family
- [x] 1.4 Rewrite the intro prose per the `num.md` pattern: state "Thirty-three laws", link `lean/DeepCausalityFormal/Algebra/` on GitHub, name `deep_causality_algebra/tests/formalization_lean/` as the witness location, note every row is `proved`
- [x] 1.5 Delete the `:::caution` "good first issue" block and remove `draft: true` from the frontmatter (keep `sidebar: order: 2`)

## 2. Link from the formalization index

- [x] 2.1 Add an `**[Algebra](/formalization/algebra/)**` bullet to "The layers" in `website/docs/src/content/docs/formalization/index.md`, between Num and Core, with a one-line summary of the layer
- [x] 2.2 Update the "being documented" sentence to name only the Complex & Dual, Haft, and Quantum layers

## 3. Verify

- [x] 3.1 Cross-check the page against the source of truth: extracted ids equal the 33 `algebra.*` ids in `THEOREM_MAP.md`; every Lean theorem name appears in `lean/DeepCausalityFormal/Algebra/*.lean`; every test name appears in `deep_causality_algebra/tests/formalization_lean/*.rs`
- [x] 3.2 Build the site (`npm run build` in `website/docs/`) and confirm it exits zero with the `/formalization/algebra/` route present
