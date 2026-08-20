<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Example Design Note: `quantum_control_loop`

**What this is.** The design for a new example under `examples/quantum_examples/`, to be reviewed
beside [`positioning.md`](positioning.md). It is the worked classical-quantum control system that
document's §9 decision 3 asks for, and it closes boundary 9 (the quantum control evidence is
currently one qubit deep and one stage long).

**Status.** Design only. No code written yet.

**The one-sentence thesis, which the example exists to earn:**

> The counterfactual does not replace the measurement. It chooses which measurement to make.

---

## 1. The job this example does

`positioning.md` makes three claims that currently rest on examples carrying *classical* plants:
the loop is inherited from the substrate (§4.1), quantum-derived decisions can be gated and logged
(C5), and the tool can produce a reasoned refusal (E4). The quantum evidence for all three is
`quantum_counterfactual`, which is four stages long and one qubit wide.

This example replaces that gap with an industry-shaped loop where the counterfactual is not a
demonstration of a language feature but the thing that makes the loop cheaper than the standard
practice it is compared against.

---

## 2. The industry system it reproduces

Kelly, O'Malley, Neeley, Neven and Martinis (Google, arXiv:1803.03226) introduced **Optimus**: qubit
calibration modelled as a directed acyclic graph, where each calibration experiment is a node and
the bootstrapping dependency between calibrations is a directed edge. Their motivation is drift:
"optimal parameters are typically different between devices and can also drift in time, which begets
the need for an efficient calibration strategy." Their result is that "calibration is reduced to a
graph traversal problem that is automatable and extensible."

The routines are four:

- **`check_state`** is *non-experimental*. It passes only if the node passed within its timeout, has
  no unresolved `calibrate` failure, has had no dependency recalibrated since, and all dependencies
  themselves pass `check_state`.
- **`check_data`** acquires a *minimal* number of points and returns one of **three** verdicts:
  **in spec** (points sit on the expected curve), **out of spec** (a systematic offset), or **bad
  data** (the points look like noise, which means the scan itself is not working because a
  dependency is bad).
- **`calibrate`** is the full scan: lots of data, new parameters.
- **`maintain`** recurses toward the root, and at the first node failing `check_state` runs
  `check_data`, then proceeds, calls `calibrate`, or hands off to `diagnose` according to the three
  verdicts.
- **`diagnose`** is invoked *only on bad data*. It makes no `check_state` calls, works backward
  through the dependencies with `check_data` and `calibrate` to find the node that actually failed,
  and resumes `maintain` once the mismatch is resolved.

The load-bearing distinction is **out of spec against bad data**. Out of spec means this node's
parameters drifted while its experiment still works. Bad data means the experiment itself is
invalid because something upstream is wrong.

This is not a historical curiosity. A April-2026 paper opens with the observation that "closed-loop
superconducting-qubit calibration has matured into DAG-orchestrated protocol chains," and its
criticism of those chains is that they treat the environment "via a Markovian master equation or a
phenomenological likelihood, absorbing bath structure into fit residuals instead of reporting it as
a diagnostic" (arXiv:2604.21458). That is the same complaint `positioning.md` files under C2: real
structure gets flattened into a fit parameter.

Google built a causal diagnosis engine over a dependency graph and called it graph traversal. This
codebase has the graph, the intervention, and the audit trail as first-class objects. The example is
the meeting point.

---

## 3. The gap: `diagnose` is destructive

Everything downstream of `check_data` depends on classifying a cheap measurement into in spec, out
of spec, or bad data. The paper's method for that classification is to overlay a few points on the
expected curve and judge whether the deviation looks systematic or like noise. When two different
physical causes produce the same minimal-data signature, the classification is undecidable from the
data `check_data` is allowed to take, and the traversal picks a branch on a prior.

From there the cost is structural. `diagnose` finds the root cause by *acting*: it recalibrates a
candidate on hardware and re-checks, moving on if the check still fails. Two properties follow, and
both are paid in device time:

1. **A wrong attribution costs a full sweep.** `calibrate` is the expensive experiment, and running
   it against the wrong cause returns to the same alarm having learned only that the guess was
   wrong.
2. **The rejected hypothesis is never recorded.** Nothing states what the other cause would have
   predicted, so the next occurrence starts from the same ignorance.

The question the traversal cannot ask is the counterfactual: *if the cause were B rather than A,
what would this device do differently, and what single measurement would tell them apart?*

