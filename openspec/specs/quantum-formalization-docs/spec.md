# quantum-formalization-docs Specification

## Purpose

The website formalization page for the Quantum layer — a complete rendering of the `quantum.*` rows of `lean/THEOREM_MAP.md` with the B1 counterexample framing, published (non-draft) at `/formalization/quantum/` and linked from the formalization index — backed by a corrected `## Quantum` section of the map whose witness pointers match the Rust tree.

## Requirements

### Requirement: THEOREM_MAP quantum witness pointers match the Rust tree
The `## Quantum` section of `lean/THEOREM_MAP.md` SHALL name the actual witness locations: prose pointing at `deep_causality_quantum/tests/formalization_lean/{partial_trace_tests,choi_tests}.rs`, the eight `quantum.partial_trace*` rows' witness cells naming `partial_trace_tests.rs`, and the two `quantum.choi.*` rows' witness cells naming `choi_tests.rs :: test_apply_choi_is_linear`. Ids, statements, and Lean cells SHALL be unchanged.

#### Scenario: Corrected cells point at existing tests
- **WHEN** every quantum row's witness cell is checked against the Rust tree
- **THEN** the named file exists in `deep_causality_quantum/tests/formalization_lean/` and contains the named `#[test]` function

#### Scenario: Correction is surgical
- **WHEN** the section is diffed against its previous state
- **THEN** only witness prose and witness cells differ; the id, statement, Lean, and Test columns are byte-identical

### Requirement: Quantum formalization page renders the complete theorem map
The website page `website/docs/src/content/docs/formalization/quantum.md` SHALL contain one table row for every `quantum.*` id in the `## Quantum` section (10 rows), with columns `id`, `statement`, `Lean proof`, `Rust witness`, and `Test`, transcribed from the corrected section, with Lean cells directory-qualified (e.g. `Quantum/PartialTrace.lean :: partialTraceRight_add`) and witness cells bare (`partial_trace_tests.rs :: test_partial_trace_linearity`).

#### Scenario: Every theorem-map row is present
- **WHEN** the ids in the page's table are extracted and compared against the `quantum.*` rows of `lean/THEOREM_MAP.md`
- **THEN** the two sets are equal and both have exactly 10 entries

#### Scenario: Cells verified against both sources
- **WHEN** any row's cells are checked
- **THEN** the Lean theorem exists in the exact named `.lean` file under `lean/DeepCausalityFormal/` and the test exists in the exact named file under `deep_causality_quantum/tests/formalization_lean/`

### Requirement: Quantum page keeps the honest framing
The page SHALL follow the `num.md`/`core.md` house style (no Kani column, prose noting every row is `proved`) and SHALL state: the B1 headline (unconditional `partial_trace_preservation` is false with a witnessed counterexample, while the conditional boundary version holds), the `/Quantum/` tree's `sorry`-gate exemption, and a one-sentence pointer to the deferred targets (CJ reconstruction and the QCM theorems) mirroring the map's closing paragraph.

#### Scenario: Draft scaffolding removed
- **WHEN** the finished page is inspected
- **THEN** the frontmatter contains no `draft: true` (keeping `sidebar: order: 7`) and the body contains no `:::caution` block

#### Scenario: Negative result framed as such
- **WHEN** the intro prose is read
- **THEN** the counterexample rows are framed as a proved impossibility (the B1 result), not as ordinary algebraic laws

### Requirement: Formalization index lists all layers as complete
The formalization index SHALL list Haft (between Algebra and Core) and Quantum (after Topology) under "The layers" and SHALL NOT contain the pending-documentation / good-first-issue sentence.

#### Scenario: All seven layers listed
- **WHEN** "The layers" section is inspected
- **THEN** it lists Num, Algebra, Haft, Core, Complex & Dual, Topology, and Quantum in sidebar order, each as a link

#### Scenario: No pending sentence remains
- **WHEN** the section's trailing prose is inspected
- **THEN** the "being documented" sentence is gone

### Requirement: Site builds with both pages published
The Astro documentation site SHALL build successfully with the Haft and Quantum pages published.

#### Scenario: Production build passes
- **WHEN** `npm run build` is run in `website/docs/`
- **THEN** the build exits zero and the output includes the `/formalization/haft/` and `/formalization/quantum/` routes
