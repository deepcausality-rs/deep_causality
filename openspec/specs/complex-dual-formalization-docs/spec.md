# complex-dual-formalization-docs Specification

## Purpose

The website formalization page for the Complex & Dual layer: a complete, accurate rendering of the `complex.*`, `quaternion.*`, and `dual.*` rows of `lean/THEOREM_MAP.md`, published (non-draft) at `/formalization/complex-dual/` and linked from the formalization index.

## Requirements

### Requirement: Complex & Dual formalization page renders the complete theorem map
The website page `website/docs/src/content/docs/formalization/complex-dual.md` SHALL contain one table row for every `complex.*`, `quaternion.*`, and `dual.*` id in `lean/THEOREM_MAP.md` (15 rows: 5 complex, 4 quaternion, 6 dual), with columns `id`, `statement`, `Lean proof`, `Rust witness`, and `Test`, and SHALL contain no rows absent from `lean/THEOREM_MAP.md`.

#### Scenario: Every theorem-map row is present
- **WHEN** the ids in the page's table are extracted and compared against the `complex.*`/`quaternion.*`/`dual.*` rows of `lean/THEOREM_MAP.md`
- **THEN** the two sets are equal and both have exactly 15 entries

#### Scenario: Cells match the source of truth
- **WHEN** any row's Lean proof and Rust witness cells are compared against the corresponding `THEOREM_MAP.md` row
- **THEN** the theorem and test names are identical, with Lean cells directory-qualified relative to `lean/DeepCausalityFormal/` (e.g. `Complex/Complex.lean :: complex_field_mul_inv`, `Dual/Dual.lean :: dual_leibniz`) and Rust cells rendered as bare `<file>_tests.rs :: <test>`

#### Scenario: Witness names are transcribed verbatim, not normalized
- **WHEN** the `dual.*` rows and the `norm_sq` rows are inspected
- **THEN** the test names match the map exactly (e.g. `test_mul_comm`, `test_complex_norm_sqr_mul`), with no invented prefixes or spelling fixes

### Requirement: Page follows the completed-layer house style
The page SHALL follow the conventions of the completed `num.md` and `algebra.md` pages: intro prose stating the law count (fifteen) with links to the Lean sources and both witness crate directories (`deep_causality_num_complex/tests/formalization_lean/`, `deep_causality_num_dual/tests/formalization_lean/`), retaining the stub's framing of the negative results (ℍ non-commutativity, `R[ε]` not a field), with no `Kani` column and no proved-status column (the prose states every row is `proved`).

#### Scenario: Draft scaffolding removed
- **WHEN** the finished page is inspected
- **THEN** the frontmatter contains no `draft: true` (keeping `sidebar: order: 5`) and the body contains no `:::caution` "good first issue" block

#### Scenario: Table shape matches the completed pages
- **WHEN** the table header is inspected
- **THEN** it is exactly `| id | statement | Lean proof | Rust witness | Test |` and every row's Test cell is `✓`

### Requirement: Formalization index links the Complex & Dual layer
The formalization index page (`formalization/index.md`) SHALL list Complex & Dual as a completed layer and SHALL NOT list it as pending documentation.

#### Scenario: Layer bullet added
- **WHEN** the "The layers" section is inspected
- **THEN** it contains a `**[Complex & Dual](/formalization/complex-dual/)**` bullet, placed between Core and Topology to match sidebar order

#### Scenario: Pending sentence updated
- **WHEN** the "being documented" sentence is inspected
- **THEN** it names only the Haft and Quantum layers

### Requirement: Site builds with the published page
The Astro documentation site SHALL build successfully with the Complex & Dual page published.

#### Scenario: Production build passes
- **WHEN** `npm run build` is run in `website/docs/`
- **THEN** the build exits zero and the output includes the `/formalization/complex-dual/` route
