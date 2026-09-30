# The essence of QCL and `deep_causality_quantum`

A distillation for writing the quantum site. Every statement below was checked against the tree on
2026-09-30. Where a number comes from a run I did not repeat, the line says so and names
`01_raw_information.md`.

Read in full: all 87 files under `deep_causality_quantum/src/` and all 61 files under
`examples/quantum_examples/`. Not read: `deep_causality_quantum/tests/`, the papers under
`papers/`, `LEAN_QUANTUM.md`, `CHANGELOG.md`. Claims about the papers rest on the crate's own
citations of them.

## 1. The essence in one paragraph

QCL, the Quantum Causal Language, is an embedded Rust builder, `QclBuilder`, that turns a question
about a quantum system into a checked decision. The caller names a subject and its candidates. The
library refuses what it cannot decide, screens the rest with checks that report how close each came
to failing, plans the cheapest experiments that tell the candidates apart, and names one survivor or
says why it cannot. The crate under it, `deep_causality_quantum`, supplies the quantum carriers, the
gates, the circuit model, the code checks and a simulated evidence source. Every verdict states what
it examined, its margin, and whether an exact or a numeric path decided it.

## 2. Three things the crate decides

| Question | What QCL does | Where it lives |
|---|---|---|
| Which causal structure produced these correlated errors? | Screens candidate structures, plans experiments, adjudicates | `pipeline/`, `design/`, `qcm/` |
| Is this quantum code sound, and which logical gates tolerate a fault? | Verifies a CSS code over 𝔽₂ and exact rationals, no simulation | `qcode/`, `abstraction/fault_tolerance.rs` |
| How far does a stack of abstractions drift from the ideal? | Measures each link's residual and bounds the composite | `abstraction/`, `circuit_model/` |

One decision form serves all three. A `Check` holds the item, the measured quantity, the threshold,
the margin (measured over threshold, rejected above 1) and whether it was accepted. A `CheckReport`
holds the checks, the number of items examined and a verdict: `Accepted`, `Rejected`, or `Vacuous`
when nothing was examined. A pass that examined nothing reads `Vacuous`, so it cannot pass as
evidence.

## 3. The two working types

`QclBuilder::config::<R, N>()` names both once, at the single origin of every configuration.

- `R: RealField` buys accuracy. Every tolerance derives from `R::epsilon()`, except the shot-noise
  member, which derives from the shot count. Switching `R` between `f32`, `f64`, `Float106` and
  `BFloat16` recomputes every threshold.
- `N: NaturalNumber` buys headroom. Counts use checked ℕ arithmetic on a caller-chosen width and
  move no threshold.

The `Tolerance` family has five members: commutator, validation, numerical rank, state and shot
noise. Each returns `None` when its threshold is not representable in `R`, and callers fall back to
`√ε`.

## 4. The pipeline

```text
config(subject, candidates)  →  validate(checks)  →  Screened  →  control(evidence)  →  report
```

- `build()` refuses what would answer unsoundly before any stage runs.
- `validate(&cfg)` runs stages and ends in `finalize()`, which returns a `Screened`.
- `control` accepts a plant configuration whose candidates are mechanisms, or a `Screened`. A plant
  configuration whose candidates are structural has no `ControlSource` implementation. The
  compiler, and no runtime check, keeps a structural candidate out of `control` without validation.
- `control(...).fork().design(MinCostCover::new(floor)).finalize()` yields a `ControlReport` with the
  forked worlds, the plan and the ledger.
- `adjudicate(&worlds, floor_bits)` folds the worlds' verdicts.

The pipeline adds ordering and naming. The categorical structure it threads is the causal monad's
`PropagatingEffect`, with `Ledger` as its state. The pipeline module sits behind the `qcm` feature,
which implies `std`.

### The four subjects

| Subject | Built with | Stages offered | Example |
|---|---|---|---|
| Plant | `over_plant(plant, observables)` then `candidates` or `mechanisms`, `probes`, `baseline`, `seed` | `validate`, then `control` | `qcl_crosstalk` |
| Model | `over_model(graph, factors, supports)` then `declare_systems` | `validate` | `qcl_qcm_freeze` |
| Code | `over_code(chain_complex)` | `validate` only: no probes, baseline or evidence | `qcl_geometric_qec` |
| Circuit | `over_circuit(circuit_model)` | `validate` on the circuit's dilation | `qcl_crosstalk_circuits` |

## 5. Deciding between causal structures

### Candidates

