<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 3 — research dossier: reentry navigation, reference data, unified-sim SOTA, relativistic timing

**What this is.** The full, cited research record behind the product decisions ⑥/⑦/⑧ resolved in
[`gap-three-resolution-3-trajectory-axis.md`](gap-three-resolution-3-trajectory-axis.md) Part F. The
resolution note carries the *decisions*; this note carries the *evidence* — three SOTA literature sweeps
(June 2026) with concrete numbers and every source URL, so the spec's reference choices are auditable.

**Framing (from the user).** The physics is well-known and already substantially **verified** in
`deep_causality_cfd/verification/` (`qtt_park2t_blackout`, `qtt_sod`, `qtt_ramc_stagline`, the rank studies,
the incompressible suite). The physics is **not** the hard problem. The hard problem is **the navigation
sensor data of a reentry vehicle** — and in a *unified* simulation envelope the simulation **generates** it
(synthetic sensors sampling the sim's own ground truth; the physics electron-density field gates GNSS
availability). The entire field of fully-integrated physics+navigation reentry simulation is slim, so a
clean synthetic-sensor-in-the-loop design is both warranted and novel.

---

## Thread 1 — Validation reference data (reentry plasma + trajectory)

### 1A. The single best RAM-C electron-density reference

> **W. L. Grantham**, *"Flight Results of a 25,000-Foot-Per-Second Reentry Experiment Using Microwave
> Reflectometers To Measure Plasma Electron Density and Standoff Distance,"* **NASA TN D-6062**, NASA
> Langley, December 1970. — ntrs.nasa.gov/citations/19710004000 ·
> ntrs.nasa.gov/api/citations/19710004000/downloads/19710004000.pdf

This is the **RAM-C II** flight; the dataset modern papers re-digitise for **peak electron density vs.
altitude** (four-frequency L/S/X/Ka microwave reflectometers). Verified numbers:

- Velocity **25,000 ft/s = 7.62 km/s** (peak 25,190 ft/s ≈ 7.68 km/s); effectively constant over the
  diagnostic window (60–80 km).
- Nose radius **R = 0.1524 m** (6 in), 9° half-cone, body length 8.5 R.
- Reflectometer electron-density range **~10¹⁰–10¹³ cm⁻³ (≈10¹⁶–10¹⁹ m⁻³)**; stagnation peak reaches
  ~10¹⁹–10²⁰ m⁻³.
- **Blackout boundaries — Figure 24** ("onset and end of RF signal blackout" vs velocity & altitude for
  VHF/C-band/X-band): attenuation onset ≈ **276,000 ft (84.12 km)**; ~30 s total blackout; large-AoA
  reflectometer oscillations near **125,000 ft (38.10 km)**.
- **Table X** ("Reflectometer-Determined Electron Densities"): onset / 20%-reflection / decay altitudes and
  times per body station (x/D) per band — the station-resolved peak-`n_e`-crossing-critical-density table.

**Companion primary source** (wall-normal electron-density rake profiles, `n_e(y)`):

> **W. L. Jones, Jr. & A. E. Cross**, *"Electrostatic-Probe Measurements of Plasma Parameters for Two
> Reentry Flight Experiments at 25,000 Feet Per Second,"* **NASA TN D-6617**, NASA Langley, 1972. —
> ntrs.nasa.gov/citations/19710011633 · also **N. D. Akey & A. E. Cross**, *"Radio Blackout Alleviation
> and Plasma Diagnostic Results …,"* NASA (1970) — source for the three-altitude RAM-C-II case.

**Caveat to verify before locking the spec:** NTRS metadata surfaces "NASA SP-252" for the probe paper while
modern CFD papers cite **TN D-6617 (1972)**. Cite TN D-6617 (the number production papers use) and confirm
against NTRS full text.

### 1B. Trajectory profile to drive the unified sim

Use the **RAM-C II trajectory itself** (community benchmark; lets the physics arc and validation share one
envelope). Standard **constant-velocity, three-altitude** discretisation (Park/Candler/Boyd lineage):

| Altitude | Velocity | M∞ | T∞ | Notes |
|---|---|---|---|---|
| 61 km | 7.62–7.65 km/s | 23.9 | 255.9 K | p_dyn ≈ 8 kPa; p∞ ≈ 19.962 Pa |
| 71 km | 7.62–7.65 km/s | 25.9 | 217.9 K | p_dyn ≈ 2.28 kPa; p∞ ≈ 4.844 Pa |
| 81 km | 7.65 km/s | 28.3 | low-density slip (Kn ~0.001–0.1) | needs Park-1993 forward control temp |

