<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The Plasma Blackout Corridor — a flagship EPP demonstrator

**What this is.** A specification for the single example that exercises the EPP's whole thesis on the
hardest real problem there is: **autonomous atmospheric reentry through plasma blackout.** It combines
**quantum-inspired tensor-network compression**, **counterfactual reasoning**, **multiphysics**, and
**regime change** in one auditable causal process — and it does so honestly, with the relativity placed
where it actually bites and the open research seams named, not papered over.

Honesty convention (as elsewhere): **[holds]**, **[holds under precondition]**, **[open]**,
**[speculative]**.

---

## 1. The hero narrative

During the 3–4 minute comms-and-GNSS **blackout** when a returning vehicle is utterly on its own, it
**reasons over counterfactual future flowfields** to choose a bank-angle profile that stays inside the
**thermal corridor** *and* hits the landing ellipse — **switching its governing physics** as it crosses
the continuum→plasma boundary, **keeping its clock honest with relativity** while GNSS is gone, and
**recording exactly what evidence each decision rested on**. All by one bind law. Fully auditable after
splashdown.

**Hero framing (non-weaponized):** a crewed capsule / Starship-class reentry — a real, funded industry
(SpaceX, Sierra Space, NASA Orion/Artemis), and plasma blackout is iconic (Apollo). **Dual-use sibling:**
the terminal phase of a hypersonic glide vehicle is the *same physics*; the corridor becomes a
thermal-survival + accuracy + rules-of-engagement problem.

The point is not "EPP does reentry CFD." It is that **no tool today couples relativistic-trajectory
acceleration, nonequilibrium plasma regime change, and compressed counterfactual aerothermo in one
auditable causal process** — and the EPP's composition law does exactly that.

---

## 2. Where relativity *actually* bites (and how it is solved today)

A discipline-honesty section, because the credibility of the whole example rests on getting this right.

**For the airflow — the CFD — General Relativity does not bite, and special relativity barely does.** At
Mach 20, v/c ≈ 2×10⁻⁵, so Lorentz corrections to the fluid are ~10⁻¹⁰ against shock chemistry and
turbulence. Gravitational curvature of the *flowfield* is nil. "GR-coupled hypersonic CFD" for the
aerodynamics is wrong; do not claim it.

Relativity bites in a **different stovepipe — Position-Navigation-Timing (PNT) and guidance:**

- **Clocks (SR + GR).** Onboard time dilation (velocity) and gravitational-potential shift (altitude)
  perturb the navigation clock at the nanosecond level over a flight; 1 ns ≈ 0.3 m of ranging error. Same
  physics that forces GPS's ~38 µs/day net correction (+45 gravitational, −7 velocity).
- **Gravity model (mostly Newtonian + relativistic margin).** Trajectory EOM use a spherical-harmonic
  geopotential (EGM2008; J2 oblateness ~10⁻³ of the monopole), Earth rotation, and — at the highest
  fidelity — **post-Newtonian (Schwarzschild, frame-dragging) terms** from the IERS conventions. The
  dominant effect is high-order *Newtonian* geopotential; true GR EOM terms sit at the sub-meter margin.
- **GNSS-denial during blackout.** When the ionized shock layer blacks out GNSS, the vehicle falls back
  to **INS-only** dead reckoning, and the relativistic clock correction must be carried *internally* with
  no satellite fix to reset it.

**How it is solved today: decoupled stovepipes + bias corrections + a fidelity split.** Aerothermodynamics
(CFD + chemistry/ionization + ablation/TPS + radiation) is run *offline* to build aero/heating tables.
GNC is a *separate* silo (trajectory EOM + gravity model + strapdown INS + GNSS/celestial aiding +
Kalman filtering); relativity enters as **clock-bias corrections**, not as a physics regime. The two
silos exchange **tables**, not live coupled state. Regime transitions (laminar→turbulent,
continuum→rarefied, perfect-gas→reacting→**plasma**, GNSS→INS-only) are stitched by **bespoke glue**,
vehicle by vehicle.

**The EPP opening is the coupling and the regime transitions, not any single physics.** EPP unifies the
stovepipes — aerothermo ↔ plasma ↔ relativistic timing ↔ regime switching ↔ safety — under one
composition law with provenance, exactly as the `grmhd/` coupling layer already does for curvature↔metric.

---

## 3. The three-physics stack (and the "2T" collision, resolved)

Two unrelated physics share the abbreviation **2T**. Separated, they give the demonstrator two
complementary accelerators on its two expensive axes, plus the physics that drives the regime change.

### 3.1 Two-Time physics (Itzhak Bars) — the trajectory/gravity/relativity accelerator

