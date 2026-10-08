<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL Gen 3

**What this is.** The scope of QCL's third generation, distilled from three notes:
[`quantum-backlog.md`](quantum-backlog.md) (31 open items), [`qcl-next.md`](qcl-next.md) (Martinis
papers ranked by impact) and [`Lorenz-Gap.md`](Lorenz-Gap.md) (QCL against Lorenz's quantum causal
models). Gen 1 decides among named candidates with the cheapest experiment that separates them. Gen 2
(QCL-2) carries that decision across levels of description: abstraction, codes, decoders.

**Status.** Proposal, 2026-10-08, against `main` at `0f4a690c0`. No code written. Facts about the
tree were checked on that date; §1 marks where they differ from the source notes.

**Finding.** Gen 3 is 16 work packages in three tracks, each small or moderate except one stretch:

- **V, validity (Lorenz).** QCL admits a candidate only if it is a valid, faithful process.
  Cyclic candidates become decidable: H₄ in `qcl_crosstalk` is rejected as an invalid process, and a
  SWITCH-type candidate is accepted. QCL derives causal structure from operators and proposes
  candidates itself by induced-graph discovery.
- **E, evidence (Martinis).** Campaigns take evidence per experiment, which recorded device data
  needs. The rank-1 paper becomes `qcl_t1_attribution`, the core of the rank-3 paper becomes a
  calibration example, and three QEC tools lay the ground for rank 2.
- **H, hardening.** The stale citation, mutation runs, stage-cost benchmarks and documentation fixes.

Gen 3 closes seven backlog items and the admission half of B-30; cyclic dilation and abstraction
stay research. Rank 2's record-valued observable is large, and with it the attribution of Willow's
correlated-error floor; both head Gen 4 (§5).

---

## 1. The three inputs, distilled

**Effort.** S: one function, one check, or one example on today's API. M: one or two library
types, or a public-API addition. L: a new subsystem, or a scope still unsettled.

### 1.1 The backlog

All 31 rows. **Gen 3** marks the eight selected, with their work package.

| # | Item | Effort | Value for Gen 3 | Verdict |
|---|---|---|---|---|
| B-1 | Plant path on bare metal | M | Low: no tie to Lorenz or Martinis | Later |
| B-2 | `intervene_instrument` | M | High: Lorenz's intervention is an instrument | **Gen 3, V7** |
| B-3 | Induced factorization of a composite | M–L | Medium | Later |
| B-4 | `Observe(Ō)` on a code | M | Medium, code path only | Later |
| B-5 | H₃ as a circuit | L | Blocked by leg sizes (D18) | Later |
| B-6 | Mutation runs over QCL kernels | S–M | High: Gen 3 adds numeric kernels | **Gen 3, H2** |
| B-7 | Amplification after marginalisation | S | Low, optional | Later |
| B-8 | Markov-rejecting plant candidate | S if one exists | Medium | Later; revisit after V3 |
| B-9 | Tick latency | S–M | Medium, fast-loop claims | Later |
| B-10 | Stage cost | S–M | High: sets the caps of V2 and V6 | **Gen 3, H3** |
| B-11 | Price of precision | S, given B-10 | Medium | **Gen 3, H3** |
| B-12 | Bounded-time log | M | Low at supervisory rate | Later |
| B-13 | `check_degeneracy` on a DEM | S | High: rank 2 | **Gen 3, E4** |
| B-14 | DEM export | S–M | High: rank 2, Stim interop | **Gen 3, E4** |
| B-15 | Code distance by enumeration | S | Low | Later |
| B-16 | Non-lattice complex in the code example | S | Low | Later |
| B-17 | Failing class-invariance case | S if one exists | Low | Later |
| B-18 | Gate catalogue | L | Track T | Out |
| B-19 | Lean: the dilation is Markov | M–L | Medium | Later, formalization track |
| B-20 | Diamond norm | L | Needs an SDP; the workspace has none | Out |
| B-21 | ε-abstraction write-up | S | Medium | Later; check the archived change first |
| B-22 | Deferred Lean targets | L | Medium | Later, formalization track |
| B-23 | `deep_causality_do_calculus` | L | High, outside the crate | Out; V8 is the quantum-side start |
| B-24 | Relativistic order | L | Low | Out |
| B-25 | Classical indefinite causal order | M–L | Low | Out |
| B-26 | The direct-sum generator `⊕` | L | Research upstream | Out |
| B-27 | Audit trail for quantum evidence | M | Medium | Later |
| B-28 | Fusion example | L | CFD scope | Out |
| B-29 | QPU-calling solve | L | Hardware-gated | Out |
| B-30 | Cyclic causal structures | M in parts | High | **Gen 3, V1 to V4** |
| B-31 | Reference ML decoder | M | High: measures assumption A7 | **Gen 3, E4** |