For the **GNC/blackout-window arc**: descend ~84 km (attenuation onset, Grantham Fig. 24) → ~30 s blackout →
VHF reacquisition near ~38 km (125 kft). Alternative high-energy continuous-entry references: **Fire-II** and
**Stardust** (11+ km/s) — but RAM-C II is the right choice for a unified physics+nav envelope at glide speeds.

### 1C. Secondary / canonical datasets

- **RAM-C I** — ablating, alkali-seeded; Grantham TN D-6062 Tables V/VI compare blackout times.
- **OREX** (NAL/NASDA, 1994) — 1.35 m nose, 50° half-cone, 5-probe rake; `n_e` 80–100 km. Doihara & Nishida,
  *Shock Waves* 11(5):331 (2002), doi 10.1007/s001930200119.
- **Fire-II / Stardust** — high-velocity radiation/ionization CFD-validation standards.
- **EXPERT** (ESA) — NATO RTO-EN-AVT-130-13.
- **Apollo / Gemini / Mercury** — VHF blackout *durations* (not `n_e`). JPL IPN PR 42-150C; DOT/Volpe
  RF-blackout review (rosap.ntl.bts.gov/view/dot/12493).
- Recent ground/theory (2020+): DSMC ionization, *Advances in Aerodynamics* 2:30 (2020); Kim & Jo, *Int. J.
  Heat Mass Transfer* 169:120950 (2021).

### 1D. Production-code accuracy band vs RAM-C (sets the gate tolerance)

Best-in-class **CFDWARP** (Parent, U. Arizona): Parent et al. arXiv:2111.09432 (2021); Rodriguez-Fuentes &
Parent, Phys. Fluids 37:013609 (2025, arXiv:2410.12760) and 38:036114 (2026, arXiv:2512.18163).

- **Axial peak `n_e` along the body** (vs reflectometer): "good" at 61 & 71 km (within **a few percent** near
  the leading edge), "fair" at 81 km.
- **Wall-normal `n_e` at the probe rake (x/R ≈ 8):** all solvers **underpredict the peak standoff by ~2×** at
  71 km — and Parent attributes this to **experimental inconsistency**: the rake (probe) peak `n_e` differs
  from the reflectometer peak `n_e` by **~2× at the same station**. So the ~2–3× spread is **partly
  dataset-internal**, which sets an *honest* validation tolerance.
- 81 km is the hard case (slip regime, freestream uncertainty). Park's model gives the **largest** `n_e`
  deviation; adjusted Dunn-Kang / Marrone-Treanor do better; 2-T underperforms 3-T.

**Spec tolerance:** few-percent on *axial peak `n_e`* at 61/71 km vs Grantham; accept **~2–3×** on
wall-normal peak location/magnitude at 71 km and on 81 km (reflects genuine reflectometer-vs-probe
disagreement, not just model error).

---

## Thread 2 — Reentry GNC through plasma blackout (the navigation problem)

The defining constraint is the 3–4 min GNSS/comms blackout. SOTA splits into (1) coast on a high-grade INS
and bound the drift, and (2) aid the INS through the sheath with a sensor that survives it — the 2025-26
breakout being **celestial/optical navigation imaging stars and LEO satellites *through* the plasma**.

### 2A. Sensor configuration (tiered by blackout-survivability)

| Sensor | During blackout? | Grade / noise | Role |
|---|---|---|---|
| Strapdown IMU/INS | **Yes — primary** | nav-grade: gyro bias **< 0.01 °/hr**, accel **< 10 µg** (RLG/HRG/FOG); tactical ~1 °/hr, ~1 mg | dead-reckoning backbone |
| Optical/celestial (star + LEO-sat tracker) | **Yes — the SOTA aid** | Draper sliced-lens star tracker **~50 m** GNSS-denied; Varda/Rhea **AutoNav**: 2 cameras + flight computer, images *through* plasma | bounds INS drift mid-blackout |
| MHD / traveling-magnetic-field "magnetic window" | partial (research) | reduces plasma density to pass L/S/C-band | comms/RF mitigation, **not** a nav sensor |
| Skin-temp / heat-flux + aero-density aiding | indirectly | stagnation heat flux ∝ density; weak observability | soft altitude/density constraint |
| GNSS receiver | **No** (attenuated/lost) | — | pre/post-blackout only |

