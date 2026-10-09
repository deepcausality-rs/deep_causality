# QCL for Quantum Sensing: Design

Oct 3, 2026 · @Marvin Hansen · revised Oct 7, 2026

The two examples stay. They are rebuilt on three properties of dynamic causality: counterfactual
worlds, dynamic decisions and dynamic context. QCL in its current form cannot run them as designed,
because it plans and adjudicates over typed numbers and never executes its own plan. Section 5
names nine pieces of groundwork, in the order they depend on each other; the examples in sections 6
and 7 assume all nine. Section 8 lists ten published results a separate verification step can
reproduce. Every number in this note was checked against its source or recomputed; the papers sit
in `../../../../deep_causality_quantum/papers`.

## 1. Motivation

I first tried to navigate by clocks, found the hard limit, and moved to quantum sensing, where the
hardware already reaches the precision navigation needs.

**The clock route.** In the weak-field limit of general relativity, a clock's rate depends on the
gravitational potential it sits in. Two clocks at different potentials drift apart by a fractional
frequency of ΔΦ/c². Inverting that relation recovers the potential difference from the measured
time-dilation difference, and the mass distribution follows from Poisson's equation. My prototype
did exactly that, using clock data and modelling from the Galileo constellation work.

```latex
\frac{\Delta f}{f} \approx \frac{\Delta \Phi}{c^2}, \qquad \nabla^2 \Phi = 4 \pi G \rho
```

**Why it stops.** Near Earth's surface, 1 m of height changes the clock rate by about 1.1 × 10⁻¹⁶.
Two problems make that unusable on a vehicle:

- **Clock precision and separation.** Resolving useful potential differences needs clocks at the
  10⁻¹⁸ level, compared over a link, with enough height separation between them. That is
  laboratory equipment.
- **The velocity circularity.** A moving clock also slows from its own speed, by v²/2c². At
  250 m/s that term is about 3.5 × 10⁻¹³, about 3,000 times the signal of 1 m of height. Removing
  it to 1 m of height needs velocity known to about 4 cm/s, and precise velocity is what a vehicle
  loses when GNSS is denied.

**Quantum sensing, and the velocity term that remains.** Atom-interferometer gravimeters measure
gravity through the phase of free-falling atoms, with no reference clock and no relativistic
velocity term. A moving platform still carries a classical one, the Eötvös term
2Ω v cos φ sin α + v²/R. At the vessel trial's latitude of about 15° S, 1 m/s of eastward velocity
error costs about 14 mGal, or 7.25 mGal per knot. A 4 cm/s error costs 0.56 mGal, which is small
against the 15 to 40 mGal features the trial resolved along its track; for the clock route the same
4 cm/s consumes the whole 1 m resolution. An unaided inertial unit is not that good: the trial's
drifted at about 5 km/h, about 1.4 m/s, which could cost up to 20 mGal. The navigation filter
therefore estimates velocity jointly with the gravity match. The difference between the two routes
is the ratio of the velocity error to the signal, and whether a filter can bound it.

