<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A type alignment carries a channel and its section for every high-level type

`TypeAlignment` SHALL record, for each high-level type `X`, the list `π(X)` of low-level types, a
QC morphism `τ_X : π(X) → X` as a `QcMorphism` (a Kraus family, no Choi operator), and a section
`E_X : X → π(X)` with `τ_X ∘ E_X = id_X` checked against `Tolerance::state()`, refusing with
`QuantumError::SectionNotInverse` otherwise. Each entry SHALL name the side it applies to,
`AlignmentSide::{Any, Input, Output}`: a query's input and output types may align through different
morphisms, as they do for a code in the shape of Lorenz & Tull's Example 58, where the low-level
model is `encoder ; program`, its input type is the logical space aligned by the identity, and its
output type is the physical space aligned by the ideal decoder. Wires SHALL be disjoint among the
entries that can apply to one side.

Lorenz & Tull Definition 14 asks for an epic `τ_X`. In QC the witness of surjectivity is a section:
every high-level sharp state is `τ_X ∘ s` for the low-level sharp state `s = E_X ∘ (that state)`,
which is what Proposition 18 needs to derive an upward abstraction. For a code `E_X` is the
code-space isometry and `τ_X` the ideal decoding channel built from the stabilizer generators
`LogicalBasis` carries. On the exact path the section is the statement that `τ_X` inverts the
encoding on the code space. Products of types align as monoidal products of the `τ_X` (Definition
14, Eq. 14).

#### Scenario: A code's decoder and isometry align

- **WHEN** the alignment of `CodeAbstraction::numeric_abstractions` is built on the `[[8,2,2]]`
  fixture
- **THEN** `τ ∘ E` on the `4 × 4` logical space equals the identity within `Tolerance::state()`,
  the output-side entry's `π(logical qubits)` lists all eight physical qubits, the input-side entry
  aligns the two logical wires by the identity, and the alignment is admitted

#### Scenario: A channel that does not invert its section is refused

- **WHEN** a `τ_X` is supplied with an `E_X` such that `τ_X ∘ E_X` differs from the identity by more
  than the state tolerance
- **THEN** construction returns `SectionNotInverse` carrying the residual and the tolerance, and no
  `TypeAlignment` value is produced

#### Scenario: A non-CPTP `τ` is refused by the carrier

- **WHEN** a `τ_X` is given as a Kraus family whose Choi operator has a negative eigenvalue and is
  validated through the shipped `Channel::from_kraus` before `QcMorphism::from_channel`
- **THEN** `Channel::from_kraus` returns `NonCptpChannel` and the alignment is not built; a
  `QcMorphism` built from Kraus operators directly is not validated, and its `τ ∘ E = id` check is
  the alignment's own guard

#### Scenario: Sided entries answer their side only

- **WHEN** an alignment has an `Input` entry aligning high wire 0 with low wire 0 by the identity
  and an `Output` entry aligning high wire 0 with low wires 0 and 1 through a partial trace
- **THEN** the input-side `τ` for wire 0 has input dimension 2 and the output-side `τ` has input
  dimension 4, two `Input` entries sharing a wire are refused, and an unsided alignment answers
  both sides

### Requirement: Query signatures are validated at construction

`QuerySignature` SHALL admit `Io` (declared inputs to declared outputs), `Open(S)` (delete the
mechanisms of `S` and make them inputs), `Inc(S₁, …, Sₙ)` (interchange on pairwise disjoint,
each parallelisable, subsets of non-input vertices) and `Observe(O)` (measurement of the named
outputs), and SHALL refuse an `Inc` whose sets are not parallelisable with
`QuantumError::NotParallelisable` naming the directed path.

`Open(S)` is the paper's quantum generalisation of the abstract Do-query (§7.2) and is the
mechanism-level intervention v1 names `intervene_mechanism`; the wrapper adds a name and changes no
semantics. Parallelisable means no directed path between two members of one set (§7.2), which is a
reachability question on the low-level DAG and is decided before any semantics is asked for.

#### Scenario: An interchange on a chain is refused

- **WHEN** `Inc({X}, {Y})` is built on a DAG with the path `X → Y`
- **THEN** construction returns `NotParallelisable` naming `X → Y`, because the paper defines the
  query only on parallelisable sets

#### Scenario: Opening is the mechanism-level intervention