---

## 4. The physics: one check, three causes, one reading

A single qubit driven by a nominal π pulse. Let `a` be the amplitude scale (1 when calibrated) and
`d = Δ/Ω` the fractional detuning (0 when calibrated). With the pulse duration fixed at the nominal
`t = π/Ω`, the rotation is by `θ = π·√(a² + d²)` about the tilted axis `(a, 0, d)`, and the excited
population starting from `|0⟩` is

```
P(|1⟩) = a²/(a² + d²) · sin²( π·√(a² + d²) / 2 )
```

Three faults a calibration engineer meets constantly, each tuned here to produce the *same* reading:

| Fault | Parameters | `check_data` reading at N = 1 |
|---|---|---|
| A. Amplitude drift | `a = 1.09`, `d = 0` | **0.9801** |
| B. Detuning drift | `a = 1`, `d = 0.1414` | **0.9802** |
| C. Decoherence | `a = 1`, `d = 0`, depolarizing `p = 0.0397` per pulse | **0.9802** |

Against a 0.999 spec all three fail, and they fail identically. The single-pulse check is
degenerate: three causes, one number.

The three map onto Optimus's own trichotomy, which is why this fault set is the right one to
demonstrate against:

| Fault | Optimus verdict it should produce | Correct action |
|---|---|---|
| A. Amplitude | out of spec at this node | `calibrate` here |
| B. Detuning | bad data: the upstream qubit-frequency node drifted | `diagnose` upstream |
| C. Decoherence | neither; no node's parameters can fix it | escalate, do not tune |

A cheap check that cannot separate them is a cheap check that cannot pick the branch.

The third cause is the one that matters operationally. A decoherence fault is **not fixable by
retuning**. Amplitude cannot beat it: under depolarizing the best any pulse achieves is `1 − p/2`,
so the sweep runs, converges on the parameters it already had, and reports failure. The correct
action is to flag the qubit as degraded and escalate, and no amount of graph traversal reaches that
conclusion without first burning the sweeps.

---

## 5. The discriminator: error amplification

Repeating the pulse `N = 9` times separates the three cleanly, because the three faults accumulate
differently. Amplitude error accumulates linearly in the rotation angle. Detuning mostly tilts the
axis, so its angle error is second order and stays small. Depolarizing shrinks the Bloch vector
geometrically toward the maximally mixed state.

| Fault | Predicted reading at N = 9 | Mechanism |
|---|---|---|
| A. Amplitude | **0.0865** | total angle `9π·1.09`, error amplified 9× |
| B. Detuning | **0.9611** | axis tilt; angle error second order |
| C. Decoherence | **0.8472** | `(1 + (1−p)⁹)/2`, Bloch vector shrinks |

Three predictions, well separated, from one cheap probe. At a realistic 1024-shot budget the
binomial standard error runs from 0.0060 to 0.0112 across the three, so C sits 10σ from B and 68σ
from A. The discrimination is statistically decisive, not merely visually obvious.

A second, free discriminator falls out of the operator layer: `DensityMatrix::purity()` drops only
under C. The example reports it as a cross-check, not as the primary evidence.

Error amplification is standard laboratory practice, not an invention of this note. What the example
contributes is deciding *which* amplified sequence to run, before running it, by asking each
hypothesis what it predicts.

---

## 6. The pipeline

Six stages, one `CausalFlow`, with the plant on the state channel.

| Stage | What happens | Substrate feature |
|---|---|---|
| 1. `check_data` | Evolve the plant through one nominal π pulse; sample the readout under a shot budget | `DensityMatrix`, `apply_kraus`, Born read-out, `Uncertain` |
| 2. Envelope | Compare against the 0.999 spec; below it, raise the alarm | `guard` / `branch` |
| 3. Counterfactual fork | Fork the plant state three ways, one per hypothesis, and evolve each forward under the N = 9 amplification sequence *on the model* | state-channel fork, `alternate_value` per world |
| 4. Adjudicate | Run the N = 9 probe **once** on the device; abduce the hypothesis whose prediction matches within the shot uncertainty | `Uncertain` comparison, typed verdict |
| 5. Commit or refuse | For A or B, apply the correction and log it. For C, report that no tuning parameter can reach spec and escalate | `alternate_value`, `EffectLog` |
| 6. Verify | Re-run `check_data` at N = 1 and N = 9; report both | the same stage-1 machinery |

