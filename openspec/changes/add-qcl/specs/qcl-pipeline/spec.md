<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Configuration has one origin naming two working types

Every QCL configuration SHALL originate at `QclBuilder::config::<FloatType, NumberType>()`, the single
site where the two working types are named.

`FloatType` buys accuracy. Every tolerance in a run is discharged from it, as the shipped policies
already are: `CommutatorTolerance::threshold` derives Q-TOL from `R::epsilon()`,
`Projection::default_tolerance` from `√ε`, and `DensityMatrix::default_tolerance` from the scalar.
`IntType` buys headroom against overflow and moves no threshold, so it names the width of the
`Ledger<R, N>` counts under `N: NaturalNumber` and nothing else. No stage constructs a tolerance
from a hardcoded scalar, and no count field carries a hardcoded width.

#### Scenario: The accuracy parameter retypes the thresholds

- **WHEN** the same pipeline is built once at `config::<f64, u64>()` and once at
  `config::<Float106, u64>()`, with no other change to the program
- **THEN** both runs compile, every acceptance threshold in both is derived from `R::epsilon()`, and
  the `Float106` run's thresholds are tighter

#### Scenario: The headroom parameter moves no threshold

- **WHEN** `NumberType` changes from `u32` to `u64` at the config site
- **THEN** the `Ledger<R, N>` count fields retype and every acceptance threshold is unchanged,
  because overflow has no `epsilon()` to derive a tolerance from

#### Scenario: A stage's parameter belongs to the stage

- **WHEN** a program calls `.gate(spec)`, `.design(objective)` and `.evolve(n)` on a config that
  names no spec, no objective and no depth
- **THEN** `build()` succeeds, and the spec, the objective and the depth are arguments of their own
  stages

### Requirement: The config branches on the subject through three constructors

A configuration SHALL name exactly one subject, through `.over_plant(plant)`,
`.over_model(graph, factors, supports)` or `.over_code(complex)`.

`.over_plant` takes a system that evolves and is measured; its candidates are `Hypothesis` values,
and it reaches `control` in every case and `validate` when its candidates are structural.
`.over_model` takes a Choi–Jamiołkowski factorization over a frozen graph and is the degenerate case
of `.over_plant` with one structural candidate and no evidence; it is what the shipped
`freeze_quantum` callers map onto. `.over_code` takes a chain complex, evaluated exactly, and
carries no probe family.

#### Scenario: The model subject maps onto the shipped freeze

- **WHEN** a config built with `.over_model(graph, factors, supports)` and `.declare_systems(&inputs,
  &outputs)` runs `validate().check_markov().check_decomposable()`
- **THEN** the two checks are the two level checks `freeze_quantum` runs inside
  `freeze_verified_with_check`, the commutativity check over intersecting supports and C₃-exclusion
  over the frozen graph's reachability, with no probe family and no evidence policy in play

#### Scenario: A plant with mechanism candidates reaches control directly

- **WHEN** a config built with `.over_plant(transmon)`, `Evidence::shots(1024).seed(20260821)` and
  candidates constructed by `Hypothesis::mechanism` is passed to `QclBuilder::control`
- **THEN** it typechecks without a `Screened<R>`, because mechanism candidates carry no structural
  claim for `validate` to screen

#### Scenario: The code subject has validate stages only

- **WHEN** a config built with `.over_code(LatticeComplex::<2, FloatType>::square_torus(4))` is built
- **THEN** `validate` offers `derive_code`, `check_ldpc_weights` and `check_class_invariance`, and
  the config holds no probes and no baseline experiment

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
  requirement, and no commutativity or decomposability check runs, because `remove_node` tombstones
  a slot without compacting, so a live node can hold an id past `number_nodes()` and its parent
  edges would be dropped silently in the C₃ gate

#### Scenario: A cyclic structural candidate is rejected as scope, not as a C₃

- **WHEN** a config's structural candidates include one whose frozen graph contains a directed
  cycle
- **THEN** `build()` returns `QuantumErrorEnum::CyclicStructureUnsupported` naming the candidate, no
  check runs, and the message says cyclic causal structures are outside v1's scope rather than that
  the structure fails a criterion

#### Scenario: A vacuous or unreachable configuration is rejected

- **WHEN** a config names a zero shot count, or an empty candidate set, or a probe whose observable
  the plant does not expose
- **THEN** `build()` returns a structured `QuantumError` for that case and yields no config value

### Requirement: validate terminates in Screened and control requires one

`validate` SHALL terminate in a `Screened<R>` carrying the config, the admitted candidate subset and
the `CheckReport<R>`, and `control` SHALL accept a plant config or a `Screened<R>`.

A config carrying structural candidates therefore has no path into `control` that skips validation.

