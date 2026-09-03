<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: A structural hypothesis is a factorization with a derived causal structure

A structural candidate SHALL be the triple `{ name, ProcessFactors<R>, FactorSupports }`, and its
`CausalStructure` SHALL be derived on demand rather than stored beside it.

The factor store, the support registry and the structure derivation all ship today:
`ProcessFactors<R>` in `qcm/process_factors.rs` keys `CjFactor<R>` by intrinsic graph node index,
`FactorSupports` records each node's ascending leg-ids with their Hilbert dimensions, and
`CausalStructure::from_graph_reachability` in `qcm/faithfulness.rs` computes the bipartite influence
relation from a frozen graph's `contains_edge` adjacency over caller-declared input and output
systems. `Hypothesis` adds the name and the pairing. It adds no second copy of the structure, so a
stored structure can never disagree with the factorization it is supposed to describe.

Mechanism candidates are the other constructor. A mechanism declares no factorization, so it exposes
no derived structure and reaches `control` without passing through the structural checks.

*As built.* The structure is derived two ways, and neither is stored. `structure(graph, inputs,
outputs)` reads a frozen graph's reachability, for the model subject. `structure_from_supports`
reads the supports alone: under the flat convention a leg of a node's support that is itself a
factor node is a parent, so the supports carry the DAG and its reachability, and a structural
candidate can be screened for decomposability with no graph in sight. The crosstalk consumer needs
the second, because each of its candidates implies a structure of its own.

#### Scenario: A structural candidate carries factors and supports, and no structure field

- **WHEN** `Hypothesis::structural("Q1->Q2", factors, supports)` is constructed
- **THEN** the value holds the name, the `ProcessFactors<R>` store and the `FactorSupports`
  registry, and it exposes no `CausalStructure` field; asking for the structure runs
  `CausalStructure::from_graph_reachability` against the declared systems each time

#### Scenario: The derived structure feeds the decomposability gate

- **WHEN** `check_decomposable` runs on a structural candidate
- **THEN** it calls `check_c3_exclusion` on the freshly derived `CausalStructure`, and a candidate
  whose reachability contains a C₃ is rejected with `QuantumError::NotFaithfullyRepresentable`
  naming the three witnessing inputs and three witnessing outputs

#### Scenario: A mechanism candidate has no structure to derive

- **WHEN** `Hypothesis::mechanism("amplitude", amp_drift)` is constructed
- **THEN** it carries no `ProcessFactors<R>`, the structure accessor is unavailable on it, and the
  structural checks are not applicable to it

### Requirement: The C₃ criterion is Definition 3.1, and the stage is named for what it decides

`check_c3_exclusion` SHALL decide the C₃-exclusion property of van der Lugt & Lorenz
(arXiv:2508.11762, Definition 3.1), where `C₃` is the causal structure of Example 2.12's two
commuting CNOTs: seven edges of nine, with the two missing input–output pairs sharing neither an
input nor an output. The QCL stage over it SHALL be named `check_decomposable`, and SHALL NOT be
named `check_faithfulness`.

A 3×3 induced block is `C₃` exactly when its sorted row degrees and sorted column degrees are both
`[2, 2, 3]`. Seven edges force the row degrees to be `{3, 2, 2}` or `{3, 3, 1}`, and only the first
leaves the two non-edges in distinct rows; likewise the columns. Over all 512 relations on three
inputs and three outputs that test agrees with isomorphism to `C₃` and with the paper's
Theorem 4.9(v) on every one. The bipartite 6-cycle, `K₃,₃` minus a perfect matching, has six edges
and every degree two; it is not `C₃`, and Theorem 4.9(v) admits it because any two of its outputs
share exactly one parent. The shipped check tested for the 6-cycle and answered wrongly on both
canonical cases; the publication is the reference and the code follows it.

"Faithful" in the paper is Lorenz–Barrett's `G_U = G_C`, a circuit decomposition whose
connectivity equals the unitary's causal structure (Definition 2.11). It is not Pearl's, where a
distribution has no independences beyond those its graph implies, and a practitioner of causal
models reads the second. `NotFaithfullyRepresentable` MAY keep its name, and its doc comment SHALL
say which sense it carries.

#### Scenario: Two commuting CNOTs are rejected

- **WHEN** the relation on inputs `{1, 2, 3}` and outputs `{1, 2, 3}` with every pair influencing
  except `(1, 3)` and `(3, 1)` is checked
- **THEN** `find_c3` returns the witness `([1, 2, 3], [1, 2, 3])` and `check_c3_exclusion` returns
  `NotFaithfullyRepresentable`, because this is Example 2.12's `C₃` and Theorem 3.2 says no unitary
  causally faithful decomposition is implied

#### Scenario: The 6-cycle is accepted

