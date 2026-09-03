<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Example Design Note: `crosstalk_attribution`

**What this is.** The design for a second example under `examples/quantum_examples/`, showcasing the
QCL sketch from [`qcl-dsl-liftback.md`](qcl-dsl-liftback.md). Companion to
[`positioning.md`](positioning.md) and [`example-quantum-control-loop.md`](example-quantum-control-loop.md).

**Status.** Design only. Every number below was computed, including the faithfulness result, which
was checked against a transcription of the crate's own `is_c3_block` algorithm rather than assumed.

**The thesis:**

> Two qubits error together. Nothing you can observe tells you whether one drives the other or both
> answer to something else. Only an intervention does, and which intervention to run is a
> calculation, not a habit.

---

## 1. Why this example and not another

`positioning.md` names correlated noise as challenge C2 and marks its demonstrator `[planned]`,
because the decoder is Track D and Track D has no number yet. That leaves the P2 audience, device
and noise characterization, with nothing addressed to them. This example fills that gap without
touching the decoder claim.

It also does something the calibration example cannot: it exercises **both halves** of the QCL
sketch. The validate pipeline screens the hypothesis set, and the control pipeline discriminates
what survives. One problem, both pipelines, which is the showcase.

And it is the crate's own thesis doing a day's work. Allen et al. (2017) state that complete common
cause *is* factorization into commuting Choi factors. The freeze gate tests exactly that. Here the
question "direct cause or common cause" is the working question, not an illustration of one.

---

## 2. The problem

Two qubits on a device show correlated error events. Three structures explain the correlation:

| Hypothesis | Mechanism | The fix it implies |
|---|---|---|
| **H₁** Q1 → Q2 | ZZ coupling, or control-line spillover from Q1's drive | reschedule gates, add echo |
| **H₂** Q2 → Q1 | the mirror | reschedule the other way |
| **H₃** Q1 ← B → Q2 | a shared two-level-system defect or common bath | no scheduling fix exists; move the qubit frequency or accept the loss |

The three imply completely different engineering responses, and an operator who guesses wrong spends
a scheduling campaign on a problem no schedule can reach.

### 2.1 The degeneracy is structural, not tuned

In the calibration example the three faults were tuned to collide on one reading. Here nothing needs
tuning. Over two binary error indicators, the three structures are **Markov equivalent**: each has
three free parameters and the observed joint distribution has three degrees of freedom, so each fits
any observation exactly. That is Reichenbach's common cause principle, and it is a theorem rather
than a coincidence.

Taking `P(e₁) = P(e₂) = 0.10` and `P(e₁, e₂) = 0.04`, against `0.01` under independence, so a
correlation coefficient of `0.3333`:

| Hypothesis | Fitted parameters | Reproduces the observation |
|---|---|---|
| H₁ | `P(e₁) = 0.1000`, `P(e₂｜e₁) = 0.4000`, `P(e₂｜¬e₁) = 0.0667` | exactly |
| H₂ | the mirror image | exactly |
| H₃ | `λ = 0.10`, `c_hi = 0.619615`, `c_lo = 0.042265` | marginal `0.100000`, joint `0.040000` |

No amount of passive data separates them. The calibration example had to arrange its degeneracy;
this one cannot avoid it.

---

## 3. Grounding, and what the literature does not say

[Binney et al., arXiv:2603.16494](https://arxiv.org/html/2603.16494) (Oliver group, MIT and Lincoln
Laboratory, August 2026) is the evidence that this class of investigation is real and expensive.
They found two populations of spatiotemporally correlated errors coexisting in one device, one from
ionizing radiation producing quasiparticles and one from pulse-tube cryocooler vibration, and
separated them with a campaign: matched filtering, frequency-domain analysis, a **pulse-tube
shutdown**, a spatial asymmetry metric, a **swap into a second refrigerator**, and accelerometer
measurements. Eighteen authors. The shutdown and the refrigerator swap are interventions, and the
campaign is a hypothesis set against a family of experiments with wildly different costs.

Two corrections, because they change what may be claimed.

The first is narrow. A search summary reported that the process-tensor literature already frames this
in the vocabulary of common cause and direct cause, citing White's thesis (arXiv:2405.05416). Reading
the source did not confirm that vocabulary.

