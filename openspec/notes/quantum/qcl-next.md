<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# QCL next: which Martinis publication to build on

**What this is.** A ranking of 23 publications by John M. Martinis by how far implementing each
would advance QCL. Each is weighed on two axes: what it costs to build on the pipeline as it stands,
and what it is worth to teams that operate superconducting quantum processors. The target is a paper
that is cheap to implement on what exists and returns value out of proportion to that cost. The
ranking is checked against what recognised experts rank as the field's highest-impact areas (§2).

**Status.** Assessment, 2026-10-08, against `main` at `4101ce439` plus the uncommitted working tree.
No code written. Titles and venues were checked against the Martinis group publication list and
online; the full texts were not read, so content claims rest on abstracts and prior knowledge. Patent
data comes from Google Patents pages, summarised rather than read claim by claim. The expert check in
§2 moved the error-model paper from third to second and the calibration-DAG paper from second to
third. §11 reads what the five most recent papers leave open and finds an unclosed gap.

**Finding.** Three papers, in this order:

1. **Klimov et al. 2018**, *Fluctuations of energy-relaxation times in superconducting qubits*.
   Attributing a T1 drop to its cause has the shape of `qcl_crosstalk`. It gives the pipeline
   features added on 2026-10-07 their first quantum-processor example and needs one library change
   (§7). It sits in the field's second-ranked area, material-limited coherence, where Google's
   Willow processor still loses performance to a two-level-system defect that moved faster than
   forecast.
2. **Fowler et al. 2014**, *Scalable extraction of error models from the output of error detection
   circuits*. It sits in the field's top-ranked area, error correction. A 2025 study of Willow's
   syndromes found correlated detector flips that the standard error model does not capture. Whether
   one mechanism drives them or several share a cause is the question QCL decides. It costs more: an
   experiment cannot yet observe a detection event.
3. **Kelly et al. 2018**, *Physical qubit calibration on a directed acyclic graph*. Calibration at
   scale is a recognised bottleneck, but an open-source graph-based framework already serves it, and
   Martinis's own company uses that framework. Active patents held by Google (to about 2036) and
   D-Wave (to 2040) claim the graph traversal the paper contributes.

**Unclosed gap.** Nobody has published the cause of the rare correlated error bursts that set the
logical-error floor on Google's Willow processor. Google calls identifying the mechanism "integral to
running larger quantum algorithms". The industry's error-correction agenda, as summarised, centres on
decoding. Rank 2 is the route to the gap, and rank 1 is its first step (§11).

---

## 1. Scope and scoring

**Population.** Martinis's landmark papers, plus every paper of his found on calibration,
characterization and error models, which are the classes QCL can use. That gives 23 rows. His full
record runs to several hundred papers; one outside this set is unranked, not rejected.

**Columns.**

- **Size.** S: one example on today's API. M: an example plus one or two library types. L: a new
  subsystem. X: outside the crate's stated scope ([`positioning.md`](positioning.md) §2.2 and §8).
- **Impact.** A judgement of value to teams operating superconducting processors, anchored where
  possible in the sources of §2. Nothing here was measured.
- **Area.** The expert-ranked areas of §2. Q1: fault tolerance through error correction. Q2: scaling
  the hardware, including material-limited coherence. Q3: useful applications.
- **Gap.** The gaps [`positioning.md`](positioning.md) §5.5 names. A: diagnosis is projected out of
  the detector error model. B: no discrete experiment selection under a cost budget. E: drift
  attribution over time.

---

## 2. What the field ranks highest

Two framings answer different questions.

**Across quantum technology**, the three standard pillars are computing, communication, and sensing.
The European Commission's Quantum Europe Strategy (July 2025) and McKinsey's Quantum Technology
Monitor 2025 both organise around them. QCL's atom-interferometer groundwork sits in the sensing
pillar; everything ranked here sits in computing.

**Within quantum computing**, the sources converge on three areas:

