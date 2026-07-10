## ADDED Requirements

### Requirement: Evaluate is the unique catamorphism per fixed carrier

The formalization SHALL prove `core.causaloid.catamorphism_unique`: for a fixed carrier (a fixed
`Verdict`/semantic algebra instance, a fixed fan-in `∇`, fixed Λ's), `evaluate` is the **unique**
F-algebra homomorphism from `Causaloid ≅ μX.F(X)` into the Kleisli category of the causal monad —
any function satisfying the three algebra equations (the `Atom`, `Coll`, and `Graph` cases) equals
`evaluate`. Uniqueness SHALL be scoped **per semantic algebra** (different carriers give different,
each-unique interpreters — assumption #6, correctly scoped; goal B2). The closed-world premise is
the sealed three-form surface (assumption #11a, DECIDED) and the initiality of the Stage-2
fixpoint.

#### Scenario: Any algebra-respecting interpreter equals evaluate

- **WHEN** `Core/Catamorphism.lean` is checked
- **THEN** `core.causaloid.catamorphism_unique` is closed: a hypothesis interpreter satisfying the
  three case equations is proven pointwise equal to the catamorphism, by induction on the fixpoint

#### Scenario: Uniqueness is per carrier, stated honestly

- **WHEN** the theorem statement is read
- **THEN** the carrier (semantic algebra) is an explicit parameter held fixed, and the header
  records that uniqueness across carriers is neither claimed nor true

### Requirement: Encapsulation is flat

The formalization SHALL prove `core.causaloid.encapsulation_flat`: folding a nested causaloid (a
`Coll`/`Graph` node containing sub-causaloids) equals folding the flattened structure —
catamorphism fusion, inherited from the monad's associativity — so wrapping a subgraph in a
causaloid does not change the evaluated semantics (the QCM-on-EPP Layer-B property, generalized to
the whole causaloid).

#### Scenario: Nested fold equals flat fold

- **WHEN** a graph containing a causaloid-wrapped subgraph and its flattened equivalent are both
  evaluated
- **THEN** the results are equal on every channel (value, error, state; logs up to the join
  multiset ruling), machine-checked in Lean and witnessed in Rust

### Requirement: The arrow fragment is the free term language

The formalization SHALL prove `core.causaloid.arrow_fragment`: the `Atom`/`compose`/`split`
fragment of the causaloid (including the `⊕`-enlarged generators from Stage 2b) is isomorphic to
the reified `ArrowTerm` term language, and on that fragment `evaluate = interpret_kleisli`. The
statement SHALL distinguish `T` (the free term) from `T/≈` (the quotient by the proven Arrow laws)
and show the interpreter factors through `T/≈` (assumption #8).

#### Scenario: Evaluate agrees with the interpreter on the fragment

- **WHEN** a causaloid in the arrow fragment and its `ArrowTerm` image are both run
- **THEN** `evaluate` on the causaloid equals `interpret_kleisli` on the term, including terms
  containing `choice`/`fanin`

#### Scenario: The quotient is explicit

- **WHEN** two terms related by an Arrow law (e.g. associativity of `compose`) are interpreted
- **THEN** their interpretations are equal — the factoring through `T/≈` — stated as its own lemma

### Requirement: The keystone is tested and proved in Lean

`core.causaloid.{catamorphism_unique,encapsulation_flat,arrow_fragment}` SHALL be proved in Lean
under `DeepCausalityFormal/Core/Catamorphism.lean` (bare-`lean`, zero `sorry`), citing the Stage-2
fixpoint, `haft.arrow_term.*`, and `haft.interpreter.*` as the base, bound by `THEOREM_MAP.md` rows
with Rust witnesses under `deep_causality/tests/formalization_lean/` (Bazel-registered).

#### Scenario: Both bridge sides exist for the keystone

- **WHEN** `THEOREM_MAP.md` is checked
- **THEN** each of the three `core.causaloid.*` keystone ids has a proved Lean location and a
  passing Rust witness, and `Core/Catamorphism.lean` typechecks standalone with bare `lean`