Stage 3 is the quantum counterfactual, and it is load-bearing rather than decorative: without it,
stage 4 has no reason to prefer one probe over another, and the loop degenerates into Optimus.

The binary runs the loop twice, in the manner the corrective-control examples already use:

- **Run 1, ground truth C (decoherence).** The baseline burns an amplitude sweep and a detuning
  sweep, both of which return to the same alarm, then escalates. The counterfactual loop spends one
  cheap probe, identifies C, and refuses to retune.
- **Run 2, ground truth B (detuning).** The baseline burns one amplitude sweep before finding it.
  The counterfactual loop spends one cheap probe, commits the frequency correction, and verifies.

The scoreboard is device operations, which is the currency a calibration engineer actually counts.

---

## 7. What each crate in the ecosystem contributes

The example is deliberately built across the stack rather than against the quantum crate alone,
because the positioning claim is that the loop is substrate-level.

| Crate | Contribution | Necessity |
|---|---|---|
| `deep_causality_core` | `CausalFlow` pipeline, `alternate_value` as the intervention, `branch`/`guard` for the envelope, `EffectLog` for the audit trail | core |
| `deep_causality_quantum` | `DensityMatrix` and `purity`, `apply_kraus` for the depolarizing channel, `born_projective_prob` for the read-out verdict, the gate kernels | core |
| `deep_causality_multivector` | `HilbertState` as the ideal target ket, `Metric::Euclidean` | core |
| `deep_causality_num_complex` | `Complex` amplitudes | core |
| `deep_causality_tensor` | `CausalTensor` is the matrix carrier beneath `DensityMatrix` and `Projection` | transitive |
| `deep_causality_uncertain` | Probe readings as `Uncertain<f64>` rather than exact floats, so the adjudication is a statistical decision with a stated confidence. Reached through the quantum crate's `qpu` feature and its `shots_to_qubit_bernoulli` bridge. `Uncertain<f64>` and `Uncertain<bool>` already implement `Verdict`, so a shot-noise reading is a lawful lattice carrier and needs no special case at a join | core |
| `deep_causality_algebra` | The `Verdict` bounded lattice the readings and the diagnosis verdict are carried in; `Prob` is the MV carrier the Born read-out lands in | core |
| `deep_causality_haft` | `SymMonoidal::merge`, the `∇` two branches fuse through, with Lean-checked monoid laws | transitive |
| `deep_causality_rand` | Seeded binomial shot draws, so a run is reproducible | core |
| `deep_causality` | The Optimus DAG itself as a causaloid graph, with `maintain` and `diagnose` as traversals over real hyperedges | option |

The `Uncertain` row is the one that changes the example's character. A calibration decision made on
exact floating-point probabilities is a toy. A calibration decision made on 1024 shots, with the
margin between hypotheses reported against the shot noise, is the real problem, and the workspace
already has the type for it.

---

## 8. Sketch of the output

```
=== Qubit calibration: counterfactual diagnosis ===
device: transmon-q3   spec: F >= 0.9990   shots: 1024   seed: 20260820

--- Run 1: hidden fault = decoherence -------------------------------
[check_data] N=1  P(|1>) = 0.9802 +/- 0.0044   FAIL (spec 0.9990)
[alarm] three causes predict this reading:
          amplitude  a=1.090          detuning  d=0.1414         decoherence p=0.0397

  Optimus baseline
    sweep amplitude  ... 2 048 pulses, 16 384 shots -> 0.9802  still failing
    sweep detuning   ... 2 048 pulses, 16 384 shots -> 0.9802  still failing
    escalate: qubit degraded                       device ops: 4 096 pulses

  Counterfactual loop
    fork -> 3 worlds, amplified probe N=9 predicted on the model
       world A amplitude    -> 0.0865
       world B detuning     -> 0.9611
       world C decoherence  -> 0.8472
    probe N=9 on device ... 9 pulses, 1 024 shots -> 0.8461 +/- 0.0113
    abduce: world C matches (0.1 sigma); A rejected (68 sigma), B rejected (10 sigma)
    purity cross-check: 0.741  (pure states would read 1.000)
    [REFUSE] decoherence is not a tuning fault. Qubit marked degraded, escalated.
                                                   device ops: 9 pulses
  no !!ValueAlternation!! recorded: no tuning parameter reaches spec under decoherence

--- Run 2: hidden fault = detuning ----------------------------------
  ... same shape, ends in a committed frequency correction and a passing verify ...

=== Summary ===
  Run 1  baseline 4 096 pulses -> escalate    counterfactual 9 pulses -> escalate
  Run 2  baseline 2 057 pulses -> corrected   counterfactual 9 pulses -> corrected
```

