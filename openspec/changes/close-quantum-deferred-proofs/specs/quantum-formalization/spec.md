## ADDED Requirements

### Requirement: The Choi–Jamiołkowski reconstruction is proved

The formalization SHALL prove `quantum.choi.reconstruction`: for every `R`-linear map
`E : Matrix α α R →ₗ[R] Matrix β β R` over a commutative ring, `applyChoi (choiOf E) A = E A` for
every `A`, using the definitions already in `Quantum/Choi.lean`. The id SHALL carry a
`THEOREM_MAP.md` row and a Rust witness under
`deep_causality_quantum/tests/formalization_lean/` that builds a channel's Choi operator, applies it,
and compares the result with the channel's direct action.

#### Scenario: The reconstruction closes with no sorry

- **WHEN** `Quantum/Choi.lean` is built
- **THEN** the reconstruction theorem typechecks with zero `sorry`, and the comment that deferred it
  is gone

#### Scenario: The witness agrees with the theorem

- **WHEN** the Rust witness runs on a channel with a non-diagonal Choi operator
- **THEN** applying the reconstructed action and applying the channel give the same matrix within
  the operator tolerance

## MODIFIED Requirements

### Requirement: The in-scope quantum causal-model theorems

The formalization SHALL prove three quantum causal-model theorems on the pair-indexed matrix model,
each with a Rust witness and a `THEOREM_MAP.md` row:
`quantum.markov_commutativity` (Lorenz 2022, Def 3.3): factors on disjoint legs commute,
`(A ⊗ 1)(1 ⊗ B) = (1 ⊗ B)(A ⊗ 1)`, and the product of a finite family of pairwise-commuting factors
does not depend on the order of the factors;
`quantum.classical_embedding`: diagonal factors commute pairwise, so every family of diagonal
factors meets the Markov condition, and the Kronecker product of two diagonal matrices is diagonal;
`quantum.no_influence` (Lorenz & Barrett 2021, Def 1, stated for any linear map on the bipartite
model): the marginal on the output D factors through the partial trace over the input A exactly when
the D marginal of every product input depends on the A part only through its trace.
`quantum.unitary_factorization` (Lorenz & Barrett 2021, Thm 1) SHALL remain an open target,
recorded with its reason: its proof rests on the commutant and direct-sum structure of
finite-dimensional C*-algebras, which the pinned Mathlib lacks. `quantum.cyclic_support` SHALL NOT be
a target: the crate refuses cyclic structures at `build()` by decision, and acyclicity as a separable
parameter is proved as `core.context_graph.acyclicity_separable`. General (all-unitary) causal
faithfulness SHALL be out of scope, recorded as an upstream-open target.

#### Scenario: Each in-scope id is proved and witnessed

- **WHEN** the CI traceability gate runs
- **THEN** `quantum.markov_commutativity`, `quantum.classical_embedding` and `quantum.no_influence`
  each have a bare-`lean` proof (exit 0, zero `sorry`), a Rust witness, and a `THEOREM_MAP.md` row,
  and the witness-search scope includes `deep_causality_quantum`

#### Scenario: The open target states its reason

- **WHEN** `LEAN_QUANTUM.md` and the quantum section of `THEOREM_MAP.md` are read
- **THEN** `quantum.unitary_factorization` is the only open quantum target, each names the missing
  C*-algebra machinery, and neither lists `quantum.cyclic_support`

#### Scenario: Faithfulness is scoped, not overclaimed

- **WHEN** the crate documents its faithfulness claims
- **THEN** they are limited to the C₃-exclusion (traditional-circuit) regime of van der Lugt & Lorenz
  (arXiv:2508.11762), and the general routed/direct-sum Lorenz–Barrett hypothesis plus the operator-
  level direct sum are named as deferred/open

### Requirement: The orthomodular carrier extends the Verdict carrier theorem

The formalization SHALL extend `core.verdict.carriers` with `quantum.verdict.orthomodular` over the
lattice of subspaces of a finite-dimensional complex inner product space, the ranges of the Rust
`Projection` carrier: the lattice is bounded, the orthocomplement `ᗮ` is an antitone involution with
`K ⊓ Kᗮ = ⊥` and `K ⊔ Kᗮ = ⊤`, and `K₁ ≤ K₂` implies `K₁ ⊔ (K₁ᗮ ⊓ K₂) = K₂`. A witness in `ℂ²`
SHALL show distributivity failing for the spans of `|0⟩`, `|1⟩` and `|+⟩`.

#### Scenario: The orthomodular carrier is proved and witnessed

- **WHEN** the verdict-carrier extension is checked
- **THEN** `quantum.verdict.orthomodular` has a bare-`lean` proof and a Rust witness over the
  `Projection` carrier that includes non-commuting projections

#### Scenario: Distributivity fails on the documented triple

- **WHEN** the distributivity witness is checked
- **THEN** Lean proves `K₀ ⊓ (K₁ ⊔ K₊) ≠ (K₀ ⊓ K₁) ⊔ (K₀ ⊓ K₊)` for the three lines in `ℂ²`, the
  same triple the Rust carrier's documentation and tests use
