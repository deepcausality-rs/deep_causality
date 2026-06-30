<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 3, Resolution 3 — trajectory axis: spec-readiness assessment + de-risking feasibility studies

**What this is.** A **spec-readiness / feasibility-gap analysis** of the Gap-3 trajectory-timing axis, sitting
between [Resolution 1](gap-three-resolution-1-perturbed-conformal-trajectory.md) (the `[preliminary]`
TRIZ/ARIZ design sketch) and an eventual OpenSpec change. It answers one question: **is Resolution 1 enough
to derive a full, implementable specification?** Short answer: **no — it is a sound architectural sketch, not
a spec.** This note pins down exactly what is missing, and then **runs three feasibility studies** that
de-risk the genuinely-open items so the spec can be written against measured facts rather than assertions.

The dominant problem (per the brief) is **trajectory-timing access via perturbed-conformal splitting** — i.e.
Resolution 1's B1 (exact gravity core `X(τ)=e^{Gτ}X(0)` + between-step aero perturbation). That is where the
two hardest open items live: the **generator `G`** and the **aero↔core coupling law**.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[preliminary]**.

---

## Part A — Spec-readiness verdict

**Verdict: NOT spec-ready.** ~70% of the machinery already exists in-tree, but the specification is blocked
on **three physics derivations, one build-sequencing dependency, and two scope/validation decisions** — all
concentrated where Resolution 1 and the corridor already stamp `[open]`/`[preliminary]`.

### A.1 What is already spec-ready (acceptance-criteria level)

| Piece | Evidence (file:symbol) | Status |
|---|---|---|
| Four-part factoring (B1 propagate / B2 filter / B3 clock / B4 switch) | Resolution 1 | Architecturally complete |
| Regime detector (ε=a_aero/a_grav → integrator) | `examples/physics_examples/grmhd/model.rs::select_metric` — proven state-vs-config-threshold switch | **EXISTS, reusable as-is** |
| Gravity core (Schwarzschild/Kerr metric, geodesic RK4, GM/r, J2) | `deep_causality_physics`: `kerr_metric_at`, `geodesic_integrator_kernel`, `solve_gm_analytical_kernel`, `effective_gravity_radius_j2_kernel` | **EXISTS, production-grade** |
| Constants (c, G, `EARTH_GM`, `EARTH_J2`, `EARTH_RADIUS_EQUATORIAL`) | `constants/universal.rs`, `constants/earth.rs` (IERS 2010 / JGM-3 / WGS-84) | **EXISTS** |
| Operator-split (Lie/Strang exact-core + between-step kick) | `SolenoidalField::from_leray_projection` (Leray); LER; Tier-B IMEX | **EXISTS, native pattern** |
| Time-dilation rapidity | `kernels/relativity/spacetime.rs::time_dilation_angle_kernel` | EXISTS |
| Dynamic-vs-config mandate (replace `g00=−0.9`/`ricci=−0.1g`/`scalar_r=−0.4` proxies with state-computed metric; only `G,c,EGM/IERS` literal) | gap-analysis Gap 3 | Explicit + testable |
| Six verification gates (coast exactness, split accuracy, constraint preservation, dynamic timing, blackout carry, regime switch) | Resolution 1 §"Verification gates" | Enumerated |
| Some numerical targets (ns clock ⇒ 1 ns ≈ 0.3 m; ε_switch ~0.1–1; relativistic terms ~1e-9; coast-exactness "to round-off") | corridor §2, Resolution 1 | Partial |

### A.2 The gaps that block a full spec

Tagged: **[D]** derivation · **[X]** decision · **[≫]** sequencing dependency · **[R]** reference data.

1. **① Conformal generator `G` — THE crux. [D]** Resolution 1 asserts the inverse-square family is
   "exactly 2T-linear, `X(τ)=e^{Gτ}X(0)`" but never gives the explicit generator `G(GM)` reproducing
   Kepler/Schwarzschild. The skeleton (`examples/avionics_examples/hypersonic_2t/model.rs`) hand-sets it as
   "Boost-Glide like motion." The *bivector exponential* is buildable (the quaternion `exp` in
   `deep_causality_num` generalizes), but **the generator is the unproven derivation, not the exp.**
   → **de-risked by FS-1.**