### 1.2 QCL next

| Item | State on 2026-10-08 | Gen 3 |
|---|---|---|
| Rank 1, Klimov 2018: attribute a T1 drop | S, smaller than the note's S–M. `qcl_gradiometer_crosstalk` and `qcl_gravimeter_systematics` now use `timed`, `ObservedContext`, `baseline_with` and `OutsideTheModel`. The note's "no example uses `timed` … or the instrument context" no longer holds. No example uses `combining`, `Recorded` or `Published` | E2 |
| Rank 2, Fowler 2014: error models from detection events | L. An experiment cannot observe a detection event | E4 lays the ground; the campaign heads Gen 4 |
| Rank 3, Kelly 2018: the calibration DAG | S for the core. The traversal is patent-claimed and waits for an IP review | E3, core only |
| Per-experiment evidence (§7) | Open: `Campaign::step` takes one `&EvidenceSource<R>` | E1 |
| Hygiene (§9) | Open: the crate README lists none of the 9 QCL examples; `LEAN_QUANTUM.md` names Lean 4.15.0 against the pinned v4.32.0; the site notes still quote 0.377, 100.1 bits and cost 200. The precision defect is fixed (`ba74b9e71`) | H4 |
| The correlated-error floor (§11) | Open in the field | Gen 4 |
| Open question 1, sequencing | Settled: both sensing examples shipped | — |

### 1.3 The Lorenz gap

| Lorenz-Gap §6 | Effort | Gen 3 |
|---|---|---|
| 1. Input/output legs, process validity, faithfulness | M | V1, V2, V3 |
| 2. Admit cyclic candidates; a SWITCH-type positive case; the C₃ screen's scope | M | V4 |
| 3. Structure from no-influence; induced-graph discovery | M | V5, V6 |
| 4. d-separation predictions | M–L | V8, stretch |
| 5. The v2 citation fix | S | H1 |
| 6. Research items | — | Out (§5) |
| Instrument intervention (§4) | M | V7 |

**One correction to Lorenz-Gap.** The paired input/output leg is already public.
`FactorSupports::set_leg_output_dim` declares it, the dilation sets it, and `check_normalization`
honours it. V3 therefore extends an opt-in convention, and the breaking change that Lorenz-Gap's
open question 1 weighs is avoidable.

---

## 2. Gen 3

**Selection rule.** Effort S or M. Each package closes a backlog item, closes a Lorenz gap, or
serves a ranked Martinis paper. Each has an acceptance test that fails on today's tree. V8 is
M–L and stays a stretch.

### 2.1 Track V: validity

| # | Work | Closes | Effort | Acceptance |
|---|---|---|---|---|
| V1 | **Faithfulness check** (cyclic paper, Eq. 4): every factor signals from every parent. A `Check` per (node, parent). | B-30; LG-1 | S | A factor that ignores one parent is rejected and the report names that parent. H₁ to H₃ pass. With V2, every two-node cyclic candidate is rejected by one check or the other (Proposition 1). |
| V2 | **Process-validity check** on the joint operator from `Hypothesis::joint_operator`: positivity, `Tr σ = ∏ d_out`, and only the allowed Hilbert–Schmidt term types (cyclic paper, Methods). Capped before allocation. | B-30; LG-1 | M | H₄ is rejected, in agreement with the classical totals of Lorenz-Gap §3.1: 1.037037 with identity at every node, 0.962963 with a NOT at Q1. H₁ to H₃ are accepted. |
| V3 | **Paired legs for structural candidates**: `Hypothesis::structural` takes a paired-leg declaration, and a child's factor acts on its parent's output half. Additive; the flat convention stays the default. | LG-1 | M | A candidate declared with paired legs has the same joint operator as the dilation of the circuit it describes. |
| V4 | **Admit cyclic candidates.** `build()` stops refusing a cycle on the structural path; V1 and V2 decide instead. The C₃ screen reports itself not applicable to a cyclic candidate. | B-30; LG-2 | M | `qcl_crosstalk`'s H₄ ends rejected as an invalid process, where today it is refused at `build()`. A SWITCH-type candidate (P, A, B, F) is accepted as valid and faithful. |
| V5 | **Structure from no-influence** (2019 Theorem 4.3; cyclic paper, Definitions 3 and 4) for circuits and unitary processes, beside the wiring-derived structure. For a unitary process, nonseparable ⇔ cyclic (Theorem 4) gives the unitary half of SPEC-Q6 as a cycle test. | LG-3; SPEC-Q6, part | M | A CNOT followed by the same CNOT: the wiring reports an edge, the operator reports none. The SWITCH tests nonseparable, a unitary comb separable. |
| V6 | **Induced-graph discovery** (2019 §11): `n(n−1)` linear no-signalling checks on a process operator, then `check_markov` on the induced graph. The result enters as a candidate. With V4 in place, a cyclic induced graph is a candidate rather than a stop. | LG-3 | M | H₁'s simulated process yields Q1 → Q2, Markov. H₃ with the bath traced out yields no edges and fails Markov, reported as a missing common cause. |
| V7 | **Instrument intervention**, `intervene_instrument(node, instrument)`: outcome-resolved predictions through `Hypothesis::evaluate`. | B-2 | M | A do-instrument agrees with `intervene_mechanism`. The outcome-resolved predictions of a measure-and-prepare instrument sum to its unresolved prediction. |
| V8 | *Stretch.* **d-separation predictions** (2019 Theorem 8.3; rule analogues 10.7, 10.17, 10.26): equalities that hold for every factor value of a candidate, offered to `design` as experiments. | LG-4 | M–L | H₁ and H₃ are separated by a rule-3 equality that holds for any parameter values. |

