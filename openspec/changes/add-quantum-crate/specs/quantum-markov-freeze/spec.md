## ADDED Requirements

### Requirement: The quantum Markov condition as a freeze-time commutativity check

The crate SHALL carry the operator-valued process (Choi–Jamiołkowski) factorization
`σ = ∏ ρ_{Aᵢ|Pa(Aᵢ)}` as an external node-keyed `ProcessFactors` store (consulted at the freeze
boundary, NOT on the runtime STATE channel — see the dedicated requirement below), and SHALL enforce
the quantum Markov condition (Lorenz 2022, Def 3.3) at the freeze boundary as a pairwise-commutativity check
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

### Requirement: σ is external node-keyed freeze-time decoration, not runtime state

The crate SHALL represent `σ = ∏ᵢ ρ_{Aᵢ|Pa(Aᵢ)}` as an external, node-index-keyed store
`ProcessFactors<R>` of factors `CjFactor<R> = CausalTensor<Complex<R>>` — the node-keyed operator
analogue of the edge-keyed `LambdaEdges<V>` — passed as a parameter to the freeze, and SHALL NOT carry
σ on the runtime arity-5 STATE channel. Keys SHALL be intrinsic graph node indices (never
enumeration/insertion order); an absent key SHALL denote a node with no operator factor. Each factor's
Hilbert support SHALL be derived from the frozen graph as `{Aᵢ} ∪ Pa(Aᵢ)` via `inbound_edges(Aᵢ)`.

#### Scenario: σ never rides the runtime state channel

- **WHEN** a quantum-Markov model is evaluated after a successful freeze
- **THEN** forward evaluation reads no operator factor from the STATE channel (PS remains the model's
  ordinary runtime state) and the factors are consulted only by the freeze-boundary check

### Requirement: The Markov check runs as the existing freeze hook, surfacing QuantumError

The crate SHALL enforce `[ρ_j, ρ_k] = 0` by invoking `freeze_verified_with_check(state_writers, hook)`
where `hook` is a closure that CAPTURES the external `ProcessFactors<R>` / support registry, computing
commutators only for factor pairs whose supports intersect. Because the hook returns
`Result<(), CausalityGraphError>`, the crate SHALL bridge its `QuantumError` through that channel via
`impl From<QuantumError> for CausalityGraphError` (orphan-rule-legal; `QuantumError` is crate-local),
preserving the offending operator pair in the message, and SHALL expose a public `freeze_quantum`
wrapper returning `Result<(), QuantumError>` so callers receive the structured error. A hook failure
SHALL roll the graph back to the dynamic state (the built-in `unfreeze` on error).

#### Scenario: A non-commuting pair aborts the freeze and is named

- **WHEN** two shared-support factors have `‖[ρ_j, ρ_k]‖ > ε` and the model is frozen via `freeze_quantum`
- **THEN** it returns `Err(QuantumError)` naming `j` and `k`, and `is_frozen()` is false afterward

#### Scenario: Disjoint-support factors impose no obligation

- **WHEN** two factors have non-intersecting Hilbert supports
- **THEN** no commutator is computed for that pair and it never blocks the freeze

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