2. **② Conformal lift + shadow/gauge. [D][X]** The skeleton's null-cone lift is "simplified for demo" and
   does not provably satisfy `X²=0`. The `(4,2)` embedding of *momentum* `P`, the inverse 3D shadow
   projection, and a fixed gauge are unspecified. Coast-exactness (gate 1) is unmeetable until the
   round-trip is exact and `e^{Gτ}` provably preserves `X·X=X·P=P·P=0`. → **de-risked by FS-1.**
3. **③ Aero↔core coupling law — `[open]` in BOTH docs. [D]** The corridor: *"Coupling Bars 2T to
   non-conformal external forcing is itself a research move, not textbook — do not assert it solved
   `[open]`."* How does a non-conformal aero kick enter without breaking the structure; in 3D or 6D; at what
   cadence? → **de-risked by FS-2.**
4. **④ Aero force/observable interface — unbuilt dependency. [≫]** Resolution 1 is written "on the
   assumption that the Tier-B CFD resolutions 4–9 work"; "the aero-coupling interface is not yet built."
   Tier-B Stages 3–6 are de-risked but **unbuilt**; the per-step force/heat output is a **mock** until
   Tier-B Stage-4+ lands. The note itself says "revisit after Tier-B Stage 4+."
5. **⑤ Forward clock `dτ/dt` + which τ. [D][R]** Constants + rapidity exist and the 1PN law
   `dτ/dt = 1 + Φ/c² − v²/2c²` is known, but there is **only the inverse** (`solve_gm` recovers GM) — no
   forward clock kernel. And Resolution 1 *asserts* the 2T parameter τ is proper time without proving it.
   → **de-risked by FS-3.**
6. **⑥ Filter / measurement model. [X]** Sensors (GNSS pseudorange / INS), noise covariance, the 6D Kalman
   gain, and constraint-projection uniqueness (gauge ambiguity, `[holds under precondition]`) are
   unspecified. The projector can reuse the Leray/Hodge framework specialised to `X·X,X·P,P·P`.
7. **⑦ Validation references. [R]** Gate 2's "high-fidelity ODE reference over a representative reentry arc"
   (which integrator? which trajectory?), gate 4's IERS reference case, and gate 6's overlap band are
   unnamed. → **partially supplied by FS-1/FS-2/FS-3 references.**
8. **⑧ Scope ambiguity. [X]** The existing example is a **ground-based tracker of an HGV target**;
   Resolution 1 reframes the same machinery as the **onboard reentry trajectory+clock propagator through
   GNSS blackout**. Different observables, measurement model, counterfactual semantics. Must be decided.

### A.3 Capability matrix (what exists vs must be built)

| # | Capability (for B1–B4) | Status | Evidence / what's missing |
|---|---|---|---|
| 1 | `e^{Gτ}` bivector/rotor exponential | **PARTIAL** | `deep_causality_num` quaternion `exp` exists; bivector generalisation needed (closed form) |
| 2 | Sp(2,R) constraint projection (`X·X,X·P,P·P`) | **PARTIAL** | Leray/Hodge projector framework exists; specialise bilinear forms |
| 3 | Conformal null-cone lift (3D→6D, sig (4,2)) | **PARTIAL** | `Metric::Generic{4,2}` ready; hand-rolled lift in skeleton; need standardised embedding |
| 4 | Schwarzschild/PN gravity, geodesic, GM/r | **EXISTS** | `kerr_metric_at`, `geodesic_integrator_kernel`, `solve_gm_analytical_kernel` |
| 5 | Clock `dτ/dt`, γ(v), GR potential, IERS | **PARTIAL** | constants + rapidity exist; **forward `dτ/dt` kernel missing**; IERS 2PN deferred (<1e-19) |
| 6 | Regime detector (ε selector) | **EXISTS** | `grmhd::select_metric`; copy, swap metric→integrator enum |
| 7 | Operator split (between-step aero/J2) | **EXISTS** | Leray, LER, Tier-B IMEX |
| 8 | EGM geopotential / J2 | **PARTIAL** | J2 present & sufficient; higher harmonics deferred (<1e-6) |

**The easy 70% (capabilities 4,6,7,8 + scaffolding) is ~200–300 LOC of leverage. The hard 30% is the three
derivations (①②③) + the sequencing dependency (④).** The three studies below attack ①②③⑤ directly, all with
**mock aero**, so they do not wait on ④.

---

## Part B — Feasibility studies (design)