A structural candidate, `Hypothesis::structural`, is a set of process factors (one diagonal or
general operator per node) plus the Hilbert legs each factor acts on, its supports. Under the flat
convention `support(A) = {A} ∪ Pa(A)`, so the supports carry the causal structure. A mechanism
candidate is a channel applied to the plant.

### What `build()` and `validate` refuse

- **Cycles.** A structural candidate whose supports contain a directed cycle fails `build()` as
  `CyclicStructureUnsupported`. This is a scope decision. Under van der Lugt and Lorenz's
  Definition 3.1 the C₃ criterion does not reject a cycle, so the builder does and names the limit.
- **Non-commuting factors.** `check_markov` tests every pair of factors whose supports intersect
  (Lorenz 2022, Definition 3.3). It compares ‖[ρⱼ, ρₖ]‖_F with the threshold
  `C·(‖ρⱼ‖·bₖ + ‖ρₖ‖·bⱼ + 2γₙ‖ρⱼ‖‖ρₖ‖)`, with C = 8 by default. It stops at the first rejecting pair.
  On a plant a rejected candidate leaves the admitted set and no error is raised. On a model
  `finalize` returns `CommutatorNonZero` naming both nodes.
- **The C₃ sub-relation.** `check_decomposable` derives a causal structure from the supports and
  rejects the structure of two commuting CNOTs (van der Lugt and Lorenz).

`freeze_quantum` runs the same two checks at the graph's freeze boundary. On failure it aborts and
the graph rolls back to dynamic. `qcl_qcm_freeze` shows the pipeline and the shipped freeze
examining the same number of pairs.

### Planning

`design` finds the minimum-cost set of experiments whose best separation covers every pair of
candidates at or above `floor_bits`. It solves an exact dynamic program over 2^C(n,2) subsets and
refuses more than 7 candidates by default. Pairs no experiment separates come back as uncovered.

Separation of two predicted read-outs p and q over n shots is
`n · (−log₂(√(pq) + √((1−p)(1−q))))` bits, the Bhattacharyya distance of n independent draws. The
plan's report takes, per pair, the best separation over all experiments offered, chosen or not.

### Adjudication

`adjudicate` names a survivor when exactly one world's verdict holds and its read-out separates from
every rival's by the floor. Otherwise it returns an `Ambiguity`: no survivor, several survivors, one
survivor that is not separated, non-commuting projections, or a single world.

Projection-valued verdicts get a commutation test between worlds, because projectors on a quantum
system form an orthomodular lattice that fails distributivity. Read-out verdicts do not: a threshold
on a real number is a classical proposition.

Forked worlds keep separate ledgers of shots, experiments, predictions, device time, cost and bits.
The ledgers are compared and never summed.

### The boundary the library states

Partial trace does not preserve commutation; this is proved in Lean. `partial_trace_preservation_boundary`
returns a bound with a √d_B amplification and a `holds` flag, and `Hypothesis::marginalise` raises
`BoundaryNotHeld` when it fails. The crate marks where a marginal model stops licensing a
conclusion.

## 6. Evidence

- **Default build.** Shots come from a seeded Bernoulli draw at the Born probability Tr(Pρ) of a
  density matrix. A fixed seed reproduces the histogram.
- **`qpu` feature.** Adds `QpuSampler`, an in-process state-vector simulator `SimQpu` capped at 24
  qubits, `Evidence`, `ShotBudget` (naming a budget compiles only under `qpu`), and `qpu_effect`,
  which lifts a sampler call into the causal monad. No vendor adapter ships.
- **Two senses, kept apart.** The verifiable path (deterministic simulated Choi–Jamiołkowski
  operators, checked at the freeze boundary) is the default and the target of the Lean proofs. The
  emergent path, a physical QPU call as a monadic effect, is a seam only.

## 7. Verifying codes exactly

A code subject is a chain complex read as a CSS code: qubits on the 1-cells, Z checks from the
columns of ∂₂, X checks from the columns of δ₀, and `k = β₁` over 𝔽₂. The type carries no distance.

`validate` on a code runs four stages, none of which simulates anything:

| Stage | Decides | Method |
|---|---|---|
| `derive_code` | n, k, the check families | counts and columns of the complex |
| `check_ldpc_weights(bound)` | both weights of both check matrices against a bound | one record per row and column; stops at the first offender |
| `check_class_invariance` | Z̄, S̄, T̄ act on the homology class, not the representative | Haruna Eq. (3.20); exact rational phases (`Turns`); no tolerance |
| `check_clifford_action` | H̄ swaps Z̄(γ) and X̄(γ̃) and fixes the other logical qubits | symplectic tableau over 𝔽₂ |

