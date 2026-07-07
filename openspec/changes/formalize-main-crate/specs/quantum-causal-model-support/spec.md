## ADDED Requirements

### Requirement: CJ operators over tagged leg-sets

The formalization SHALL define a Choi–Jamiołkowski operator type `CJOp` over an ordered set of tagged Hilbert-space legs (each leg `H_i` with a primal/dual tag `H_i ⊗ H_i*`), with a leg-embedding (identity-padding) operation and a partial-trace (`traceOut`) operation, so that factors on different leg-sets can be multiplied in a common space and marginalized. `Quantum/CJOp.lean` MUST typecheck standalone with bare `lean`; each definition/theorem carries a `THEOREM_MAP.md` row. The Rust implementation of `CJOp` is deferred.

#### Scenario: Padding enables multiplication on a shared space

- **WHEN** two `CJOp`s on overlapping-but-distinct leg-sets are multiplied
- **THEN** the model embeds each into the common leg-space with identity padding before composing, matching the suppressed-identity convention `ρ_{B|A} ρ_{C|A} := (ρ_{B|A} ⊗ id_C)(ρ_{C|A} ⊗ id_B)`

#### Scenario: Partial trace is marginalization

- **WHEN** `traceOut` removes an output leg from a `CJOp`
- **THEN** it yields the marginal CJ operator (`ρ_{B|A} = Tr_C[ρ_{BC|A}]`)

### Requirement: No-influence and direct-cause predicates

The formalization SHALL define `NoInfluence U A D`, `DirectCause`, and the parent-set `Pa` derived from them, per Lorenz–Barrett Def 1 / Lorenz Def 4.1: `A ↛ D` iff `Tr_rest[ρ^U] = ρ^M_{D|·} ⊗ 1_{A*}` for some channel `M`; `DirectCause A D := ¬ NoInfluence U A D`; `Pa(D) := { A | DirectCause A D }`.

#### Scenario: No-influence factors identity onto the input leg

- **WHEN** `NoInfluence U A D` holds
- **THEN** the reduced operator carries `1_{A*}` on the `A` input leg (the marginal is independent of the `A` input)

### Requirement: Quantum Markov condition as factorize-and-commute

The formalization SHALL define `IsMarkov σ G := Factorizes σ G ∧ PairwiseCommute (factors σ G)`, where `Factorizes` is `σ = ∏_i ρ_{A_i | Pa(A_i)}` (identity-padded) and `PairwiseCommute` is `∀ i j, [ρ_i, ρ_j] = 0`. The parent sets `Pa(A_i)` SHALL be the same labeled (parent-index-keyed) sets exposed by the `comonoid-graph-join` substrate — one attachment surface for classical evaluation and quantum factorization, with overlapping parent sets expressible because contributions are keyed. `PairwiseCommute` plays for the operator product the role `schedule_invariance` plays for the classical fold: both state that the chosen linearization does not matter. The `n = 2` commutativity SHALL be provided as a derived lemma (from Hermiticity of the product); for `n ≥ 3` commutativity SHALL be an explicit axiom/obligation, since it does not follow from Hermiticity of `σ` when parental sets overlap.

#### Scenario: Two-factor commutativity is derived

- **WHEN** a two-factor factorization `ρ_{BC|A} = ρ_{B|A} ρ_{C|A}` is given
- **THEN** the model proves `[ρ_{B|A}, ρ_{C|A}] = 0` from Hermiticity (a lemma, not an axiom)

#### Scenario: Three-or-more-factor commutativity is an obligation

- **WHEN** `IsMarkov` is asserted for `n ≥ 3` overlapping parental sets
- **THEN** pairwise commutativity is an explicit hypothesis/obligation, not derived

### Requirement: Valid-process predicate independent in the cyclic case

The formalization SHALL define `ValidProcess σ` (positivity + normalization for all input CPTP maps) as a SEPARATE predicate. In the acyclic/unitary case, factorize-and-commute imply validity; in the cyclic case, validity is independent and MUST be a distinct requirement (a product of commuting channels need not be a valid process operator).

#### Scenario: Cyclic validity is not implied by commutativity

- **WHEN** a set of pairwise-commuting factors is given over a cyclic graph
- **THEN** `ValidProcess` is a separate check that may fail, and the model does not fold it into `IsMarkov`

### Requirement: Compatibility, obligation slots, and open hypotheses

The formalization SHALL state `Compatible G σ` (existence of a unitary dilation with uncorrelated product-state ancillas whose no-influence relations match `G`) and the theorem slot `Compatible G σ → IsMarkov σ G` (Barrett–Lorenz–Oreshkov), with the converse recorded as an open `Hypothesis`. It SHALL also state the obligation `traceOut_preserves_commute` (Layer-D: partial trace preserves pairwise commutativity under encapsulation) as the single genuinely-open quantum proposition. Hard proofs MAY be deferred (e.g. `sorry`-marked or stated as `axiom`/`Hypothesis`) provided each is explicitly labelled as an obligation, never silently assumed.

#### Scenario: Compatibility implies Markov is stated with a witness slot

- **WHEN** `Quantum/QCM.lean` is typechecked
- **THEN** `Compatible G σ → IsMarkov σ G` is present (proved or clearly marked as an obligation) and its converse is an explicit open `Hypothesis`

#### Scenario: The Layer-D obligation is explicit

- **WHEN** the partial-trace-preserves-commutativity property is referenced
- **THEN** it appears as a named obligation with no hidden `sorry` masquerading as a completed proof

### Requirement: Operator-valued state inherits the monad laws

The formalization SHALL keep the arity-5 state channel generic so that instantiating `State` with operator-valued (`CJOp`/matrix) payloads inherits the causal-monad laws from `Core/CausalMonad.lean` with no re-proof. A Rust witness SHALL exercise the monad-law property tests with a matrix-valued state payload (not only scalar state).

#### Scenario: Matrix-valued state threads lawfully

- **WHEN** the causal monad is instantiated with an operator-valued `State`
- **THEN** left/right identity and associativity hold by the generic `core.causal_monad.*` theorems, and the Rust witness confirms law-3 threading with a matrix payload