- **WHEN** the relation with every input influencing exactly two outputs and every output influenced
  by exactly two inputs is checked
- **THEN** `find_c3` returns `None` and `check_c3_exclusion` returns `Ok(())`, because six edges are
  not seven and Theorem 4.9(v) admits the relation

#### Scenario: The search agrees with Theorem 4.9(v) on every small relation

- **WHEN** every relation on three inputs and three outputs is enumerated
- **THEN** `find_c3` fires on exactly the eighteen labelled copies of `C₃`, and on each relation its
  verdict matches the paper's condition (v): for all outputs `b₁, b₂, b₃`, the parent sets of
  `{b₁, b₂}` and `{b₂, b₃}` are disjoint or nested

### Requirement: A hypothesis is admitted only when its supports validate against its factors

Construction of a structural `Hypothesis` SHALL run `FactorSupports::validate(&factors)` and SHALL
return the shipped `QuantumError` on failure, and construction from a graph SHALL reject a graph that
is not frozen.

`validate` already checks that every factor is a square, non-empty matrix whose dimension equals the
product of its declared support leg dimensions, and that no factor is keyed on a node with no
declared support. `FactorSupports::from_graph` already rejects an unfrozen graph and a factor keyed
past `number_nodes()`. `Hypothesis` calls these rather than repeating them.

#### Scenario: A factor whose dimension disagrees with its support is rejected

- **WHEN** node 2 carries a 4×4 factor while its declared support is the single leg `[2]` of
  dimension 2
- **THEN** construction fails with `QuantumError::DimensionMismatch` reporting that the factor at
  node 2 has dim 4 while its support implies 2, and no `Hypothesis` value is produced

#### Scenario: An unfrozen graph yields no hypothesis

- **WHEN** a structural candidate is built from a graph that has not been frozen
- **THEN** `FactorSupports::from_graph` returns `QuantumError::CalculationError` stating that a
  frozen graph with dense node ids is required, and the failure propagates out of construction

#### Scenario: A factor keyed outside the graph is rejected

- **WHEN** the factor store carries a factor keyed by node 9 and the frozen graph has 4 nodes
- **THEN** construction fails with `QuantumError::CalculationError` naming node 9 and the valid id
  range, rather than declaring node 9 as a lone qubit detached from the graph

### Requirement: `intervene_mechanism` is a keyed factor replacement followed by revalidation

`intervene_mechanism(do(node ← factor))` SHALL replace exactly the named node's entry through
`ProcessFactors::insert` and SHALL then re-run `FactorSupports::validate` over the whole store,
returning the revalidated hypothesis or the shipped error. The operation SHALL carry the
`_mechanism` suffix, and v1 SHALL NOT expose an unqualified `intervene`.

This is Pearl's cut expressed against a store that is already keyed for it: the surgery touches one
key and leaves every other factor identical. The revalidation is what makes the cut safe, because a
replacement factor of the wrong dimension would otherwise sit in a store that `quantum_markov_check`
embeds through `embed_on_legs`.

The suffix is load-bearing because a QCM has two interventions and they differ. The factor
`ρ_{A|Pa(A)}` is the *mechanism* delivering A's input from its parents' outputs, and replacing it is
the mechanism-level `do()`, the classical analogue. Barrett–Lorenz–Oreshkov's canonical intervention
fixes the *instrument* at the node, what happens between A's input and A's output. `predict` differs
under the two. v1 supplies only the first and models a probe as a mechanism replacement;
`intervene_instrument(node, instrument)` is the name reserved for the second, and it is not built.

#### Scenario: A probe is a mechanism replacement, and the model says so

- **WHEN** the calibration or crosstalk consumer applies a probe to a structural hypothesis
- **THEN** the probe is applied through `intervene_mechanism`, and the hypothesis's documentation
  states that the probe is modelled as a factor replacement rather than as an instrument at the node

#### Scenario: The cut touches one key

- **WHEN** `intervene` replaces the factor at node 3 of a store keyed `{0, 1, 2, 3}`
- **THEN** `get(3)` returns the new factor, `get(0)`, `get(1)` and `get(2)` return factors equal to
  the originals, `len()` is unchanged at 4, and `FactorSupports::validate` returns `Ok(())`

#### Scenario: A replacement of the wrong dimension fails the cut

- **WHEN** `intervene` supplies an 8×8 factor for a node whose declared support implies dimension 4
- **THEN** revalidation returns `QuantumError::DimensionMismatch` naming the node, the intervened
  hypothesis is not produced, and the hypothesis passed in is left unchanged

#### Scenario: Intervening on an undeclared node fails the cut

- **WHEN** `intervene` names a node that carries no declared support in the registry
- **THEN** revalidation returns `QuantumError::DimensionMismatch` stating that the node has a factor
  but no declared support, and no intervened hypothesis is produced