| Area | Who ranks it | What they say |
|---|---|---|
| **Q1. Fault tolerance through error correction**, including real-time decoding | Preskill (Caltech), *Beyond NISQ: The Megaquop Machine* (2025) | Machines of broad practical value "must be protected against noise using quantum error correction and fault-tolerant protocols" |
| | Riverlane, *Quantum Error Correction Report 2025*, built on interviews with 25 experts, Martinis among them | Real-time error correction is "the industry's defining engineering hurdle" (The Quantum Insider's summary) |
| | National Academies, *Quantum Computing: Progress and Prospects* (released December 2018) | Error correction is among the technical problems to overcome before a functional machine |
| | DARPA, Quantum Benchmarking Initiative (Stage B, November 2025) | "Our opening position is skepticism … that a fully fault-tolerant quantum computer with a sufficient number of logical qubits can ever be built" |
| **Q2. Scaling the hardware**: manufacturing, wiring, material-limited coherence | Martinis (The Quantum Insider, November 2025) | "The complexity of the plumbing completely overwhelms the quantum device itself" |
| | McKinsey, *Quantum Technology Monitor 2025* | In 2024 the industry shifted "from growing quantum bits (qubits) to stabilizing qubits" |
| | IBM, Quantum Developer Conference 2024 | Heron r2 adds two-level-system mitigation "to help reduce the impact of an important source of noise" |
| | Fermilab SQMS Center (DOE) | Researches mitigation of two-level-system noise in superconducting devices |
| **Q3. Useful applications and verified advantage** | Babbush et al. (Google), *The Grand Challenge of Quantum Applications* (PRX Quantum, 2026) | Identifying advantage instances and tying them to real uses are "essential and currently under-resourced" |
| | DARPA, Quantum Benchmarking Initiative | Utility scale means computational value exceeds cost, by 2033 |

**QCL's reach.** QCL touches Q1 and Q2 at the diagnosis and characterization layer: the error models
decoders depend on, and the coherence drift that limits the hardware. Two parts of these areas are
out of its reach. Real-time decoding, the centre of Q1 in Riverlane's report, runs on microsecond
rounds, and QCL works at supervisory rate ([`positioning.md`](positioning.md) boundary 8). Q3 has no
paper in this set, and QCL does not address it.

---

## 3. The ranking

| # | Publication | What QCL would build | Size | Impact | Area | Gap and notes |
|---|---|---|---|---|---|---|
| 1 | Klimov et al., *Fluctuations of energy-relaxation times in superconducting qubits*, PRL 121, 090502 (2018), arXiv:1809.01043 | Attribute a T1 drop to a TLS defect, quasiparticles or readout loss | S–M | High | Q2, feeds Q1 | B, E. No direct patent found; Google US 12,050,964 is adjacent (§6.2) |
| 2 | Fowler, Sank, Kelly, Barends, Martinis, *Scalable extraction of error models from the output of error detection circuits*, arXiv:1405.1454 (2014) | Decide whether correlated detection events share a mechanism or a cause | M–L | High | Q1 | A. Needs record-valued observables (§5) |
| 3 | Kelly, O'Malley, Neeley, Neven, Martinis, *Physical qubit calibration on a directed acyclic graph*, arXiv:1803.03226 (2018) | Separate "out of spec", "bad data" and decoherence; the calibration-graph traversal | S core, M graph | Medium | Q2 | B. Open-source competition; the traversal is patent-claimed (§6) |
| 4 | Martinis et al., *Decoherence in Josephson qubits from dielectric loss*, PRL 95, 210503 (2005) | Resonant TLS absorption as a candidate mechanism | S | via 1 | Q2 | Candidate physics for row 1 |
| 5 | Martinis, Ansmann, Aumentado, *Energy decay in Josephson qubits from non-equilibrium quasiparticles*, PRL 103 (2009), arXiv:0904.2171 | Quasiparticle decay as the rival that does not depend on frequency | S | via 1 | Q2 | Candidate physics for row 1 |
| 6 | Martinis, *Saving superconducting quantum processors from decay and correlated errors generated by gamma and cosmic rays*, npj QI (2021), arXiv:2012.06137 | A radiation burst as a candidate mechanism | S as a candidate | via 1, 2 | Q1, Q2 | Candidate physics for rows 1 and 2; Arms et al. see radiation signatures in Willow's syndromes (§5.1) |
| 7 | Kelly et al., *Scalable in situ qubit calibration during repetitive error detection*, PRA 94, 032321 (2016) | Calibration while error detection runs | M | High | Q1 | Optimization, not discrimination. Google US 10,692,009, to 2036 |
| 8 | Arute et al., *Quantum supremacy using a programmable superconducting processor*, Nature 574, 505 (2019) | Isolated against simultaneous benchmarking: does the product-of-fidelities error model hold, or does a pair cross-talk? | M | Med–High | Q1 | Needs a cross-entropy estimator. The 7-candidate cap forces per-pair campaigns |
| 9 | Chen et al., *Measuring and suppressing quantum state leakage in a superconducting qubit*, PRL 116, 020501 (2016) | Leakage as a mechanism candidate on a qutrit plant | M | Med–High | Q1 | Candidate physics for row 2 |
| 10 | Martinis, *Qubit metrology for building a fault-tolerant quantum computer*, npj QI 1, 15005 (2015) | An error budget as `Check` records with margins | S | Medium | Q1 | A framing more than an algorithm |
| 11 | Fowler & Martinis, *Quantifying the effects of local many-qubit errors and non-local two-qubit errors on the surface code*, PRA 89, 032316 (2014) | Price the logical damage of an attributed correlated error | L | Medium | Q1 | Needs threshold simulation |
| 12 | Neill et al., *A blueprint for demonstrating quantum supremacy with superconducting qubits*, Science 360, 195 (2018) | Cross-entropy benchmarking on 9 qubits | M | Medium | Q3 | Cirq already ships it |
| 13 | Boixo et al., *Characterizing quantum supremacy in near-term devices*, Nature Physics 14, 595 (2018) | Cross-entropy benchmarking theory | M | Medium | Q3 | As row 12 |
| 14 | Klimov, Kelly, Martinis, Neven, *The Snake optimizer for learning quantum processor control parameters*, arXiv:2006.04594 (2020) | Frequency reallocation after an attribution | L | High | Q2 | Optimization, outside "decide, do not optimize" |
| 15 | Neeley et al., *Process tomography of quantum memory in a Josephson-phase qubit coupled to a two-level state*, Nature Physics 4, 523 (2008) | A tomography estimator turning counts into Choi factors | M | Low–Med | Q2 | Tomography is the cost QCL plans around |
| 16 | Kelly et al., *Optimal quantum control using randomized benchmarking*, PRL 112, 240504 (2014) | Benchmarking-driven parameter optimization | M | Medium | Q2 | Optimization |
| 17 | Fowler, Mariantoni, Martinis, Cleland, *Surface codes: Towards practical large-scale quantum computation*, PRA 86, 032324 (2012) | Decoding and resource estimates | X | High | Q1 | The crate holds no decoder; Stim and PyMatching cover this |
| 18 | Barends et al., *Superconducting quantum circuits at the surface code threshold for fault tolerance*, Nature 508, 500 (2014) | The threshold experiment | X | Low | Q1 | Its usable content is rows 2 and 10 |
| 19 | Kelly et al., *State preservation by repetitive error detection in a superconducting quantum circuit*, Nature 519, 66 (2015) | The repetition-code experiment | X | Low | Q1 | As row 18 |
| 20 | Martinis & Geller, *Fast adiabatic qubit gates using only σz control*, PRA 90, 022307 (2014) | CZ pulse shaping | X | Low | Q2 | The pulse layer is out of scope |
| 21 | Ansmann et al., *Violation of Bell's inequality in Josephson phase qubits*, Nature 461, 504 (2009) | Classical common cause against a quantum causal model | S | Low | — | A strong showcase with no buyer |
| 22 | Martinis, *Surface loss calculations and design of a superconducting transmon qubit with tapered wiring*, npj QI (2022), doi:10.1038/s41534-022-00530-6 | Surface-loss design | X | Low | Q2 | Design-time electromagnetics |
| 23 | Martinis, Devoret, Clarke, *Energy-level quantization in the zero-voltage state of a current-biased Josephson junction*, PRL 55, 1543 (1985) | — | X | None | — | The 2025 Nobel basis; no QCL path |

Rows 4, 5, 6 and 9 rank by what they feed. Implemented alone, each is a channel and a rate law; inside
rows 1 and 2, each is a rival explanation the campaign must rule in or out.

---

## 4. Rank 1: attributing a T1 drop

### 4.1 The problem

A qubit's energy-relaxation time drops, and an operator must decide whether to retune, escalate or
wait. Klimov et al. resolved T1 in frequency and in time on frequency-tunable qubits. They found that
two-level-system defects cause the largest fluctuations, because the defects' transition frequencies
wander (spectral diffusion).

Other causes also produce a drop, and Martinis's own papers supply them: resonant TLS absorption
(row 4), non-equilibrium quasiparticles (row 5) and radiation bursts (row 6). Each cause calls for a
different repair. Moving the qubit frequency cures a TLS that has drifted onto the operating point.
No retune cures quasiparticles or radiation.

In standard practice, someone reads T1 across a frequency sweep and a person interprets the plot.
That is gap E.

### 4.2 The challenge is live at the frontier

- Google's Willow processor (Nature 638, 920, 2025) chooses qubit frequencies by "extracting
  frequency-temporal trajectories of TLS from historical T1 data, forecasting them to future times,
  and excluding these frequencies". Even so, "the performance of the worst distance-3 quadrant appears
  to fluctuate due to a transient TLS moving faster than our forecasts" (Supplementary Information,
  §III).