**Where the field stands.** A strapdown quantum gravimeter aboard a 29 m vessel navigated 83 km by
gravity map matching with GNSS excluded. Over about 6 h it ended 4.1 km (2.2 nmi) from the truth,
against 26 km (14 nmi) for the unaided inertial solution; the paper prints no DRMS figure. The
sensor is a hybrid: a classical accelerometer is bias-stabilised by the atoms, and an inertial unit
runs independently ([arXiv:2608.25563](https://arxiv.org/pdf/2608.25563)). Coupling a
high-fidelity gradiometer model to a map-matching filter shows non-Gaussian errors from platform
dynamics as the key challenge, with rotation rate and rotational acceleration dominating, and
derives a tilt requirement below 3.3° ([arXiv:2504.11119](https://arxiv.org/pdf/2504.11119)). The
open problems have moved to error budgets, noise attribution, and integrating sensor models with
navigation filters. Attributing an effect to its cause is a causal question.

**Why one framework.** The same DeepCausality stack handles reentry flow and guidance, relativistic
gravity, error-correcting codes and sensor systematics. The two experiments below show that range
on problems a sensing team recognises.

## 2. What dynamic causality adds

QCL plans the cheapest experiments that separate rival causes and names the cause the evidence
leaves standing. Three properties of dynamic causality turn that into a campaign a sensing lab runs.

| Property | In this design | DeepCausality mechanism |
|---|---|---|
| Counterfactual worlds | Every candidate cause is a world with its own ledger. Every experiment is an alternate configuration of the instrument, and a candidate's prediction is that candidate evaluated in that configuration. A correction is an intervention whose outcome is predicted in every world before it is observed. | QCL `fork` and per-world `Ledger` (exist); `Hypothesis::predict`, a mechanism-level `do()` (exists); configured experiments (Q-2); logged context alternation on `CausalFlow` (C-1) |
| Dynamic decisions | The campaign runs the cheapest informative experiment, adjudicates, and stops as soon as one survivor separates; otherwise it re-plans over the survivors. A verdict branches to its corrective action. A baseline observation refuses candidates that cannot produce it. "Outside the model" is a decision outcome. | `CausalFlow::iterate_until`, `branch_with` (exist); sequential campaign (Q-6); campaign adjudication (Q-4); baseline stage (Q-0) |
| Dynamic context | The instrument's configuration, its model (sensitivity, contrast, interrogation time) and its environment (Earth tide, atom temperature, bias field) live in a context. Predictions are evaluated in the context at observation time; a change between planning and observation re-plans. | `deep_causality_context::Context`, `get_data_by_id`, time nodes (exist); instrument context (Q-1); evidence tagged with its context (Q-3) |

## 3. The read-out arithmetic

**The read-out is a probability.** An atom interferometer reports the fraction of atoms in one
state, P = ½(1 − C cos Δφ), with contrast C. At mid-fringe a small phase δφ moves it to about
½ + (C/2)δφ. For a Mach–Zehnder gravimeter Δφ = k_eff g T². With rubidium at 780.24 nm,
k_eff = 1.611 × 10⁷ rad/m, and at T = 100 ms one µGal (10⁻⁸ m/s²) is 1.611 mrad of phase. At
C = 0.5, an offset of −5 µGal moves the read-out from 0.500 to 0.498.

**Separation adds up, and bounds the error.** QCL measures the separation of two predicted
read-outs p and q over n shots as n·(−log₂(√(pq) + √((1−p)(1−q)))) bits, the Bhattacharyya
distance. It is additive over independent draws, so the bits of two experiments on the same pair
add. Under equal priors it bounds the probability of confusing the pair by ½·2^(−bits); a 5-bit
floor keeps it at or below 1/64 per pair.

**Mid-fringe against a dark baseline.** An additive leak of 6 % of the light read at mid-fringe
(0.56 against 0.50) separates at 0.00261 bits per shot: 1,915 shots reach a 5-bit floor, and
1,024 shots reach 2.67 bits. The same leak read against a dark baseline (0.06 against 0)
separates at 0.0446 bits per shot and needs 113 shots; with a 1 % or 2 % background it needs 246
or 330.

**Shots are effective, and they cost instrument time.** Real sensors sit above projection noise, so
a shot here is an effective Bernoulli draw calibrated from the instrument's sensitivity S. At
mid-fringe one effective draw carries phase variance 1/C², so the instrument yields
n_eff = 1/(C k_eff T² S)² effective draws per second. Reaching a floor of F bits between offsets
that differ by Δg takes τ = 8 ln2 · F · S² / Δg².

| Sensitivity S | Source | Δg | τ for 5 bits | effective draws |
|---|---|---|---|---|
| 24 µGal/√Hz | HUST-QG, Metrologia 59 (2022) | 5 µGal | 639 s | 1.71 × 10⁶ |
| 24 µGal/√Hz | | 10 µGal | 160 s | 4.28 × 10⁵ |
| 50 µGal/√Hz | AQG, Ménoret et al. (2018) | 5 µGal | 2,773 s | 1.71 × 10⁶ |
| 50 µGal/√Hz | | 10 µGal | 693 s | 4.28 × 10⁵ |

At 24 µGal/√Hz, C = 0.5 and T = 100 ms the instrument yields 2,677 effective draws per second.
The relation σ(τ) = S/√τ holds only while the noise is white. The AQG paper reports white noise
between 100 and 2,000 s: its 10-minute and 1-hour averages scatter by 25.2 and 10.7 nm/s² against
24.5 and 10.0 predicted, while its 1-day averages scatter by 9.4 nm/s² against 2.6 predicted. A
separation that needs more averaging than the white-noise range allows cannot be bought with time.

## 4. What the quantum crate does today

Read in full for this revision: `deep_causality_quantum/src/types/{pipeline,design,decision,qpu,carriers}`,
most of `qcm/hypothesis.rs`, the decoder abstraction, and the control-stage tests.

**What works and carries over.**

| Capability | Where |
|---|---|
| `Experiment::new(name, cost, shots, predictions)`: one typed probability per candidate | `design/experiment_design.rs:36` |
| `design`: the exact minimum-cost cover of all candidate pairs, a dynamic program over 2^C(n,2) pair sets, 7 candidates by default and raisable with `with_max_hypotheses` | `design/experiment_design.rs:233` |
| `control` stages `observe`, `gate`, `fork`, `predict`, `compare`, `design`, `adjudicate` | `pipeline/control.rs` |
| Mechanism candidates are attributed through `observe` (root) → `fork` → `predict` → `compare` → `adjudicate`; `predict` computes each mechanism's Born read-out from its own channel | `pipeline/control.rs:370-483`; test `test_a_mechanism_world_may_be_compared` |
| A systematic as a real channel: `Channel::unitary(QubitOperator::rotation(Axis::Z, δφ))`; reduced contrast as a dephasing `Channel::from_kraus` | `carriers/qubit_operator.rs:111`, `carriers/quantum_channel.rs` |
| A mechanism-level `do(node ← factor)` on a structural candidate, then evaluation | `qcm/hypothesis.rs:491` (`predict`) |
| `adjudicate`: one survivor separated by the floor, or a named ambiguity (no survivor, several, unseparated) | `design/adjudicate.rs:178` |
| `CausalFlow`: `branch_with` (the predicate reads value, state and context; the arms receive the whole flow), `iterate_until`, `try_step_with`, `update_context`, `alternate_value` | `../../../../deep_causality_core/src/types/causal_flow` |
| `Context` with `get_data_by_id`, `freeze`, neighbour listing and time nodes | `deep_causality_context` |

**What blocks the examples.**

| Id | Gap | Evidence |
|---|---|---|
| G0 | A plan cannot be executed. An `Experiment` holds a name, a cost, shots and typed predictions; `observe` measures an observable on the plant; nothing links a planned experiment to a configuration or an intervention. `qcl_crosstalk` runs its plan's first experiment by hand. | `experiment_design.rs:22`, `control.rs:234`, `qcl_crosstalk/main.rs` (`observe_under`) |
| G1 | Cost does not depend on shots, and device time is charged at one unit per shot. | `control.rs:228-264`, `pipeline/ledger.rs` |
| G2 | A world carries one read-out, so a campaign of several experiments cannot be adjudicated. | `ControlWorld.read_out`, `adjudicate.rs:216` |
| G3 | `control` cannot take measured data: `observe` always samples the model plant. | `control.rs:258-264` |
| D1 | `Config::baseline` and, under `qpu`, `Config::evidence` are accepted and stored, and no stage reads them. A declared shot budget is not enforced. | `pipeline/config.rs:402-415`; no reader in `src/` |
| N1 | `design` counts a pair as covered only when one experiment alone reaches the floor; bits from two experiments are not combined. | `experiment_design.rs:283-292` |
| C0 | `CausalFlow` has no `alternate_context`. The logged counterfactual swap exists on the process (`AlternatableContext`) and not on the flow facade. | `../../../../deep_causality_core/src/types/causal_flow/intervene.rs` |

## 5. Groundwork, in order

Nine pieces, each with the property it serves and what proves it done. The examples start after
Q-6; Q-7 is the regression that proves the new pipeline on the existing keystone example.

```text
Q-0 ──┐
C-1 ──┼─► Q-1 ─► Q-2 ─► Q-3 ─► Q-4 ─► Q-5 ─► Q-6 ─► Q-7 ─► examples ─► verification
      │                                  ▲
      └──────────────────────────────────┘ (Q-5 also reads Q-1)
```

| # | Crate | Change | Serves | Done when |
|---|---|---|---|---|
| Q-0 | quantum | Close D1. `baseline` names the experiment observed first; candidates that cannot produce the baseline observation are refused before planning, with the reason. `evidence` sets the shot budget `observe` draws from, and an overdraw fails. | dynamic decisions | A config with a budget smaller than the plan's shots fails in `observe`; a baseline that excludes a candidate removes it before `design`; both have tests that fail on today's code. |
| C-1 | core | `CausalFlow::alternate_context`, logged like the process's `AlternatableContext`. | counterfactual worlds | A flow that alternates its context carries the alternation marker in its log and the new context in its channel. |
| Q-1 | quantum, context | The instrument context: a `Context<Data<D>, NoSpace<R>, DiscreteTime, NoSpaceTime<R>>` alias over the instrument's payload, keyed by contextoid id. It holds the configuration (wave-vector sign, heading, tilt offset, bias field, atom temperature, Rabi frequency, cloud position, accelerometer correction), the instrument model (k_eff, T, C, S, cycle time, white-noise range, setup time per configuration change) and time-stamped environment readings (Earth tide, temperature, field). It supersedes the earlier `DeviceContext` proposal; a QPU uses the same alias with T1, T2 and gate errors. | dynamic context | A context round-trips through snapshot and restore with a `Storable` payload; reads by contextoid id return each field. |
| Q-2 | quantum | Configured experiments. An experiment names a configuration (a context, or a mechanism-level intervention for structural candidates) and an observable. A response model, one generic type with static dispatch, maps (candidate, configuration) to a channel; QCL computes each prediction as the Born read-out of the plant evolved by that channel. Structural candidates compute theirs through `Hypothesis::predict`. Typed predictions remain for callers without a model. Closes G0. | counterfactual worlds | `qcl_crosstalk`'s structural predictions computed by Q-2 equal `Hypothesis::evaluate` on the intervened factors; a mechanism prediction equals the Born read-out of plant → candidate → configuration channels. |
| Q-3 | quantum | Evidence sources. `observe` runs an experiment against simulated evidence (the Born sampler, today's behaviour) or recorded evidence: counts a lab measured, or a published value with its standard error, converted to effective draws at the fringe slope. Each observation records its experiment, its shots and the context at observation time. Closes G3. | dynamic context | Recorded counts reproduce a simulated run's verdict when they equal its histogram; a published value and its error convert to effective draws and back without loss; an observation carries its context snapshot. |
| Q-4 | quantum | Campaign adjudication. A world carries one read-out per observed experiment; its verdict holds when every observation agrees; the separation of a pair is the sum over shared experiments. `design` may combine experiments per pair (closes N1, opt-in). Closes G2. | dynamic decisions | Law test: a campaign's pair separation equals the sum of its per-experiment separations. Regression: a one-experiment campaign reproduces today's `adjudicate`. |
| Q-5 | quantum | Instrument time. Cost = setup time + integration time; `design` sizes each experiment's shots to the smallest count that reaches the floor for the pairs it serves, within the instrument's white-noise range; the ledger's device time is seconds. Closes G1. | dynamic context | Sizing law: n reaches the floor and n − 1 does not; a pair that needs more averaging than the white-noise range allows is reported uncovered. |
| Q-6 | quantum, core | Sequential campaign. A campaign step for `CausalFlow::iterate_until`: plan over the survivors, run the next experiment, record, adjudicate; stop at a separated survivor or an ambiguity that no remaining experiment can resolve. A context change beyond a tolerance between planning and observation re-plans. | dynamic decisions | On fixed seeds the sequential cost never exceeds the static plan's; a context change mid-campaign triggers a re-plan; each stop reason has a test. |
| Q-7 | examples | Port `qcl_crosstalk` to Q-2 to Q-6: the pipeline runs its own plan, with no hand-run observation. | regression | Same survivor and plan as today; the hand-run `observe_under` and manual `adjudicate` are gone. |

## 6. Experiment 1: Is the gradiometer's correlation benign?

**The situation.** A gradiometer runs two atom clouds, A below and B above, off one shared laser
and mirror. Their read-outs fluctuate together. If the cause is common-mode noise, differential
phase extraction removes it and the gradient estimate stays unbiased. If light from one cloud's
detection reaches the other's detection signal, or the platform rotates, the correlation biases the
gradient, and the bias becomes map-matching error downstream. Passive data cannot tell these apart.

Detection layouts differ. Some instruments image both clouds at once onto separate detectors or
quadrants; others pass both clouds through one detector in sequence, as Sorrentino et al. do. There
the leak is one cloud's stray or residual light inside the other's detection window: minimising
stray light at their photodiodes cut the phase noise from 99 to 56 mrad/√τ
([arXiv:1312.3741](https://arxiv.org/pdf/1312.3741)). No paper read for this note measures leakage
between the clouds directly, so its size is the lab's measurement.

**Candidates** (structural, screened by `validate`)

| Candidate | Structure | Consequence |
|---|---|---|
| H1 | L → A, L → B, plus L → B's signal: A's fluorescence, which follows A's fringe, reaches B's detection signal | Gradient biased |
| H2 | The mirror image: B's fluorescence reaches A's detection signal | Gradient biased |
| H3 | L → A, L → B: shared laser phase and mirror vibration alone | Benign, removed by differential extraction |
| H4 | L → A, L → B ← Ω: platform rotation, a differential centrifugal phase 2bk_eff T²(Ω_x² + Ω_y²) over the baseline b | Gradient biased, always positive ([arXiv:2504.11119](https://arxiv.org/pdf/2504.11119)) |

The factors are classical conditional tables, diagonal Choi factors. Each cloud's atoms and its
detection signal are separate nodes, so a leak runs from one cloud into the other's signal and no
structure is cyclic. The shared phase L takes four equally spaced values, which averages every
product of at most three fringe terms exactly. A leak carries the sending cloud's whole population,
so it depends on L through that cloud's fringe. Each candidate is fitted exactly to the passive
read-out through its two fringe centres and its differential phase, so the passive read-out cannot
separate them. The instrument operates a quarter fringe from the gradient's phase, where every fit
exists. The leak ε, the dark background b and the setup times are placeholders for the lab's
measurements; no paper measures leakage between the clouds.

**Experiments as counterfactual configurations.** Each experiment is a mechanism-level
intervention computed by `Hypothesis::predict` (Q-2), not a typed row.

| Experiment | Intervention | H1 | H2 | H3 | H4 |
|---|---|---|---|---|---|
| E0 Passive | none; read both | passive correlation | passive correlation | passive correlation | passive correlation |
| E1 Brighten A, B dark | do(A ← bright), do(B ← dark, no interferometer pulses); read B | b + ε(1 − b) | b | b | b |
| E2 Brighten B, A dark | the mirror image; read A | b | b + ε(1 − b) | b | b |
| E3 Rotation step | rotate the sensor head about a horizontal axis at 1 mrad/s; read the two signals disagreeing | unchanged | unchanged | unchanged | shifts by the centrifugal phase |

Reading against a dark baseline instead of mid-fringe cuts the shots by a factor of 6 to 17
(section 3) and removes a problem of the first draft: in a gradiometer, common-mode vibration
scrambles each cloud's phase shot to shot, so a single cloud cannot be held at mid-fringe. E3
reads the two signals disagreeing, which depends on the differential phase alone because the
shared phase averages out of it. Q-5 sizes the draws: E1 and E2 take 340 at ε = 0.06 and a 2 %
background, and E3 about 60,000 for the step's 0.247 rad. At Earth's horizontal rate the
centrifugal phase is 6.9 × 10⁻⁴ rad, so E3 rotates the sensor head deliberately.

**The campaign** (Q-6). The campaign runs E1 first. If E1 reads b + ε, H1 is the only candidate
consistent with it and the campaign stops after one experiment. If E1 reads b, H1 is rejected; E2
then separates H2 from H3 and H4, and E3 separates H4 from H3. The static plan needs all three,
because only E1 separates H1 from H3; the sequential campaign stops earlier whenever a survivor
separates.

**Dynamic decisions.** The verdict branches (`branch_with` on the survivor):

- H1 or H2: realign the detection optics, add a baffle or shift a detection window. The change is
  an intervention on the instrument context, ε → ε′, and the same experiment is re-run and judged
  against the prediction recomputed from the updated context. A residual leak reopens the campaign.
- H3: no action; differential extraction stays.
- H4: correct with the gyroscope's rotation rate, or compensate the rotation; verify with E3.
- Leakage both ways: with atoms and signals as separate nodes the two-way leak is acyclic, and it
  is no candidate. The static plan reports it outside the model, because E2 shows a leak H1 does
  not predict. The sequential campaign stops at H1 after E1: a survivor ends it before the
  experiment that would contradict it runs.

The example prints the action for each survivor. The re-run after a realignment is not built.

**Counterfactual consequence.** The instrument's reading is the benign fit of the passive
read-out, the ellipse fit's differential phase. A flow reads it in the survivor's world, alternates
the context to the same world without the mechanism, and reads it again. The difference is the bias
the survivor implies, which a map-matching filter downstream would absorb: −0.044 rad, −356 E, for a
6 % leak, and +6.9 × 10⁻⁴ rad, +5.5 E, for rotation at Earth's horizontal rate.

**Gates** (the example exits nonzero when one fails)

- For each of H1 to H4 as the generating cause, on fixed seeds, the survivor is that cause.
- The static plan reports a two-way leak outside the model.
- The sequential cost never exceeds the static plan's.

**Assumptions specific to this experiment**

- Brightening one cloud changes only the other cloud's detection signal, not its atoms. A lab must
  confirm this.
- Recent gradiometer work names ellipse-fitting bias and rotation × gradient × velocity
  cross-couplings as leading systematics. This experiment asks whether the correlation between the
  clouds is causal; an estimator bias in the differential extraction is a different question.
- This is the sensing counterpart of a question Sarovar et al. leave open for qubits: their protocol
  localises crosstalk and states that it cannot separate detection crosstalk from correlation through
  a shared environment (section 8, V7).

## 7. Experiment 2: Which systematic explains the gravimeter's offset?

**The situation.** After the Earth-tide correction, an atom gravimeter reads −5 µGal against a
trusted reference value. Seven causes could produce the offset, and a passive run cannot separate
them. The first draft listed four and omitted the largest.

Wavefront aberration of the beam splitters is the dominant term of current gravimeter accuracy
budgets, with Coriolis and the two-photon light shift the next largest
([arXiv:2410.07720](https://arxiv.org/pdf/2410.07720)). In Louchet-Chauvet et al. (2011) its
uncertainty is 4.0 µGal of a 5.2 µGal budget. Karcher et al. (2018) measured it at 5.6(1.3) µGal
at 1.8 µK by extrapolating to zero temperature with ultracold atoms
([arXiv:1804.04909](https://arxiv.org/pdf/1804.04909)). A seventh cause comes from Farah et al.
(2014): detection clipping combined with a shifted starting position of the cloud produced a
Coriolis-like shift of the opposite sign to the one expected, and a 180° turn cannot separate it
from true Coriolis because both flip ([arXiv:1406.5998](https://arxiv.org/pdf/1406.5998)).

**Candidates** (mechanisms; each a phase channel R_z(δφ_h(configuration)) on the interferometer's
two-level read-out, with the contrast as a dephasing channel). Seven candidates reach the
planner's default cap.

| Candidate | Cause | Response to the configuration context |
|---|---|---|
| H1 | Coriolis from the atoms' mean transverse velocity | Retained in the k-reversal half-difference; flips sign under a 180° turn about vertical |
| H2 | Quadratic Zeeman shift | Independent of the wave-vector sign, so it largely cancels under k-reversal; a residual remains because the two paths do not overlap in space. With the field split as B = s·B_sn + B_bg, s the coil current over nominal, eq. 9 of arXiv:1805.05159 gives a bias a·s² + b·s + c: a from the coil's own inhomogeneity, b from the coil field across the background's variation, c from the background alone, which a field step does not move. The instrument context carries a, b and c; GAIN's come from its two field maps (Fig. 6(a), (b)), digitised by `../../../../deep_causality_quantum/papers/digitised/digitise_hu2017_fig6.py`: a −0.038, b 1.005, c 0.033 of the nominal bias, which the nominal map puts at 2.01 µGal against the printed 2.04 |
| H3 | Tilt of the beam axis from vertical | Reads low, g cos θ; a tilt scan traces a parabola whose vertex marks vertical; unchanged by a 180° turn when fixed to the instrument |
| H4 | Synchronous vibration of the reference mirror | Removed by the correlation with a co-located classical accelerometer |
| H5 | Wavefront aberration | Retained under k-reversal; fixed to the instrument; vanishes at zero atom temperature for aberrations added in a single plane, with a residual when they propagate (0.27 µGal in arXiv:2410.07720). Not linear in temperature: flat from 2 to 7 µK and non-monotonic below, so a linear extrapolation from laser-cooled temperatures is wrong (arXiv:1804.04909) |
| H6 | Two-photon light shift | Odd in the wave vector, so it survives the k-reversal half-difference; linear in the Rabi frequency at constant pulse area; part of it depends on the bias field, so a field step moves it too (arXiv:0809.0149, eq. 5) |
| H7 | Detection clipping × cloud position offset | Coriolis-mediated, so it also flips under a 180° turn; separated from H1 by a cloud-position scan or velocity selection (arXiv:1406.5998) |

**Experiments as configurations** (each an alternate instrument context; predictions computed by
the response model, Q-2)

| Experiment | Configuration change | Moves |
|---|---|---|
| E0 Passive | wave vector up | baseline (Q-0) |
| E1 k-reversal | reverse the wave vector | H2 (its apparent sign flips, eq. 2 of Louchet-Chauvet et al.) |
| E2 Turn 180° | rotate the sensor head about vertical, re-level | H1, H7 (sign flip) |
| E3 Accelerometer correction | apply the classical correlation to the same shots | H4 (to 0) |
| E4 Tilt step | tilt by θ₀ = 0.5 mrad | H3, through the cross term 2θθ₀; every world shifts by the common g θ₀²/2 |
| E5 Bias-field step | change the bias field | H2, and the field-dependent part of H6 |
| E6 Temperature scan | vary the atom temperature down to tens of nK, extrapolate to zero | H5 |
| E7 Rabi-frequency step | halve the Rabi frequency at constant pulse area | H6 |
| E8 Cloud displacement | displace the initial cloud by 0.5 mm east-west | H7, at 14.2 µGal/mm |

The table shows why the predictions are computed rather than typed. A bias-field step moves two
candidates, and a 180° turn moves two; a typed table either hides those cross-dependencies or
repeats them by hand, while the response model carries them from the physics.

**The baseline stage** (Q-0). E0 is observed first, through `baseline_with`, which observes a
configured experiment from an evidence source rather than the plant. Each candidate's magnitude is calibrated to the
passive offset within its physical range. A candidate that cannot produce the observation is
refused before planning, with the reason: for a positive offset, tilt is refused because g cos θ
only reads low. The physics in the response model carries this; the builder needs no special rule.

**Cost** (Q-5). Each experiment costs its setup time plus the integration time to reach the floor.
The instrument is Louchet-Chauvet et al.'s: one configuration reads 22.1 µGal/√Hz (70 µGal at 1 s
for the four-configuration protocol, √10 above one), so separating 5 µGal at 5 bits takes about
540 s and separating 10 µGal (a sign flip) about 135 s. E6 is the expensive one: ultracold operation in Karcher et al. ran at 1,200 to
3,000 nm/s² at 1 s with a 4.49 s cycle, which puts each temperature point at 10⁴ s or more. The
planner therefore leaves wavefront to elimination whenever the other isolations cover its pairs,
and that trade is the question QCL answers. Setup times come from the lab.

**The plan and the campaign.** A pair is covered when an experiment that moves either candidate and
not the other runs. The static plan isolates six of the seven candidates and identifies the last
by elimination, which the run prints as such: the wavefront, since E6 costs 20,000 s. The
sequential campaign (Q-6) runs the cheapest isolating experiment first and stops at the first
separated survivor. Its cost is at most the static plan's when every context move re-plans; a
drift tolerance can break the bound.

**Dynamic context.** The Earth tide moves gravity by far more than 5 µGal over the hours a campaign
takes; in Louchet-Chauvet et al. the tidal correction alone is −18.8 µGal. Every prediction
includes the tide at its observation time, read from the context, and after each step the campaign
moves its contexts to the new time. Every move re-plans: with a drift tolerance, draws sized at an
earlier tide can leave a pair a fraction of a bit short of the floor, and the campaign then buys
another experiment to close it, up to the 20,000 s temperature scan. The atom-temperature drift is
not built.

**Counterfactual correction** (not built). The survivor's correction is an intervention on the instrument
context: average opposite headings or select the velocity class for Coriolis, re-centre the cloud
for clipping, re-level for tilt, reduce the field gradient for quadratic Zeeman, use a colder source
for wavefront, recalibrate the Rabi frequency for the light shift. Before the verification run,
every world predicts the residual after the correction: zero under the survivor, unchanged under the
rivals. If the observed residual does not vanish, the correction failed and the campaign reopens
over the remaining candidates.

**Gates**

- For each of the seven generating causes, on fixed seeds, the survivor is that cause.
- A positive-offset run refuses tilt at the baseline stage and says why.
- A run whose contexts record no tide refuses every candidate at the baseline, which shows the
  context is necessary.
- A run that drops H7 from the list while H7 generates the data misattributes to H1: without H7
  no pair needs E8, and a turn flips both alike (section 8, V2).

**Assumptions specific to this experiment**

- The response of each systematic to each configuration is instrument specific. Whether a field or
  a misalignment turns with the sensor head depends on the design; the lab confirms every row.
- Elimination is only as good as the candidate list. An eighth cause would be assigned to whichever
  candidate is eliminated last, and adding it raises the cap to 2^28 planner states.

**A moving-platform variant follows the site section.** On a vessel the Eötvös term adds a
velocity-error candidate, reciprocal survey lines provide the heading reversal of E2, and the
verdict feeds counterfactual navigation solutions, one per surviving candidate's correction. V8 and
V9 bring their own candidate sets, which do not overlap H1 to H7, and the navigation solutions need
a gravity map and a map-matching filter that the workspace does not have. V8 and V9 run with the
variant.

## 8. Verification against published results

Verification is a separate step after the groundwork and the examples. Its only purpose is to show
that this QCL implementation reproduces published conclusions from the same published inputs,
whether the authors used another implementation or only analysis. Where a paper offers calculations
and no data, replicating it substantiates the method rather than testing an implementation.

**Common conditions.**

- No paper ships code or raw data; the AQG, Pagot and Lellouch papers offer data on request. Values
  printed in text and tables are used as printed; values that exist only in figures are digitised,
  and the digitisation error enters the observation's uncertainty.
- Published values with standard errors enter through Q-3's recorded evidence, converted to
  effective draws.
- An equivalence passes when QCL reaches the paper's attribution, or the paper's ambiguity where the
  paper reaches none, and its numbers agree within the published uncertainty.
- The sources contain inconsistencies, listed per entry; each is resolved and documented before its
  numbers are encoded.

**The targets**, in priority order.

| # | Source | What QCL reproduces | Equivalence | Notes |
|---|---|---|---|---|
| V1 | Karcher et al., NJP 20, 113041 (2018), arXiv:1804.04909 | Candidates wavefront, Coriolis, one-photon light shift, atom interactions; experiments temperature scan (50 nK to 7 µK), 180° turn, k-reversal, atom number (25,000 to 5,000 at 650 nK) | Wavefront is the sole survivor; its bias g(0) − g(1.8 µK) agrees with −56(13) nm/s² (Table I); a wavefront model linear in temperature fitted to the 2–7 µK points alone does not reproduce it; adding the two-photon light shift with no separating experiment returns an ambiguity | Fig. 2 must be digitised; its axis label reads mm/s² and means nm/s². The cleanest target: one paper, four configuration changes, a published attribution |
| V2 | Farah et al., PRA 90, 023606 (2014), arXiv:1406.5998 | The wrong-sign Coriolis result: +46(2) µGal per unit molasses imbalance where Coriolis predicts a negative shift | With only {Coriolis, wavefront}, no candidate is consistent with the sign, reported as outside the model; with clipping × position added, it is named dominant; the planner reports that a 180° turn does not separate Coriolis from clipping, and that a coil displacement (14.2(1.1) µGal/mm) or velocity selection does | The text prints "kHz/mm" where Hz/mm is meant; it uses two Coriolis conversion factors, 10.95 and 9.72 µGal/(mm/s) |
| V3 | Louchet-Chauvet et al., NJP 13, 065025 (2011) | The four-configuration protocol (k up/down at Ω and Ω/2), the 180° turn, and the temperature scan from 2 to 6 µK | The planner reports that the k and Rabi configurations cannot separate Coriolis from wavefront and that the turn can; Coriolis 0.5 ± 0.4 µGal; the temperature scan returns an ambiguity for wavefront (fits range from −4 to 15 µGal), not a point value; the budget sums reproduce −24.3, −19.7 and −44.0 µGal (Table 1) | Eq. (4) as printed is off by a factor of 2; the device total uncertainty of 5.1 µGal does not follow from the rows (quadrature sum 4.16) |
| V4 | Sarovar et al., Quantum 4, 321 (2020), arXiv:1908.09855 | The crosstalk scenarios of §7 regenerated with pyGSTi from the published error models: one-way depolarisation, coherent ZZ, detection crosstalk, and the six-qubit ladder | The pair set implied by QCL's survivor equals the published edge set ({S0–R1}; {R0–R1, R0–S1, S0–R1}; {R0–R1}; {R0–S5, R1–S4, R2–S3}); the survivor is the generating model; a crosstalk-free control yields no edge; dropping the ZZ candidate on its data yields an ambiguity | Edge weights are not compared: the paper calls them no physical measure, and the pyGSTi code halves the sum that eq. (12) prints. Beyond the paper, simulator ground truth checks QCL separating detection crosstalk from a shared bath |
| V5 | Gauguet et al., PRA 78, 043615 (2008), arXiv:0809.0149 | The two-photon light shift on the T = 50 ms gravimeter | The phase that is odd in k and linear in the Rabi frequency at constant pulse area is attributed to the light shift, not to quadratic Zeeman; 32 mrad fitted against 33 expected; eq. (8) gives 21.7 mrad for the counter-propagating part | Eq. (2) and the text disagree on a sign convention. Do not mix these numbers with the T = 70 ms instrument of V3 |
| V6 | Ménoret et al., Sci. Rep. 8, 12300 (2018), arXiv:1809.04908 | Q-5's time model at S ≈ 500 to 600 nm/s²/√Hz | 10-minute averages 25.2 against 24.5 nm/s² and 1-hour averages 10.7 against 10.0; the 1-day scatter of 9.4 against 2.6 is reported as outside the white-noise range rather than sized; the tilt scan's quadratic response (0 to 1.5 mrad, about 10 µm/s²) | No accuracy budget, so no attribution target |
| V7 | Sorrentino et al., PRA 89, 023607 (2014), arXiv:1312.3741 | The gradiometer's parameter sweep of Table I and three single-cause checks | Mirror tilt attributed to Coriolis (−37 ± 5 against −34 mrad/mrad predicted); magnetic pulse 69 ± 1 against 72 mrad/mA predicted; the MOT intensity ratio attributed to cloud temperature by elimination, as the paper says "most probably"; the dominance set matched by set inclusion with tolerance | Text and table disagree on the tilt slope's sign and on two upper limits; the text's dominance list omits Raman intensity, which ranks third by slope × fluctuation |
| V8 | Everitt et al. (Q-CTRL), arXiv:2608.25563 (2026) | (a) Gimbaled against strapdown over identical traversals; (b) atom correction on and off; (c) map against sensor near the coast | (a) Below 0.3 m/s² of heave the configuration-independent floor survives and the strapdown penalty is eliminated; above it QCL returns the ambiguity gimbal saturation or heave noise, since no strapdown run sampled that range, and proposes a strapdown run there; (b) classical bias and drift survive (570 mGal offset); (c) map degradation survives, from the independent GA-0106 survey line | Sea state differs between the configurations, which confounds (a). The endpoint error is printed as 4.0 and 4.1 km; the duration as 6 h and as a 5 h window |
| V9 | Lellouch & Holynski, arXiv:2504.11119 (2025) | The in-silico ablation of ship motion and the closed-form requirements | Rotation rate and rotational acceleration survive, static tilt and transverse acceleration are eliminated; α < 3.3° and Ω < 7.07 × 10⁻⁵ s⁻¹ reproduce exactly; the Ω̇ bound of 7 × 10⁻⁴ s⁻² reproduces as sufficient, not tight | A method check: per-ablation numbers are not printed, code on request |
| V10 | Hu et al., PRA 96, 033414 (2017), arXiv:1805.05159 | The quadratic Zeeman bias from the measured field map, and H2's coefficients a, b and c from the nominal and half-current maps | 2.04 µGal from the digitised Fig. 6(a) map and the printed trajectory; a·s² + b·s + c at s = 1 returns the same value | A response-model check, not a discrimination. γ₂ is printed in Hz/nT² and only works in Hz/µT² |

**Results.** The harnesses are `deep_causality_quantum/verification/sensing/verification_v*.rs`,
53 checks, all passing; the directory's README lists the data, its provenance and the run commands.
Where a result departs from the equivalence above, the departure is the finding.

| # | Equivalence | Result |
|---|---|---|
| V1 | Partly | The scan attributes to the wavefront alone among the wavefront, a linear wavefront fitted above 2 µK, Coriolis of a centred cloud and the one-photon light shift; the linear extrapolation lands 4.4σ from −56(13). Interactions are not eliminated: the printed bound, 1 nm/s² per thousand atoms, is (7 + 12)/20, one standard error, and at three they stay a possible cause of the rise at 650 nK. Refitting Fig. 2 with Fig. 4's responses gives 30.7, 24.3, −21.7 and −43.1 against Table I's 37, 9, −19 and −56, each within one standard error; the correlation coefficients do not reproduce. Fig. 4 computes each of Fig. 2's temperatures above the lowest at 1.5 times its value, and the drawn fit (χ² 17.7) is better than any combination of Fig. 4's responses (36.7). A light shift with the wavefront's response returns an ambiguity; the planner needs the turn, the atom-number change and a Rabi change |
| V2 | Yes | Outside the model with {Coriolis, wavefront}; with clipping, only clipping with Coriolis holds, unseparated from clipping alone at the floor. The turn separates nothing, a 1 mm displacement does. §VII uses 10.94 and 10.96 µGal/(mm/s); "kHz/mm" reads as Hz/mm |
| V3 | Yes | Budget sums reproduce; 5.1 against a quadrature of 4.16; eq. (4)'s factor 2. Fig. 9(c): 4 of 5 fits hold, matching the paper's "most"; the holding fits extrapolate from −4.50 to 4.82 µGal, against the text's −4 to 15 |
| V4 | Yes | Survivor and published edge set in all four scenarios; the control establishes none; dropping ZZ leaves no survivor. Detection crosstalk and a shared bath separate though both imply only R0–R1. pyGSTi's PC on the same draws misses the published edges in three scenarios and finds them in 9, 7, 9 and 8 of ten further draws. pyGSTi's edge weights halve eq. (12)'s sum |
| V5 | Yes | The error bars alone reject every candidate; with the printed ±10 % or the repeated ratios' pooled scatter (2.74 mrad), eq. (7) survives over eq. (8) alone and quadratic Zeeman. Fit 32 against 33. Eq. (8) gives 21.7 mrad from the printed Doppler shifts, 20.0 from the timing. The ratio-½ triplet spreads ±16 %, beyond the printed ±10 % |
| V6 | Yes | 24.5 against 25.2 ± 0.7 and 10.0 against 10.7 ± 0.7; the 1-day 2.55 against 9.4 lies outside white noise; tilt −11.03 µm/s² at 1.5 mrad, quadratic |
| V7 | Yes | Coriolis survives (eq. (10) from the printed inputs: −35.2); the pulse's 69 ± 1 against 72 holds at 2.7σ with the prediction's rounding; the MOT ratio is outside the quantified candidates; the text omits Raman total intensity, third by slope × fluctuation |
| V10 | Yes | 2.010 against 2.04 µGal, within the digitisation's 0.040; γ₂ in Hz/µT²; the coil's field 5424.07 ± 6.14 nT against 5424.98 ± 6.24 |

**Not an equivalence target.** Stace et al., Phys. Rev. Applied 21, 014012 (2024),
arXiv:2211.09090, estimate continuous parameters with an adaptive Bayesian loop, publish single runs
and no code. QCL discriminates among discrete candidates with a static cover. A fair comparison
would be static against static: offer QCL the paper's 300 equally spaced Rabi points over
discretised hypotheses and report how many it needs. That is a cost comparison, not a verification.

## 9. Limits and assumptions

- **Evidence.** Simulated by default. Recorded evidence enters through Q-3 when a lab or a paper
  supplies it; until then every run demonstrates the logic, not the physics, and the release says
  so on its first screen.
- **The physics is supplied.** QCL plans, predicts from the response model, and adjudicates. The
  response model's parameters and every instrument-specific row come from a cited budget or a lab.
- **One dominant cause per candidate.** Mixtures enter as extra candidates. The planner solves
  exactly up to 7 candidates by default, and Experiment 2 uses all 7; raising the cap costs
  2^C(n,2) states.
- **Causes outside the list.** A cause that is no candidate shows up as no survivor only when an
  experiment it contradicts runs. The static plan runs every planned experiment; a sequential
  campaign stops at the first survivor, and the two-way leak stops it at H1.
- **The quantum screens do not engage here.** Experiment 2's candidates are mechanisms, which skip
  `validate`; Experiment 1's factors are diagonal, so the Markov and decomposability screens pass.
  The value is planning, prediction, adjudication and the campaign; the write-up claims nothing more.
- **Modelled drift only.** Tide, temperature and field drift enter through the context. A drift the
  context does not model breaks the stationarity a campaign assumes.
- **White noise within a range.** Integration time is sized on σ(τ) = S/√τ, which holds only inside
  the instrument's white-noise range.
- **Prior art.** Adaptive Bayesian experiment design is established for calibrating quantum devices
  (Stace et al., 2024). QCL's campaign is an exact static cover, which bounds the worst case, plus
  sequential stopping. It is not a Bayes-optimal policy, and the write-up does not claim efficiency
  against one.

## 10. Build plan, sources and open decisions

**Build plan**

- [x] Groundwork Q-0, C-1, Q-1 to Q-7, in the order of section 5, each with its tests.
- [x] Experiment 1 as `qcl_gradiometer_crosstalk`, on the structural path with configured
      interventions.
- [x] Experiment 2 as `qcl_gravimeter_systematics`, on the mechanism path with the response model.
- [x] Replace every placeholder with values from the cited papers, and cite them in the examples.
      Sourced: the Zeeman shares (digitised), the bias field and Rabi frequency (Farah et al.), the
      tilt step (Ménoret et al.), the cloud step (inside Farah et al.'s scan), the contrast (the
      AQG's 40 %, the CAG papers state none). No published source, so still placeholders: the
      gradiometer's leak and dark background (the lab's measurement), every setup time, and the
      tide (the site's).
- [x] Verification V1 to V7 and V10 (section 8), as its own step after the examples:
      `../../../../deep_causality_quantum/verification/sensing`.
- [ ] A "Quantum sensing" section on the quantum site, opened with the field's state as cited
      facts and carrying the limits on the Boundaries page.
- [ ] The moving-platform variant (section 7), with V8 and V9.

**Decisions**

1. Q-0: `baseline` and `evidence` are wired.
2. Q-2: one generic trait, `ResponseModel<R, C>`, over (candidate, configuration).
3. H2's response to a field step is the polynomial a·s² + b·s + c, its coefficients in the
   instrument context.
4. The moving-platform variant follows the site section.
5. Verification reads V1, V3 and V5 from the PDFs' vector paths and V10 from a raster, with the
   scripts in `../../../../deep_causality_quantum/papers/digitised`; V4 regenerates its data from the published
   models with `papers/regenerated/regenerate_sarovar2020.py` and pyGSTi.

**Sources** (local copies in `../../../../deep_causality_quantum/papers` unless marked)

- P. J. Everitt, D. H. White et al. (Q-CTRL), *GNSS-free quantum gravity-aided navigation and
  fine-scale marine surveying with a strapdown quantum gravimeter*, arXiv:2608.25563 (2026)
- S. Lellouch, M. Holynski, *Integration of a high-fidelity model of quantum sensors with a
  map-matching filter for quantum-enhanced navigation*, arXiv:2504.11119 (2025)
- L. Pagot, S. Merlet, F. Pereira Dos Santos, *Influence of optical aberrations on the accuracy of an
  atomic gravimeter*, Opt. Express 33, 18843 (2025), arXiv:2410.07720
- N. Mouelle, J. Mitchell, V. Gibson, U. Schneider, *Wavefront curvature and transverse atomic motion
  in time-resolved atom interferometry: impact and mitigation*, arXiv:2510.26739 (2025)
- R. Karcher, A. Imanaliev, S. Merlet, F. Pereira Dos Santos, *Improving the accuracy of atom
  interferometers with ultracold sources*, New J. Phys. 20, 113041 (2018), arXiv:1804.04909
- A. Louchet-Chauvet et al., *The influence of transverse motion within an atomic gravimeter*, New
  J. Phys. 13, 065025 (2011)
- T. Farah et al., *Effective velocity distribution in an atom gravimeter: effect of the convolution
  with the response of the detection*, Phys. Rev. A 90, 023606 (2014), arXiv:1406.5998
- A. Gauguet et al., *Off-resonant Raman transitions impact in an atom interferometer*, Phys. Rev. A
  78, 043615 (2008), arXiv:0809.0149
- Q.-Q. Hu et al., *Mapping the absolute magnetic field and evaluating the quadratic Zeeman effect
  induced systematic error in an atom interferometer gravimeter*, Phys. Rev. A 96, 033414 (2017),
  arXiv:1805.05159
- V. Ménoret et al., *Gravity measurements below 10⁻⁹ g with a transportable absolute quantum
  gravimeter*, Sci. Rep. 8, 12300 (2018), arXiv:1809.04908
- F. Sorrentino et al., *Sensitivity limits of a Raman atom interferometer as a gravity gradiometer*,
  Phys. Rev. A 89, 023607 (2014), arXiv:1312.3741
- M. Sarovar et al., *Detecting crosstalk errors in quantum information processors*, Quantum 4, 321
  (2020), arXiv:1908.09855
- T. M. Stace et al., *Optimized Bayesian system identification in quantum devices*, Phys. Rev.
  Applied 21, 014012 (2024), arXiv:2211.09090
- Not stored (the publisher blocks scripted downloads): differential phase extraction in atom
  gradiometers, EPJ Quantum Technology (2024), doi:10.1140/epjqt/s40507-024-00292-4; HUST-QG
  evaluation, Metrologia 59 (2022), 24 µGal/√Hz and 3 µGal
- [ESA: probing navigation via the quantum realm](https://www.esa.int/Applications/Satellite_navigation/ESA_probing_navigation_via_the_quantum_realm)
- [DeepCausality Quantum and the crosstalk example](https://quantum.deepcausality.com)
