# algebra-formalization-docs Specification

## Purpose

The website formalization page for the Algebra layer: a complete, accurate rendering of the `algebra.*` rows of `lean/THEOREM_MAP.md`, published (non-draft) at `/formalization/algebra/` and linked from the formalization index.

## Requirements

### Requirement: Algebra formalization page renders the complete theorem map
The website page `website/docs/src/content/docs/formalization/algebra.md` SHALL contain one table row for every `algebra.*` id in `lean/THEOREM_MAP.md` (33 rows), with columns `id`, `statement`, `Lean proof`, `Rust witness`, and `Test`, and SHALL contain no rows that are absent from `lean/THEOREM_MAP.md`.

#### Scenario: Every theorem-map row is present
- **WHEN** the ids in the page's table are extracted and compared against `grep '| \`algebra\.' lean/THEOREM_MAP.md`
- **THEN** the two sets are equal and both have exactly 33 entries

#### Scenario: Cells match the source of truth
- **WHEN** any row's Lean proof and Rust witness cells are compared against the corresponding `THEOREM_MAP.md` row
- **THEN** the theorem names and test names are identical, rendered relative as `<File>.lean :: <theorem>` and `<file>_tests.rs :: <test>`, with multiple theorems separated by ` / `

### Requirement: Page follows the completed-layer house style
The page SHALL follow the conventions of the completed `num.md` page: intro prose stating the law count with a GitHub link to `lean/DeepCausalityFormal/Algebra/` and the witness directory `deep_causality_algebra/tests/formalization_lean/`, no `Kani` column, and no proved-status column (the prose states every row is `proved`).

#### Scenario: Draft scaffolding removed
- **WHEN** the finished page is inspected
- **THEN** the frontmatter contains no `draft: true` and the body contains no `:::caution` "good first issue" block

#### Scenario: Table shape matches num.md
- **WHEN** the table header is inspected
- **THEN** it is exactly `| id | statement | Lean proof | Rust witness | Test |` and every row's Test cell is `✓`

### Requirement: Formalization index links the Algebra layer
The formalization index page (`formalization/index.md`) SHALL list Algebra as a completed layer and SHALL NOT list it as pending documentation.

#### Scenario: Layer bullet added
- **WHEN** the "The layers" section is inspected
- **THEN** it contains an `**[Algebra](/formalization/algebra/)**` bullet, placed between Num and Core to match sidebar order

#### Scenario: Pending sentence updated
- **WHEN** the "being documented" sentence is inspected
- **THEN** it names only the Complex & Dual, Haft, and Quantum layers

### Requirement: Site builds with the published page
The Astro documentation site SHALL build successfully with the Algebra page published.

#### Scenario: Production build passes
- **WHEN** `npm run build` is run in `website/docs/`
- **THEN** the build exits zero and the output includes the `/formalization/algebra/` route