**Through-plasma optical nav — the credible existence proof:** Varda's **W-6** (March 2026) flew Rhea Space
Activity's **AutoNav** — two vented cameras + flight computer, the camera design informed by spectrographic
surveys of Starlink satellites at Magdalena Ridge Observatory specifically to image LEO satellites/stars
*through the plasma envelope*, cross-referenced against the US Space Force Unified Data Library; algorithm
lineage JPL Deep Impact / Spitzer / OSIRIS-REx ("AutoNav"). Draper's patented **sliced-lens star tracker**
quotes **50 m** accuracy in GNSS-denied environments, used to "discipline the IMU." (Exact AFRL/Varda/Rhea
accuracy is redacted; ~50 m is Draper's published spec and a reasonable planning value.)

**Recommended blackout config:** nav-grade strapdown INS (gyro < 0.01 °/hr, accel < 10 µg) propagation core,
aided by an optical celestial/RSO tracker (~50 m), with aero-density/heat-flux as a weak altitude
constraint. MHD magnetic-window = comms recovery, not nav.

### 2B. Filter + Q/R + INS-only drift

**Filter: 15-state error-state (indirect) EKF (ESKF).** Standard for INS/GNSS — estimates the small,
near-linear *error* state, keeps attitude error in 3-parameter form. UKF / cubature KF (CKF) is justified
*only* for the strongly-nonlinear optical-bearing measurement update (a robust/adaptive CKF for GNSS-denied,
noise-varying INS is an active 2025 result, MDPI Micromachines 16(10):1116).

**15-state vector:** position(3), velocity(3), attitude-error(3), gyro bias(3), accel bias(3); biases as
random-walk / first-order Gauss-Markov (keeps biases calibrated through the coast → fast reacquisition).
Tightly coupled to GNSS adds **receiver clock bias + clock drift → 17 states**.

**Q (process noise):** drive bias random-walk from the IMU spec (gyro ARW ~0.001–0.005 °/√hr, accel VRW tens
of µg/√Hz); position/velocity Q from un-modelled acceleration (aero/buffet) — inflate during reentry buffet.
**R (measurement noise):** optical fix ~50 m (1σ) position-equivalent (or angular arc-sec–arc-min for a
bearing; inflate for plasma-induced jitter); GNSS pseudorange m-level pre/post.

**INS-only drift over 3–4 min (the load-bearing number):**

- Error growth laws: position error from **gyro bias ∝ t³**, from **accel bias ∝ t²** (polynomial for
  intervals ≪ the 84-min Schuler period — so 3–4 min is squarely polynomial, no Schuler bounding).
- Magnitudes: a quiescent nav-grade unit (~500 m/hr) ≈ 25–50 m over 3–4 min; a tactical-maneuver case in the
  literature hit **~100 m in 3 min (~33 m/min)**. For nav-grade through a *high-dynamic* hypersonic reentry
  coast, plan **~10²–10³ m (hundreds of m to ~1–2 km)**, dominated by the t³ gyro term + high-g cross-coupling.
  Worked floor: 10 µg accel bias double-integrated over 240 s ≈ ½·(10 µg·9.81)·240² ≈ **~28 m** from that
  term alone. Tactical-grade (1 °/hr, 1 mg) blows out to **km** — why the optical through-plasma aid matters.

### 2C. GNSS reacquisition after blackout

Tightly-coupled INS/GNSS; carry bias + clock states through the coast. Because the ESKF keeps IMU biases
calibrated, the inertial coast holds position tight enough that **carrier-phase integer-ambiguity resolution
re-converges rapidly** on signal return; the receiver clock bias/drift coast on their random-walk model
(unobservable during blackout) and **a single returning pseudorange** starts correcting INS drift. Recipe:
hold the 17-state ESKF on IMU-only through blackout (Q inflated), keep clock states alive, feed pseudoranges
on first fix (loose → tight). Well-bounded inertial position makes ambiguity fixing near-instant.

### 2D. The unification gap (navigation side)

**No published environment closes aerothermo → plasma sheath (and its GNSS/comms/optical effect) → GNC
filter in one simulation.** Pieces exist: Dymos/OpenMDAO (trajectory + aerothermal *load* constraint, not a
plasma-EM-to-filter model); thermochemical-nonequilibrium N-S + ablation (Phys. Fluids 34:126103, 2022) and
DLR CoNF stop at the electron-density field; plasma→RF link models (arXiv:1407.6635; MHD magnetic-window AIP
Advances) are link-budget level; AIAA 2023-3097 *uses* GNSS attenuation to reconstruct the sheath (inverse
coupling, a diagnostic). The forward, fused loop is the deep_causality opportunity.

---

## Thread 3 — Unified-simulation SOTA + relativistic timing standard

### 3A. The standard reentry toolchain (what it couples)

| Tool | Discipline | Couples to | Note |
|---|---|---|---|
| **POST2** (NASA Langley) | 3-/6-DOF trajectory + GNC | reads aero/aerothermal **databases** | trajectory optimiser; consumes decks, no live flowfield |
| **DPLR** (NASA Ames) | structured N-S aerothermo, nonequilibrium | trajectory via offline DB | CEV/Orion aero database code |
| **LAURA** (NASA Langley) | structured aerothermo + radiation | trajectory via offline DB | co-validated w/ DPLR on FIRE II |
| **US3D** (UMN) | unstructured implicit N-S, nonequilibrium | offline-DB pattern | FIRE II code-to-code w/ DPLR/LAURA |
| **DSMC** (DAC/SPARTA/MONACO) | rarefied/transitional | loosely coupled to continuum + photon-MC radiation | "loosely coupled DSMC-PMC" |
| **OpenMDAO + Dymos** (NASA) | MDO + optimal-control trajectory | embeds subsystem models in one optimisation graph | closest to a single envelope — but for *design optimisation*, not live physics+plasma+nav co-state |

Cross-verification of the flagship aerothermo solvers: **Hash et al., AIAA 2007-605** (FIRE II: DPLR/LAURA/US3D).

### 3B. The documented stovepipe gap

**No published framework couples aerothermo + plasma/ionization + GNC as live shared state — they exchange
tables.** Survey-language evidence:

1. Plasma decoupled from flowfield: "a decoupled modeling approach … stagnation region approximated as a 1-D
   inviscid normal shock" (J. Spacecraft & Rockets, doi 10.2514/1.A33122).
2. Multiphysics partitioned: "separate solvers … exchanging boundary data … once during each time step";
   past attempts "either computationally inefficient or numerically unstable" (HYPATE aerothermoelastic).
3. GNC its own stovepipe: blackout treated as an external outage to coast on IMU; *Chinese J. Aeronautics*
   (2021) review — full-state coupled guidance-control "remains underexplored."

**Net:** physics (CFD/aerothermo), plasma (decoupled 1-D shock or separate EM solver), and GNC are three
separate processes exchanging **pre-computed databases/decks**, not one auditable shared-state simulation.
That gap is the unified envelope's target — confirmed, not refuted, in the reviews.

### 3C. Prior unified / digital-twin / counterfactual work

- **Kapteyn, Knezevic, Willcox**, *Toward predictive digital twins via component-based ROMs and interpretable
  ML*, AIAA SciTech 2020-0418 / arXiv:2004.11356 — the canonical aerospace **predictive digital twin**:
  component-ROM library + interpretable classifier ingests onboard sensor data, infers state, replans a safe
  mission. Closest published analog to the envelope — but **structural health, not plasma + nav**.
- *Adaptive planning for risk-aware predictive digital twins*, arXiv:2407.20490 (2024); *Uncertainty-Aware
  Digital Twins (robust MPC, deep quantile)*, arXiv:2501.10337 (2025).
- Counterfactual/what-if trajectory reasoning exists only in autonomous-driving/world-models (CICR,
  PMC12653532; CounterScene, arXiv:2603.21104) — **no entry-specific counterfactual trajectory framework
  found**, supporting the novelty of `continue_with` reasoning in entry GNC.

### 3D. Relativistic timing reference standard (for the onboard clock)

**Authoritative:** Petit, G. & Luzum, B. (eds.), *IERS Conventions (2010)*, **IERS Technical Note No. 36**,
**Chapter 10** ("General relativistic models for space-time coordinates and equations of motion"). —
iers-conventions.obspm.fr/content/chapter10/tn36_c10.pdf

- **§10.1 Time scales:** TT/TCG/TCB/TDB; `L_G = 6.969290134×10⁻¹⁰`, `L_B = 1.550519768×10⁻⁸`, `TT = TAI +
  32.184 s`.
- **§10.2 proper↔coordinate time near Earth, Eqs. 10.6–10.9** — the section to cite for an onboard clock.
  Eq. 10.7: **`dτ_A/dt = 1 − (1/c²)[v_A²/2 + U_E(x_A)]`** — *identical to the FS-3 kernel*
  `dτ/dt = 1 + Φ/c² − v²/2c²` with Φ ≡ −U_E. GPS-altitude combined shift ≈ 4.5×10⁻¹⁰ (constant part
  −4.4647×10⁻¹⁰ hardware pre-offset), periodic terms ≤ 10⁻¹¹.

**Canonical numbers:** **Ashby, N., *Relativity in the Global Positioning System*, Living Reviews in
Relativity 6:1 (2003)**, doi 10.12942/lrr-2003-1 — gravitational **+45.7 µs/day**, velocity **−7.1 µs/day**,
net **+38 µs/day = 38,000 ns/day**; uncorrected positioning error ~**10–11 km/day**; required clock
knowledge ~20–30 ns (NIST/Ashby IAU-261; Pogge, Ohio State).

**Reentry clock + the right anchor — confirmed.** A reentry vehicle's rate is **velocity-dominated** and far
smaller per day than GPS (deep in the well, minutes not days): single-digit ns accumulated over a ~5–10 min
entry. **There is no reentry flight-clock dataset.** So the validation strategy is exactly FS-3's: validate
against the **analytic GPS split (Ashby, a measured operational standard)** + the **IERS TN36 §10.2 Eqs.
10.6–10.9** re-evaluated at the entry trajectory's (Φ, v). The GPS numbers verify the implementation at
orbit; the IERS equations are the unit-checked physics re-evaluated at entry. The right and only available
anchor.

---

## Consolidated source list

**Reference data (Thread 1).**
- Grantham, NASA TN D-6062 (1970) — ntrs.nasa.gov/citations/19710004000
- Jones & Cross, NASA TN D-6617 (1972) — ntrs.nasa.gov/citations/19710011633
- Parent et al., arXiv:2111.09432 (2021); Rodriguez-Fuentes & Parent, Phys. Fluids 37:013609 (2025,
  arXiv:2410.12760), 38:036114 (2026, arXiv:2512.18163)
- Doihara & Nishida, *Shock Waves* 11(5):331 (2002), doi 10.1007/s001930200119 (OREX)
- DOT/Volpe RF-blackout review — rosap.ntl.bts.gov/view/dot/12493 ; JPL IPN PR 42-150C
- DSMC ionization, *Adv. Aerodynamics* 2:30 (2020), doi 10.1186/s42774-020-00030-1

**Navigation (Thread 2).**
- Rhea Space Activity AutoNav on Varda W-6 (2026) — rheaspaceactivity.com ; Varda W-6 — varda.com/mission/w-6
- Draper sliced-lens celestial nav — insidegnss.com/the-stars-return-draper-patents-celestial-navigation-system/
- AIAA 2023-3097 (GNSS to reconstruct plasma sheath) — arc.aiaa.org/doi/10.2514/6.2023-3097
- Inside GNSS, *Inertial Error Propagation* (gyro t³/accel t², Schuler 84 min) —
  insidegnss.com/inertial-error-propagation-understanding-inertial-behavior/
- IMU grades — skymems.com/what-are-the-4-grades-of-imu/ ; advancednavigation.com IMU intro
- ESKF/CKF GNSS-denied INS — MDPI Micromachines 16(10):1116 (2025); MDPI Machines 14(2):217 (2026)
- Tight coupling / clock state / ambiguity recovery — PMC7014498 ; sbg-systems.com/glossary/tight-coupling/
- Navy SBIR N242-075 (alt-nav for hypersonic, GPS-denied, 2024); Nature Sci. Rep. (2022) SINS/BDS hypersonic
- Plasma/MHD — AIP Advances 7:105314, 7:025114 (2017); arXiv:1407.6635 (2014)

**Unified sim + timing (Thread 3).**
- Hash et al., AIAA 2007-605 (DPLR/LAURA/US3D, FIRE II)
- CEAS Space J. (2024), coupled trajectory aerothermal-load (ATDB) — doi 10.1007/s12567-024-00588-2
- Gray et al., OpenMDAO, *Struct. Multidiscip. Optim.* 59 (2019) ; Dymos — openmdao.github.io/dymos/
- J. Spacecraft & Rockets, decoupled 1-D shock blackout — doi 10.2514/1.A33122
- *Chinese J. Aeronautics* (2021), review of hypersonic guidance-control — S1000936121004167
- Kapteyn/Knezevic/Willcox, arXiv:2004.11356 (2020); arXiv:2407.20490 (2024); arXiv:2501.10337 (2025)
- IERS Conventions (2010) TN36 Ch.10 — iers-conventions.obspm.fr/content/chapter10/tn36_c10.pdf
- Ashby, *Living Reviews in Relativity* 6:1 (2003), doi 10.12942/lrr-2003-1

---

## Demonstrated (working example, 2026-06-30)

The research above answered the hard question — *how do you get reentry navigation data?* — with: **the
unified simulation generates it** (synthetic sensors on the sim's own ground truth, physics-gated GNSS).
That reframe is now a **running artifact**, on **real data**:

**`examples/avionics_examples/ins_gnss_blackout`** — the navigation/timing **core** of the blackout problem,
isolated and run on the **real Galileo E14** SP3 orbit + `.clk` clock products. It composes the three native
mechanisms this dossier and the resolution call for, in one auditable `CausalFlow`:

- **grmhd `select_metric` regime detector** → a plasma-intensity indicator (n_e proxy, Thread 1's RAM-C
  band) vs a critical threshold flips **GNSS available ↔ denied** — the two regime changes.
- **`intervene` / `branch_with`** → the GNSS fix disciplines the INS error when available and is **withheld**
  through the blackout (the chain runs open-loop in the dark, then snaps back) — the corrective-control idiom.
- **`relativistic_clock_drift_rate_kernel`** (shipped, FS-3; Thread 3's IERS/Ashby physics) → predicted from
  the **real orbit geometry** and **carried** across the outage; it beats a naive last-rate hold against the
  **real measured E14 clock** (≈3532 ns vs 3663 ns).

Measured: open-loop pure INS drifts **~375 km** over the day; the closed loop stays **bounded** and reacquires;
**two** regime changes + the interventions are recorded in the `EffectLog`. Five self-verifying gates pass.

**New reusable infrastructure:** the real-data ingestion (Thread-1 datasets) was factored into a dedicated
crate, **`deep_causality_file`** — RINEX SP3/CLK loaders expressed over the **haft IO monad** (lazy
`IoAction`s run at the edge), with an opaque `DataLoadingError` (public type, private evolvable enum) ready
for future CSV/parquet. The chronometric `gm_recovery` example now also rides it (Earth's mass still recovered
from the real clocks to 0.2%). This is the ingestion path the CFD crate's reference datasets (RAM-C) can reuse.

**What this validates:** decisions ⑥ (filter/sensor model — synthetic sensor-in-the-loop + the regime-gated
corrective filter) and ⑧ (the one-envelope scope) are no longer just decided but **demonstrated**, and the
FS-3 clock kernel is exercised by a real-data consumer. The clock-holdover thesis — *physics-gated GNSS
denial + carried relativistic clock + corrective reacquisition* — is the **core of the blackout problem**, and
it runs **without plasma CFD**. The full 2T conformal trajectory lift + aero coupling (resolutions 1/3) remain
the open trajectory research.

---

## Related

- [`gap-three-resolution-3-trajectory-axis.md`](gap-three-resolution-3-trajectory-axis.md) — the decisions
  (Part F) this dossier backs.
- `examples/avionics_examples/ins_gnss_blackout/` — the demonstrator above (real Galileo data).
- `deep_causality_file/` — the reusable RINEX/GNSS ingestion crate (haft IO monad) it rides.
- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) §2 (where relativity bites), §8
  (validation anchors).
- `deep_causality_cfd/verification/` — the already-gated physics (`qtt_park2t_blackout`, `qtt_ramc_stagline`,
  `qtt_sod`).
- `deep_causality_cfd/.../dec_cylinder_wake_verification` — the existing sensor-fed-uncertain-inflow +
  dropout/`EffectLog` substrate the unified envelope reuses.
</content>
