<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# What Lifts Back Into the Quantum Crate: a QCL Sketch

**What this is.** An answer to a specific question: given that the calibration control system is
already a DAG in industry practice, and that the causaloid plus the join algebra fit it, what can be
lifted *back* into `deep_causality_quantum` as a high-level API in the manner of the CDL DSL in
`deep_causality_discovery`?

**Status.** Revision 2, updated against three designed examples:
[`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md) and
[`example-geometric-qec.md`](example-geometric-qec.md). Companion to
[`positioning.md`](positioning.md).

**The short answer, revised.** Revision 1 said the DSL should wait, because it would have been an
abstraction over a use case of one. That caution is now discharged: three independent consumers
exist as designs, and two of them changed the API. The composites in §3 still lift first, and the
pipeline now has a shape that three problems agree on rather than one problem asserts. What has not
changed is that all three consumers are designs. None is code, and §7 keeps that distinction.

---

## 1. The reading is right, with one correction

Google established that the calibration control system is a dependency graph and that maintaining it
is graph traversal. The causaloid adds what the traversal lacks: a node that carries its own causal
function, a defined answer at a reconvergent join, and a freeze that can refuse a graph.

The correction is one of ownership. The join algebra is not the causaloid's. `Verdict` is a
`deep_causality_algebra` trait; `∇` is `SymMonoidal::merge` in `deep_causality_haft` with
Lean-checked monoid laws. The causaloid graph supplies the *policy* that a join folds `∇` over the
value channel, concatenates logs, and refuses to merge state. Keeping that straight matters here,
because it determines what a quantum-crate DSL has to build and what it merely calls.

---

## 2. What the three examples say about lifting

### 2.1 The original caution, and what discharged it

The test for a new abstraction is repetition that already exists. Counting `CausalTensor::new` in
the quantum crate's tests and the quantum examples gives 50 call sites, and 24 uses of
`DensityMatrix::`. The mass is in tests, where raw construction is correct and should stay; on the
example side there were four in `quantum_geometric_tensor` and one in `qcm_freeze_check/model.rs`.
On that evidence alone a pipeline DSL would have been designed against one prospective consumer.

Three designed consumers now exist, and the two that came after the first each changed the API
rather than confirming it. A fourth arrived from outside the design: `deep_causality_topology`'s
cup product, shipped 2026-08-21, is a real consumer of the same cochain-as-slice convention §3.2 and
Decision 2 argue for, and it landed on that convention without being asked to. An abstraction that survives being pushed on by independent problems is a
different object from one that was fitted to a single problem, and that is the difference between
revision 1 and this one.

### 2.2 What each example forced

| Example | What it forced |
|---|---|
| `quantum_control_loop` | The pipeline shape: `prepare, evolve, observe, gate, fork, design, predict, adjudicate`. Established `Ambiguous` as a required verdict (§5) and the experiment designer (§8). |
| `crosstalk_attribution` | `design` returns a **plan**, not an experiment. Its hypothesis set defeats the single-experiment criterion outright (§8.8). |
| `geometric_qec` | `validate` takes a **code**, not only a factorization, and gains a third structural gate. Also established that the two pipelines are not uniformly applicable (§2.3). |

Two amendments from two consumers is the ratio a design should want. A second and third example that
merely confirmed the first would have been weaker evidence, not stronger.

### 2.3 Which pipeline each example actually uses

| Example | validate | control | design | adjudicate |
|---|---|---|---|---|
| `quantum_control_loop` | no | yes | yes, single experiment | yes |
| `crosstalk_attribution` | yes, Markov and C₃ | yes | yes, min-cost plan | yes |
| `geometric_qec` | yes, class invariance | **no** | partial, and a different computation | no |

The empty cell is the honest finding. `geometric_qec` is construction and verification with no
feedback loop in it, and the natural control loop over a code is decoding, which is Track D and
carries no claim yet. A DSL whose two halves are not equally applicable everywhere is the normal
case; one that claimed otherwise is what this audience disbelieves on sight.

`design` appears in all three, which makes it the most general stage. It is also the one place where
the three do **not** fully agree: see §8.9.

---

## 3. What lifts back today

### 3.1 `QuantumPlant<R>`: a validated state-and-channel pair

Today a caller builds a `DensityMatrix`, hand-assembles a Kraus set, and calls `apply_kraus`
repeatedly. The CPTP checks (`check_completely_positive`, `check_trace_preserving`) exist but
nothing binds them to the channel, so they are either re-run per application or skipped.

A plant bundles the two, runs the CPTP check once at construction, and exposes a single `step`. The
invariant moves from "the caller remembered" to "the type was constructed."

### 3.2 `Observable<R, D>`: a named projector with its read-out attached

`born_projective_prob` takes a `Projection` the caller constructs separately. Binding a name to a
projector and hanging the Born read-out off it removes the two-step and makes the read-out
self-describing in a log line, which matters because the log is the audit trail.

### 3.3 `ShotBudget`: one call from probability to `Uncertain`

This is the least ergonomic corner of the crate today. To use `shots_to_qubit_bernoulli` a caller
must construct a `CountHistogram`, fill it, and only then bridge. A shot budget carrying a count and
a seed, with a `sample` that fills the histogram internally, turns the emergent seam into one call.

The payoff here is larger than its size suggests. `Uncertain<f64>` and `Uncertain<bool>` already
implement `Verdict`, so once a read-out is sampled it is a lawful lattice carrier and joins with
classical verdicts without a special case. The bridge is the only awkward step in an otherwise
complete chain.

### 3.4 `CssCode<'a, K>`: the QEC reading of a chain complex

From [`example-geometric-qec.md`](example-geometric-qec.md). A CSS code is a chain complex: the
boundary maps are the parity checks and `β₁` counts the logical qubits. `deep_causality_topology`
already supplies all of it, verified by running it: an `L × L` periodic `LatticeComplex` returns
`β₁ = 2` and boundary matrices of the right shape and weight, which is the toric code family with no
code-specific code written.

What is missing is the **naming**. Nothing says that `boundary_matrix(1)` is `H_X`, that
`num_cells(1)` is the physical qubit count, or that `betti_number(1)` is `k`. A `CssCode` borrowing a
`ChainComplex` and exposing `n`, `k`, `h_x`, `h_z` and the LDPC weights is a thin, honest composite:
it adds QEC vocabulary over topology objects and computes nothing new.

It belongs in the quantum crate rather than in topology, because the mapping is a quantum-information
reading of a topological object, and topology should not learn about codes. That direction also
respects the tier ordering; see §6 item 5.

---

## 4. The DSL

CDL's structure is worth copying almost exactly, because it solves the same problem: many small
typed stages that must run in a valid order, parameterised from one config.

### 4.1 Two sub-pipelines, genuinely disjoint

CDL isolates SURD and BRCD at compile time and converges them on a shared analyze/finalize tail. The
quantum crate has an equally real split, and it is not "monitor versus diagnose" (diagnose is monitor
plus more, so forcing that parallel would be false). The real split is **freeze-time structure**
against **run-time control**:

- **Validate**, over a factorization or a code: `declare_* → check_* → validate_analyze → finalize`
- **Control**: `prepare → evolve → observe → gate → fork → design → predict → adjudicate →
  control_analyze → finalize`

The validate pipeline has three gates across the three examples, and its subject is not fixed:

| Gate | Subject | Source |
|---|---|---|
| quantum Markov commutativity | a Choi factorization | shipped, `freeze_quantum` |
| C₃-exclusion faithfulness | a declared causal structure | shipped, `check_c3_exclusion` |
| homology-class invariance | a code and its logical operators | `geometric_qec`, buildable today |

That the third gate's subject is a *code* rather than a factorization is `geometric_qec`'s amendment.
The pipeline is the home for structural correctness criteria in general, not for QCM-specific ones,
and the stage names should read that way.

Calling a control stage on a validation pipeline should not compile, exactly as in CDL. Both end at
`finalize().print_results()`, and a `QclOutcome::{Validation, Control}` selects which section the
report renders, mirroring `CdlDiscoveryOutcome`.

### 4.2 Sketch

```rust
let cfg = QclConfigBuilder::build_control_config::<FloatType>()
    .with_plant(plant)                       // §3.1, CPTP-checked at construction
    .with_observable(excited_population)     // §3.2
    .with_spec(Spec::at_least(ft(0.999)))
    .with_shots(ShotBudget::new(1024, seed)) // §3.3
    .with_hypotheses(&[amplitude, detuning, decoherence])
    .build()?;   // spec in [0,1], shots > 0, hypothesis names distinct

QclBuilder::build_control(&cfg)
    .prepare()          // seed the plant state
    .evolve(1)          // one protocol step
    .observe()          // Born read-out, sampled -> Uncertain<FloatType>
    .gate()             // spec test
    .fork()             // one world per hypothesis
    .design(&experiments, DesignObjective::MinCostCover { floor_bits: ft(5.0) })  // §8
    .predict()          // evolve each world under the chosen probe
    .adjudicate()       // rank by sigma margin
    .control_analyze()
    .finalize()
    .print_results();
```

The validate side, from `geometric_qec`:

```rust
QclBuilder::build_validate(&cfg)
    .declare_complex()          // a ChainComplex
    .derive_code()              // §3.4: n from 1-cells, k from beta_1, checks from the boundaries
    .check_ldpc_weights()       // bounded row and column weight, off the CsrMatrix structure
    .check_class_invariance()   // the Haruna gate acts on the class, not the representative
    .validate_analyze()
    .finalize()
    .print_results();
```

`crosstalk_attribution` runs the other declaration path, `declare_factors → declare_supports →
check_markov → check_faithfulness`, and then hands the surviving hypotheses to the control pipeline.
That hand-off is worth supporting explicitly: **validate screens the hypothesis set, control
discriminates what survives.** In that example one of four hypotheses is rejected at freeze as
containing a C₃, so no device time is spent discriminating a model no circuit can faithfully realize.

### 4.3 Where the quantum DSL should be *thinner* than CDL

CDL invented `CdlEffect` because it needed a short-circuiting monad that threads warnings. The
quantum crate does not need to invent anything at that layer: it already sits on `PropagatingEffect`,
which short-circuits on error and carries an `EffectLog`. Every stage returns one, and the DSL is a
typestate skin over stages that already compose.

Stated as a rule for whoever builds it: **the DSL adds ordering and naming, not a monad.** If a
`QclEffect` type appears in the design, something has gone wrong.

---

## 5. The one genuinely new concept

Everything above is packaging. One thing in the control pipeline is not, and it is the piece worth
building carefully.

`adjudicate` cannot return a hypothesis. It has to return one of:

- **`Abduced { hypothesis, margins }`** — one hypothesis matches and the others are rejected, each
  with the sigma distance that rejected it.
- **`Ambiguous { candidates, separation }`** — two or more hypotheses sit within tolerance of each
  other *at this shot budget on this observable*. The honest output is not a guess; it is "take more
  shots, or measure something else," with the numbers that say so. §8 supplies those numbers: the
  experiment designer both selects the observable and inverts the separation floor into a required
  shot count.
- **`NoneAdmissible { reason }`** — the measurement matches no hypothesis, which means the fault
  model is incomplete.

Compare Optimus. Its `check_data` returns in spec, out of spec, or bad data, and "bad data" is a
judgement that the problem is upstream, inferred from the points looking like noise. It has no way to
say "these two causes are indistinguishable from the data I am allowed to take." That verdict is
exactly what the degenerate reading in the control example produces, and expressing it is the
API-shaped version of the claim in `positioning.md` that the tool has to be able to say no.

`Ambiguous` is the one type in this sketch that would be worth writing even if nothing else here
ever gets built.

---

## 6. What must not be lifted

1. **Graph traversal.** `maintain` and `diagnose` are traversals over a dependency graph. That is
   `deep_causality`'s `CausaloidGraph` and `ultragraph`'s algorithms. A quantum crate that grows its
   own traversal has duplicated the graph layer and broken the tier ordering.
2. **Plant physics.** The rotation-with-amplitude-and-detuning model belongs to the example. The
   crate provides channels and states, not device models.
3. **Device adapters.** Ruled out already by the modality split; a vendor driver crosses the
   verifiable/emergent line the crate is built to keep.
4. **A verdict lattice.** `Verdict` is `deep_causality_algebra`'s. The quantum crate contributes
   `Projection` as a carrier and stops there.
5. **Topology.** `geometric_qec` makes this one concrete. Chain complexes, boundary and coboundary
   matrices, Betti numbers, cell and lattice complexes and the gauge field machinery are
   `deep_causality_topology`'s, and the sparse carrier under the parity checks is
   `deep_causality_sparse`'s. The quantum crate may name their QEC meaning (§3.4) and must not
   reimplement them. The cup product **arrived in topology on 2026-08-21**, as `SPEC-T2` said it
   should, alongside the `SplittableCell` trait that gives each cell family its own decomposition.
   That it landed there rather than here is the rule working: the quantum crate names the QEC
   meaning of a topological object and the topological operation stays where the complex lives.

---

## 7. Sequencing

Revision 1 sequenced around a single unwritten example. Three designs change the order but not the
principle: composites first, pipeline once real code has exercised the shape.

1. **Write the examples with the raw APIs.** All three are designed and none is code. Start with
   `quantum_control_loop`, which is self-contained, then `crosstalk_attribution`, which needs the
   validate-to-control hand-off, then `geometric_qec`, which needs no control pipeline at all and is
   the most independent of the three.
2. **Lift §3.1 to §3.4 as the dances repeat.** Each is small and independently useful, and none
   presumes the pipeline. `CssCode` (§3.4) can land first if `geometric_qec` is written first; it has
   no dependency on the others.
3. **Then build §4.** The shape is now attested by three problems rather than asserted by one, which
   is the bar revision 1 set. It should still wait for at least two examples to be running, because
   a design that three notes agree on is weaker evidence than two programs that compile.
4. **`Ambiguous` (§5) and the experiment designer (§8) land together**, ahead of the pipeline, for
   the reason in §8.7. Both are used by two of the three examples.
5. **`SPEC-T1` and `SPEC-T2` are done** and sat outside this crate, as expected. The branching
   structure turned out to be present already, so T1 cost documentation rather than construction,
   and the cup product shipped on top of it. `SPEC-T3` (higher cup products) is now the rung
   `geometric_qec` stops short of, and the catalogue is what it feeds; see that note's §6.

The failure mode to avoid has not changed: writing the DSL first and shaping examples to justify it.
What has changed is that the examples now push back. Two of the three amended the API before a line
of it was written, which is exactly the service designs are supposed to render.

---

## 8. Designing the experiment instead of guessing it

`predict` as sketched above takes the discriminating protocol as an argument, which means a human
picked the amplification depth. That is the weakest joint in the pipeline, and closing it turns "the
counterfactual chooses which measurement to make" from a description of what the example does into
something the library does.

### 8.1 The score

Each hypothesis `hᵢ` predicts a probability `pᵢ(e)` for candidate experiment `e`. With `S` shots the
outcome is Bernoulli, so the natural question is how well `S` samples separate two predicted
distributions. The Bhattacharyya coefficient answers it in closed form:

```
BC(p, q) = √(pq) + √((1−p)(1−q))
```

and for `S` independent shots the probability of confusing the two hypotheses under the optimal test
is bounded by `½·BC^S`. Working in log space keeps it readable and numerically stable:

```
separation(i, j, e) = −S · log₂ BC(pᵢ(e), pⱼ(e))          [bits]
score(e)            = min over all pairs (i, j) of separation(i, j, e)
```

Minimising the worst-case confusion is the right criterion when every hypothesis must be
distinguished, not just the likely pair. The chosen experiment is `argmax score(e)`.

### 8.2 Why not a sigma score

A z-score, `|pᵢ − pⱼ| / σ`, is what an engineer would reach for and what the example's own output
sketch reports. It is fine for *reporting* and unreliable for *selecting*, because the normal
approximation needs roughly ten expected counts in each outcome and the good cheap experiments do
not have them.

On the example's own fault set at 1024 shots, the two scores agree closely in the bulk: both rank
`N = 12, 14, 10, 16` at the top, differing only in order. They part company where it matters. The
most cost-efficient experiment, `N = 4`, puts the detuning hypothesis at `p = 0.0038`, which is 3.9
expected counts out of 1024. The Bhattacharyya score is exact there; the z-score is not defined in
any trustworthy sense.

So: **select on bits, report in sigma.** The score that picks the experiment and the number that
explains it to a human need not be the same number.

### 8.3 What the scan actually finds

Running the score over depths 1 to 40 on the three faults from
[`example-quantum-control-loop.md`](example-quantum-control-loop.md), at 1024 shots:

| N | amplitude | detuning | decoherence | worst-pair bits | bits per pulse |
|---:|---:|---:|---:|---:|---:|
| 1 | 0.9801 | 0.9802 | 0.9802 | **0.0** | 0.0 |
| 4 | 0.2871 | 0.0038 | 0.0748 | 34.5 | **8.6** |
| 8 | 0.8187 | 0.0152 | 0.1384 | 49.5 | 6.2 |
| 9 | 0.0865 | 0.9611 | 0.8472 | 30.7 | 3.4 |
| 12 | 0.9843 | 0.0341 | 0.1925 | **53.9** | 4.5 |
| 20 | 0.0955 | 0.0927 | 0.2776 | **0.0** | 0.0 |

Three findings, and all three argue for building this.

**The hand-picked depth is dominated.** `N = 9` was chosen by the reasoning that nine repetitions
amplify an amplitude error ninefold. It yields 30.7 bits for nine pulses. `N = 4` yields 34.5 bits
for four. More separation, less than half the cost, and no human would have found it by reasoning
about amplification.

**The landscape has holes.** Worst-pair separation falls below one bit at `N = 1, 20, 25` and `40`.
At `N = 20` the amplitude and detuning hypotheses both land at about 0.094 and the experiment
discriminates nothing at all. "Deeper amplifies more" is false, and the failure is silent: the probe
runs, the answer is ambiguous, and nothing distinguishes a badly chosen depth from an incomplete
fault model.

**The method reproduces the known-bad case.** `N = 1` scores 0.0 bits. That is the degenerate
single-pulse check the whole example is built around, rediscovered by the scorer without being told.
A scoring rule that independently finds the degeneracy you already knew about is a scoring rule
worth trusting on the depths you did not check.

### 8.4 Cost, and the shape of the objective

Only the caller knows what a protocol costs, so cost is supplied per experiment rather than inferred
from depth. Three objectives cover the real cases:

```rust
pub enum DesignObjective {
    /// Maximise worst-pair separation, cost ignored.
    MaxSeparation,
    /// Maximise worst-pair separation subject to a cost ceiling.
    MaxSeparationUnderCost { budget: u64 },
    /// Maximise worst-pair separation per unit cost.
    MaxSeparationPerCost,
    /// Cheapest set of experiments that separates every pair at or above the floor (§8.8).
    MinCostCover { floor_bits: FloatType },
}
```

`MaxSeparation` picks `N = 12`; `MaxSeparationPerCost` picks `N = 4`. A calibration engineer
counting device time wants the second, and the difference between them is the whole reason the
objective is a parameter rather than a constant.

### 8.5 Where it closes the loop with `Ambiguous`

This is the part that makes §5 actionable rather than merely honest. When no experiment in the
family reaches the separation floor, the pipeline does not have to discover ambiguity after
measuring; it can state it before, and answer the operator's next question. The floor inverts
directly, since separation is linear in `S`:

```
shots_needed(target_bits) = ⌈ target_bits / (−log₂ BC(p_i, p_j)) ⌉
```

At `N = 4` the binding pair yields 0.03365 bits per shot, so the answer is a table rather than a
shrug:

```
binding pair: detuning / decoherence
  10 bits ->   298 shots
  20 bits ->   595 shots
  40 bits ->  1189 shots
  60 bits ->  1784 shots
```

"Two causes are indistinguishable at this budget, and here is the budget that would distinguish
them" is a different class of answer from "bad data."

### 8.6 API and cost

The stage sits between `fork` and `predict`, and produces a reportable artifact rather than a
silent choice:

```rust
QclBuilder::build_control(&cfg)
    .prepare()
    .evolve(1)
    .observe()
    .gate()
    .fork()
    .design(&experiments, DesignObjective::MaxSeparationPerCost)  // picks e*
    .predict()                                                    // evolves each world under e*
    .adjudicate()
    .control_analyze()
    .finalize()
    .print_results();
```

`DesignReport` carries the chosen experiment, the full ranking with bits and cost, the binding pair,
the shot-count solve, and the list of candidates that scored below the floor. The rejected
alternatives belong in the audit trail for the same reason the rejected hypotheses do: the next
occurrence should not start from the same ignorance.

Compute cost is negligible. The scan above is `|E| × |H|` model evaluations plus
`|E| × C(|H|, 2)` closed-form coefficients, which for 40 depths and 3 hypotheses is 120 plant
evolutions and 120 square roots.

### 8.7 What this is, and is not

Optimal experimental design is an established discipline, and the quantum-adaptive branch of it is
well developed: Ferrie, Granade and Cory used Bayesian experimental design to choose Hamiltonian
estimation experiments, with a posterior updated after each observation. Two honest differences:

- **This is batch, not adaptive.** One experiment is chosen against a fixed hypothesis set, before
  measuring. The sequential-posterior version is strictly stronger and is not what is proposed here.
- **This is discrimination, not estimation.** The objective separates a discrete hypothesis set
  rather than shrinking a continuous parameter's variance. That is the model-discrimination branch
  of the classical literature, not the parameter-estimation one the quantum papers mostly occupy.

The contribution is not the criterion. It is that the chosen experiment, the rejected alternatives,
the binding pair and the shot solve land in the same causal audit trail as the intervention they
justify, and that a failure to discriminate becomes a typed verdict instead of a confusing result.

**Sequencing.** In §7 this was a v2. It should move up, to land with `Ambiguous` rather than after
it. `Ambiguous` without the designer reports a dead end; `Ambiguous` with it reports a dead end, the
reason, and the budget that would clear it. The two are one feature, and the scan in §8.3 is cheap
enough that separating them buys nothing.

### 8.8 Amendment: `design` returns a plan, not an experiment

[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md) is the second consumer this
API needed, and it breaks the formulation above. On its hypothesis set every cheap experiment leaves
exactly one pair unseparated, so the worst-pair criterion of §8.1 scores all of them at 0.0 bits and
the only single experiment that resolves everything costs 200 units against the others' 1 or 2.

The criterion is not wrong; the return type is. The answer there is a *pair* of interventions, and
`design` as specified cannot express one.

The correct formulation is **minimum-cost set cover**. The elements are the `C(n,2)` hypothesis
pairs; each experiment covers the pairs it separates at or above the floor, at its cost; the plan is
the cheapest set whose union covers every pair. On that example it returns two interventions at a
combined cost of 2 against the 200 of the single experiment that would otherwise be chosen.

Consequences for the API:

- `design` returns a **`DesignPlan`**: an ordered set of experiments, the total cost, the pair each
  resolves, and any pairs left uncovered when the family is insufficient.
- The single-experiment case is a plan of length one, so §8.3's calibration result is unchanged and
  nothing is lost.
- `DesignObjective` gains `MinCostCover { floor_bits }` alongside the three in §8.4.
- Set cover is NP-hard in general and exactly solvable by enumeration at the scale this operates on.
  Above a threshold the greedy cover is the standard fallback, and its logarithmic approximation
  factor should be reported rather than hidden.
- `Ambiguous` from §5 strengthens accordingly: not merely "no experiment separates this pair" but
  "no plan within the cost budget separates it," with the shot-count inversion of §8.5 still
  available per pair.

The closure with Optimus is worth noting. `diagnose` recurses through dependencies in an order the
graph fixes. A plan is the same recursion with the order computed from what each experiment would
resolve and what it would cost.

### 8.9 Where the three examples do not agree

`design` appears in all three, which is why §2.3 calls it the most general stage. It is also the one
place the three do not reduce to a single computation, and forcing them to would be a mistake.

`quantum_control_loop` and `crosstalk_attribution` **are** the same computation. Both cover
hypothesis pairs with experiments under a cost objective; the first is the degenerate case where a
plan of length one suffices. `MinCostCover` subsumes both, and §8.8 is the honest generalisation.

`geometric_qec` is not. Its selection problem is over **cell complexes** against a requirement of
`k` logical qubits, distance `d` and a gate set, with cost in physical qubits. There is no hypothesis
set, no shot noise and no Bhattacharyya coefficient. It shares the shape, choose from a family to
meet requirements at least cost, and none of the content.

The right conclusion is that `design` is a stage parameterised by a requirement set and a coverage
relation, with `DesignObjective` naming the objective over them. Two instances share a coverage
relation; the third supplies its own. Unifying the enum across all three would produce a variant that
means something different in each arm, which is worse than two named cases.

One further honesty note on the third: two of its three requirement axes compute today, `n` and `k`
exactly and `d` by enumeration on small complexes. The gate-catalogue axis does not, and that is a
topology gap rather than a DSL one.

---

## 9. Sources

The three consumer designs: [`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`example-crosstalk-attribution.md`](example-crosstalk-attribution.md),
[`example-geometric-qec.md`](example-geometric-qec.md).

CDL's structure: `deep_causality_discovery/README.md` (typestate config builder, compile-time
isolated sub-pipelines, shared analyze/finalize tail, effect monad with short-circuit).

Crate surfaces referenced: `deep_causality_quantum/src/types/{density_matrix.rs,
qgates/channel.rs, verdict/{born.rs, projection.rs}, qpu/{bridge.rs, sampler.rs}}`,
`deep_causality_algebra/src/algebra/verdict.rs`, `deep_causality_haft/src/monoidal/mod.rs`,
`deep_causality_uncertain/src/types/uncertain/uncertain_verdict.rs`,
`deep_causality_topology/src/traits/chain_complex.rs`,
`deep_causality_topology/src/types/lattice_complex/mod.rs`,
`deep_causality_sparse/src/types/sparse_matrix/`.

Kelly, J. et al. (2018). *Physical qubit calibration on a directed acyclic graph.* arXiv:1803.03226.
The `check_data` trichotomy §5 compares against.

Experiment design (§8), for attribution rather than as a method source:

- Ferrie, C., Granade, C.E. & Cory, D.G. *Adaptive Hamiltonian Estimation Using Bayesian
  Experimental Design.* arXiv:1111.0935.
- Granade, C.E., Ferrie, C., Wiebe, N. & Cory, D.G. (2012). *Robust online Hamiltonian learning.*
  New J. Phys. **14**, 103013.

Both are adaptive, sequential, and aimed at parameter estimation. The design stage proposed here is
batch and aimed at discrimination; §8.7 states the difference rather than eliding it.