### Requirement: An intervention invalidates the report gathered before it

A `QuantumMarkovReport<R>` obtained before an intervention SHALL NOT be carried onto the intervened
hypothesis, and a decision about the intervened factorization SHALL come from a check re-run against
it.

`quantum_markov_check` measures `‖[ρ_j, ρ_k]‖_F` pair by pair over the factors it is given, so the
margins in a report describe the store as it stood. Replacing a factor changes the operators the
margins were computed from. A `Screened<R>` whose factorization is later intervened on therefore
carries an invalidated report.

#### Scenario: The intervened hypothesis starts without a report

- **WHEN** a screened hypothesis carrying a `QuantumMarkovReport<R>` with `tested_pairs() == 6` is
  intervened on
- **THEN** the resulting hypothesis carries no report, and reaching a stage that needs one requires
  running the check again

#### Scenario: The re-run reports its own count

- **WHEN** `quantum_markov_check` is re-run on the intervened factorization
- **THEN** the returned report exposes its own `tested_pairs()` and `worst_margin()`, and a
  factorization whose replacement factor overlaps nothing reports `tested_pairs() == 0`, which is a
  vacuous pass and is visible as one

### Requirement: `predict` is model evaluation over the forked worlds

`predict` SHALL evolve each forked world under the chosen probe, applied as a mechanism-level
intervention through `intervene_mechanism`, SHALL count each evaluation on the ledger's
`predictions` field, and SHALL leave `shots`, `experiments` and `device_time` untouched.

The contraction it needs ships. `FactorSupports::space_map` returns the leg-to-dimension map for the
union of the supports involved, `embed_on_legs` lifts each factor onto that space as the identity
elsewhere, and the product of the embedded factors is the joint operator the probe reads. The whole
evaluation stays on the full union of legs, so it invokes no partial trace and needs no warrant.

#### Scenario: Predictions are counted, never billed

- **WHEN** `predict` runs over a fork of three hypothesis worlds
- **THEN** each world's ledger advances `predictions` by one through `NaturalNumber::succ`, and
  `shots`, `experiments` and `device_time` hold the values they had before the stage

#### Scenario: The joint operator is built by embedding, not by tracing

- **WHEN** `predict` evaluates a world whose factors are supported on legs `{0, 1}` and `{1, 2}`,
  each leg of dimension 2
- **THEN** it calls `space_map` on the union `{0, 1, 2}`, embeds both factors with `embed_on_legs`
  onto the resulting 8×8 space, multiplies them there, and makes no call to `partial_trace`

### Requirement: Marginalising a validated factorization is a separate operation gated on the boundary warrant

Reducing a validated factorization onto fewer legs SHALL be an operation distinct from `predict`, it
SHALL obtain a `BoundaryWarrant<R>` from `partial_trace_preservation_boundary` before any call to
`partial_trace`, and it SHALL refuse when the warrant does not hold.

`quantum.partial_trace_preservation` is false, refuted in Lean by two commuting operators whose
partial traces have commutator `[[0, 4], [−4, 0]]`. The sound path is the unconditional transport
identity plus the contraction `‖Tr_B(E)‖_F ≤ √(d_B)·‖E‖_F`, which
`partial_trace_preservation_boundary` reports as `hypothesis_residual`, `tolerance`,
`amplification`, `conclusion_bound` and `holds`. Marginalising a certified factorization without
that warrant produces a model whose Markov property the certificate no longer stands for, and it is
forbidden.

#### Scenario: A failed warrant refuses the marginalisation

- **WHEN** the returned `BoundaryWarrant<R>` has `holds == false`, its `hypothesis_residual`
  exceeding the named `tolerance`
- **THEN** the operation returns an error carrying the warrant, and no traced operator is produced

#### Scenario: A held warrant travels with the marginalised model, degraded by its amplification

- **WHEN** the warrant holds for a trace over a factor of dimension `d_B == 4`
- **THEN** the traced operator is returned together with the warrant, `amplification` reads `2`,
  `conclusion_bound` reads `2 · hypothesis_residual`, and any Markov margin carried onto the
  marginalised model is degraded by `amplification` or the report is marked invalidated

#### Scenario: The boundary form is built rather than asserted

- **WHEN** a caller marginalises with a kept-factor operator `Z` and the dimensions `[d_A, d_B]`
- **THEN** the operation passes `Z` to `partial_trace_preservation_boundary`, which constructs
  `Z ⊗ 1_B` itself, so the theorem's shape hypothesis holds by construction and the entry point
  exposes no path that traces a validated factorization without a warrant

### Requirement: Hypothesis bookkeeping separates the real axis from the count axis