The rest of the exact machinery:

- **Gauge-field gates.** A diagonal logical gate is a phase table over the parities of its blocks,
  `O_k = exp(iπ/2^{k−1} · p₁⋯p_m)` (Haruna Eq. 3.63). Z̄, S̄, T̄, CZ̄ and C^{m−1}Z̄ are instances.
- **Emitters.** `logical_z`, `logical_x`, `logical_s`, `logical_t`, `logical_hadamard`,
  `logical_cz` and `logical_multi_cz` emit the physical programs of Haruna's Table 1.
- **Logical equivalence.** `LogicalBasis` decides when an operator acts trivially on the code space:
  the normalizer condition first, then pairings against the logical generators. Every predicate is an
  inner product over 𝔽₂.
- **Fault tolerance.** `check_fault_tolerance` pushes a Pauli through a gate's program. A fault is
  tolerated when the diagonal remainder is constant and the propagated Pauli weighs no more than the
  fault. `haruna_filter` reports which Table 1 gates tolerate every weight-one fault. The report
  claims no threshold, distance or asymptotic suppression.
- **Width.** The checks run on supports, so cost tracks chain weight, not register width. The 32-qubit
  toric code is decided although the simulator stops at 24.

## 8. Abstractions and the composition law

An abstraction, after Lorenz and Tull (arXiv:2602.16612), says a high-level model summarises a
low-level one. It holds:

- a type alignment: for each high-level type, the low-level wires it abstracts, a channel τ down and
  a section E back, with τ∘E = id checked at construction (`SectionNotInverse`);
- a query map from high-level queries to low-level ones. Query kinds are `Io`, `Open`, `Inc`,
  `Observe` and `Fault`.

**Naturality** is the square `τ ∘ ⟦π(Q)⟧_L = ⟦Q⟧_H ∘ τ` for every query. `check_naturality` measures
its residual in Frobenius norm on Choi operators against `Tolerance::state()` and returns the
two-sided diamond bound `r/d_in ≤ ‖·‖⋄ ≤ √(d_in·d_out)·r` for the worst residual. An abstraction whose
squares commute to residual ε is an ε-abstraction, the crate's own definition.

**Composition.** Two abstractions L→M and M→H compose to L→H. For every query the composite residual
obeys `ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂`, and a `LawRow` records ε₁, ε₂, both constants, the bound
and the measured residual. The constants are Frobenius-induced norms and are not 1 in general: a
partial trace over a factor of dimension d has norm √d. The exact case, ε₁ = ε₂ = 0 ⇒ ε = 0, is the
Lean statement in `lean/DeepCausalityFormal/Quantum/Abstraction.lean`. The approximate law is the
crate's own statement.

**Structure.** `check_alignment_structure` decides Definition 49's predicates (simple, extra-simple,
full) for a partition. Between classical models the scope reads `Equivalent` (Theorem 51); with a
quantum model it reads `Necessary` (Remark 56).

### The models abstractions relate

A `CircuitModel` is typed wires, boxes and a grouping of boxes into nodes. Boxes are `Encoder`,
`Unitary`, `Channel`, `Kraus`, `Instrument` and `Measurement`. The induced DAG is derived from the
wiring and never stored. Queries are rewired circuits: `opened`, `observed`, `interchanged`,
`faulted`. Two semantics evaluate a model:

| Path | Carries | Limit |
|---|---|---|
| Exact | Pauli and Clifford layers by tableau, diagonal gates by phase table | none on register width; 16 blocks per gate; 20-qubit support for reading a diagonal program |
| Numeric | Kraus families evaluated on state vectors, Choi operators formed once | 2^24 entries, 2^12 Kraus operators, 12 qubits per gate |

Every report names its path, so a numeric verdict is not read as exact.

### Chains over CSS codes

| Chain | What it builds | What the run shows (from `01_raw_information.md`) |
|---|---|---|
| `concatenated_code` | inner code's qubits grouped into blocks encoded by an outer code | Z̄ and X̄ compose exactly; CZ̄ across outer blocks is refused by name |
| `code_switching` | decode from code A, optional noise, encode into code B | noiseless is exact; with depolarising 1/10, ε₁ = 0.4619, measured 0.4619, bound 3.695 |
| `distillation_round` | encode, depolarise every qubit, run T̄H̄ | p = 1/100: ε₁ = 0.1639, measured 0.1638, bound 0.3277 |