The second is not narrow, and it retires an earlier draft of this section. **Statistical causal
inference was applied to crosstalk in quantum processors six years ago.** Sarovar, Proctor, Rudinger,
Young, Nielsen & Blume-Kohout, *Detecting crosstalk errors in quantum information processors*
(Quantum **4**, 321, 2020) adapts the PC algorithm, tests conditional independence under the Markov
condition, and resolves direction as well as existence, at O(n²) to O(n³) experiments, demonstrated
on 2- and 6-qubit processors. The causal framing for crosstalk belongs to that paper and this example
must cite it rather than present the framing as new.

What the example adds sits inside that framing, and it is narrower than the earlier draft implied.
The Sandia protocol runs a **fixed experiment set**. It does not enumerate named hypotheses, so an
inadmissible structure is never considered and never rejected; it does not screen for admissibility
before device time is spent; and it does not select experiments against a cost budget. This example
enumerates four structures including a cyclic one, rejects that one as a `C₃` before any shots, and
then buys the cheapest experiment subset that separates the survivors by a stated number of bits.
That conjunction — discrete structural hypotheses, admissibility screening, cost-bounded design — is
the contribution. The causal vocabulary is not.

---

## 4. The interventions

Passive observation is exhausted, so the pipeline intervenes. Each experiment is a preparation, an
idle or echo sequence, and a measurement, with a cost in device cycles supplied by the caller.

| Experiment | What it does | Observable | Cost |
|---|---|---|---:|
| **E0** | both qubits idle, as observed | `P(e₁, e₂)` | 1 |
| **E1** | `do(Q1 = ｜1⟩)`, hold Q1 excited | `P(e₂)` | 1 |
| **E2** | `do(Q2 = ｜1⟩)`, hold Q2 excited | `P(e₁)` | 1 |
| **E3** | echo both qubits | `P(e₁, e₂)` | 2 |
| **E4** | two-qubit process tensor tomography | full process tensor | 200 |

The reasoning behind each is ordinary device physics. Holding Q1 excited maximally exposes a ZZ
shift on Q2, so under H₁ Q2's error rate jumps while under H₂ and H₃ it does not move. An echo
refocuses a quasi-static ZZ coupling but not a bath fluctuating on the sequence timescale, so it
suppresses the correlation under H₁ and H₂ and leaves it under H₃. The echo behaviour is a
**modelling assumption** about the bath spectrum and belongs in the example's stated assumptions,
not in its conclusions.

Predicted readings:

| | H₁ | H₂ | H₃ |
|---|---:|---:|---:|
| E0 `P(e₁,e₂)` | 0.04 | 0.04 | 0.04 |
| E1 `P(e₂)` | **0.40** | 0.10 | 0.10 |
| E2 `P(e₁)` | 0.10 | **0.40** | 0.10 |
| E3 `P(e₁,e₂)` | 0.01 | 0.01 | **0.04** |

---

## 5. What the design scan finds, and how it breaks §8

Scoring with the Bhattacharyya separation of `qcl-dsl-liftback.md` §8, at 1024 shots with a 5-bit
floor:

| Experiment | H₁/H₂ | H₁/H₃ | H₂/H₃ | worst pair | pairs covered | cost |
|---|---:|---:|---:|---:|---:|---:|
| E0 passive | 0.0 | 0.0 | 0.0 | **0.0** | 0 of 3 | 1 |
| E1 `do(Q1)` | 99.5 | 99.5 | 0.0 | **0.0** | 2 of 3 | 1 |
| E2 `do(Q2)` | 99.5 | 0.0 | 99.5 | **0.0** | 2 of 3 | 1 |
| E3 echo | 0.0 | 7.6 | 7.6 | **0.0** | 2 of 3 | 2 |
| E4 tomography | resolves the structure directly | | | | 3 of 3 | 200 |

Two results, and the second is the important one.

**E0 scores zero, which validates the scorer.** The passive experiment separates nothing, exactly as
the Markov-equivalence argument in §2.1 requires. The scoring rule independently rediscovers the
degeneracy without being told, which is the same check the calibration example's `N = 1` provided.

