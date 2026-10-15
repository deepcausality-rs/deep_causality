## MODIFIED Requirements

### Requirement: build() rejects the configurations that would answer unsoundly

`build()` SHALL reject an unfrozen graph, a cyclic structural candidate, a probe naming an
observable the plant does not expose, a zero shot count, and an empty candidate set, returning a
structured `QuantumError` and running no stage.

The unfrozen-graph rejection is the precondition `FactorSupports::from_graph` already enforces, and
`build()` enforces it for both graph bridges.

*As built.* `.over_plant(plant, observables)` takes the plant and the observables it exposes
together, so the observables' dimension is what `build()` checks the plant against. The candidate
kind is a type: `.candidates(&[..])` marks the plant `Structural` and `.mechanisms(&[..])` marks it
`Mechanisms`, and `build()` refuses a candidate of the other kind at runtime as well. A structural
candidate's cycle is read off its supports, since under the flat convention every leg of a node's
support that is itself a factor node is a parent; the model subject's cycle is read off its graph.

The cyclic rejection is a scope decision and SHALL be reported as `CyclicStructureUnsupported`.
Cyclic QCMs exist (Barrett, Lorenz & Oreshkov, arXiv:2002.12157), and the C₃ criterion does not
reject them: the crosstalk example's cyclic H₄ satisfies C₃-exclusion under Definition 3.1, and its
reachability on a cyclic graph is complete, which satisfies it more plainly still. So if a cyclic
candidate is to be kept out of `validate` and `control`, `build()` has to do it, by decision, before
any check runs, and the error has to name the scope limit rather than an obstruction.

#### Scenario: An unfrozen graph is rejected before any check runs

- **WHEN** `.over_model` is given a graph whose `is_frozen()` is `false`
- **THEN** `build()` returns `QuantumErrorEnum::CalculationError` naming the frozen-graph
  requirement, and no commutativity or decomposability check runs

#### Scenario: A cyclic structural candidate is rejected as scope, not as a C₃

- **WHEN** a config's structural candidates include one whose frozen graph contains a directed
  cycle
- **THEN** `build()` returns `QuantumErrorEnum::CyclicStructureUnsupported` naming the candidate, no
  check runs, and the message says cyclic causal structures are outside v1's scope rather than that
  the structure fails a criterion

#### Scenario: A cycle through an index above number_nodes is found

- **WHEN** the model subject's frozen graph has a removed causaloid at index 0 and a directed cycle
  through index 4 of five indices
- **THEN** `build()` returns `QuantumErrorEnum::CyclicStructureUnsupported`

#### Scenario: A vacuous or unreachable configuration is rejected

- **WHEN** a config names a zero shot count, or an empty candidate set, or a probe whose observable
  the plant does not expose
- **THEN** `build()` returns a structured `QuantumError` for that case and yields no config value