- **WHEN** `Open({X})` is given semantics on a `CircuitModel`
- **THEN** the box `c_X` is deleted, `X` becomes an input of the diagram, and the resulting channel
  equals the one `Hypothesis::intervene_mechanism` yields on the dilation with the factor at `X`
  replaced by the identity instrument, within Q-TOL

### Requirement: An abstraction is a downward abstraction, and upward ones are derived

`Abstraction<L, H>` SHALL consist of a `TypeAlignment` and a query map `π : Q_H → Q_L` sending each
high-level query to a low-level query of type `π(X) → π(Y)`, per Lorenz & Tull Definition 15, and
SHALL derive upward abstractions by composition with low-level sharp states per Proposition 18
rather than storing them.

#### Scenario: A query map is total on the declared signature

- **WHEN** an `Abstraction` is built with a signature of five high-level queries and a query map
  defined on four
- **THEN** construction returns `QuantumError::CalculationError` naming the unmapped query

#### Scenario: The concrete intervention is derived, not stored

- **WHEN** a caller asks for the upward abstraction of `Do(S = s)` on a downward abstraction whose
  signature contains `Open(S)`
- **THEN** the result is `Open(π(S))` composed with the low-level sharp state `E_S(s)`, computed on
  demand, and the abstraction value holds no field for it

### Requirement: The naturality check is a `Check<R>` that names its path, its norm and its factor

`check_naturality` SHALL evaluate, for every query `Q_H` in the declared signature, the two sides
`τ ∘ ⟦π(Q_H)⟧_L` and `⟦Q_H⟧_H ∘ τ` under the semantics path the query admits, SHALL report a
`CheckReport<R>` with one record per query carrying the residual, the threshold and the margin, and
SHALL carry beside the report the `SemanticsPath` that decided, the norm used, the
Frobenius-to-diamond amplification factor and the count of queries examined.

On the exact path the residual is zero or one: the images agree up to stabilizers through
`LogicalBasis::are_logically_equivalent` and the phase ratio is integral through
`DiagonalPhase::phase_at`, or they do not. On the numeric path the residual is the Frobenius norm
of the difference of the two composite Choi operators against `Tolerance::state()`, under the
entry-count cap of `qcl-circuit-model`. An abstraction whose squares commute to residual `ε` is an
**ε-abstraction**. That notion is this crate's definition; the paper gives the exact case, and the
doc block says so and cites it.

#### Scenario: The strict code abstraction is exact on the wide torus

- **WHEN** `check_naturality` runs on `CodeAbstraction` of the `[[18,2,3]]` torus for `Z̄`, `S̄`,
  `T̄` on each logical qubit and `H̄` on each
- **THEN** every record has residual zero, the report reads `SemanticsPath::Exact`, its examined
  count is the number of `(gate, logical qubit)` pairs, and no matrix was formed

#### Scenario: The two paths agree where both reach

- **WHEN** `check_naturality` runs on the `[[8,2,2]]` fixture of `square_torus(2)` for `Z̄`, `S̄`,
  `T̄` and `H̄` once under each path
- **THEN** both verdicts agree per query, the numeric residuals are below `Tolerance::state()`, and
  the numeric report names its amplification factor

#### Scenario: A vacuous signature is visible

- **WHEN** `check_naturality` runs on an abstraction whose signature is empty
- **THEN** the report's verdict is `Vacuous` with examined count zero, not `Accepted`

#### Scenario: A failing square names the query

- **WHEN** the low-level program for `S̄` is replaced by `Z̄`'s program in the query map
- **THEN** the record for `S̄` rejects, `first_rejection` names it, and the report's verdict is
  `Rejected`

### Requirement: The Frobenius proxy carries the two-sided diamond bound

The numeric path's report SHALL state, for a Frobenius residual `r` between the two Choi operators
of a square, the bounds `r / d_in ≤ ‖E − F‖_⋄ ≤ √(d_in d_out) · r`, the docstring SHALL carry the
derivation, and one test SHALL compare both ends against a channel pair whose diamond distance has
a closed form.

The derivation, for any linear map with the crate's unnormalised Choi operator `J`: every unit
vector on `X ⊗ X` is `(I ⊗ A)|Ω̃⟩` with `‖A‖_F = 1`, so `‖Φ‖_⋄ ≤ ‖J‖_1`; Cauchy–Schwarz on the
singular values gives `‖J‖_1 ≤ √(d_in d_out) · ‖J‖_F`; and `J / d_in = (Φ ⊗ id)(ω)` for the maximally
entangled state `ω` gives `‖J‖_F ≤ ‖J‖_1 ≤ d_in ‖Φ‖_⋄`. A Frobenius residual of zero therefore
certifies a diamond distance of zero.