The bound in code switching sits 8 times above the measurement, because ‖τ₂‖_post = 8. The
distillation round is a non-strict quantum-to-quantum abstraction, the case the paper defers (§7.1).
Each chain is an example with checks, and claims the residual it measures and the bound the law
records.

### The decoder as an abstraction

`DecoderAbstraction` relates a circuit to a detector error model. τ is a stochastic matrix lifted
into QC. The signature is `Io` plus one fault query per named circuit location. A location the model
omits gets a phantom mechanism that never fires, so its square asks whether the model may ignore that
location, and its residual is of order √2. `attribute` ranks faults by residual. The crate holds no
decoding algorithm and no `Decoder` trait.

## 9. What the crate refuses, by name

The library states what it cannot decide instead of returning an answer that would not hold.

| Refusal | Cause |
|---|---|
| `CyclicStructureUnsupported` | a candidate structure with a cycle |
| `CommutatorNonZero` | factors on a shared leg that do not commute |
| `BoundaryNotHeld` | a marginal outside the partial-trace bound |
| `NotParallelisable` | an interchange set joined by a directed path |
| `NoPropagationNormalForm` | a non-diagonal Clifford layer after a non-constant remainder |
| `NaturalityDimensionExceeded`, `KrausFamilyExceeded` | a Choi operator or Kraus family above its cap, counted before allocation |
| `SectionNotInverse` | a type alignment whose τ∘E is not the identity |
| refused T̄ concatenation | the outer code has no controlled-S† emitter (Haruna's Table 1 has none) |
| refused cross-block gate | no transversal gadget between code blocks in this construction |
| refused narrower switch | switching into a code with fewer qubits |

## 10. Limits, in one place

| Limit | Value |
|---|---|
| Candidates `design` covers by default | 7 |
| In-process simulator, `SimQpu` | 24 qubits |
| Ideal recovery (dense) | 10 qubits |
| Numeric semantics, Choi operator | 2^24 entries |
| Numeric semantics, Kraus family | 2^12 operators |
| Detector error model | 20 mechanisms, 20 variables |
| Dilation, one factor | 2^24 entries; this is why H₃ stays a factorization in `qcl_crosstalk_circuits` |
| Gate program in a box | 12 qubits per gate |
| Class-invariance enumeration | 2^22 states per shift |
| Pipeline build | needs `std`, through `qcm` |
| Evidence | simulated; no vendor adapter |
| Decoder | none in the crate |
| Cycles | refused by decision |

## 11. The keystone example, split into inputs, computed and sampled

`qcl_crosstalk` decides whether two qubits' correlated errors have a direct cause or a shared bath.

| Typed into `constants.rs` and `model.rs` | Computed by the library | Sampled |
|---|---|---|
| the four structures' factors | the refusal of H4 at `build()` | the observation: 1024 shots from the Born sampler |
| five experiments' costs (1, 1, 1, 2, 200, "in the same arbitrary unit") | Markov screen (5 pairs) and C₃ screen (3 blocks) | drawn at H1's own predicted read-out, seed 20260821 |
| each experiment's predicted read-out under H1, H2, H3 | the minimum-cost plan: E1 and E2, cost 2 | |
| shots, floor (5 bits), agreement window (3 standard errors) | separations in bits, per pair | |
| | the adjudication of the three worlds | |

Facts that follow from that table and from `constants.rs`:

- The costs 2 and 200, and the predictions behind the plan, are inputs. The example states them as
  modelling assumptions.
- The run observes one experiment, E1, on data drawn from H1. The example then checks that the
  survivor is the structure the observation came from.
- E1 predicts 0.10 under both H2 and H3. Had the truth been H2 or H3, an E1 read-out near 0.10 would
  leave both standing, and E2 would split them (0.40 under H2, 0.10 under H3). That is why the plan
  holds two experiments. The run exercises the case where E1 suffices.
- The printed "tightest pair separates at 164.8 bits" is the smallest, over the three pairs, of the
  best separation any offered experiment reaches; tomography reaches it. Under the chosen plan the
  tightest pair separates at 99.5 bits. I confirmed both by hand: n = 1024, (0.9, 0.5) gives 164.8
  and (0.4, 0.1) gives 99.5.
- The survivor's 100.1 bits is the same distance between the two predictions rounded to whole shots
  (410 and 102 of 1024).
- The Markov check admits all three candidates because the factors are diagonal, which puts the whole
  weight of the discrimination on the interventions.
- The crate can compute a prediction from a candidate's factors: `Hypothesis::evaluate` and `predict`
  return `Re Tr(σ·τ)`. `Experiment::new` takes predictions as data, and this example does not call
  `evaluate`.

`qcl_crosstalk_circuits` writes H₁, H₂ and the cyclic H₄ as circuits. H₁ and H₂ share the same two
boxes and differ only in how the boxes group into nodes, which reverses the induced edge. H₃ stays a
factorization. The predictions and costs are the same constants, so the circuits do not change the
plan or the verdict; the example shows the circuit route through `build()`, the dilation and the
screen.

## 12. The fourteen examples

Nine import `deep_causality_quantum`. Five are quantum in subject and use other crates.

| Example | Imports the quantum crate | What it shows |
|---|---|---|
| `qcl_crosstalk` | yes | the pipeline end to end on one problem (section 11) |
| `qcl_crosstalk_circuits` | yes | the same decision over circuit-derived candidates |
| `qcl_qcm_freeze` | yes | validate on a model; the shipped freeze rolls back a dynamic graph |
| `qcl_geometric_qec` | yes | the [[32, 2]] toric code verified exactly; a bound of 3 rejects with margin 4/3 |
| `qcl_concatenated_code` | yes | composition law, Z̄ and X̄ exact, cross-block gate refused |
| `qcl_code_switching` | yes | the gadget's cost as the first link's residual |
| `qcl_distillation_round` | yes | the case the paper defers |
| `qcm_freeze_check` | yes | factors sharing a leg must commute; `freeze_quantum` aborts and rolls back |
| `ikkt_matrix_model` | yes | commutator kernel; action falls from 0.048 to 8.2e-7 in 40 steps at fixed norm |
| `quantum_counterfactual` | no | three-qubit repetition code; `alternate_value_if` forces a syndrome |
| `quantum_geometric_tensor` | no | metric and Berry curvature; flat-band Drude weight 0.037778 meV·nm² |
| `gauge_electroweak` | no | W mass at tree level and one loop against the measured value |
| `topological_insulator` | no | Chern number by quadrature and by Wilson loop |
| `hopf_fibration_multivector` | no | global phase moves the state and leaves the Bloch vector fixed |

All fourteen alias `FloatType` (default `Float106`) so a hard-coded `f64` fails to compile. Three of
the physics examples record a `BFloat16` failure: `gauge_electroweak` misses its tolerance,
`topological_insulator` disagrees between routes, and `ikkt_matrix_model` stalls. Those are stated in
their READMEs.

## 13. Claims in the examples I could not verify from the tree

- `gauge_electroweak`: the README says the W mass follows from "three measured numbers" and is
  "measured to better than a part in ten thousand". The run prints a top Yukawa coupling and a Higgs
  quartic derived inside `ElectroweakParams::standard_model_precision()`, which I did not read, so I
  did not count its inputs. The PDG uncertainty on M_W of roughly 12 MeV on 80.377 GeV is about 1.5
  parts in 10⁴, which is not better than one in ten thousand. Check both before the site repeats
  either.
- `qcl_crosstalk_circuits/README.md`: the H₃ cost argument (d = 16, a leg of 256, 2^32 entries) is
  arithmetic on the dilation's leg convention; I did not recompute it.
- The Lean counts in `01_raw_information.md` (14 rows named `quantum.*` proved; seven deferred
  targets) come from `lean/THEOREM_MAP.md` and `LEAN_QUANTUM.md`, which I did not read.

## 14. What the site can say without qualification

- QCL refuses, screens, plans and adjudicates, and each verdict carries examined count, margin and
  path.
- A structural candidate cannot reach `control` without validation; the compiler enforces it.
- Every tolerance derives from the working real type's epsilon.
- Code checks are exact over 𝔽₂ and rationals and decide a 32-qubit code the simulator cannot hold.
- The composition law is the crate's statement; the exact case is proved in Lean.
- The library names what it will not do: no decoder, no vendor adapter, no cycles, capped scale.

## 15. What the site must qualify

- "Cost 2 against 200" and the predictions behind it are inputs.
- The tightest pair under the plan separates at 99.5 bits; 164.8 is tomography's.
- "Two experiments" is the plan; the run observes one, drawn from H1.
- The circuit example reproduces the pipeline route, and its plan and verdict come from the same
  typed predictions.
- The composition bound holds and can sit far above the measurement (a factor of 8 in code
  switching).