Numbers in the sketch are the model predictions computed in §4 and §5; the sampled figures are
illustrative until the code runs.

---

## 9. File layout and build

Following the example layout convention and the Bazel rule in `AGENTS.md`:

```
examples/quantum_examples/quantum_control_loop/
  README.md          the standard example README
  main.rs            wiring: two runs, baseline against counterfactual loop
  constants.rs       spec threshold, shot budget, N_AMPLIFY, seed, fault magnitudes
  model_config.rs    device configuration and the hidden ground-truth fault
  model_types.rs     Hypothesis, PlantState, ProbeReading, Diagnosis, Action
  model.rs           the rotation and channel physics, the probe, the six stages
  utils_print.rs     output formatting
```

Registration is needed in three places: an `[[example]]` entry in
`examples/quantum_examples/Cargo.toml`, a matching `rust_binary` in
`examples/quantum_examples/BUILD.bazel` with the crates from §7 in `deps`, and a row in the package
README table. `make check_examples` fails if the Bazel target is missing.

Conventions that apply: a per-example `FloatType` alias with no raw `f64` above the display
boundary, config separated from execution, no test module (examples are verified by running).

---

## 10. What the example must not claim

Every one of these belongs in the README and in the site copy that points at it.

1. **The counterfactual runs on the model, not on the device.** It predicts what each hypothesis
   implies for a probe that has not been run. The device still adjudicates. A model that is wrong
   about the fault mechanism will propose the wrong probe, and the example should say so.
2. **The plant is a simulation.** There is no hardware in the loop, and the `qpu` seam here is the
   in-process path. Nothing about vendor latency, queueing or calibration drift over real time is
   demonstrated.
3. **One qubit, three faults.** Real calibration graphs have dozens of nodes and the fault space is
   not three-valued. The example demonstrates the mechanism, not coverage.
4. **The device-operation counts are model bookkeeping**, not measured wall-clock on hardware. They
   should be reported as pulse and shot counts, which are honest, rather than as time.
5. **No speedup claim.** The saving is device operations avoided by not running the wrong
   experiment. That is an experimental-design saving, not a computational one.

---

## 11. Scope options

**Core** (recommended floor). Sections 4 through 8, with the crates marked "core" in §7. Runnable,
honest, and enough to carry the positioning claim.

**Option: the real Optimus DAG.** Model the calibration dependency graph as a graph, with
`maintain` and `diagnose` as traversals, matching Kelly et al. node for node. Which graph is the
open question, and the honest answer depends on whether the goal is to reproduce Optimus or to
exceed it.

*`ultragraph` suffices to reproduce it.* The crate already carries the dual-state machine
(`DynamicGraph` for construction, frozen `CsmGraph` for analysis), `has_cycle`, `find_cycle`,
`topological_sort`, `is_reachable`, `shortest_path`, and `inbound_edges` / `outbound_edges`.
`maintain` is a walk up the inbound edges to the root; `diagnose` is the same walk under a different
stopping rule. Nothing in the published algorithm needs more than that, and a faithful reproduction
should say so rather than reach for machinery it does not use.

*`CausaloidGraph` earns its place only for what Optimus cannot do.* Five things it adds are not
recoverable by writing more code on top of `ultragraph`:

1. **A freeze that verifies and rolls back.** `freeze_verified_with_check` runs acyclicity, then
   the single-writer invariant, then a caller-supplied level check, and returns the graph to its
   dynamic state if any of them fails. `ultragraph`'s `freeze()` is a representation change with no
   verification contract. This is the same hook `freeze_quantum` already uses.
2. **The single-writer invariant at reconvergent joins.** At most one incoming branch of a join may
   write the state channel, checked structurally against declared writers and tied to
   `core.causaloid.graph_fold_order_invariant`. In calibration terms this is a real hazard, not a
   formality: two calibration routines that both write the same physical parameter, reachable from
   different branches of a join, are a race. A calibration DAG is genuinely reconvergent, because a
   two-qubit gate node sits above two single-qubit chains. `ultragraph` gives the topology of that
   join and nothing about the conflict.
