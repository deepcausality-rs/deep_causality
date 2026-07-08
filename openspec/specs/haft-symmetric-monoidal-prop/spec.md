# haft-symmetric-monoidal-prop Specification

## Purpose
TBD - created by archiving change haft-categorical-machinery. Update Purpose after archive.
## Requirements
### Requirement: A symmetric-monoidal category with copy (Δ) and merge (∇) generators

`deep_causality_haft` SHALL provide a symmetric-monoidal / free-Markov structure over the effect monad with explicit generators: a **copy comonoid** (`Δ: A → A ⊗ A`, discard `ε: A → I`), a **merge** (`∇: A ⊗ A → A`), and a **symmetry** (swap). It MAY build on the existing `CoMonad` and `MonoidalMerge` where they help, but SHALL supply the comonoid/monoid *objects* and symmetry those do not. It SHALL introduce no `dyn`, no `unsafe`, and no `/src` macros. This is the categorical substrate the deferred reconvergence-merge (∇) extension consumes (`algebraic-causaloid-assumptions.md` #2 Q2); wiring it into the graph engine is out of scope here.

#### Scenario: Fan-out is a copy comonoid

- **WHEN** a value is fanned out to two consumers via `Δ`
- **THEN** both receive the same value, and `Δ` satisfies coassociativity and the counit law with `ε`

#### Scenario: Fan-in is a merge monoid

- **WHEN** two wires are merged via `∇`
- **THEN** `∇` satisfies associativity and the unit law, and (for a commutative merge) `∇ ∘ swap = ∇`

### Requirement: PROP laws are tested and proved in Lean

The comonoid laws (coassociativity, counit), the merge monoid laws (associativity, unit), and the symmetry/naturality laws SHALL be exercised by Rust law-tests (Bazel-registered) and proved in Lean under `DeepCausalityFormal/Haft/SymmetricMonoidal.lean` (bare-`lean`, transcribed self-contained without heavy Mathlib), bound by `THEOREM_MAP.md` ids (`haft.monoidal.comonoid_laws`, `haft.monoidal.merge_monoid_laws`, `haft.monoidal.symmetry`) with Rust witnesses.

#### Scenario: The merge substrate has both bridge sides

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** the `haft.monoidal.*` ids have `proved` Lean locations and passing Rust witnesses, and `Haft/SymmetricMonoidal.lean` typechecks standalone with bare `lean`