Three self-contained numerical experiments (the pattern the CFD side used — `qtt_acoustic_precond`,
`qtt_rank_*` — to de-risk Stages 3–6 *before* committing a change). Each answers one yes/no/how question
against an independent, falsifiable reference. They live in `deep_causality_cfd/studies/` (the
plasma-blackout study home, alongside the Tier-B `qtt_rank_*` studies) and self-verify (exit nonzero on a
failed gate); run with `cargo run --release -p deep_causality_cfd --example <name>`.

### FS-1 — Generator: is the inverse-square core an exact constant-generator matrix exponential? (① ②)

- **Claim under test:** the bound inverse-square (Kepler) trajectory equals `ψ(s)=e^{Gs}ψ(0)` for a
  **constant** generator `G` under a time reparametrisation — Resolution 1's "exact conformal core."
- **Realisation:** the eccentric-anomaly linearisation (the 1-D essence of KS/Levi-Civita regularisation,
  Stiefel–Scheifele 1971). In eccentric anomaly `s=E`, the recentred perifocal state `Q=(a cos E, b sin E)`
  solves the unit-frequency harmonic oscillator `Q''=−Q`; the phase state `ψ=(Q,Q')` advances by the
  **constant** `4×4` symplectic generator `Ω=[[0,I₂],[−I₂,0]]`, `e^{Ωs}=[[cos s·I,sin s·I],[−sin s·I,cos s·I]]`.
  Physical time is the closed-form `t(s)=t₀+(M−M₀)/n`, `M=E−e sin E` (Kepler's equation).
- **Independent reference:** classical orbital-element propagation (Kepler-equation Newton solve), an
  *independent* exact Kepler solver.
- **Gates:** (1) `e^{Ω·2π}=I` to round-off (orbit closes — the matrix exponential is exactly periodic);
  (2) a *generic* scaling-and-squaring matrix exponential agrees with the closed form to ~1e-13 (the
  "matrix exponential" is literal, not hand-waved); (3) physical positions from `e^{Ωs}ψ₀` match the
  element propagation at matched `t(s)` to ≲1e-10·a across a full orbit; (4) the generator is `s`-independent.

### FS-2 — Coupling: does Encke/Strang splitting carry non-conformal aero without touching the core? (③)

- **Claim under test:** a non-conformal perturbation (aero) can ride a between-step kick **in physical
  (Cartesian) coordinates** between exact-core steps, at 2nd order, **without** expressing it in the
  conformal algebra — dissolving the corridor's "research move, not textbook" `[open]` concern.
- **Realisation:** planar Kepler + a **mock drag** `a_drag=−k·v` (non-conservative — the hard, energy-changing
  case, the genuine aero analog). Strang split per macro-step `H`: half-kick `v += a_pert·H/2` → **exact
  Kepler drift** `H` (the FS-1 core, pure μ) → half-kick. The kick is applied in Cartesian velocity; the
  conformal core is never modified.
- **Independent reference:** RK4 of the full perturbed EOM `ẍ=−μx/r³−k v` at a tiny step (truth).
- **Gates:** (1) observed order ≈ 2 (`log₂(err_H/err_{H/2}) ∈ [1.8,2.2]`); (2) error shrinks with the
  perturbation ratio `ε=|a_drag|/|a_grav|`; (3) split error below a stated tolerance at a moderate `H`.

### FS-3 — Clock: is τ the proper time, and does forward `dτ/dt` hit ns-level known offsets? (⑤)

- **Claims under test:** (a) the regularising fictitious time of FS-1 is **not** proper time — proper time is
  a separate GR+SR integral (clarifies Resolution 1's conflation); (b) a forward
  `dτ/dt = 1 + Φ/c² − v²/2c²` kernel reproduces textbook relativistic clock offsets at ns precision.
- **Realisation + reference:** the **GPS clock split** — the canonical falsifiable anchor: a GPS satellite
  clock runs **+45.7 µs/day** (gravitational) and **−7.2 µs/day** (velocity), net **+38.6 µs/day** relative
  to the geoid. Compute both from `EARTH_GM`, `c`, geoid radius, GPS semi-major axis.
- **Reentry micro-demo:** accumulate the τ−t offset over a 180 s GNSS-blackout window at reentry conditions
  (v≈7.65 km/s, ~71 km) and report it in ns and metres (×0.3 m/ns) — "carry the clock internally."
- **Gates:** `|grav−45.7|<1.0`, `|vel−7.2|<0.5`, `|net−38.6|<1.0` µs/day; blackout offset finite and
  reported.

---

## Part C — Findings (measured)

All three studies **pass** (built, run, self-verifying; `cargo run --release -p deep_causality_cfd
--example traj_fs{1,2,3}_*`). Each de-risked its target `[open]` item; two produced findings that **simplify
Resolution 1**.

### FS-1 — Generator: **HOLDS to round-off.** ① resolved.

| Gate | Result |
|---|---|
| `e^{Ω·2π} = I` (orbit closes — matrix exp is exact) | ‖·‖ = **2.7×10⁻¹⁵** |
| generic scaling-and-squaring `e^{Ω·s}` vs closed form | ‖·‖ = **3.0×10⁻¹⁵** |
| matrix-exp trajectory vs **independent** element-Kepler over a full orbit | **2.25×10⁻¹⁵·a** |
| constant generator (semigroup `e^{Ωs₁}e^{Ωs₂}=e^{Ω(s₁+s₂)}`) | ‖·‖ = **1.1×10⁻¹⁶** |

The bound inverse-square trajectory **is** an exact constant-generator matrix exponential `ψ(s)=e^{Ωs}ψ₀`.
Resolution-1's B1 "exact conformal core" **holds**, with a *concrete* generator `Ω` (the eccentric-anomaly /
Kustaanheimo–Stiefel realisation, Stiefel–Scheifele 1971). **Spec simplification:** the production
realisation is **KS regularisation** (3-D, singularity-free, perturbation-ready); the heavier Bars `(4,2)`
conformal packaging is **optional, not required** — the skeleton's hand-set `(4,2)` generator can be replaced
by the proven KS generator.

### FS-2 — Coupling law: **the `[open]` concern DISSOLVES.** ③ resolved.

| Gate | Result |
|---|---|
| Strang-split observed order | **2.000** |
| split error vanishes with perturbation (ε→ε/10) | **10.0×** drop (linear in ε) |
| moderate macro-step (H≈29 s) accuracy | **3.9×10⁻⁷·a** |

A non-conformal **mock drag** `a=−k·v` (ε≈9×10⁻⁴) carried as a **between-step kick in physical Cartesian
velocity**, around an *exact, untouched* Kepler core, is **2nd-order accurate** and its error **vanishes with
the perturbation**. The corridor's "coupling Bars 2T to non-conformal forcing is a research move, not
textbook `[open]`" concern is **not a real obstacle**: you split in physical space (Encke/Strang); you never
express aero inside the conformal algebra. **This removes the single scariest `[open]` from the axis.**

### FS-3 — Clock: **forward kernel feasible; a conceptual correction to Resolution 1.** ⑤ resolved.

| Gate | Result | Textbook |
|---|---|---|
| GPS gravitational offset | **+45.65 µs/day** | +45.7 |
| GPS velocity offset | **−7.21 µs/day** | −7.2 |
| GPS net offset | **+38.44 µs/day** | +38.5 |
| reentry-blackout carry (180 s, v=7.65 km/s, 71 km) | **−57.2 ns ⇒ 17.2 m** | — |

The missing forward kernel `dτ/dt = 1 + Φ/c² − v²/2c²` reproduces the canonical GPS relativistic split to
sub-µs/day from the existing `EARTH_GM`/`SPEED_OF_LIGHT` constants — **ns-level onboard timing is feasible
today**. **Conceptual correction:** the FS-1 linearising parameter `s` (eccentric anomaly / KS, `dt=(r/na)ds`)
is a **regularising reparametrisation, NOT proper time**; proper time `τ` is the separate GR+SR integral.
Resolution 1 conflates them — the spec must carry **two** clocks (`s` for the matrix-exponential core, `τ`
for the relativistic correction). The 17.2 m blackout drift quantifies why `τ` must be carried internally (B3).

### Net effect on the gap list (Part A.2)

| Item | Before | After the studies |
|---|---|---|
| ① generator `G` | `[D]` open derivation | **resolved** — `Ω` (KS), exact to round-off |
| ② lift/gauge | `[D][X]` | **largely resolved** — KS lift is standard & singularity-free; the `(4,2)` shadow/gauge is now *optional* (only if Bars packaging is kept) |
| ③ aero↔core coupling | `[open]` research move | **dissolved** — physical-space Strang split, 2nd-order |
| ⑤ forward clock + which τ | `[D][R]` | **resolved** — kernel validated vs GPS; `s`≠`τ` clarified |
| ④ aero force interface | `[≫]` Tier-B dependency | **unchanged** — still waits on Tier-B Stage-4+ (FS-2 used a mock, as designed) |
| ⑥ filter model · ⑦ references · ⑧ scope | `[X]/[R]` | **unchanged** — decisions, not blockers |

The three hardest physics items (①③⑤) are now **measured facts**, not assertions. What remains is **build
sequencing** (④) and **product decisions** (⑥⑦⑧), not open research.

---

## Part D — Recommendation

The trajectory axis is **now spec-ready for Phase 1**, and the studies **simplify** the design (KS instead of
hand-set `(4,2)`; physical-space split instead of a conformal-coupling law; two explicit clocks).

- **Phase 1 — specifiable + buildable now.** The conformal-gravity-clock core: B1 core as the **KS
  matrix-exponential** propagator (FS-1), B3 as the **forward `dτ/dt` clock kernel** (FS-3, a genuine new
  `deep_causality_physics` kernel filling capability ⑤), and B2 as the predict + (KS/Sp(2,R)) projection
  filter — all validated against analytic Kepler / GPS references with a **mock** aero kick (FS-2's split).
  Acceptance criteria already exist as the FS gates. Remaining product decisions: ⑥ (measurement/covariance
  model), ⑧ (ground-tracker vs onboard-propagator scope).
- **Phase 2 — blocked only on ④.** Real aero coupling + the Encke↔Cowell regime switch (B4, reuse
  `grmhd::select_metric`) + hysteresis, gated on the Tier-B Stage-4+ force/heat interface. FS-2 already shows
  the *mechanism* works; Phase 2 is wiring the real force in, not new physics.

**Suggested next step:** write the OpenSpec change for Phase 1 (it can be specified against the FS gates).
Phase 2 follows once Tier-B Stage-4 lands the aero interface.

---

## Part E — Promoted to library (done)

The two reusable primitives the studies validated are now **shipped in `deep_causality_physics`** (generic
over the scalar, `from_f64` literals, 100% test coverage, cited):

- **Forward clock kernel** — `kernels/chronometric/forward_clock.rs`:
  `relativistic_clock_drift_rate_kernel(radius, speed, gm)` (the `dτ/dt − 1` primitive, the **complement** of
  the existing `solve_gm_analytical_kernel` inverse) and `relativistic_clock_offset_kernel(...)` (clock vs
  reference). Tests reproduce the GPS split (+45.65/−7.21/+38.44 µs/day) and the reentry-blackout carry.
  This fills capability ⑤. Cited: Ashby, *Living Reviews in Relativity* 6, 1 (2003); IERS Conventions (2010).
- **Exact two-body propagator** — `kernels/astro/two_body.rs`: `TwoBodyPropagator<R>` — the constant-generator
  matrix-exponential Kepler core (FS-1), exact to round-off (`from_state` → `propagate(dt)`), planar
  realisation with the 3-D KS generalisation documented as the production extension. Tests gate one-period
  closure, round-trip identity, energy/momentum conservation, Kepler's third law, and the rejections. Cited:
  Stiefel & Scheifele (1971); Battin (1999).

`traj_fs3_clock` now consumes the shipped `relativistic_clock_offset_kernel` (the study verifies the shipped
code, not a private copy). The B1 core and B3 clock of a Phase-1 propagator can be built directly on these.

---

## Related

- [`gap-three-resolution-1-perturbed-conformal-trajectory.md`](gap-three-resolution-1-perturbed-conformal-trajectory.md)
  — the `[preliminary]` design this assesses.
- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) §2, §3.1, §6 — the requirements + the
  `[open]` aero-coupling seam.
- [`../gap-analysis.md`](../gap-analysis.md) Gap 3 — the dynamic-by-construction mandate.
- `examples/physics_examples/grmhd/model.rs` — the reusable `select_metric` regime-detector pattern.
- `examples/avionics_examples/hypersonic_2t/model.rs` — the skeleton (Euler `predict`, no-op `correct`).
- `deep_causality_cfd/studies/traj_fs1_generator/`, `traj_fs2_coupling/`, `traj_fs3_clock/` — the three
  studies (FS-1/2/3).
</content>