3. **A reconvergence policy.** The evaluator fixes what happens at a join: the value channel folds
   `∇`, logs concatenate in ascending parent order, and state never merges. Note carefully what is
   and is not CausaloidGraph's here. The *algebra* is not: `Verdict` (a bounded lattice with
   complement: `bottom`, `top`, `meet`, `join`, `complement`) lives in `deep_causality_algebra`,
   and `∇` itself is `SymMonoidal::merge` in `deep_causality_haft`, whose laws are checked in
   `Haft/SymmetricMonoidal.lean`. Both are usable from a hand-written driver over `ultragraph`.
   What CausaloidGraph contributes is the *policy*: the decision that this is what a join does, and
   the refusal to merge state.
4. **`RelayTo` as recorded control flow.** A causaloid may return `RelayTo(target, input)`, which
   ends the evaluation round and resumes at the target with the log intact. That is precisely
   `diagnose` handing back to `maintain`. The effect itself is `deep_causality_core`'s
   `CausalCommand`; what the graph adds is interpreting the target as a node and preserving the log
   across the jump.
5. **Λ edge decorations.** `evaluate_subgraph_from_cause_with_lambda_edges` transforms the value
   along an edge before the join, so the transfer relation between a dependency's parameter and its
   dependent's expectation becomes edge data instead of being buried in each node's code.

Of those five, only 1, 2 and 5 are genuinely unavailable elsewhere. The verdict lattice and the
merge monoid are separate, separately-formalized crates that any driver can call, and `RelayTo` is
a core effect. That narrowing is worth stating plainly rather than padding the list.

*Recommendation: defer either way, and if it is built, build it on `CausaloidGraph` with the
single-writer check as the stated reason.* A second example that reproduces Optimus on `ultragraph`
demonstrates that the workspace has a good graph library. One that freezes the calibration DAG under
`freeze_verified_with_check` and rejects a two-writer join demonstrates something Optimus cannot
express, which is the only reason to write it.

---

## 12. Open questions for you

1. **Name.** `quantum_control_loop` is my proposal. `calibration_counterfactual` says what it does
   more literally. Which?
2. **Keep or absorb `quantum_counterfactual`?** It is the minimal, pedagogical version of the same
   idea. *Recommendation: keep it as the small example and cross-link the two; retiring it would
   lose the four-line teaching version.*
3. **The causaloid DAG: now or as a follow-up example?** *Recommendation: follow-up.*
4. **`Uncertain` through the `qpu` feature.** That path is off by default, so the example either
   enables the feature or samples shots locally with `deep_causality_rand` and skips the bridge.
   *Recommendation: enable the feature, because exercising the emergent seam is part of the point,
   and the in-process simulator keeps it dependency-free.*
5. **A third run?** A ground-truth-amplitude run would complete the matrix, at the cost of a longer
   output. *Recommendation: two runs; the amplitude case is the easy one and adds little.*

---

## 13. Sources

- Kelly, J., O'Malley, P., Neeley, M., Neven, H. & Martinis, J.M. (2018). *Physical qubit
  calibration on a directed acyclic graph.* arXiv:1803.03226. The Optimus framework; calibration as
  graph traversal; drift as the motivation.
- Ye, J. (2026). *HEOM-in-Calibration-Loop: Exposing Non-Markovian Bath Signatures That Markovian
  Calibration Elides in Superconducting-Qubit Tune-Up.* arXiv:2604.21458. Confirms DAG-orchestrated
  calibration as current practice, and names the diagnostic loss when bath structure is absorbed
  into fit residuals.
- Battistel, F. et al. (2023). *Real-time decoding for fault-tolerant quantum computing.* Nano
  Futures 7(3), 032003. The neighbouring loop, and the reason boundary 8 in `positioning.md` exists.

Code this example builds on: `deep_causality_core/src/types/causal_flow/`,
`deep_causality_quantum/src/types/{density_matrix.rs, qgates/channel.rs, verdict/born.rs, qpu/}`,
`examples/causal_correction_examples/` (the open-loop against closed-loop reporting pattern),
`examples/quantum_examples/quantum_counterfactual/` (the state-channel history pattern).

Companion note: [`positioning.md`](positioning.md).