- Dane et al. (arXiv:2503.12514) report qubits whose "energy loss is dominated by two level systems
  (TLS)" even at T1 above 2.5 ms. They note that large T1 variation "would make it extremely difficult
  to accurately evaluate and compare qubit fabrication processes".
- IBM's Heron r2 ships two-level-system mitigation (Quantum Developer Conference 2024).

The second point ties rank 1 to Martinis's own priority, manufacturing (§2, Q2). A fabrication line
can compare processes only after it separates TLS drift from process quality, and that separation is
an attribution.

### 4.3 How it maps onto the pipeline

| Piece | T1-drop campaign | In the tree |
|---|---|---|
| Candidates | TLS near the operating point, quasiparticles, readout loss, radiation: each an amplitude-damping channel whose rate depends on the qubit frequency | `.mechanisms(...)` in [`config.rs`](../../../deep_causality_quantum/src/types/pipeline/config.rs); `Channel::from_kraus` |
| Experiments | The configuration is a frequency offset and a delay; the read-out is P(1) after the delay, one Born probability per experiment. Offer targeted probes at a few offsets, plus the full spectroscopy sweep as the expensive baseline | `ConfiguredExperiment`; a `ResponseModel` that answers `Response::Channel` |
| Price | Device seconds per shot and per configuration change, with shots sized to reach the floor | `MinCostCover::timed(InstrumentTime)` |
| Drift | Spectral diffusion as a time-stamped environment reading. `alternate_context` between steps reaches the campaign, which re-plans when a prediction moves by more than `drift` | `Campaign::advance`; `InstrumentContext<P, R>`, generic over the payload |
| Missing cause | No candidate holds | `CampaignStop::OutsideTheModel` |
| Evidence | Simulated in the example; recorded counts on a device | `EvidenceSource::Simulated` and `Recorded`, with the gap in §7 |

No example uses `timed`, `combining`, `Recorded`, `Published` or the instrument context (grep of
`examples/`, 2026-10-08). The atom-interferometer groundwork for quantum sensing built them, and this
example would use them on a quantum processor too.

### 4.4 What the example takes from the papers

Coupling strengths, TLS linewidths, diffusion rates and the baseline T1 come from Klimov et al. and
from rows 4 to 6, cited in the README; none is invented. Each candidate's damping rate as a function
of qubit frequency is the response model. Its functional form is a modelling assumption, and the
README says so.

### 4.5 The claim and its limit

