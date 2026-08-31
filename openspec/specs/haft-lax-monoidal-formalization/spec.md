# haft-lax-monoidal-formalization Specification

## Purpose
TBD - created by archiving change add-lax-monoidal-applicative. Update Purpose after archive.
## Requirements
### Requirement: The lax monoidal laws are proved in Lean under their own id prefix
`lean/DeepCausalityFormal/Haft/LaxMonoidal.lean` SHALL prove the coherence laws over the `Option` carrier under the `haft.lax_monoidal.*` `THEOREM_MAP` prefix, with these four ids: `haft.lax_monoidal.naturality` for `zip(fmap(fa, f), fmap(fb, g)) = fmap(zip(fa, fb), f × g)`; `haft.lax_monoidal.assoc` for `zip(zip(fa, fb), fc) ≅ zip(fa, zip(fb, fc))` modulo the associator; `haft.lax_monoidal.unit_laws` for `zip(unit(), fa) ≅ fa` and `zip(fa, unit()) ≅ fa` modulo the unitors; and `haft.lax_monoidal.apply_agreement` for the equality of the derived `apply` with a hand-written one. The prefix SHALL NOT be `haft.monoidal.*`, which `SymmetricMonoidal.lean` already occupies, since the `theorem-map` job matches ids by `grep -Fl` and a shared prefix would match the wrong row and hide a missing entry.

#### Scenario: The four ids have both bridge sides

- **WHEN** `THEOREM_MAP.md` is checked for the new ids
- **THEN** each has a `proved` Lean location and a passing Rust witness in `deep_causality_haft/tests/formalization_lean/`

#### Scenario: The prefix does not collide

- **WHEN** each `haft.monoidal.*` id is grepped
- **THEN** it matches only its own row, and no `haft.lax_monoidal.*` id matches a `haft.monoidal.*` row

### Requirement: Laws are stated at the trait that can carry them
`assoc` and `naturality` SHALL be stated against `Semigroupal`, and `unit_laws` against `LaxMonoidal`, mirroring the Rust split, so a witness carrying φ without η can discharge the first two without owing the third. Naturality SHALL be the first law stated, being what makes φ a natural transformation rather than an arbitrary binary function.

#### Scenario: A semigroupal-only witness owes two laws, not three

- **WHEN** the law obligations of a witness implementing `Semigroupal` alone are enumerated
- **THEN** they are naturality and associativity, and the unit laws are not among them

### Requirement: `Applicative.lean` keeps its laws unchanged and sheds a stale deviation note
`Applicative.lean` SHALL retain all four McBride-Paterson laws and the functor-compatibility law byte-identical, since `Applicative` itself does not change. Its header DEVIATION NOTE, which states that the Rust docstring lists only three laws and omits Composition, SHALL be removed: the docstring already carries all four with Composition numbered second, and the deviations note records the matching entry D1 as **RESOLVED (docs)**. No Rust docstring edit is required, because the deviation it describes was cleared before this change.

#### Scenario: No existing applicative law is restated or removed

- **WHEN** `Applicative.lean`'s theorems are compared against their prior state
- **THEN** the four laws and the functor-compatibility law are byte-identical

#### Scenario: The header no longer reports a deviation that is closed

- **WHEN** the header's claim about the Rust docstring is checked against `src/applicative/mod.rs`
- **THEN** the docstring lists Composition, and the header does not claim otherwise

### Requirement: The agreement theorem lives with its own id prefix
The theorem showing the derived `apply` equals the hand-written one SHALL carry the id `haft.lax_monoidal.apply_agreement` and SHALL be proved in `LaxMonoidal.lean`, not in `Applicative.lean`. `THEOREM_MAP.md` records exactly one Lean location per id, and `haft-formalization-docs` requires that cell to be a bare directory-qualified filename, so an id under the `haft.lax_monoidal.*` prefix SHALL resolve to the file that owns the prefix. Because every file under `Haft/` is self-contained with no imports, `LaxMonoidal.lean` SHALL transcribe both sides of the equation locally rather than importing either.

#### Scenario: The id resolves to one file matching its prefix

- **WHEN** the `haft.lax_monoidal.apply_agreement` row's Lean location is read
- **THEN** it is `Haft/LaxMonoidal.lean`, and every other `haft.lax_monoidal.*` id resolves to the same file

#### Scenario: The obligation is still stated for a witness holding both traits

- **WHEN** a witness implements both `Applicative` and `MonoidalApplicative`
- **THEN** the agreement theorem is the law its Rust witness test discharges

### Requirement: The new Lean file follows the crate's build and import discipline
`LaxMonoidal.lean` SHALL be self-contained with no imports, as every other file under `Haft/` is, so that it typechecks standalone with bare `lean`. If an import ever becomes necessary it SHALL be mirrored into `cache_roots` in `MODULE.bazel`, since that list tree-shakes the Mathlib download and a module absent from it is never fetched. No Bazel target SHALL be added, because `lean/BUILD.bazel` globs `DeepCausalityFormal/{ns}/**/*.lean` per namespace.

#### Scenario: The file typechecks alone

- **WHEN** `LaxMonoidal.lean` is compiled with bare `lean`
- **THEN** it succeeds without a Mathlib dependency

#### Scenario: The namespace target picks it up unedited

- **WHEN** `bazel test //lean:Haft` is run after the file is added
- **THEN** it builds and passes with no edit to `lean/BUILD.bazel`

