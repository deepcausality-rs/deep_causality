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

### Requirement: Encapsulation-equals-flat exercised over matrix-valued state

The layer SHALL exercise the inherited monad associativity law (`core.causaloid.encapsulation_flat`)
over the arity-5 STATE channel carrying complex-MATRIX payloads, not only scalar state, so that
"encapsulation = flat" is validated for operator-valued processes.

#### Scenario: Nested and flat evaluation agree on matrix state

- **WHEN** an operator-valued process is evaluated nested and flat
- **THEN** the two results agree up to numerical tolerance, witnessing `encapsulation_flat` on the
  matrix-valued state channel