### 2.2 Track E: evidence and examples

| # | Work | Closes | Effort | Acceptance |
|---|---|---|---|---|
| E1 | **Per-experiment evidence**: a trait the campaign calls with the chosen experiment, a generic parameter on `step` and `advance` (qcl-next §7, shape b). `Simulated`, a recorded log and the `qpu` seam implement it. | qcl-next §7 | M | A campaign over a recorded log receives each chosen experiment's counts. Existing campaign tests pass unchanged. |
| E2 | **`qcl_t1_attribution`** (rank 1). Candidates: a TLS near the operating point (rows 1 and 4), quasiparticles (row 5), readout loss, and radiation (row 6). Experiments: a frequency offset and a delay, priced by `InstrumentTime`. Spectral diffusion as a time-stamped context. | Rank 1; rows 4 to 6 | S, plus E1 for recorded data | The generating cause is named in fewer device seconds than the full sweep. A TLS that moves triggers a re-plan. A run with the cause left off the list shows how the campaign ends, as the gravimeter's third run does. |
| E3 | **Calibration diagnose core** (rank 3): one qubit, three faults that read alike after one π pulse, and the nine-pulse amplification probe. On the pipeline, rewritten from [`example-quantum-control-loop.md`](example-quantum-control-loop.md). The graph traversal is excluded. | Rank 3, core | S | `design` picks the amplification probe over single-π probes, and the campaign separates all three faults. |
| E4 | **QEC groundwork for rank 2**: (a) `check_degeneracy`, mechanisms grouped by identical target set; (b) Stim-readable DEM export; (c) an exact maximum-likelihood decoder over a `DemModel` within its 20-mechanism cap. | B-13, B-14, B-31; A7 | S, S–M, M | (a) Two mechanisms with one target set form one class. (b) Export, then import, returns the same `DemModel`. (c) Positive control: a model whose correlated mechanism dominates decodes better under the correlated model than under its independent split. Then the A7 number. A zero there is checked against the control before it is reported. |

### 2.3 Track H: hardening

| # | Work | Closes | Effort | Acceptance |
|---|---|---|---|---|
| H1 | **Citation fix**: van der Lugt & Lorenz v2 in `faithfulness.rs` (lines 22, 195, 198) and its tests (lines 17, 65, 150, 153, 198). The oracle becomes Theorem 4.9(iii) or Definition 3.1, and Proposition 6.1 becomes Proposition C.1. | LG-5 | S | The 512-relation oracle test passes against the v2 criterion. |
| H2 | **Mutation runs** over the QCL-2 kernels and V1, V2, V5, V6 and E4(c), with equivalent mutants recorded in `.cargo/mutants.toml`. | B-6 | S–M | Every survivor is killed by a new test or listed with its measurement. |
| H3 | **Stage cost and precision**: B-10 extended to V2, V5 and V6, all exponential in node count, plus B-11 at `f32`, `f64` and `Float106`. The caps of V2 and V6 are set from these measurements. | B-10, B-11 | S–M | Each cap cites its measurement on the M3 Max reference machine. |
| H4 | **Documentation**: the crate README's QCL examples table, the Lean version in `LEAN_QUANTUM.md`, and `quantum-site/01_raw_information.md` and `02_home_page_draft.md`. | qcl-next §9 | S | Each number matches current output. |