Already prototyped in `examples/avionics_examples/hypersonic_2t/` (Sp(2,R), (4,2) conformal signature;
`CausalMultiVector`; linear propagation as one `Euler`/`EndoArrow` step). Lift to 6D, where the
**inverse-square family** — Kepler/Coulomb, and by extension the **geopotential monopole plus its
post-Newtonian (Schwarzschild) perturbation** — becomes *linear free motion* `X(τ) = e^{Gτ} X(0)`, a
matrix exponential rather than an ODE solve: zero-lag, branch-cheap. This is the **natural home for the
relativistic trajectory EOM** — precisely where GR bites (§2) — and it makes each counterfactual branch's
*trajectory* propagation near-free. **[holds for the gravitational/relativistic part; see seam §6]**

### 3.2 Two-Temperature physics (Park) — the regime-change driver

The standard hypersonic thermochemical-nonequilibrium model: separate translational–rotational `T_tr`
and vibrational–electron `T_ve`, because at high Mach the vibrational excitation, dissociation, and
**ionization lag the flow**. That lagging **electron density** is what raises the plasma frequency above
the comms band and **blacks out GNSS/comms.** Park 2T governs the regime classifier and the GNSS-denial
trigger (steps [2]/[3]) and makes the regime change *physically real*, not a parametric switch. **[holds
as the governing model; surrogate fit acceptable for Tier A]**

### 3.3 Tensor-network (MPS) compression — the flowfield accelerator

Quantum-inspired matrix-product-state / tensor-train compression of the reacting/ionized flowfield keeps
the per-branch heat-flux + drag + ionization estimate cheap (Gourianov et al., *Nat. Comput. Sci.* 2022,
for the turbulence-compression precedent). This is the lever that serves the CFD *minutes-not-hours*
north-star and makes "many counterfactual rollouts in the blackout window" affordable. **[holds under
precondition: written; classical hardware]**

---

## 4. The causal chain (CausalFlow)

```text
[1] State + context ingest   → altitude, velocity, attitude, TPS temp, GNSS SNR        (context)
        ↓
[2] REGIME CLASSIFIER         → Knudsen (continuum/slip/transitional)                   ◀ REGIME CHANGE
                                + ionization fraction via Park 2T (neutral → plasma)
                                + GNSS available → DENIED
        ↓
[3] COUPLING LAYER (grmhd-style) → select governing models:                            ◀ MULTIPHYSICS
      airflow: continuum NS → slip-corrected → rarefied closure
      gas:     perfect → reacting → ionized (T_tr / T_ve)
      timing:  GNSS-aided → relativistic-corrected INS    (where GR/SR honestly bites)
        ↓
[4] TENSOR-COMPRESSED ROLLOUT → MPS flowfield → heat-flux + drag + electron density     ◀ TENSOR COMPRESSION
      trajectory arc via Two-Time linear propagation (matrix exponential)               (zero-lag dynamics)
        ↓
[5] COUNTERFACTUAL BRANCHES   → continue_with: N bank-angle profiles, each a            ◀ COUNTERFACTUALS
      compressed multiphysics rollout to the landing ellipse, returning
      (peak heat flux, integrated thermal load, miss distance, blackout dwell)
        ↓
[6] EFFECT ETHOS GATE         → reject branches breaching thermal corridor / g-load /   (safety)
                                (crewed) physiological limits / (weapon) ROE;
                                among survivors choose best accuracy
        ↓
[7] PROVENANCE LOG            → per-step: active regime, physics model, relativistic     (audit)
                                clock correction applied under GNSS-denial,
                                evidence each branch rested on
```

The four required elements, mapped:

- **Regime change** → step [2]/[3]: Knudsen + Park-2T ionization + GNSS state select the governing
  models; the coupling layer is the `grmhd/` metric-selection pattern generalized.
- **Multiphysics** → CFD + Park-2T thermochemistry + EM/plasma + flight dynamics + relativistic timing,
  composed by one bind law.
- **Counterfactuals** → step [5]: `continue_with` spawns bank-angle branches; each is a full rollout; the
  vehicle reasons over *what-if flowfields*, not just current state.
- **Tensor-network compression** → step [4]: MPS makes each branch's flowfield affordable.

---

## 5. Compute strategy — two accelerators on two axes

The reason "counterfactual multiphysics rollouts at decision speed" was aspirational is two expensive
axes. The stack covers both, under a physically-grounded regime switch:

| Axis | Cost without help | Accelerator |
|---|---|---|
| **Trajectory / dynamics** (gravity + relativistic EOM, glide arc) | nonlinear ODE integration, per branch | **Two-Time (Bars)** → linear 6D matrix exponential, zero-lag |
| **Flowfield / aerothermo** (heat flux, drag, ionization) | huge field eval, per branch | **Tensor-network (MPS) compression** |
| **Regime / plasma** (blackout onset, GNSS-denial) | thermochemistry | **Two-Temperature (Park)** governs the switch |

