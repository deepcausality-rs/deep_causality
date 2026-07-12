# quantum-formalization Specification

## Purpose
TBD - created by archiving change add-quantum-crate. Update Purpose after archive.
## Requirements
### Requirement: A Lean partial-trace and Choi–Jamiołkowski foundation

The formalization SHALL build, as the first deliverable of the Lean work, a partial-trace and
Choi–Jamiołkowski foundation in a new `Quantum/` directory that imports Mathlib's Hilbert-space and
linear-algebra machinery (the project's first DeepCausalityFormal consumer of it), defining `Tr_B`
and the CJ operator with their lemma libraries (linearity, positivity, `Tr_B(X ⊗ Y) = X · Tr(Y)`, and
the bimodule law), because the pinned Mathlib provides neither. Every file SHALL close with zero
`sorry`.

#### Scenario: The foundation typechecks and gates the theorems

- **WHEN** `lake build` runs on the `Quantum/` foundation
- **THEN** `Quantum/PartialTrace.lean` and `Quantum/Choi.lean` typecheck with zero `sorry`, and the
  downstream `quantum.*` theorems import them

### Requirement: The in-scope quantum causal-model theorems

The formalization SHALL prove, each with a Rust witness and a `THEOREM_MAP.md` row:
`quantum.no_influence` (Lorenz & Barrett 2021, Def 1 — the partial-trace marginal condition),
`quantum.markov_commutativity` (Lorenz 2022, Def 3.3 — factorization into pairwise-commuting CJ
operators, with the 2-factor free-commutation lemma explicit), `quantum.unitary_factorization`
(Lorenz & Barrett 2021, Thm 1 — the commuting factorization on the unitary fragment),
`quantum.classical_embedding` (the diagonal-σ special case, where this crate meets
`deep_causality_do_calculus`), and `quantum.cyclic_support` (cyclic QCM on the native non-DAG
hypergraph, resting on `core.context_graph.acyclicity_separable`). General (all-unitary) causal
faithfulness SHALL be out of scope, recorded as an upstream-open target.

#### Scenario: Each in-scope id is proved and witnessed

- **WHEN** the CI traceability gate runs
- **THEN** each of the five in-scope `quantum.*` ids has a bare-`lean` proof (exit 0, zero `sorry`), a
  Rust witness, and a `THEOREM_MAP.md` row, and the witness-search scope includes
  `deep_causality_quantum`

#### Scenario: Faithfulness is scoped, not overclaimed

- **WHEN** the crate documents its faithfulness claims
- **THEN** they are limited to the C₃-exclusion (traditional-circuit) regime of van der Lugt & Lorenz
  (arXiv:2508.11762), and the general routed/direct-sum Lorenz–Barrett hypothesis plus the operator-
  level direct sum are named as deferred/open

### Requirement: `partial_trace_preservation` is stated conditionally with a witnessed counterexample

The formalization SHALL NOT state `partial_trace_preservation` unconditionally. It SHALL prove
`quantum.partial_trace_nonpreservation` — the refuting counterexample
(`X = σx ⊗ |0⟩⟨0| + σz ⊗ |1⟩⟨1|`, `Y = σx ⊗ |0⟩⟨0| − σz ⊗ |1⟩⟨1|`: `[X,Y] = 0` but
`[Tr₂X, Tr₂Y] = +4i·σy ≠ 0`) — and `quantum.partial_trace_preservation_boundary` — the conditional
sufficient theorem (boundary-only / single-node interface ⇒ preservation, via
`Tr_B((1_B ⊗ Z) · M) = Z · Tr_B(M)`). The exact necessary-and-sufficient "valid encapsulation"
condition SHALL remain an explicit open target, to be narrowed empirically via the instrumented
freeze. The crate SHALL NOT claim quantum-subgraph encapsulation as a supported semantic operation:
flat QCM models are the supported path, and nesting-transparency — whose physical meaning is itself
unestablished — is an open research question, so the conditional theorem and counterexample serve to
document the boundary rather than to enable a promised feature.

#### Scenario: The counterexample and the conditional theorem both land

- **WHEN** the partial-trace preservation work is checked
- **THEN** `quantum.partial_trace_nonpreservation` proves the commuting-operators / non-commuting-
  marginals witness, `quantum.partial_trace_preservation_boundary` proves preservation under the
  boundary condition, and no unconditional preservation statement exists anywhere in the corpus

### Requirement: The orthomodular carrier extends the Verdict carrier theorem

The formalization SHALL extend `core.verdict.carriers` with `quantum.verdict.orthomodular`: the
orthomodular projection-lattice carrier satisfies the bounded-lattice + orthocomplement + orthomodular
laws, with a Rust witness for the Phase-4 newtype.

#### Scenario: The orthomodular carrier is proved and witnessed

- **WHEN** the verdict-carrier extension is checked
- **THEN** `quantum.verdict.orthomodular` has a bare-`lean` proof and a Rust witness over the commuting
  projection-family newtype