---

## 3. Order

1. **Independent, small:** H1, H4, V1, E4(a), E4(b).
2. **The two spines:** E1 → E2, and V3 → V2 → V4.
3. **On the spines:** V5 → V6, V7, E3, E4(c).
4. **Alongside:** H3 before the caps of V2 and V6 are fixed; H2 after each kernel lands.
5. **Stretch:** V8.

E2 runs on simulated evidence before E1 lands. V4 needs V1 and V2, because once cycles are admitted
only validity and faithfulness keep invalid candidates out.

---

## 4. What Gen 3 adds, and its limits

- **Physical admission.** Today a candidate passes on normalization and Markov. Under Gen 3 it must
  also be a valid process and faithful. Cycles are decided by physics.
- **Candidates proposed.** Discovery turns a process operator into a candidate. It needs a full
  process operator, which is exponential in node count, so it serves few-node systems and simulated
  truths, and on hardware it is priced as tomography.
- **Device-shaped evidence.** E1 and E2 give QCL its first attribution on a quantum processor that
  can run on recorded counts. The claim is device seconds against the sweep, inside the candidate
  set.
- **QEC ground.** Degeneracy names what syndrome data cannot separate. The reference decoder
  produces the A7 number. Neither attributes a correlated burst; that needs Gen 4.

---

## 5. Left for Gen 4 or research

| Item | Reason | Where |
|---|---|---|
| Record-valued observable; the rank-2 campaign | L; it must resolve detection events across the array and across rounds | **Gen 4 headline** |
| Attributing Willow's correlated-error floor (qcl-next §11) | Needs the observable above; each step costs hours of device time | Gen 4 |
| Passive screening on the Willow archive (Zenodo 13273331) | Follows the observable; whether the archive holds the bursts is unchecked | Gen 4 |
| Cyclic dilation and abstraction | Lorenz & Tull §8 leave it open | Research |
| Cyclic Hypothesis 1; routed decompositions (B-26); latent common causes; quantum counterfactuals; genuine quantum causal abstraction | Open upstream | Research |
| General causal separability; the diamond norm (B-20) | Need an SDP | Out until one exists |
| The calibration traversal | Patent-claimed (qcl-next §6.2) | After an IP review |
| B-1, B-3, B-4, B-5, B-7, B-8, B-9, B-12, B-15 to B-17, B-19, B-21, B-22, B-27 | Outside the selection rule, or low value now | Backlog |
| B-18, B-23 to B-25, B-28, B-29 | L, or outside the crate | Their own tracks |

---

## 6. Open questions

1. **V3's default.** Keep paired legs opt-in, or make them the structural default in a later
   breaking release?
2. **V2's reach.** Run validity on every candidate, where it is implied for acyclic ones, or only on
   cyclic candidates?
3. **The SWITCH case.** A new example, or a fixture in the tests? The crosstalk plant has no root
   and leaf nodes.
4. **E4(c) and Track D.** The reference decoder produces SPEC-D1's number, which is a proposal
   deliverable. Build it in Gen 3, or hold it for that proposal?
5. **V8.** In Gen 3 as a stretch, or the first item of Gen 4?

---

## 7. Sources

- [`quantum-backlog.md`](quantum-backlog.md): B-1 to B-31
- [`qcl-next.md`](qcl-next.md): §3 ranking, §6.2 patents, §7 evidence shapes, §9 hygiene, §11 gap
- [`Lorenz-Gap.md`](Lorenz-Gap.md): §3 cyclic gaps, §3.1 H₄, §4 framework gaps, §5 citations, §6
  order
- [`dynamic-qcm.md`](dynamic-qcm.md): assumption A7, SPEC-Q6, Track D
- [`example-quantum-control-loop.md`](example-quantum-control-loop.md): the E3 design
- Papers in [`Lorenz/`](Lorenz/): Barrett, Lorenz & Oreshkov 2019 and 2021; van der Lugt & Lorenz
  v2
- In the tree, read on 2026-10-08:
  [`process_factors.rs`](../../../deep_causality_quantum/src/types/qcm/process_factors.rs)
  (`set_leg_output_dim`),
  [`hypothesis.rs`](../../../deep_causality_quantum/src/types/qcm/hypothesis.rs)
  (`joint_operator`, `evaluate`, `check_normalization`),
  [`dilation.rs`](../../../deep_causality_quantum/src/types/qcm/dilation.rs),
  [`campaign.rs`](../../../deep_causality_quantum/src/types/pipeline/campaign.rs),
  [`faithfulness.rs`](../../../deep_causality_quantum/src/types/qcm/faithfulness.rs)
