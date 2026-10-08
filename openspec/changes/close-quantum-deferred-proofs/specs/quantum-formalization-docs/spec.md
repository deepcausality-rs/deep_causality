## MODIFIED Requirements

### Requirement: Quantum formalization page renders the complete theorem map
The website page `website/docs/src/content/docs/formalization/quantum.md` SHALL contain one table row for every `quantum.*` id in the `## Quantum` section, with columns `id`, `statement`, `Lean proof`, `Rust witness`, and `Test`, transcribed from the section, with Lean cells directory-qualified (e.g. `Quantum/PartialTrace.lean :: partialTraceRight_add`) and witness cells bare (`partial_trace_tests.rs :: test_partial_trace_linearity`).

#### Scenario: Every theorem-map row is present
- **WHEN** the ids in the page's table are extracted and compared against the `quantum.*` rows of `lean/THEOREM_MAP.md`
- **THEN** the two sets are equal

#### Scenario: Cells verified against both sources
- **WHEN** any row's cells are checked
- **THEN** the Lean theorem exists in the exact named `.lean` file under `lean/DeepCausalityFormal/` and the test exists in the exact named file under `deep_causality_quantum/tests/formalization_lean/`

### Requirement: Quantum page keeps the honest framing
The page SHALL follow the `num.md`/`core.md` house style (no Kani column, prose noting every row is `proved`) and SHALL state: the B1 headline (unconditional `partial_trace_preservation` is false with a witnessed counterexample, while the conditional boundary version holds), the `/Quantum/` tree's `sorry`-gate exemption, and a one-sentence pointer to the single open target, `quantum.unitary_factorization`, with the C*-algebra machinery it needs, mirroring the map's closing paragraph.

#### Scenario: Draft scaffolding removed
- **WHEN** the finished page is inspected
- **THEN** the frontmatter contains no `draft: true` (keeping `sidebar: order: 7`) and the body contains no `:::caution` block

#### Scenario: Negative result framed as such
- **WHEN** the intro prose is read
- **THEN** the counterexample rows are framed as a proved impossibility (the B1 result), not as ordinary algebraic laws

#### Scenario: One open target, with its reason
- **WHEN** the page's closing pointer is read
- **THEN** it names `quantum.unitary_factorization` as the only open quantum target and the missing machinery, and it names no retired target
