## ADDED Requirements

### Requirement: An operator/channel layer over complex tensors

The crate SHALL provide an operator/channel layer built on `CausalTensor<Complex<R>>` (`R: RealField`,
including `Float106`): a density-matrix (mixed-state) type with positivity and unit-trace checks, the
partial trace `Tr_B`, the Choi–Jamiołkowski isomorphism between channels and operators, a CPTP / Kraus
representation, and the operator commutator `[A, B]`. `HilbertState` remains the pure-state ket and is
reused, not duplicated.

#### Scenario: Partial trace obeys its defining identities

- **WHEN** `Tr_B` is exercised on product and general operators
- **THEN** it is linear, satisfies `Tr_B(X ⊗ Y) = X · Tr(Y)` and the bimodule law
  `Tr_B((1_B ⊗ Z) · M) = Z · Tr_B(M)`, and preserves positivity and total trace

#### Scenario: Choi–Jamiołkowski round-trips a channel

- **WHEN** a CPTP map is converted to its CJ operator and back
- **THEN** the round-trip is identity up to numerical tolerance, and a CPTP map yields a positive,
  trace-preserving CJ operator

### Requirement: A ket↔matrix bridge grounds pure states in the operator layer

The state layer SHALL provide a bijective bridge between `HilbertState<R>` (a minimal-left-ideal
Clifford multivector) and its d-dimensional complex column, so that `ρ = |ψ⟩⟨ψ|` and `A|ψ⟩` are formed
on `CausalTensor<Complex<R>>`. It SHALL reuse the existing `to_matrix`/`from_matrix` isomorphism and the
gamma basis, adding no new numeric substrate: `to_ket` SHALL return the distinguished minimal-left-ideal
column (`KET_COLUMN = 0`, the primitive idempotent `E = e₀e₀ᵀ`) of `to_matrix()`, and `from_ket` SHALL
embed a d-vector as that column and apply `from_matrix`. The bridge SHALL be defined only for
even-dimensional metrics (where `to_matrix` is a bijection, `d² = 2ⁿ`), including `Cl(0,10)` (d = 32),
using the metric-signature type from `deep_causality_metric` (the SSOT), and SHALL error otherwise. The
ket inner product SHALL agree with the existing `QuantumOps::bracket` (the Dirac product `⟨φ|ψ⟩`) and the
adjoint with `QuantumOps::dag` (`reversion` + coefficient-conjugation); it SHALL NOT reuse the unrelated
multifield Lie-commutator.

#### Scenario: Ket embedding round-trips and agrees with the Dirac product

- **WHEN** a d-vector `v` is embedded via `from_ket` and read back via `to_ket`, and two states are
  combined as `to_ket(φ)† · to_ket(ψ)`
- **THEN** `to_ket(from_ket(v)) == v` up to tolerance, `from_ket(to_ket(ψ)) == ψ` up to the MLI (column-0)
  embedding, and the column product equals `QuantumOps::bracket(φ, ψ)` up to tolerance

#### Scenario: Density matrix from a ket is a valid state

- **WHEN** `ρ = to_ket(ψ) · to_ket(ψ)†` is formed for a normalized ψ
- **THEN** `ρ` is Hermitian, positive semidefinite, unit-trace, and idempotent (rank-1 purity), feeding
  the density-matrix checks

### Requirement: Encapsulation-equals-flat exercised over matrix-valued state

The layer SHALL exercise the inherited monad associativity law (`core.causaloid.encapsulation_flat`)
over the arity-5 STATE channel carrying complex-MATRIX payloads, not only scalar state, so that
"encapsulation = flat" is validated for operator-valued processes.

#### Scenario: Nested and flat evaluation agree on matrix state

- **WHEN** an operator-valued process is evaluated nested and flat
- **THEN** the two results agree up to numerical tolerance, witnessing `encapsulation_flat` on the
  matrix-valued state channel