*As built.* `QclBuilder::control` takes any `ControlSource`, and the two implementations are
`&Config<.., PlantSubject<.., Mechanisms>>` and `&Screened<.., PlantSubject<.., Structural>>`. A
structural config has no implementation, which is the compiler refusing the skipped screen. The
structural plant's `check_decomposable(inputs, outputs)` screens each candidate on the structure
its own supports encode, and `check_decomposable_with(graph, ..)` is there for candidates whose
structure lives in a graph.

#### Scenario: Structural candidates cannot enter control unscreened

- **WHEN** a config whose candidates are built by `Hypothesis::structural` is passed to
  `QclBuilder::control` without running `validate`
- **THEN** the program fails to compile, and `QclBuilder::control(screened)` on the `Screened<R>`
  returned by `validate(&cfg).check_markov().check_decomposable().finalize()` compiles

#### Scenario: A vacuous pass is visible in the screened report

- **WHEN** `validate` runs `check_markov` over a factorization whose factor supports are pairwise
  disjoint
- **THEN** the `Screened<R>` admits the candidate and its `CheckReport<R>` records zero pairs
  examined, matching `QuantumMarkovReport::tested_pairs()` on the same input

#### Scenario: Marginalisation invalidates the screened report

- **WHEN** the factorization inside a `Screened<R>` is marginalised through
  `partial_trace_preservation_boundary`
- **THEN** the report either is marked invalidated or carries the `√(d_B)` amplification from the
  returned `BoundaryWarrant`, and the pre-trace margins are unreadable as current margins

### Requirement: Failure is transactional and carries the structured error

A failing stage SHALL leave the subject in its pre-stage state and SHALL carry the structured
`QuantumError` out, rather than a `Display` rendering of it.

This is the behaviour `freeze_quantum` already has: the hook returns `CausalityGraphError`, so the
structured error is recovered from a `RefCell` stash across that bridge, and the graph rolls back to
its dynamic state on failure.

*As built.* The model subject of a configuration takes a frozen graph, because `build()` requires
one, and `validate` mutates nothing: its checks run on the frozen graph as it stands, so a failed
validation leaves the subject exactly as built and the structured error is what `finalize`
returns. The rollback scenario below is the shipped freeze's behaviour, reached through
`QclBuilder::freeze_model` on a dynamic graph; the two entry points serve the two starting states.
The pipeline module is compiled under the `qcm` feature, because candidates are `Hypothesis` values
and the model subject reaches the causal graph; bare-metal reach for the plant path is a follow-up
that splits the graph-dependent half of `Hypothesis` from the rest.

#### Scenario: A rejected pair rolls the graph back and names itself

- **WHEN** `validate` runs `check_markov` over a factorization holding one pair whose commutator
  exceeds the Q-TOL threshold
- **THEN** the returned error is `QuantumErrorEnum::CommutatorNonZero { node_j, node_k, detail }`
  naming that pair, `graph.is_frozen()` is `false` afterwards, and no part of the graph is left
  frozen

#### Scenario: A built-in freeze failure stays distinguishable

- **WHEN** the graph fails the built-in acyclicity or single-writer check before the quantum hook
  runs
- **THEN** the returned error is `QuantumErrorEnum::CalculationError` carrying the graph error's
  message, and it is distinguishable from a quantum-check rejection

### Requirement: The ledger counts on NaturalNumber and holds three invariants

The threaded state SHALL be `Ledger<R, N>` with `shots`, `experiments` and `predictions` bound on
`N: NaturalNumber` and `device_time`, `cost` and `bits` on the real scalar `R`.

Its three invariants: `observe` is the only stage that touches `shots`, `experiments` and
`device_time`; `fork` is QCL's rather than core's; forked ledgers are compared rather than joined
under ∇, because at a counterfactual fork exactly one branch was factual.

#### Scenario: The draw-down is checked arithmetic rather than a guard

- **WHEN** a stage draws more shots than the budget holds
- **THEN** `NaturalNumber::checked_difference` returns `None` and the draw-down reports an overdraft,
  with `monus` available where a floor at zero is the wanted answer, and no tolerance is applied to a
  count

#### Scenario: Only the device boundary increments the device fields

- **WHEN** a fork into three hypothesis worlds runs one hardware experiment and two model evaluations
- **THEN** `experiments` advances by one, `predictions` advances by two, and `shots` and
  `device_time` change only inside `observe`

#### Scenario: The ledger is a copyable monad state

- **WHEN** `Ledger<R, N>` is threaded as the `State` of the causal monad
- **THEN** it is `Copy`, holds no `Vec` and no `String`, and its `Default` is hand-written from
  `R::zero()` and `N::zero()`, because the `CausalMonad` implementation requires `State: Default` and
  a derived `Default` would impose `R: Default`

### Requirement: fork is built above core, because a counterfactual fork is a product

`fork` SHALL produce one live world per candidate, each holding its own cloned `Ledger<R, N>` and
no read-out and no verdict, SHALL fail with `CalculationError` when there is no candidate to fork,
and SHALL NOT be built on `Either` or `CausalFlow::either`.