#### Scenario: Both bounds hold on a closed-form pair

- **WHEN** two single-qubit unitary channels differing by a rotation by `θ` about `Z` are compared
- **THEN** the residual is `2√2 sin(θ/2)`, the upper bound `2r` is at least the closed-form
  diamond distance `2 sin(θ/2)` and the lower bound `r/2` is at most it, at every `θ` in a sweep
  over `(0, π]`

#### Scenario: A zero residual certifies zero distance

- **WHEN** the numeric path reports residual zero on a square
- **THEN** the report's lower bound reads zero and the diamond distance is stated as zero, not as
  "below the proxy"

### Requirement: The structural precheck computes Definition 49 and states what it licenses

`check_alignment_structure` SHALL compute, for each high-level vertex `X`, the set `α(X)` of
low-level vertices with a directed path to `π(X)` not passing through `π(Pa(X))`, SHALL decide the
predicates *simple*, *extra-simple* and *full* of Lorenz & Tull Definition 49, and SHALL report
`Equivalent` when both models are classical causal models and `Necessary` when the low-level model
is a `CircuitModel`.

Theorem 51 characterises mechanism-level abstraction by those predicates for causal models in a
Cartesian, Markov or cd structure category. Quantum compositional models of DAGs have no copy maps
(the paper's remark after Example 62), and the paper proves no characterisation for them. For a
quantum low-level model the precheck is a necessary condition by Remark 56: when `α(X)` meets the
low-level inputs, no normalised network diagram with inputs `π(Pa(X))` exists, so `π^S(c_X)` cannot
be defined. The stage runs in `validate` before any operator is formed, and a failure names the
offending pair of high-level vertices.

#### Scenario: Example 54 is simple and not extra-simple

- **WHEN** the paper's Example 54 is checked with `π :: X ↦ {X}, Y ↦ {Y}, W ↦ {W}`
- **THEN** the report reads `simple: true`, `extra_simple: false` with witness `(X, Y)` because
  `α(X) = {X, Z}` and `α(Y) = {Y, Z}` share `Z`, and `full: true`

#### Scenario: Example 54 with the alternative partition is extra-simple

- **WHEN** Example 54 is checked with `π :: X ↦ {X}, Y ↦ {Y}, W ↦ {W, Z}`
- **THEN** `α(N) = π(N)` for each of `X`, `Y`, `W`, and the report reads extra-simple and full

#### Scenario: Example 55 is not simple

- **WHEN** the paper's Example 55 is checked with `π :: X ↦ {X}, Y ↦ {Y, Y'}, Z ↦ {Z, Z'}`
- **THEN** the report reads `simple: false` with witness `(X, Y)`, because `X ∈ π(X) ∩ α(Y)`

#### Scenario: The quantum case says necessary

- **WHEN** the precheck runs with a `CircuitModel` on the low-level side
- **THEN** the report's scope reads `Necessary`, and its documentation states that Theorem 51 is
  proved for classical models and that the quantum case is a necessary condition by Remark 56

### Requirement: A CSS code is an abstraction, and the generation regression pins it to v1

`CodeAbstraction` SHALL build the strict `Abstraction<L, H>` of a CSS code from a `LogicalBasis`,
with `π` sending each logical qubit to its block, `τ` the ideal decoding channel, `E` the code-space
isometry, and the query map sending a logical gate to the program the Table 1 emitter produces and
`Open(S̄)` to `Open(π(S̄))`, whose fresh inputs align through the recovery. `Observe(Ō)`, the
measurement of the corresponding logical operator, needs a classical coarse-graining in the
alignment that this change does not carry and is deferred (design D16). For a noiseless physical
model, `check_naturality` on it SHALL agree with
`check_class_invariance` on every diagonal Table 1 gate and with `check_clifford_action` on `H̄`,
on every gate and every fixture, in verdict and in witness.

On the exact path the naturality check *is* those two predicates, so the regression pins a
reduction rather than comparing two independent computations. It is the first test written in this
group, and nothing about a noisy or composite abstraction is claimed until it passes.

#### Scenario: The regression holds on both torus fixtures

- **WHEN** `check_naturality` on `CodeAbstraction` and the v1 stages `check_class_invariance` and
  `check_clifford_action` run on `[[18,2,3]]` and `[[32,2,4]]` for every Table 1 gate the emitters
  produce
- **THEN** the verdicts agree per `(gate, logical qubit)`, and for every rejection the naturality
  witness names the same shift or the same Pauli image the v1 stage names

#### Scenario: A defect in either side breaks the regression

- **WHEN** the query map's program for `S̄` omits its CZ pairs
- **THEN** `check_naturality` rejects `S̄` where `check_class_invariance` on the correct emitter
  accepts, and the regression test fails on that gate

### Requirement: Abstractions compose under a law with two computed constants

`Abstraction::compose(self, next)` SHALL produce the composite abstraction with
`π = π₁ ∘ π₂` and `τ = τ₂ ∘ τ₁`, and its report SHALL carry the bound
`ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` with both constants computed as the Frobenius-induced norms
of the channels `τ₂` and `τ₁`, the largest singular value of each channel's `d_out² × d_in²` natural
representation obtained through the Gram matrix and the shipped `eigen_hermitian`, and recorded with
the norm in the report's provenance.

Post-composition acts on a Choi operator as `id ⊗ τ₂` and pre-composition as a transpose of `τ₁`
on the input factor; neither changes the induced norm on a Hilbert-space norm, so the constants are
the channels' own norms and the `(d_in d_out)²`-dimensional composition superoperators are never
formed.

Exact composition, `ε₁ = ε₂ = 0 ⇒ ε = 0`, is Lorenz & Tull Proposition 17 and is the Lean statement
in `lean/DeepCausalityFormal/Quantum/Abstraction.lean`, bound through `lean/THEOREM_MAP.md`. The
approximate law is this crate's theorem, elementary by the triangle inequality on the pasted
squares, with its derivation in the docstring. In the diamond norm both constants are one; in
Frobenius norm on Choi operators neither is in general. This composition replaces certificate
inheritance for callers who hold abstractions: each link is checked once and the residuals add
under the law, and `CertificateNotInherited` is never reached on this path.

#### Scenario: Exact links compose exactly

- **WHEN** two abstractions each with residual zero on the exact path compose
- **THEN** the composite's naturality check has residual zero on every query of the composed
  signature, which is the Rust witness of Proposition 17

#### Scenario: The bound is tight on a constructed case

- **WHEN** two numeric abstractions are constructed so that `‖τ₂‖_post > 1` and `‖τ₁‖_pre > 1`, with
  `ε₁, ε₂ > 0`
- **THEN** the composite's measured residual is at most the recorded bound, and a bound computed
  with either constant replaced by one is exceeded by the measured residual

#### Scenario: Provenance records the law

- **WHEN** a composite report is inspected
- **THEN** it carries `ε₁`, `ε₂`, both constants, the norm and the resulting bound, so a reader can
  recompute the bound from the record

### Requirement: Three consumers are abstraction chains

The change SHALL express three consumers as composites of `Abstraction` values: a concatenated
code (inner code abstraction composed with outer), code switching (into code A, identity on the
logical level, out of code B, with the switching gadget as the low-level query), and a distillation
round (noisy inputs to a logical magic state, a non-strict quantum-to-quantum abstraction). Each
SHALL report its composite bound and SHALL be labelled an example with checks, not a theorem.

The distillation round is the case the paper defers (§7.1); the example builds it and claims only
the residual it measures.

#### Scenario: The concatenated code's bound holds

- **WHEN** the hand-built `[[4,2,2]]` complex is concatenated with itself as inner and outer code
  and `check_naturality` runs on the composite for `Z̄` and `H̄`
- **THEN** the measured residual is at most the composite bound recorded by `compose`

#### Scenario: The opened square commutes on the small code

- **WHEN** `check_naturality` runs on the `[[4,2,2]]` code abstraction for `Z̄`, `X̄` and `CZ̄`
- **THEN** both the `Io` square and the `Open` square have residual zero, the opened low-level
  input type is the two logical wires and four fresh physical wires, and on `[[8,2,2]]` the opened
  square is refused by the default cap with `n: 10, k: 8, entries: 2^26` while the `Io` square
  alone is checked through `check_naturality_on`