The claim: deciding among named causes takes fewer device seconds than a full sweep. When a TLS
moves, the campaign re-plans, and when no named cause fits, it says `OutsideTheModel`.

The limit: the saving holds only inside the candidate set. Klimov's dense map of T1 against
frequency and time stays the tool for finding a cause nobody named. The example compares against the
sweep in device seconds, priced by `InstrumentTime`, and claims nothing about wall-clock time.

### 4.6 What to build

1. The per-experiment evidence change in §7. Simulated runs do not need it; recorded data does.
2. A payload type for `InstrumentContext` holding the operating frequency, the TLS frequency
   estimate and time-stamped T1 readings. It must implement `Storable`, and it can live in the
   example first.
3. The example under `examples/quantum_examples/qcl_examples/`, laid out per the example convention,
   with its `rust_binary` in `BUILD.bazel`.

Amplitude damping needs no library constructor: two Kraus operators through `Channel::from_kraus`
suffice.

---

## 5. Rank 2: error models from error-detection output

### 5.1 The challenge is live at the frontier

Fowler et al. extract an error model from the output of error-detection circuits that run
continuously. It is the only paper in the set aimed at gap A, which
[`positioning.md`](positioning.md) §5.5 ranks with B as the target, and it sits in Q1, the area
every source in §2 ranks first.

Two 2025 results show the open problem:

- Willow's abstract: in repetition codes up to distance 29, "logical performance is limited by rare
  correlated error events occurring approximately once every hour, or 3 × 10⁹ cycles".
- Arms et al. (arXiv:2512.10814) fit detector error models to Willow's repetition-code syndromes and
  find "two artifacts … that are not well-modeled by a DEM: correlated flipping of pairs of adjacent
  detectors in many consecutive rounds of QEC, and signatures consistent with radiation events
  occurring more frequently than previously reported".

A detector error model carries a weight per mechanism and no direction. It cannot say whether two
detectors flip together because one mechanism drives both or because separate mechanisms share a
cause. QCL decides that kind of question. The candidates come from the set: leakage (row 9), a
radiation burst (row 6), a TLS near a coupler frequency (row 1), and crosstalk between neighbours
(row 8). Each predicts a different response to an intervention the operator controls, such as
switching leakage removal on or off or moving a qubit's frequency. That is the experiment `design`
would choose.

### 5.2 Why it ranks below rank 1

- **Detection events have no observable.** Detection events are functions of mid-circuit
  measurement records across rounds. An `Observable` is a projector read on the final plant state,
  so an experiment cannot name a detection-event correlation. A record-valued observable is new
  design work, and it is the bulk of the cost.
- **Caps.** `DemModel` caps at 20 mechanisms and 20 variables, and `SimQpu` at 24 qubits. A
  distance-5 repetition code, nine qubits, fits. A surface code of useful distance does not.
- **Patents nearby.** Row 7 optimizes parameters on the same detection events, and Google's
  US 10,692,009 claims that closed loop. Attribution is a different act from the one claimed;
  optimization is the same one.

`DecoderAbstraction` and `DemModel` exist for the comparison between a circuit and its error model.

---

## 6. Rank 3: the calibration DAG

### 6.1 What fits

[`example-quantum-control-loop.md`](example-quantum-control-loop.md) designs an example on Kelly
et al. It has one qubit and three faults (amplitude, detuning, decoherence) that read alike after one
π pulse, and a nine-pulse amplification probe separates them. On today's pipeline the three faults
are mechanism candidates, the N-pulse sequences are configured experiments, and `design` picks the
probe. That note predates the pipeline and plans a hand-built `CausalFlow`; the pipeline version is
smaller.

### 6.2 What the patents claim

| Patent | Holder | Priority | Expires | Claim 1, summarised |
|---|---|---|---|---|
| US 10,606,720 B2, *Automatic qubit calibration*; continuations US 10,997,044, US 11,567,842, US 12,380,355 | Google (inventor J. S. Kelly) | 2016-06-09 | 2036-10-25 | A directed graph of parameters and dependencies. The descendants of a root are processed in order: a calibration test, then on failure a calibration or a diagnosis that walks the ancestors |
| US 9,940,212 B2, *Automatic qubit calibration* | Google | — | — | Same title. Its family relation to the row above is unconfirmed |
| US 11,288,073 B2, *Systems and methods for calibrating devices using directed acyclic graphs* | D-Wave (Berkley et al.) | 2019-05-03 | 2040-07-17 | Map a processor's parameters to vertices and their dependencies to directed edges, order the vertices, and measure each parameter in that order |
| US 10,692,009 B2, *In-situ quantum error correction* | Google (Kelly) | 2015-11-06 | 2036-06-21 | Closed-loop optimization of readout, single-qubit and CNOT parameters on detection events, over qubits partitioned into hardware patterns (row 7) |
| US 12,050,996 B2, *Qubit calibration* | Google (Vainsencher, Kelly) | 2017-12-14 | 2040-07-05 | Train a supervised model on calibration logs to say which parameters need calibration |
| US 12,050,964 B1, *Systems and methods to measure quantum gate fidelity through swap spectroscopy* | Google (Niu, Smelyanskiy, Boixo) | 2019-09-04 | 2043-05-01 | Gate control values from a model of non-Markovian qubit–TLS dynamics, to reduce TLS noise (adjacent to row 1) |

These summaries come from the Google Patents pages. Read the claims in full before relying on this
table; it is not legal advice.