Each counterfactual branch is cheap on *both* the dynamics side (2T matrix-mult) and the field side
(MPS), under a physically-real regime change (Park 2T). That is what turns the flagship from research
aspiration into buildable demonstrator: running 5–20 counterfactual corridors inside the ~3-minute
blackout window becomes trivial. This is the real meaning of "get the math up to speed."

---

## 6. Honest seams (named, not hidden)

- **Aero forcing is NOT Bars-2T-linearizable.** Lift/drag during a bank are empirical and nonlinear in
  α/Mach/ρ — outside the Sp(2,R) family. Correct factoring: **2T-exact gravitational/relativistic
  propagation + the aero force as a perturbation** fed from the tensor-compressed flowfield (the 3D
  shadow gets an aero kick each step). Coupling Bars 2T to non-conformal external forcing is itself a
  **research move**, not textbook — phrase it as a contribution, do not assert it solved. **[open]**
- **The existing `hypersonic_2t` example is a proof-of-concept skeleton.** Its conformal embedding is
  explicitly "simplified for demo" (`model.rs:41-48`), the generator is hand-set, and `correct()` is a
  stub (`model.rs:84-86`) — no real 6D measurement update yet. "Zero-lag tracking" is the formalism's
  *aspiration*, shown in toy form. Carrying the flagship needs the real conformal lift + a genuine 6D
  filter update. **[holds under precondition: example hardened]**
- **Park 2T at Tier A is a surrogate fit.** A parametric ionization-fraction model is acceptable for the
  demonstrator; the validated reacting-flow closure is Tier B.
- **EPP is a macroscope, not the inner solve loop.** It composes, gates, and audits; the heavy compute
  lives behind the causaloid boundary. Reentry guidance is latency-bound — state the value as
  orchestration + auditable safety + counterfactual decision, not as the CFD hot kernel.

---

## 7. Feasibility tiers

- **Tier A — buildable demonstrator ("minutes" → really milliseconds for the reduced model).** Quasi-1D/2D
  reduced hypersonic flow; parametric Park-2T thermochemistry + ionization; a *real* Knudsen/ionization
  regime classifier; a *genuine* MPS compression on the model flowfield; Two-Time linear propagation for
  the gravitational/relativistic arc (built on the existing example); 2–3 counterfactual bank branches; a
  real Effect Ethos gate; full provenance. Surrogate physics, but **real composition + real tensor
  compression + real counterfactuals + real regime switch + real relativistic-timing causaloid.** This is
  the deliverable.
- **Tier B — research.** Higher-fidelity coupled CFD / Park-2T / plasma, validated; the 2T-exact-gravity +
  perturbative-aero coupling (§6) worked out properly; a genuine 6D conformal filter.

---

## 8. Validation anchors (verify before any paper use)

- **RAM-C II flight (NASA Langley, 1970)** — the canonical ionized-reentry **electron-density / blackout**
  dataset; the reference target for the Park-2T ionization and blackout-onset timing.
- **Apollo reentry blackout durations** — public; sanity check on blackout dwell.
- **Park two-temperature model** — the standard thermochemical-nonequilibrium reference for `T_tr / T_ve`.
- **IERS conventions** — the relativistic EOM / clock-correction terms used in the timing causaloid.
- **Bars two-time physics (Sp(2,R))** — the conformal-lift formalism the trajectory accelerator rests on.

---

## 9. Why this is the flagship

It makes the EPP's entire thesis *physical* on the hardest real-world problem: **dynamic causality,
regime change, counterfactual reasoning, multiphysics, and a verifiable safety gate, in one auditable
process.** It is the avionics flight-envelope example (Preprint EPP, `introduction.tex` §examples) taken
to its limit. The wow is *causal*: the vehicle reasons over counterfactual future flowfields, switches
its governing physics across the continuum→plasma boundary, keeps its clock honest with relativity while
GNSS is gone, and picks a survivable, accurate maneuver — by one composition law, fully auditable. No
CFD code does counterfactuals; no GNC code does CFD; no quantum tool does either; the EPP does all four
in one bind.

---

## 10. Related

- [`quantum-epp.md`](quantum-epp.md) §9 — the Quantum × CFD value ranking and industry map this example
  instantiates (fusion/aerospace/reentry); the macroscope-not-microscope boundary.
- `examples/avionics_examples/hypersonic_2t/` — the Two-Time (Bars) trajectory accelerator, in
  proof-of-concept form; the foundation for the trajectory axis (§3.1, seam §6).
- `examples/physics_examples/grmhd/` — the regime-adaptive coupling-layer pattern (metric selection by
  curvature) generalized here to airflow/gas/timing model selection.
- Preprint EPP `introduction.tex` §examples — the flight-envelope monitor this demonstrator extends.
- CFD design notes (Flow DSL, `.couple` multi-physics seam) — the MPS compression and the multiphysics
  coupling slot in as Flow sub-processes.
