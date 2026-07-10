## ADDED Requirements

### Requirement: The quantum Markov condition as a freeze-time commutativity check

The crate SHALL carry the operator-valued process (Choi–Jamiołkowski) state on the arity-5 STATE
channel as hyperedge parent-set factors `σ = ∏ ρ_{Aᵢ|Pa(Aᵢ)}`, and SHALL enforce the quantum Markov
condition (Lorenz 2022, Def 3.3) at the freeze boundary as a pairwise-commutativity check
`[ρ_j, ρ_k] = 0` over the operators sharing a Hilbert support, aborting the freeze when a required
commutator is non-zero. The check SHALL be sound (it never accepts a non-commuting model) and MAY be
incomplete (it may reject a valid model whose nesting and commutation conflict — see the conditional
`partial_trace_preservation`).

#### Scenario: A commuting model freezes; a non-commuting one aborts

- **WHEN** a simulated-CJ process with pairwise-commuting factors is frozen, and separately one with a
  non-zero required commutator is frozen
- **THEN** the first freezes and evaluates, and the second aborts at freeze with an error naming the
  offending operator pair

#### Scenario: The check is confined to the freeze boundary

- **WHEN** forward evaluation runs
- **THEN** no commutativity check runs during the directional `bind` (the global Markov property is
  a whole-graph fact checked only at freeze), matching the existing `freeze_dag` model

### Requirement: Depth-aware numerical tolerance for the commutator test

The commutativity check SHALL use a near-zero tolerance that scales with the conditioning accumulated
through encapsulation depth (iterated partial trace), so that deeply nested valid models do not
spuriously abort, and the freeze SHALL be instrumented to record which structural patterns fail across
a test battery.

#### Scenario: Nested valid models do not spuriously abort

- **WHEN** a deeply nested but valid commuting model is frozen
- **THEN** the depth-aware tolerance admits it, and the instrumented freeze records no failure for that
  pattern

### Requirement: A C₃-exclusion faithfulness check at freeze

The freeze SHALL enforce the traditional-circuit faithfulness criterion of van der Lugt & Lorenz
(arXiv:2508.11762, Thm 3.2): a declared causal structure `G` is faithfully representable in the
crate's (non-routed) wiring iff it has the **C₃-exclusion property** (no `C₃` sub-relation between
three inputs and three outputs). A `G` that contains a `C₃` (canonically two commuting CNOTs) SHALL be
rejected at freeze with a `QuantumError`, never silently mis-represented, because such a structure
provably has no traditional-circuit causally faithful decomposition. The general routed/direct-sum
faithfulness case remains out of scope (open upstream).

#### Scenario: A C₃-containing structure is rejected

- **WHEN** a model whose causal structure contains a `C₃` sub-relation is frozen
- **THEN** the freeze aborts with a `QuantumError` identifying the C₃ obstruction, while a
  C₃-exclusion structure freezes and is treated as faithfully representable

### Requirement: The environmental preparation is immutable context

Residual environmental quantum data (the Bell-preparation `ρ_A`) SHALL be carried as an immutable
context handle whose write methods are unreachable, keeping the simulated-CJ model deterministic and
in the verifiable region.

#### Scenario: The preparation cannot be mutated mid-pass

- **WHEN** a model threads `ρ_A` through evaluation
- **THEN** `ρ_A` is read-only for the duration of the pass and the model's result is reproducible