A hypothesis and the operations over it SHALL carry real quantities on the scalar parameter and
counting quantities on `NaturalNumber`, and SHALL NOT mix them. The factor operators, the boundary
warrant's residual and its amplified bound are real and follow `FloatType`. The node keys, the leg
identifiers, the per-leg dimensions and the prediction count are ℕ and follow `NumberType`.

A leg dimension SHALL be treated as a count rather than as a measurement: `space_map`'s dimensions
multiply into a total that can overflow, and the shipped `FactorSupports::validate` is
overflow-checked for that reason. The same discipline SHALL apply to any dimension arithmetic QCL
adds.

#### Scenario: The boundary warrant's bound follows the scalar

- **WHEN** a marginalisation is gated and the warrant returns its residual and the `√(d_B)` factor
- **THEN** both are carried on the run's scalar, and the amplified bound re-types when `FloatType`
  changes

#### Scenario: A dimension product that would overflow is reported

- **WHEN** declared leg dimensions multiply to a total exceeding the integer width
- **THEN** validation reports the overflow as a typed error rather than wrapping, because a count
  has no `epsilon()` and a wrapped dimension would silently mis-shape every factor

### Requirement: A composed factorization inherits a Markov certificate only at two factors

A `CheckReport<R>` certifying the quantum Markov condition SHALL NOT transfer to a composite
factorization of more than two factors. Composing two validated parts and carrying the parts'
certificates forward SHALL be rejected, and the composite SHALL re-run `check_markov`.

**The literature leaves this open, and QCL claims no more than the literature does.** Def 3.3 of the
quantum Markov condition requires the factors to commute pairwise, and Lorenz (2022) records at
footnote 11 that while pairwise commutation is a simple consequence of hermiticity in the two-factor
case, *commutativity of the factors does not follow from the hermiticity of `σ` when there are more
than two factors*. Lorenz & Barrett (arXiv:2001.07774) leave the general case to future work,
naming the missing ingredient as results in operator algebra "pertaining to sets of three or more
pairwise commuting algebras". An inheritance rule beyond two factors would assert that theorem.

**At exactly two factors the inheritance is sound and SHALL be available as a fast path**, because
there the commutation relation follows from the hermiticity of the composite. A composite of two
factors MAY carry the certificate forward; every other arity re-checks.

**Composition over a shared wire is the marginalisation of the shared node**, so the F9 guard
applies to it. `choi_compose` is a double contraction over the shared wire, verified to a relative
Frobenius residual of 3.198e-16 over 500 random CPTP pairs, and that figure establishes that it
computes the right *channel*, not that it preserves the *factorization*. The sum over the shared
indices is the partial trace whether or not a function of that name runs. What makes composition
sound in v1 is this requirement: the certificate is re-derived, not inherited.

**A failed re-check on inherited factors SHALL be reported as `CertificateNotInherited`**, distinct
from `CommutatorNonZero`, and the report SHALL carry the factorization's provenance as `Inherited` or
`Rederived`. Barrett–Lorenz–Oreshkov's representation theorem is an existence statement: a composite
of two QCM-representable parts always has a Markov factorization for the induced DAG with the
induced factors. It need not be Markov for the naive product of the parts' factors, which is what
the re-check tests. The failure therefore says "this factorization does not certify the composite",
and the message SHALL say that a Markov factorization for the composite may exist under a different
factor assignment. Constructing that induced factorization is not built in v1.

#### Scenario: A three-factor composite re-runs the check

- **WHEN** two validated factorizations compose into a factorization of three or more factors
- **THEN** the parts' reports do not transfer, `check_markov` runs on the composite, and the
  composite's own report is what `Screened<R>` carries

#### Scenario: A two-factor composite may carry its certificate

- **WHEN** a composite has exactly two factors and both parts validated
- **THEN** the certificate transfers without re-running the check, and the report records that it
  was inherited under the two-factor rule rather than measured on the composite

#### Scenario: An inherited certificate is distinguishable from a measured one

- **WHEN** a caller inspects a `Screened<R>` whose certificate was inherited
- **THEN** the report says so, so that a reader cannot mistake an inherited certificate for pairs
  actually tested on the composite

#### Scenario: A failed re-check on inherited factors is a certificate failure, not a physics one

- **WHEN** a three-factor composite is re-checked on the parts' inherited `ProcessFactors` and a
  pair fails to commute
- **THEN** the error is `CertificateNotInherited` naming the pair, the report's provenance reads
  `Inherited`, and the message says a Markov factorization for the composite may exist under a
  different factor assignment, so a sound model is not rejected with a message that reads as physics

#### Scenario: A failure on rederived factors is the physics failure

- **WHEN** a factorization whose provenance is `Rederived` fails a pair
- **THEN** the error is `CommutatorNonZero` as shipped, because the factors under test are the
  model's own