`Either<L, R>` in `deep_causality_haft` is the coproduct, and `CausalFlow::either` consumes the flow
and runs one arm with the state moved into it. `Either` is the carrier on the way out, where
`adjudicate` returns one surviving hypothesis against a residual ambiguity.

*As built.* A world inherits the ledger and nothing else. The root keeps its read-out and verdict
as the baseline, and a world's evidence comes from `observe` and `gate` after the fork, or from
`compare`, so a world evolved by a mechanism is never adjudicated on a measurement of the
unevolved plant. A screen that admitted no candidate, or a config that declared none, leaves
nothing to fork; `fork` fails naming which, rather than producing no worlds for `finalize` to
report as success.

#### Scenario: Three candidates give three live worlds

- **WHEN** `fork` runs on a screened config holding three admitted hypotheses
- **THEN** all three worlds are live simultaneously, each carrying an independent copy of the ledger,
  and none of the three states was moved into an arm

#### Scenario: A world inherits the ledger and no evidence

- **WHEN** `observe` and `gate` run on the root and `fork` follows
- **THEN** every world's ledger equals the root's, every world's read-out and verdict are absent,
  and `gate` on the worlds before `observe` fails with `CalculationError` naming `observe`

#### Scenario: A fork with no candidate is refused

- **WHEN** `control` takes a screen that admitted no candidate and `fork` runs
- **THEN** `finalize` returns `CalculationError` naming that the screen admitted none or the config
  declared none, and no world exists

#### Scenario: Forked ledgers are compared rather than joined

- **WHEN** two forked worlds reach `adjudicate`
- **THEN** their ledgers are read side by side to score the candidates, and no ∇ join is applied to
  them

#### Scenario: Adjudicate returns the coproduct

- **WHEN** `adjudicate` resolves to one surviving hypothesis, or leaves a residual ambiguity
- **THEN** the result is an `Either` from `deep_causality_haft` rather than an ad-hoc enum

### Requirement: compare turns predictions into evidence against the root's read-out

`compare(sigmas)` SHALL give every forked world a read-out and a verdict from its prediction: the
read-out is `ShotEstimate::from_probability(prediction, baseline.shots())`, and the verdict is one
`Check` of `|prediction − baseline.estimate()|` against `sigmas · baseline.standard_error()`,
examined over the baseline's shots, where the baseline is the root's read-out taken by `observe`
before `fork`.

The prediction is carried with the shot noise it would have at the baseline's shots, so
`adjudicate` separates worlds by their predictions, and a world's verdict holds when its prediction
agrees with the observation. `compare` refuses, with `CalculationError` naming the missing step, a
`sigmas` that is not finite or is negative, a root with no read-out, no worlds, and a world with no
prediction. A mechanism world with a prediction may be compared; nothing forbids it.

#### Scenario: A prediction becomes a read-out at the baseline's shots

- **WHEN** `observe(o, n)` runs on the root, `fork` and `predict(o)` follow, and `compare(3)` runs
- **THEN** every world's read-out has the world's prediction as its estimate and `n` as its shots,
  and its verdict examined `n` items and accepted exactly when the prediction lies within three
  standard errors of the root's estimate

#### Scenario: A missing step is named

- **WHEN** `compare` runs before `predict`, before `fork`, without a root read-out, or with a NaN
  or negative `sigmas`
- **THEN** `finalize` returns `CalculationError` whose message names `predict`, `fork`, `observe`
  or `sigmas` respectively

#### Scenario: A mechanism world may be compared

- **WHEN** a mechanism config runs `observe → fork → predict → compare → adjudicate`
- **THEN** each world's prediction is judged against the root's read-out, and the world whose
  channel leaves the read-out unchanged survives

### Requirement: control has two paths to adjudicate, one per kind of candidate

The control stage SHALL support two paths to `adjudicate`: mechanism candidates run
`fork → observe → gate → adjudicate`, and structural candidates run
`observe → fork → predict → compare → adjudicate`.

A mechanism world carries the plant evolved by its channel, so its evidence is a measurement of
that plant after the fork. A structural world carries the plant unchanged, so a measurement after
the fork would read the same in every world; the root is measured once as the baseline, and the
worlds are told apart by what each model predicts for it.

#### Scenario: The mechanism path adjudicates each world on its own measurement

- **WHEN** a mechanism config with a flipping and an identity channel runs `observe` on the root,
  then `fork → observe → gate(at_least 0.9) → adjudicate`
- **THEN** the flipping world's read-out is its own evolved plant's rather than the root's, its
  ledger counts two experiments to the root's one, and it is the survivor

#### Scenario: The structural path selects the candidate whose prediction matches the plant

- **WHEN** two structural candidates whose predictions differ enter `control` through the screen,
  the plant is prepared so one candidate's prediction is its Born read-out, and
  `observe → fork → predict → compare → adjudicate` runs
- **THEN** that candidate is the survivor, its verdict accepted and the other's rejected, and the
  separation credited to the ledger is between the two predictions at the observed shots
