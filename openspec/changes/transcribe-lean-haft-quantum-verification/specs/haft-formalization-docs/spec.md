# Spec: haft-formalization-docs

## ADDED Requirements

### Requirement: Haft formalization page renders the complete theorem map
The website page `website/docs/src/content/docs/formalization/haft.md` SHALL contain one table row for every `haft.*` id in the `### Haft layer` table of `lean/THEOREM_MAP.md` (49 rows), with columns `id`, `statement`, `Lean proof`, and `Test`, and SHALL NOT contain the planned ids from the "Not yet on the map" table (`haft.traversable.composition`, `haft.effect_unbound.laws`).

#### Scenario: Every Haft-layer row is present and no planned id leaks in
- **WHEN** the ids in the page's table are extracted and compared against the `haft.*` rows of the `### Haft layer` table
- **THEN** the two sets are equal with exactly 49 entries, and neither planned id appears

#### Scenario: Cells match the source of truth
- **WHEN** any row's Lean proof cell is compared against the corresponding map row's Lean-location cell
- **THEN** they are identical directory-qualified filenames (e.g. `Haft/Functor.lean`) with no theorem names, and the named file exists under `lean/DeepCausalityFormal/`

#### Scenario: Every id has a Rust witness despite the absent column
- **WHEN** each page id is searched as a `THEOREM_MAP:` annotation in `deep_causality_haft/tests/formalization_lean/`
- **THEN** every one of the 49 ids is found

### Requirement: Haft page follows the topology-page house style
The page SHALL follow `topology.md`: no per-row Rust-witness column, with intro prose stating the law count (forty-nine), linking `lean/DeepCausalityFormal/Haft/` on GitHub, and stating the witness convention (mirrored test files in `deep_causality_haft/tests/formalization_lean/`, one test per id carrying a `THEOREM_MAP:` annotation) so witnesses are findable without a column.

#### Scenario: Draft scaffolding removed
- **WHEN** the finished page is inspected
- **THEN** the frontmatter contains no `draft: true` (keeping `sidebar: order: 3`) and the body contains no `:::caution` "good first issue" block

#### Scenario: Table shape matches topology.md
- **WHEN** the table header is inspected
- **THEN** it is exactly `| id | statement | Lean proof | Test |` and every row's Test cell is `✓`