### 6.3 The field already has graph calibration

Calibration is a recognised scaling bottleneck, and several funded efforts serve it:

- **QUAlibrate** (Quantum Machines, open source, May 2025) composes calibrations into graphs and ships
  "its first calibration graph for superconducting quantum computers". Qolab, Martinis's company,
  reports "full calibrations in less than 10 minutes" with it.
- Quantum Machines and Rigetti demonstrated AI-driven calibration (December 2024). Q-CTRL offers
  autonomous calibration.
- *Vibe Calibration* (arXiv:2606.22376) brings up a 112-qubit superconducting processor with a
  language agent.

### 6.4 Consequence

The paper's contribution is the dependency-graph traversal, `maintain` and `diagnose`.
US 10,606,720 claims that traversal, and US 11,288,073 claims ordering a processor's parameters as a
graph and measuring them in that order. A free graph framework already serves the people who would
use it.

What stays distinctive is QCL's choice of the cheapest probe that separates named faults at one
node. The control-loop note's core option ships exactly that, and neither claim describes it. Its
value is highest as a diagnose step inside an existing graph framework rather than as a second
framework. The established frameworks are Python, so that route needs a bridge.

The control-loop note's §11 option, "the real Optimus DAG", reproduces the claimed traversal. It should not ship
before an IP review. Kelly et al. was public in March 2018, before D-Wave's May 2019 priority date;
what that means for validity is a question for counsel.

---

## 7. The library change the top three need