**The §8 designer fails outright here.** Its criterion is the worst-pair separation of a single
experiment, and every cheap experiment leaves one pair at zero bits. Minimax over single experiments
therefore ranks them all equal at 0.0 and, on a tie, returns nothing useful. The only single
experiment that resolves everything is E4, at 200 units.

That is not a flaw in the physics. It is a missing assumption in the API: **`design` returns an
experiment, and the answer here is a plan.**

---

## 6. What this contributes back to the DSL

The correct formulation is minimum-cost set cover. The elements to cover are the `C(n,2)` hypothesis
pairs; each experiment covers the pairs it separates at or above the floor, at its cost; the plan is
the cheapest set of experiments whose union covers every pair.

Solved on this family:

```
minimum-cost plan: { E1 do(Q1=|1>), E2 do(Q2=|1>) }   total cost 2
  E1 covers H1/H2, H1/H3        E2 covers H1/H2, H2/H3        union: all three pairs
versus E4 alone: cost 200   ->   100x cheaper
```

Two cheap interventions, chosen by computation, replace a process tensor tomography. That is the
example's headline number and it is the kind a characterization engineer converts into calendar time
without help.

So `qcl-dsl-liftback.md` §8 needs one amendment, and this note is the second consumer that
establishes it:

- `design` returns a **`DesignPlan`**: an ordered set of experiments, their total cost, the pair each
  one resolves, and the pairs left uncovered if the family is insufficient.
- The single-experiment case is the plan of length one, so nothing is lost.
- Set cover is NP-hard in general and trivially exact here by enumerating the `C(n,2)` pairs rather
  than the experiments: the DP over covered-pair subsets is linear in the experiment count.
  Above a threshold the greedy cover is the standard fallback and its logarithmic factor should be
  reported rather than hidden.
- `Ambiguous` becomes richer: not only "no experiment separates these two," but "no *plan* within the
  cost budget separates them," with the shot-count inversion from §8.5 still available per pair.

There is a pleasing closure here. Optimus's `diagnose` recurses through dependencies in an order
fixed by the graph. A plan is the same idea with the order computed from what each experiment would
resolve and what it would cost.

---

## 7. Both pipelines on one problem

This is what the calibration example cannot demonstrate.

### 7.1 Validate screens the hypothesis set

Each hypothesis is a causal structure over the systems `{Q1, Q2, B}`, expressed as the bipartite
input-to-output influence relation `CausalStructure` already carries. A fourth hypothesis is
physically reasonable and worth putting in the set: **H₄, a cyclic influence chain** where Q1 drives
Q2, Q2 heats the bath, and the bath feeds back to Q1, each system also influencing itself.

Running the crate's C₃-exclusion criterion over all four, as corrected on 2026-09-02:

```
H1  Q1->Q2 (direct)          passes
H2  Q2->Q1 (direct)          passes
H3  Q1<-B->Q2 (common)       passes
H4  Q1->Q2->B->Q1 (cyclic)   passes the C₃ criterion; rejected at build() as cyclic
```

**H₄ is not a `C₃`, and this section used to say it was.** H₄ as drawn is `K₃,₃` minus a perfect
matching, the bipartite 6-cycle: every input influences two of the three outputs and every output
is influenced by two of the three inputs. Van der Lugt and Lorenz's `C₃` (arXiv:2508.11762,
Example 2.12) is the causal structure of two commuting CNOTs, which has *seven* edges: one input
reaches every output, one output is reached by every input, and the two missing pairs share neither
an input nor an output. Definition 3.1 excludes exactly that relation, and the 6-cycle satisfies
the property — Theorem 4.9(v) admits it directly, since any two of its outputs share exactly one
parent. The earlier result was computed against a faithful transcription of a wrong
`is_c3_block`, which tested for the 6-cycle; the check is corrected and its tests are now held to
the paper. The record is `qcl-corrections.md`, X-16.

Nor is the 6-cycle what reachability on a cyclic graph produces. `from_graph_reachability` takes
the transitive closure, and on `Q1 → Q2 → B → Q1` every input reaches every output: the derived
relation is complete, which satisfies C₃-exclusion more plainly still.

