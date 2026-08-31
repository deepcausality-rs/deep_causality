## ADDED Requirements

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

### Requirement: `Applicative.lean` keeps its laws and gains an agreement theorem
`Applicative.lean` SHALL retain all four McBride-Paterson laws and the functor-compatibility law unchanged, since `Applicative` itself does not change. It SHALL gain a theorem showing the derived `apply` equals the transcribed one, carried under `haft.lax_monoidal.apply_agreement` because it is a statement about the pair of traits rather than about either alone. The deviation this file already reports, that the Rust docstring omits the Composition law, SHALL be cleared in the same change.

#### Scenario: No existing applicative law is restated or removed

- **WHEN** `Applicative.lean` is compared against its prior state
- **THEN** the four laws and the functor-compatibility law are byte-identical, and only the new theorem is added

#### Scenario: The docstring lists every law that is proved

- **WHEN** the `Applicative` Rust docstring's law list is compared against `Applicative.lean`
- **THEN** Composition appears in both

### Requirement: The new Lean file follows the crate's build and import discipline
`LaxMonoidal.lean` SHALL be self-contained with no imports, as every other file under `Haft/` is, so that it typechecks standalone with bare `lean`. If an import ever becomes necessary it SHALL be mirrored into `cache_roots` in `MODULE.bazel`, since that list tree-shakes the Mathlib download and a module absent from it is never fetched. No Bazel target SHALL be added, because `lean/BUILD.bazel` globs `DeepCausalityFormal/{ns}/**/*.lean` per namespace.

#### Scenario: The file typechecks alone

- **WHEN** `LaxMonoidal.lean` is compiled with bare `lean`
- **THEN** it succeeds without a Mathlib dependency

#### Scenario: The namespace target picks it up unedited

- **WHEN** `bazel test //lean:Haft` is run after the file is added
- **THEN** it builds and passes with no edit to `lean/BUILD.bazel`