`Campaign::step` takes one `EvidenceSource` for the whole campaign
([`campaign.rs:173`](../../../deep_causality_quantum/src/types/pipeline/campaign.rs#L173)). With
`Simulated` that works for any experiment, because the truth's prediction is computed per
experiment. `Recorded` holds one histogram, and the campaign chooses its next experiment only inside
`advance`. A campaign on device data therefore cannot be handed the right counts.

| Shape | Serves | Cost |
|---|---|---|
| (a) `EvidenceSource::PerExperiment`, counts keyed by experiment name | Logs recorded in advance | The log must hold every experiment the campaign might pick |
| (b) A trait the campaign calls with the chosen experiment, which returns that experiment's evidence; a generic parameter on `step`, so dispatch stays static | Logs, and a live device behind the `qpu` seam | One trait, one generic parameter |

(b) covers (a): a log lookup is one implementation of the trait. Recommendation: (b). See open
question 2.

---

## 8. Where the ranking could be wrong

- **Impact is still judgement.** §2 anchors the areas, not the per-paper scores. A hardware partner
  who names a different daily pain outranks this table.
- **Rank 2's cost.** The record-valued observable is unscoped. If it turns out large, rank 2 waits
  behind rank 1 without losing its place in value.
- **Seven candidates.** `design` covers at most seven by default. One qubit's T1 fits, and so does
  the four-mechanism set of §5.1. Row 8, crosstalk across a whole device, would run as many small
  per-pair campaigns.
- **Full texts unread.** The content claims rest on abstracts, the group list and prior knowledge.

---

## 9. Other findings from the same review

- **Precision defect in the gate kernels, fixed in the working tree.** `gate_unitary` built 1/√2 and
  π/4 through `f64`, and `GaugeFieldGate::pauli_coefficients` built 2π and each phase the same way.
  At `Float106` the distillation round showed ε₂ = 3.283e-15 at p = 0, a residual of `f64` size. The
  uncommitted fix forms these constants at `R`. Re-run on 2026-10-08, ε₂ at `Float106` is 0 for every
  p. At `f64` the p = 1/20 row moved from 5.53e-16 to 5.28e-15, the cost of deriving the constants
  instead of reading rounded literals; the composition law holds at every precision.
- **Stale site notes.** [`01_raw_information.md`](../quantum-site/01_raw_information.md) and
  [`02_home_page_draft.md`](../quantum-site/02_home_page_draft.md) still describe `qcl_crosstalk`
  with typed predictions, a fifth experiment (tomography at cost 200), a 0.377 read-out and
  100.1 bits. The example now computes its predictions through a `ResponseModel`, offers four
  experiments, runs a `Campaign`, reads 0.396 ± 0.015 and names H1 at 99.5 bits.
  [`quantum-essence.md`](../quantum-site/quantum-essence.md) was updated in `4101ce439` and matches.
- **Crate README.** Its examples table lists the seven non-QCL examples and none of the seven QCL
  examples.
- **`LEAN_QUANTUM.md`** names Lean 4.15.0; `lean/lean-toolchain` pins v4.32.0.

---

## 10. Open questions

1. **Build rank 1 next?** The two quantum-sensing examples are also queued, and they build on the
   same instrument context and campaign. Sequencing is the maintainer's call. A working name is
   `qcl_t1_attribution`.
2. **Evidence shape.** Recommendation (b) in §7, the trait.
3. **Rank 2's observable.** A record-valued observable is the price of entering Q1. Scope it now, or
   after rank 1 ships?
4. **Start rank 2 on Willow's archive?** The repetition-code datasets are public (§11.5). They
   support screening candidates, not separating explanations that predict the same passive
   statistics. Whether they contain the hourly bursts is unchecked.
5. **Rank 3's form.** A diagnose step for an existing graph framework, or a stand-alone example on
   the core option only? Either way, the traversal waits for an IP review.
6. **Rank 2's target.** Should rank 2 aim at the correlated-error floor of §11 from the start?

---

## 11. The unclosed gap: what causes the correlated-error floor

Found on 2026-10-08. The 23 papers were sorted by date, the five most recent were read for what
they leave open, and that was matched against the priorities of §2.

### 11.1 The papers by date

Ordered by publication date, and arXiv-only papers by arXiv date. Where it was not checked, the
month order within a year is from memory.

| Year | Rank | Paper |
|---|---|---|
| 1985 | 23 | Martinis, Devoret, Clarke: energy-level quantization |
| 2005 | 4 | Martinis et al.: dielectric loss from TLS |
| 2008 | 15 | Neeley et al.: process tomography of TLS memory |
| 2009 | 5 | Martinis, Ansmann, Aumentado: quasiparticles |
| 2009 | 21 | Ansmann et al.: Bell violation |
| 2012 | 17 | Fowler et al.: surface codes |
| 2014 | 11 | Fowler & Martinis: local many-qubit errors |
| 2014 | 18 | Barends et al.: surface-code threshold |
| 2014 | 2 | Fowler et al.: error models from error detection |
| 2014 | 16 | Kelly et al.: optimal control by benchmarking |
| 2014 | 20 | Martinis & Geller: fast adiabatic gates |
| 2015 | 19 | Kelly et al.: repetitive error detection |
| 2015 | 10 | Martinis: qubit metrology |
| 2016 | 9 | Chen et al.: leakage |
| 2016 | 7 | Kelly et al.: in-situ calibration |
| 2018 | 3 | Kelly et al.: calibration DAG |
| 2018 | 12 | Neill et al.: supremacy blueprint |
| 2018 | 13 | Boixo et al.: characterizing supremacy |
| **2018** | **1** | **Klimov et al.: T1 fluctuations** |
| **2019** | **8** | **Arute et al.: quantum supremacy** |
| **2020** | **14** | **Klimov, Kelly, Martinis, Neven: Snake optimizer** |
| **2021** | **6** | **Martinis: radiation and correlated errors** |
| **2022** | **22** | **Martinis: surface loss and tapered wiring** |

### 11.2 What the five most recent leave open

Arute's quotes come from the Nature main text and Klimov's from the conclusion of the arXiv v1 PDF.
The other three come from the arXiv full texts.

| Paper | Forward-looking statement | Status, 2026-10-08 |
|---|---|---|
| Martinis 2022 | Hopes to "encourage researchers to precisely test surface loss theory, and measure in additional experiments the various surface loss parameters" | Active materials research. Dane et al. (2025) note that T1 variation makes it "extremely difficult to accurately evaluate and compare qubit fabrication processes" |
| Martinis 2021 | "It is imperative to further understand this error mechanism, not just reducing the effects of radiation, but slashing the resultant quasiparticle density, time and length scales by a factor of 100 or more each." Also: "There are many interesting experiments to do soon" | Largely closed for high-energy impacts. Gap engineering (McEwen et al. 2024) cut bursts from about once every ten seconds to about once an hour, in Preskill's account. A different burst remains (§11.3) |
| Snake 2020 | "A calibration system must learn optimal control parameters much faster than system drift. This requirement poses a significant scaling hurdle." The error mechanisms its model covers are "typically determined through physics research and machine learning" | Speed partly closed by calibration automation (§6.3). The mechanism set stays a manual research input |
| Arute 2019 | "To be successfully described by a digitized error model, a system should be low in correlated errors." "Our model assumes that entangling larger and larger systems does not introduce additional error sources." "The engineering of quantum error correction will need to become a focus of attention" | The agenda is met: error correction is the field's focus. The assumption fails at depth. Arute decorrelated errors with random circuits at about 0.2 % system fidelity; Willow's repetition codes hit a floor set by correlated events |
| Klimov 2018 | "Defect data should guide qubit calibration protocols … defects' diffusivity and coherence properties should inform the algorithms that are used to select tunable qubits' frequencies and how often those algorithms are run." "Correlation studies can be used to identify defective circuit components and materials" | Partly closed, in-house. Willow forecasts TLS trajectories when it chooses frequencies but recalibrates on a fixed cadence, and "a transient TLS moving faster than our forecasts" still degraded one patch (§4.2) |

### 11.3 The gap

Three lines meet in one open problem.

1. **Arute 2019** made low correlated error the precondition for the error model that error
   correction relies on.
2. **Martinis 2021** named radiation as the main correlated threat, and gap engineering since
   removed that threat.
3. **Willow** (Nature 638, 2025) found what remains. Its repetition codes show two failure modes
   (§IV). The milder one has a named cause: "These less damaging failures could be caused by
   transient TLS's appearing near the operation frequencies of a qubit, or by coupler excitations",
   which is the territory of Klimov 2018. The catastrophic one has none. It "manifests as many
   detectors experiencing a larger spike in detection probability simultaneously", and the bursts
   are "spatially localized to neighborhoods of roughly 30 qubits". Unlike the earlier impact
   events, "they occur approximately once an hour, rather than once every few seconds, and they
   decay with an exponential time constant around 400 µs, rather than tens of milliseconds". On
   these: "We do not yet understand the cause of these events, but mitigating them remains vital to
   building a fault-tolerant quantum computer." The Outlook: "Identifying and mitigating this error
   mechanism will be integral to running larger quantum algorithms."

The problem is still open:

- Preskill (2025, §2): the origin of the hourly bursts "is not yet clearly understood", and such
  rare events "will need to be understood and mitigated".
- Arms et al. (December 2025, revised March 2026) fit error models to Willow's repetition-code
  syndromes. They find "correlated flipping of pairs of adjacent detectors in many consecutive rounds
  of QEC" and "signatures consistent with radiation events occurring more frequently than previously
  reported".
- A search on 2026-10-08 found no publication that identifies the cause. That is absence of
  evidence, not proof.

It counts as unclosed, and not merely open, on three grounds:

- **Recognised.** The leading hardware team names it, and so does Preskill.
- **Unowned by the industry agenda.** The Quantum Insider's summary of Riverlane's QEC Report 2025
  covers decoding throughput, classical bandwidth and talent. It mentions neither correlated errors
  nor characterization.
- **Unmet by current tools.** Error-model estimation fits rates to a model, and decoders absorb the
  correlation. Neither names the physical mechanism to fix.

### 11.4 What it means for QCL

- **Rank 2 is the route.** The gap is the problem of §5.1, now carried by Google's own statement that
  it is unresolved and integral.
- **The footprint constrains the candidates.** Willow names no cause for the bursts, but it states
  what any cause must reproduce: about 30 qubits, a decay near 400 µs, and about one an hour.
  Residual radiation with different quasiparticle dynamics is one candidate, given the radiation-like
  signatures Arms et al. report (row 6). Leakage (row 9) and coupler excitations are others to test
  against the footprint. This note claims none of them fits; the campaign decides, and an unnamed
  cause ends at `OutsideTheModel`, which still rules out the named ones.
- **Rank 1 is the first step.** Willow attributes its milder failure mode to transient TLS near a
  qubit's operating frequency, which is rank 1's subject. Rank 1 also delivers the TLS tracking, the
  re-plan on drift and the instrument context that a burst campaign needs.
- **Sequence.** Rank 1, then the record-valued observable (§5.2), then a campaign that attributes
  the bursts.

### 11.5 Limits

- **Archived data is passive.** The Willow data are public: Zenodo record 13273331
  (doi:10.5281/zenodo.13273331), 112.5 GB, with surface-code and repetition-code datasets. Passive
  data can screen out candidates whose signatures differ in space or time, such as a radiation burst
  across many qubits against a TLS local to one. It cannot separate explanations that predict the
  same passive statistics. That takes interventions on hardware, as `qcl_crosstalk` shows for
  Markov-equivalent structures. Whether the archive holds the hourly bursts was not checked.
- **Rare events are costly.** At about one burst an hour, device time dominates every experiment.
  `MinCostCover::timed` prices it, and each campaign step costs hours.
- **The record-valued observable of §5.2 is still required, and the footprint sets its resolution.**
  To tell a 30-qubit burst with a 400 µs decay from a local defect, it must resolve detection events
  in space across the array and in time across rounds.
- **Identification only.** QCL addresses which mechanism; mitigating it stays with the hardware.

### 11.6 A smaller gap

Klimov's "how often those algorithms are run" asks for recalibration triggered by drift. Willow
recalibrates on a fixed cadence, "between every four experimental runs", alongside its TLS forecast.
QCL's re-plan on context drift (§4.3) fits this. Google covers part of it in-house, so the gap is
smaller than the one in §11.3.

---

## 12. Sources

Expert priorities (§2):

- Preskill, *Beyond NISQ: The Megaquop Machine* (2025), arXiv:2502.17368,
  https://arxiv.org/abs/2502.17368
- Riverlane, *Quantum Error Correction Report 2025*,
  https://www.riverlane.com/quantum-error-correction-report-2025
- The Quantum Insider, *Quantum report says error correction now the industry's defining challenge*
  (2025-11-19),
  https://thequantuminsider.com/2025/11/19/quantum-report-says-error-correction-now-the-industrys-defining-challenge/
- National Academies, *Quantum Computing: Progress and Prospects*, https://www.nap.edu/read/25196
- DARPA, Quantum Benchmarking Initiative, Stage B,
  https://www.darpa.mil/research/programs/quantum-benchmarking-initiative/stage-b-selection
- R&D World on DARPA QBI,
  https://www.rdworldonline.com/darpas-quantum-benchmarking-push-can-a-useful-quantum-computer-exist-by-2033/
- The Quantum Insider, *Quantum progress demands manufacturing revolution, Martinis says*
  (2025-11-20),
  https://thequantuminsider.com/2025/11/20/quantum-progress-demands-manufacturing-revolution-martinis-says/
- McKinsey, *Quantum Technology Monitor 2025*,
  https://www.mckinsey.de/~/media/mckinsey/business%20functions/mckinsey%20digital/our%20insights/the%20year%20of%20quantum%20from%20concept%20to%20reality%20in%202025/quantum-monitor-2025.pdf
- The Quantum Insider, *What's Europe's quantum strategy?* (2025-07-03),
  https://thequantuminsider.com/2025/07/03/whats-europes-quantum-strategy-breaking-down-europes-coordinated-plan-for-global-quantum-leadership/
- IBM, Quantum Developer Conference 2024, https://www.ibm.com/quantum/blog/qdc-2024
- Fermilab SQMS Center, https://sqmscenter.fnal.gov/?p=21273
- Babbush et al., *The Grand Challenge of Quantum Applications*, arXiv:2511.09124,
  https://arxiv.org/abs/2511.09124

Frontier evidence (§4.2, §5.1, §6.3):

- Google Quantum AI, *Quantum error correction below the surface code threshold*, Nature 638, 920
  (2025), arXiv:2408.13687, https://arxiv.org/abs/2408.13687
- Arms, McHugh, Nyhan, Reus, Ulrich, *Estimating Detector Error Models on Google's Willow*,
  arXiv:2512.10814, https://arxiv.org/abs/2512.10814
- Dane et al., *Performance Stabilization of High-Coherence Superconducting Qubits*,
  arXiv:2503.12514, https://arxiv.org/abs/2503.12514
- The Quantum Insider, *Quantum Machines launches open-source framework …* (2025-05-19),
  https://thequantuminsider.com/2025/05/19/quantum-machines-launches-open-source-framework-that-cuts-quantum-computer-calibration-from-hours-to-minutes/
- The Quantum Insider, *AI-driven quantum calibration achieved in Rigetti and Quantum Machines
  challenge* (2024-12-10),
  https://thequantuminsider.com/2024/12/10/ai-driven-quantum-calibration-achieved-in-rigetti-and-quantum-machines-challenge/
- Q-CTRL, *Making quantum computer calibration autonomous, informative, and easy*,
  https://q-ctrl.com/blog/making-quantum-computer-calibration-autonomous-informative-and-easy
- *Vibe Calibration: Autonomous Bring-up of a 112-Qubit Superconducting Quantum Processor by a
  Skill-Orchestrating Language Agent*, arXiv:2606.22376, https://arxiv.org/abs/2606.22376

Gap analysis (§11):

- Google Quantum AI, *Quantum error correction below the surface code threshold*, Nature 638, 920
  (2025), §IV and Outlook, https://www.nature.com/articles/s41586-024-08449-y
- Willow dataset, Zenodo record 13273331, https://zenodo.org/records/13273331
- McEwen et al., *Resisting high-energy impact events through gap engineering in superconducting
  qubit arrays*, arXiv:2402.15644, https://arxiv.org/abs/2402.15644
- Preskill, *Beyond NISQ: The Megaquop Machine*, §2, https://arxiv.org/html/2502.17368v2
- Arute et al., Nature 574, 505 (2019), main text, https://www.nature.com/articles/s41586-019-1666-5;
  read from the copy at
  https://qolab.ai/research/quantum-supremacy-using-a-programmable-superconducting-processor
- Klimov et al. (2018), arXiv v1 PDF, conclusion, https://arxiv.org/pdf/1809.01043v1
- Klimov, Kelly, Martinis, Neven (2020), Outlook and §II, https://arxiv.org/abs/2006.04594
- Martinis (2021), Introduction and §5, https://arxiv.org/abs/2012.06137
- Martinis (2022), §IX and §XI, https://arxiv.org/abs/2104.01544

Ranked papers:

- Klimov et al. (2018), arXiv:1809.01043, https://arxiv.org/abs/1809.01043
- Fowler, Sank, Kelly, Barends, Martinis (2014), arXiv:1405.1454, https://arxiv.org/abs/1405.1454
- Kelly, O'Malley, Neeley, Neven, Martinis (2018), arXiv:1803.03226,
  https://arxiv.org/abs/1803.03226
- Martinis et al. (2005), arXiv:cond-mat/0507622, https://arxiv.org/abs/cond-mat/0507622
- Martinis, Ansmann, Aumentado (2009), arXiv:0904.2171, https://arxiv.org/abs/0904.2171
- Martinis (2021), arXiv:2012.06137, https://arxiv.org/abs/2012.06137
- Kelly et al. (2016), arXiv:1603.03082, https://arxiv.org/abs/1603.03082
- Martinis (2015), arXiv:1510.01406, https://arxiv.org/abs/1510.01406
- Fowler & Martinis (2014), arXiv:1401.2466, https://arxiv.org/abs/1401.2466
- Klimov, Kelly, Martinis, Neven (2020), arXiv:2006.04594, https://arxiv.org/abs/2006.04594
- Martinis (2022), arXiv:2104.01544, https://arxiv.org/abs/2104.01544
- Arute et al. (2019),
  https://research.google/pubs/quantum-supremacy-using-a-programmable-superconducting-processor
- Martinis group publication list, https://web.physics.ucsb.edu/~martinisgroup/publications.shtml
- Nobel Prize in Physics 2025, https://www.nobelprize.org/prizes/physics/2025/press-release/

Patents:

- US 10,606,720 B2, https://patents.google.com/patent/US10606720B2/en
- US 9,940,212 B2, https://patents.google.com/patent/US9940212
- US 11,288,073 B2, https://patents.google.com/patent/US11288073
- US 10,692,009 B2, https://patents.google.com/patent/US10692009B2/en
- US 12,050,996 B2, https://patents.google.com/patent/US12050996
- US 12,050,964 B1, https://patents.google.com/patent/US12050964

In this repository:

- [`positioning.md`](positioning.md), §2.2, §5.5 and §8
- [`example-quantum-control-loop.md`](example-quantum-control-loop.md)
- [`campaign.rs`](../../../deep_causality_quantum/src/types/pipeline/campaign.rs),
  [`control.rs`](../../../deep_causality_quantum/src/types/pipeline/control.rs),
  [`evidence_source.rs`](../../../deep_causality_quantum/src/types/pipeline/evidence_source.rs),
  [`instrument_context.rs`](../../../deep_causality_quantum/src/types/instrument/instrument_context.rs)