So the C₃ criterion screens nothing here, and the operational consequence survives for a different
reason. **Cyclic causal structures are out of scope for v1 by decision, not because they fail a
check.** Cyclic QCMs exist (Barrett, Lorenz & Oreshkov, arXiv:2002.12157); the C₃ criterion is
applied to acyclic influence relations, and a cyclic candidate is rejected at `build()` with
`CyclicStructureUnsupported` before any check runs. H₄ still never reaches the control pipeline,
and no device time is spent on it — but the rejection names a scope limit, not an obstruction, and
a later version that admits cyclic structures would have to say what criterion replaces this one.

### 7.2 Control discriminates what survives

Three hypotheses reach the control pipeline. The plan from §6 runs, the interventions execute, and
`adjudicate` returns the surviving structure with the rejected ones and their margins.

### 7.3 The pipeline

```rust
QclBuilder::build_validate(&cfg)
    .declare_factors()        // one Choi factorization per hypothesis
    .declare_supports()
    .check_markov()           // is each factorization a legal QCM?
    .check_decomposable()     // C3-exclusion; H4 was already rejected at build() as cyclic
    .validate_analyze()
    .finalize()
    .print_results();

QclBuilder::build_control(&cfg.with_hypotheses(surviving))
    .prepare()
    .evolve(1)
    .observe()                // E0, the passive baseline
    .gate()                   // correlated beyond independence -> alarm
    .fork()                   // one world per surviving hypothesis
    .design(&experiments, DesignObjective::MinCostCover { floor_bits: ft(5.0) })
    .predict()
    .adjudicate()
    .control_analyze()
    .finalize()
    .print_results();
```

---

## 8. Sketch of the output

```
=== Crosstalk attribution: direct cause or common cause ===
device: transmon-q3/q4   shots: 1024   floor: 5.0 bits

[validate] four declared structures
    H1 Q1->Q2        Markov ok   faithful ok
    H2 Q2->Q1        Markov ok   faithful ok
    H3 Q1<-B->Q2     Markov ok   faithful ok
    H4 Q1->Q2->B->Q1 Markov ok   REJECTED: C3 sub-relation, inputs (0,1,2) outputs (3,4,5)
                                 no traditional-circuit faithful decomposition
    3 of 4 hypotheses admitted to discrimination

[E0 passive] P(e1)=0.100  P(e2)=0.100  P(e1,e2)=0.040   correlation 0.333
    all three admitted hypotheses fit this exactly (Markov equivalent)
    worst-pair separation 0.0 bits -> observation is exhausted

[design] minimum-cost cover, floor 5.0 bits
    E1 do(Q1=|1>)  cost   1   covers H1/H2 (99.5 bits), H1/H3 (99.5 bits)
    E2 do(Q2=|1>)  cost   1   covers H1/H2 (99.5 bits), H2/H3 (99.5 bits)
    E3 echo both   cost   2   covers H1/H3 (7.6 bits),  H2/H3 (7.6 bits)
    E4 process tomography  cost 200   covers all
    plan: {E1, E2}  total cost 2   (E4 alone would cost 200, 100x more)

[E1] P(e2) = 0.401 +/- 0.015   H1 predicts 0.400, H2/H3 predict 0.100
[E2] P(e1) = 0.104 +/- 0.010   H1 predicts 0.100, H2 predicts 0.400, H3 predicts 0.100

[adjudicate] H1 confirmed. H2 rejected (E2, 29 sigma). H3 rejected (E1, 20 sigma).
    Q1 drives Q2. Direct cause, not shared bath.
    -> a scheduling or echo fix applies. Frequency reallocation is not required.
```

Sampled figures are illustrative; the predictions and the plan are computed.

---

## 9. Crates, layout, build

Same layout convention and Bazel rule as the calibration example.

| Crate | Contribution |
|---|---|
| `deep_causality_core` | the pipeline, the audit trail |
| `deep_causality_quantum` | `CausalStructure` and the C₃ gate, `ProcessFactors` and the Markov check, `DensityMatrix`, the channel layer |
| `deep_causality` | the frozen graph the structure is derived from |
| `deep_causality_multivector`, `deep_causality_num_complex`, `deep_causality_tensor` | carriers |
| `deep_causality_uncertain` | shot-noise readings as `Uncertain<f64>`, already a `Verdict` |
| `deep_causality_rand` | seeded shot draws |

```
examples/quantum_examples/crosstalk_attribution/
  README.md  main.rs  constants.rs  model_config.rs  model_types.rs  model.rs  utils_print.rs
```

---

## 10. What the example must not claim

1. **The noise model is phenomenological.** Error probabilities are declared, not derived from a
   master equation. That is honest for a demonstrator and must be stated, because the adjudication
   inherits the model's correctness.
2. **The echo assumption is an assumption.** Whether an echo suppresses the correlation depends on
   the bath spectrum relative to the sequence timescale. E3's discriminating power rests on that,
   and a device whose bath is quasi-static would invert it.
3. **Two qubits and one bath mode.** Real crosstalk is many-body and real devices have many
   defects. The example demonstrates the mechanism, not coverage.
4. **Simulated, no hardware.** No claim about wall-clock, queueing or drift over real time.
5. **The cost model is caller-supplied.** The 100× is a ratio of declared cycle costs, not a
   measured saving on a device.
6. **`do(Q1 = ｜1⟩)` is an idealisation.** A real hold has its own error mechanisms, which a serious
   version would fold into the predictions.

---

## 11. Open questions

1. **Does this displace the calibration example or accompany it?** They share a shape: hypotheses,
   degenerate observation, designed experiment. *Recommendation: both, in this order.* Calibration
   discriminates parameter faults on one system and is the gentler introduction; this discriminates
   causal structures across systems, which is the harder question and the one the design
   stage is built for.
2. **Is H₄ worth including given it is rejected?** *Recommendation: yes.* A validation gate that
   never rejects anything reads as decoration. H₄ is physically reasonable, and its rejection is the
   only demonstration in the repository of the C₃ criterion refusing a model an engineer might
   actually write down.
3. **Does `design` learn set cover now, or does this example hard-code the plan?** *Recommendation:
   implement the cover.* It is a small exact enumeration at this scale, and without it the example
   documents a limitation rather than a capability.
4. **Should the Markov check reject anything here too?** Currently all four pass it and only the
   faithfulness gate bites. A fifth hypothesis with non-commuting factors on a shared support would
   exercise the other gate. *Recommendation: investigate during implementation; do not force one.*

---

## 12. Sources

- Sarovar, M., Proctor, T., Rudinger, K., Young, K., Nielsen, E. & Blume-Kohout, R. (2020).
  *Detecting crosstalk errors in quantum information processors.* Quantum **4**, 321.
  DOI 10.22331/q-2020-09-11-321. The prior work §3 defers to: PC algorithm, conditional independence
  under the Markov condition, direction and existence, fixed experiment set.
- Ferrie, C., Granade, C.E. & Cory, D.G. (2012). *Adaptive Hamiltonian Estimation Using Bayesian
  Experimental Design.* arXiv:1111.0935. Adaptive design over continuous parameters; the `design`
  stage here selects over discrete structures under a cost budget instead.
- Allen, J.-M. et al. (2017). Complete common cause as factorization into commuting Choi factors,
  the theorem the freeze gate tests.
- van der Lugt, T. & Lorenz, R. (2025). *Unitary causal decompositions.* arXiv:2508.11762. The
  C₃-exclusion criterion that rejects H₄.
- Binney, H.P. et al. (2026). *Distinguishing types of correlated errors in superconducting qubits.*
  arXiv:2603.16494. The evidence that this investigation class is real, expensive, and
  intervention-driven.
- White, G.A.L. (2024). *Many-time physics in practice.* arXiv:2405.05416. Process tensor
  tomography, the E4 experiment. Cited for the method, not for causal vocabulary; see §3.
- Kam, J.F. et al. (2025). *Detrimental non-Markovian errors for surface code memory.* QST 10(3),
  035060. Why correlation structure, not correlation magnitude, decides the damage.

Companion notes: [`positioning.md`](positioning.md),
[`example-quantum-control-loop.md`](example-quantum-control-loop.md),
[`qcl-dsl-liftback.md`](qcl-dsl-liftback.md).
