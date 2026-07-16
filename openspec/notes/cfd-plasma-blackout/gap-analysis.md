<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Plasma Blackout Corridor — gap analysis after the tensor-train layer

**What this is.** A capability-vs-SOTA gap analysis for the [Plasma Blackout Corridor flagship](../plasma-blackout-corridor.md),
written immediately after the tensor-train (MPS/MPO) layer landed in `deep_causality_tensor`. It answers
one question: *with the tensor train added, what stands between us and a buildable example using current
state-of-the-art methods?*

---

## 1. Verdict

The tensor-train addition **closes the tensor-network *primitive* gap** — the MPS flowfield-compression
lever (chain step [4]) is now buildable, and the recently-hardened SVD/QR (overflow-safe Jacobi,
noise-floor rank revealing) plus randomized rounding directly serve the per-step recompression that
QTT/MPS solvers depend on. **It did not, by itself, unblock the example.** *(Update: **Gap 1 is now closed** — the CFD-side encoding,
the immersed-body solver, and the surface observables have since been built and verified across the
`add-cfd-qtt-*` change series; see §4. The remaining gaps are 2–4.)* At the time of writing, the largest
gap was that **nothing in `deep_causality_cfd` touched a tensor network**: the generic primitives lived in
`deep_causality_tensor`; the CFD-side encoding did not exist.

For **Tier A** (the buildable demonstrator with surrogate physics) the remaining work is bounded and no
longer blocked on missing mathematics. For **Tier B** genuine open research remains (validated coupled
reacting-plasma CFD; the Bars-2T-exact-gravity + perturbative-aero coupling).

---

## 2. State of the art (mapped to the four axes)

### Axis 1 — Tensor-network flowfield (step [4])

The field has moved onto the flagship's thesis since the original note was written:

- **MPS Simulation of Reacting Shear Flows** — Pinkston et al., arXiv:2512.13661 (Dec 2025). The direct
  precedent: matrix-product-state / tensor-train for a *reacting* flow, not just turbulence.
  <https://arxiv.org/abs/2512.13661>
- **Tensor networks for turbulence probability distributions** — Gourianov et al., *Sci. Adv.* 11 (2025);
  ~10⁶ memory and ~10³ compute reduction on a 5+1D reactive-flow PDF.
  <https://inspirehep.net/files/0ee2a95339cde99c2435a51ad0c6344a>
- The quantized tensor-train (QTT) incompressible Navier–Stokes lineage (Gourianov; Kiffner–Jaksch) is
  the building-block method: **QTT field + MPO operators + TT-cross for nonlinear terms + TT-rounding
  each step.**

### Axis 2 — Plasma / blackout (steps [2], [3])

- **Numerical simulation of air ionization in the RAM-C-II flight experiment** — *Fluid Dynamics* (Springer,
  2022); Park-2T electron density vs flight data. A modern, citable companion to the RAM-C anchor.
  <https://link.springer.com/article/10.1134/S0015462822100639>
- **Vibrational–electron heating: a thermodynamically consistent model** — arXiv:2506.11457 (2025), and
  **Impact of ion mobility on electron density and temperature in hypersonic flows** — arXiv:2410.12760
  (2024). Current refinements of the `T_ve` / electron-density physics that drives blackout onset.
- **Data-driven lookup-table reduction for hypersonic chemical nonequilibrium** — arXiv:2210.04269. The
  surrogate-table route that Tier A explicitly permits.
- **Review of nonequilibrium plasma kinetics in hypersonic flows** — Aiken, Carter & Boyd, *Plasma Sources
  Sci. Technol.* 34 (2025). The authoritative modern anchor: confirms **Park-2T is the current standard** for
  RAM-C-type electron density; gives the ionization-by-velocity bands (associative `<7` km/s, electron-impact
  `>9` km/s, mixed in between — RAM-C/orbital ~7.6 km/s is *mixed*); documents the electron-density
  **overshoot** (Lin et al. 1962). <https://iopscience.iop.org/article/10.1088/1361-6595/ae2ba2>
- **Numerical prediction of hypersonic flowfields including electron translational nonequilibrium** — Farbar,
  Boyd & Martin, *JTHT* 27 (2013); and **3-T thermochemical nonequilibrium with application to slender-body
  wakes** — Clarey & Greendyke, *JTHT* 33 (2019). A separate electron-energy equation (3T, `T_e ≠ T_ve`) cuts
  peak plasma density **~2×** vs Park's `T_e = T_ve` lumping and matters most in the wake — the named,
  quantified bias of a Tier-A 2-T closure, and an LER-native upgrade (one extra relaxing scalar).

### Axis 2′ — Kinetic tensor-networks vs the continuum/moment closure (read in full)

The same tensor-train layer could later carry a *kinetic* plasma closure (the full `f(x,v,t)` phase space),
not just a continuum/moment closure (Park-2T). The three kinetic-TN papers were read end to end; the verdict
is that **continuum Park-2T is correct for the forebody blackout deliverable, and kinetic-TN is a future
Tier-C reserved for the rarefied legs.** Corroborated four ways.

**The kinetic-TN papers work — but on the wrong regime, with rank unsolved.**

- **Quantized tensor networks for Vlasov–Maxwell** — Ye & Loureiro, arXiv:2311.07756 (J. Plasma Phys. 2024).
  Comb-tree QTT, 2D3V; `D=64` vs a full-rank `2^18`. But rank is held by a **fixed cap**, the fields'
  entanglement entropy grows over time, and the authors state plainly that convergence "must be addressed in
  order to trust results." **Collisionless.**
- **Dynamical tensor-train approximation for kinetic equations** — Wang & Hu, arXiv:2512.14950 (2025).
  Correction to the original note's title: it solves **BGK + Fokker–Planck, not the full Boltzmann `Q(f,f)`.**
  Velocity-TT *per spatial point* to avoid x–v coupling growth; ranks tiny (~5). Key fact: **the Maxwellian is
  TT-rank-1**, so the near-equilibrium (collisional) regime stays low-rank — but **stiff collisions in TT are
  explicitly unsolved** (forced small collision strength, explicit stepping).
- **Tensor-network compression for fully spectral Vlasov–Poisson** — Åsgrim, Pennati, Pasquale & Markidis,
  arXiv:2602.13092 (2026). Correction: it is **Fourier–Fourier, not Fourier–Hermite.** `χ ≈ 15–45` (Landau),
  plateaus at `120–150` (two-stream); negativity artifacts under aggressive truncation. Collisionless, no
  Maxwell coupling.
- Common to all three: **none is run on reacting hypersonic air**, and rank control is the open problem.

**Why the kinetic side points back to continuum.** Wang & Hu's rank-1-Maxwellian result is the bridge: in the
collisional sheath that drives blackout, the distribution is near-rank-1 in velocity — which is the
mathematical statement that **a moment closure is accurate there.** A kinetic solver would pay a 6-D
computation to recover what a few moments already give. And `n_e` (the quantity blackout needs) is the zeroth
moment.

**The continuum/moment side, placed by Knudsen ceiling.** Park-2T is the validated RAM-C standard on the
collisional forebody (Aiken–Carter–Boyd 2025, Axis 2). Above it sits a graded ladder — Grad-13 → **R13**
(Kn ≲ 0.5, Struchtrup–Torrilhon) → **R26** (Kn ~1–3, Gu–Emerson) → **hyperbolic HME** (Cai–Fan–Li, arbitrary
order, globally well-posed) → **HyQMOM** (Fox et al., the moment route built for *non-Maxwellian / multi-stream*
plasma VDFs, realizability-preserving). No moment hierarchy reaches free-molecular (Kn > 10) — that is the hard
wall where kinetics is mandatory — but the forebody sheath is nowhere near it.

**The architecture already embodies the right pattern.** "Tensor-train on a moment *vector*" is structurally
empty (a Park-2T state is too small to compress). But **DLRA on a moment *system* (macro–micro)** is a real,
thin, under-occupied niche: low-rank on the angular `P_N` moments (Peng–McClarren–Frank 2020, arXiv:1912.07522),
DLRA/POD on the hyperbolic shallow-water *moment* equations (Koellermeier–Krah–Kusch 2023, arXiv:2302.01391),
and POD on a Hermite moment hierarchy with a learned closure (Issan et al. 2025, arXiv:2504.09682). Their
pattern — keep the conserved low-order moments exact, carry a compressed correction toward equilibrium — **is
exactly the LER stage.** So the upgrade ladder `LER scalar → per-point BGK-DLR (Wang/Hu) → kinetic` is
non-fighting and rides the existing per-point `PhysicsStage` seam. Extending macro–micro moment reduction into
*reacting multi-temperature plasma* is itself an open niche, not a crowded one.

> **Net.** Park-2T (continuum/moment) is the right tool for the forebody blackout deliverable, corroborated by
> the kinetic side's own rank-1-Maxwellian result, the Kn-ceiling ladder, and the macro–micro framing. The
> kinetic-TN route (full `f(x,v)`) is genuine future Tier-C, valuable for the rarefied / high-altitude / wake
> legs where the near-Maxwellian assumption breaks — *not* the sheath that decides onset. Two named blockers
> gate any kinetic move and Tier-A/B sidestep both: stiff collisions in TT (open, Wang & Hu) and
> convergence/trust (open, Ye & Loureiro).

### Axis 3 — Trajectory / relativity

No single 2024 "GNSS-denied relativistic INS" paper; the relativistic-GNSS modelling is mature
(Schwarzschild-frame GNSS, arXiv:1003.5836) and the IERS clock/EOM terms are textbook — confirming the
§2 framing that this axis is **bias-correction engineering, not open physics**.

---

## 3. What the tensor train provides vs. what the SOTA method needs

The reacting-MPS / QTT-NS method depends on exactly this primitive set — all now present:

| SOTA method needs | In `deep_causality_tensor` now | Status |
|---|---|---|
| QTT / MPS field encoding | `CausalTensorTrain`, `from_dense`, QTT reshape | ✓ |
| MPO spatial operators | `CausalTensorTrainOperator`: `from_cores` (hand-build), `from_dense`, `identity`, `apply`, `compose`, `transpose`, `round` | ✓ |
| MPO operator algebra (assemble stencils) | `add` / `sub` / `neg` / `scale` — **added this session** (completes the algebra) | ✓ (new) |
| Nonlinear convective + chemical source | `hadamard`/`round`, `cross`, `apply_nonlinear` | ✓ |
| Recompression every step | `round` (+ randomized, + NaN-robust SVD/QR) | ✓ (just hardened) |
| Implicit step / linear solve | `solve::linear` (AMEn), `fit` (ALS), `eigen` (DMRG), `tdvp` | ✓ |

**Conclusion:** the primitive layer is **sufficient** for a Tier-A MPS flowfield. The operator algebra
was the one genuine hole found while drilling into Gap 1 — the MPO type had `compose`/`identity` but no
additive structure, so differential stencils like `(S₊ − S₋)/2Δx` could not be assembled. That hole is
now closed (`add`/`sub`/`neg`/`scale`, tested f64/Float106; spec change `add-tensor-operator-algebra`).
The shift operator `S₊` itself is hand-built via `from_cores` and lives in the CFD bridge, not the tensor
crate (see [`gap-one-cfd-tensor-bridge.md`](gap-1/gap-one-cfd-tensor-bridge.md) §3.2).

This table covers the **continuum/moment** flowfield (Tier-A/B). A **kinetic** closure (Axis 2′, Tier-C) would
additionally need primitives this layer does *not* have — a velocity-space discretization, a collision operator
(BGK → Boltzmann) as MPOs, Vlasov streaming, and Maxwell/Poisson coupling — so it is a separate build, not a
closure swap on the present substrate.

---

## 4. Remaining gaps (the actual answer)

### Gap 1 — CFD ↔ tensor-network bridge — **CLOSED**

*(Original state: `deep_causality_cfd` used `CausalTensor` only as a flat `Vec`; zero MPS/MPO usage.)*

**Resolved across the `add-cfd-qtt-*` change series.** `deep_causality_cfd` now has: a QTT codec (1-D/2-D
field ⇄ MPS); finite-difference MPO assembly (hand-built shift operators + the stencil algebra); a
periodic 2-D incompressible Navier–Stokes tensor-train marcher (`QttIncompressible2d`) with spectral Leray
projection and nonlinear convection; an immersed body by Brinkman volume penalization (`QttImmersed2d`, a
smoothed mask MPS — no cut cells); the surface observables the flagship's step [4] reads — **drag/lift** as
the penalization-force tensor-train contraction and a **neutral wall heat flux** via a penalized passive
scalar; and the `CfdFlow::qtt_march` DSL wiring + TT-native diagnostics. Verified: 2nd-order convergence to
the analytic Taylor–Green vortex; no-slip + accuracy-vs-bond convergence on an immersed cylinder. The
headline numerical risks (singular periodic Poisson, nonlinear rank growth, mask rank) were resolved and
verified in code. **[CLOSED — solver core + immersed body + surface observables built and verified]**

The one remaining flagship deliverable that *touches* this bridge — **electron density** and a *reacting*
heat flux — is **Gap 2** physics, not Gap 1; the neutral thermal observable is the seam it plugs into.

### Gap 2 — reacting / ionized physics — **Tier-A CLOSED**

**Tier-A resolved across `add-park2t-blackout-tier-a`.** The Park-2T pointwise kernels now exist in
`deep_causality_physics` (`kernels/hypersonic/`: vibrational relaxation as the closed-form LER step,
Arrhenius rate, Saha / Park-2T ionization surrogate, Rankine–Hugoniot temperature jump, recovery
temperature, plus the plasma-frequency kernel reusing `mhd`), each cited and validated pointwise
(papers in `deep_causality_physics/papers/`). The **Lagging-Equilibrium Relaxation (LER)** coupling —
`RecoveryTemperatureStage` → `IonizationStage` → `EosStage` — and the `BlackoutTrigger` run **inside the
QTT march**: the coupling seam was generalized (`StepContext` backing sum type) and `QttMarchRun` gained a
between-step coupling host (`run_coupled`) that transports the reacting scalars via `advance_scalar` and
emits the `n_e` / plasma-frequency / blackout-dwell observables. The self-verifying
`verification/qtt_park2t_blackout` example gates the six LER criteria (stability-at-stiffness, exponential
exactness, the mandatory RH temperature band, ionization lag + Saha limit, counterfactual path-dependence,
electrons produced) and passes. **[CLOSED — Tier-A reacting/ionization slice built and verified on the
incompressible rollout; `T_tr` is a recovery-temperature reconstruction, disclaimed.]**

### Tier-B — Stages 0–6 built and gated. — **Tier-B CLOSED**

[`add-cfd-compressible-qtt-marcher`](../../changes/add-cfd-compressible-qtt-marcher/proposal.md) (reusing every
Tier-A kernel and LER stage unchanged) is staged 0–6, **all built and gated:** Stage 0 (3-D QTT codec +
operators), Stage 1 (body-fitted coordinate + the **rank-lever gate**), Stage 2 (conservative compressible
Euler + Rusanov, **Sod exact-Riemann gate**), Stage 3 (IMEX split-acoustic with the **closed-form
constant-coefficient inverse** — the D10 ideal realized, no AMEn gamble, exact at all N), Stage 4 (RAM-C
stagnation line, peak `n_e` gate), Stage 5 (2-D body-fitted IMEX marcher + the `BlendedMap` `λ` dial,
**blunt-body rank-lever gate** `qtt_blunt_body_2d`), Stage 6 (3-D IMEX marcher, **forebody rank-lever gate**
`qtt_reentry_3d`). The six ARIZ resolutions
([4](gap-2/gap-two-resolution-4-body-fit-parameter.md)–[9](gap-2/gap-two-resolution-9-moment-closure-turbulence.md))
that de-risked the make-or-break nodes are realized in code. **Named open remainders:** (i) bounding the
*dynamic marched* rank of a flux-through-front (re-pin + exact-RH interface, Res 5 / D9); (ii) a **3-D
body-fitted `MetricProvider`** (the 3-D marcher is Cartesian-capture so far); (iii) `CfdFlow` wiring; with the
**wake/turbulence residual** out of scope (spectral pinning, Res 8; RANS mean `n_e`, Res 9). The
genuinely-irreducible residual stays **instantaneous turbulent fine structure** (never needed for `n_e`) +
**RANS-closure fidelity** (the standard hypersonic-CFD caveat). The RAM-C stagnation line (Stage 4) is the
honest first Tier-B validation point.

**Resolution / mesh strategy (the micrometer shock-sheath requirement).** Reentry needs ~µm resolution at
the shock layer and plasma sheath over a ~m vehicle — a **~10⁶ dynamic range** that forces conventional
CFD into adaptive mesh refinement. The QTT representation answers this *without* AMR: a `2^L` grid costs
`O(χ²·L)`, so a uniform micrometer grid (`L ≈ 20`) is affordable and the cost localizes to the **bond
dimension** where the sharp shock/sheath gradients live (a smooth coordinate stretch adds wall-normal
clustering through a low-rank Jacobian). "Variable mesh" becomes *tensor rank*, not a graded mesh — see
corridor §3.3. **This is now measured** (four rank studies in `deep_causality_cfd/studies/`): the rank driver
is **coordinate alignment, not sharpness or curvature** — a realistically-formed **3-D** curved shock
*captured on a Cartesian grid* measures **`χ ~ √side` (unbounded in resolution)**, while a **shock-aligned /
body-fitted coordinate** holds the same shock at **`χ ~ O(10)` (constant)**. So the coordinate stretch is
**mandatory, not optional**, and **artificial viscosity is not the lever** (it cannot remove curvature, and
over-thickening is diffusion-CFL-unstable → needs an implicit/IMEX step). The **compressible QTT marcher** this
requires is now **built** (`CompressibleMarcher2d` / `CompressibleMarcher3d`, IMEX with the closed-form acoustic
inverse), with the body-fitted rank lever gated in 2-D and 3-D (`qtt_blunt_body_2d`, `qtt_reentry_3d`). The mesh
*strategy* is sound and quantified; the shock *physics* is **Tier-B, Stages 0–6 built** — full analysis in
[`gap-2/tier-b-compressible-marcher.md`](gap-2/tier-b-compressible-marcher.md). **[measured: body-fitted
coordinate + IMEX mandatory; compressible QTT marcher built (Stages 0–6); dynamic marched-rank re-pin + 3-D
body-fit metric are the named open remainders]**

Gap 2's dedicated plan — the **physics-kernel / solver split** (kernels in `deep_causality_physics`, solver +
coupling in `deep_causality_cfd`), the Park-2T / ionization kernel list, and the two composition idioms —
is in [`gap-two-reacting-plasma.md`](gap-2/gap-two-reacting-plasma.md).

### Gap 3 — trajectory axis is a proof-of-concept skeleton (matches corridor seam §6)

`hypersonic_2t/model.rs` has a "simplified for demo" conformal embedding (`data[16] = sqrt(x²+y²+z²)`), a
hand-set generator, and a `correct()` that is a literal no-op stub — no 6D measurement update.
`grmhd/model.rs` uses hardcoded proxy curvature (`g_00 = -0.9`, `g_11 = 1.1`, `ricci = g·-0.1`,
`scalar_r = -0.4`). Both are honest skeletons, not engines. Carrying the flagship needs the real conformal
lift, a genuine 6D filter update, a relativistic-timing causaloid (IERS terms), and the **2T-exact-gravity +
perturbative-aero coupling**, which is correctly named open research — *not* something the tensor train
touches. **[open — preliminary resolution drafted:
[`gap-3/gap-three-resolution-1-perturbed-conformal-trajectory.md`](gap-3/gap-three-resolution-1-perturbed-conformal-trajectory.md)
splits an exact conformal core (2T matrix exponential) from a between-step aero+J2 perturbation, runs the 6D
filter as predict + Sp(2,R) constraint projection, and reads the clock correction off the dynamic `τ↔t` metric.
Peak dynamic pressure is **not** an open node but a **regime change** (the Encke→Cowell crossover,
`ε = a_aero/a_grav ≳ 1`), handled by adopting the built `grmhd/select_metric` detector to switch the integrator
— giving the trajectory axis its own regime change parallel to continuum→plasma. The residual is the
integrator handover (overlap-band agreement + hysteresis), and the factoring is provisional pending the Tier-B
Stage-4+ aero interface.]**

**Mandate — curvature must be dynamic, not hardcoded** (the same invariant as the Park-2T physics, see
[`gap-two-reacting-plasma.md`](gap-2/gap-two-reacting-plasma.md) §1.2): the metric `g_uv` is computed from the
physical state (`g_00 = −(1 − 2GM/rc²)` from the real `GM`/`r`, plus `γ(v)` for the SR timing term), and the
**Ricci/curvature from the metric** (the real `einstein_tensor` inputs / energy-momentum route) — replacing
the `−0.9` / `−0.1·g` / `−0.4` proxies. Only constants of nature and cited gravity coefficients (`G`, `c`,
EGM/IERS terms) stay literal, in `constants/`. The regime threshold the `select_metric` coupling compares
against is *config*; the curvature it compares *is computed from state*. **[open: hardening Gap 3 honors the
dynamic invariant]**

**Progress (2026-06-30) — the navigation/timing core is now demonstrated on real data.** Product decisions
⑥ (filter/sensor model) and ⑧ (one unified envelope) of
[`gap-3/gap-three-resolution-3-trajectory-axis.md`](gap-3/gap-three-resolution-3-trajectory-axis.md) are no
longer just decided — they have a working artifact: **`examples/avionics_examples/ins_gnss_blackout`**. It
runs on the **real Galileo E14** SP3/CLK products and composes the three native mechanisms the resolution
calls for: the **grmhd `select_metric` regime detector** flips GNSS available↔denied (two regime changes), the
**`intervene`/`branch_with`** corrective loop disciplines the INS when GNSS is up and is *withheld* through the
blackout, and the shipped **`relativistic_clock_drift_rate_kernel`** (FS-3) is *carried* across the dark — and
beats a naive last-rate hold against the real measured clock (≈3532 ns vs 3663 ns). Open-loop pure INS drifts
~375 km over the day; the closed loop stays bounded and snaps back, all in one auditable `CausalFlow` with the
regime changes + interventions in the `EffectLog`. This is the **physics-gated GNSS denial + carried
relativistic clock + corrective reacquisition** thesis — the clock-holdover *core* of the blackout problem —
realised without plasma CFD. The real-data ingestion was factored into the reusable **`deep_causality_file`**
crate (RINEX SP3/CLK loaders over the haft IO monad), which the chronometric `gm_recovery` example now also
rides. **[demonstrated for the timing/navigation core; the full 2T conformal lift + aero coupling (Gap-3
resolutions 1/3) remain the open trajectory research].**

### Gap 4 — EPP composition glue (substrate present, flagship wiring absent)

`CausalFlow` / `PropagatingEffect` (counterfactual `continue_with`), the `grmhd` metric-selection coupling
pattern, the CFD `Coupling` / `PhysicsStage` seam, and an Effect Ethos layer all exist as working
substrate. The regime classifier (Knudsen + ionization + GNSS state), the safety-gate wiring, and the
provenance log are composition work — not missing primitives. **[holds under precondition: example wired]**

---

## 5. Bottom line and smallest next step

> **Status (2026-07-02): shipped.** The corridor was built and archived as
> `openspec/changes/archive/2026-07-02-add-plasma-blackout-corridor/` (the contract-first, reordered
> plan: Stage 0 contracts → 3-D metric/marcher → nav engine in `deep_causality_cfd` → composition
> stages → the Flow-DSL alternation/fork machinery → the flagship
> `examples/avionics_examples/cfd/plasma_blackout/corridor`). The flagship's peak electron density
> lands at `1.03e19 m⁻³` against the RAM-C II `1e19` anchor via the marched Park two-temperature
> controller with sheath renewal; its coupled gates (blackout window, INS drift → reacquisition,
> regime change, counterfactual branches, tensor compression) self-verify and exit nonzero on
> regression. The remainder of this section is the pre-build analysis, kept for the record.
>
> **Status update (2026-07-02, second change): carrier upgraded from surrogate to compressible.**
> `openspec/changes/add-compressible-blackout-carrier/` hosted the 2-D compressible marcher behind
> the same coupled-loop machinery (a shared `CoupledCarrier` seam; the QTT host bit-identical):
> the flagship now flies **one continuous descent** — the truth trajectory selects the freestream
> through an atmosphere schedule, the exact Rankine-Hugoniot jump is the shock-fitted inflow
> strip, `T_tr`/`n_tot`/pressure are evolved per-cell projections (the recovery-temperature
> reconstruction and the per-station `FlightCondition` constants are gone), blackout onset and
> exit are flow-resolved events, and the gate-clamped bank command actuates a 3-DOF
> `BankSteeredLift` so branch misses are trajectory-derived. The peak `n_e` at the 61 km passage
> holds the anchor band (measured `1.43e19` vs the `1e19` anchor, 1.43×, inside the 5× band) on the evolved state.

- **Did the tensor train remove a gap?** Yes — the one that made step [4] aspirational. The
  flowfield-compression axis is now primitive-complete and the SOTA reacting-MPS method
  (arXiv:2512.13661) maps cleanly onto what we have.
- **Can the example be built now?** Closer. ~~(1) a CFD→QTT/MPO bridge + a small MPS rollout (Gap 1)~~ —
  **done** (the bridge, the immersed-body solver, and the drag/heat surface observables are built and
  verified). Tier A now needs: (2) a parametric Park-2T / ionization `PhysicsStage` surrogate (Gap 2), and
  (3) wiring the existing skeletons + Ethos gate + provenance (Gap 4). Neither is blocked on missing
  mathematics.
- **Tier B** is now **mostly de-risked engineering, not open research.** The compressible QTT marcher's Stages
  0–2 are built and gated; the shock-rank lever is **measured** (`deep_causality_cfd/studies/`: Cartesian-captured
  3-D curved shock `χ ~ √side`, body-fitted `χ ~ O(10)`) and **mandatory by design**. The six resolutions
  ([gap-2/](gap-2/) Res 4–9) discharged what used to be the open parts: shock-fitting-in-QTT is no longer "a
  research move" but the dynamic `MetricProvider` the bulk runs in (Res 5); the implicit-acoustic convergence
  gamble is replaced by a closed-form inverse (Res 6); body-fit is a generality-preserving blend parameter (Res
  4); and the wake/turbulence residual has levers (Res 8–9). What remains is **building** Stages 3–6 (Stage 4 =
  the RAM-C milestone) and **validating** against flight — with two honest standing caveats (instantaneous
  turbulent fine structure, never needed for `n_e`; RANS fidelity) and the separate **Bars-2T-exact-gravity +
  perturbative-aero coupling**, which stays genuinely **[open]** and is untouched by the tensor work. Smallest
  de-risking slice and the C1–C8 map: [`gap-2/tier-b-compressible-marcher.md`](gap-2/tier-b-compressible-marcher.md).

**Smallest honest slice that proves the thesis:** a Tier-A vertical slice — quasi-1D reacting flow as a
QTT/MPS rollout (new tensor train), a parametric ionization surrogate feeding a blackout trigger, 2–3
`continue_with` bank-angle branches, the Effect Ethos corridor gate, and provenance — leaving the
trajectory axis at its current skeleton fidelity and labelling it so.

Gap 1 is the critical path; its dedicated closing plan is in
[`gap-one-cfd-tensor-bridge.md`](gap-1/gap-one-cfd-tensor-bridge.md).

---

## 6. Related

- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) — the flagship specification this
  analysis measures against.
- [`gap-one-cfd-tensor-bridge.md`](gap-1/gap-one-cfd-tensor-bridge.md) — SOTA methodologies for closing Gap 1 (**closed**).
- [`gap-two-reacting-plasma.md`](gap-2/gap-two-reacting-plasma.md) — the Gap-2 physics-kernel/solver split + Park-2T plan.
- [`gap-2/tier-b-compressible-marcher.md`](gap-2/tier-b-compressible-marcher.md) — the **measured** Tier-B note:
  the four `deep_causality_cfd/studies/` rank studies, the `χ ~ √side` upper bound, and the body-fitted-coordinate
  + IMEX mandate for the compressible shock-capturing marcher.
- `deep_causality_tensor` tensor-network layer — the primitives Gap 1 builds on.
- `examples/avionics_examples/hypersonic_2t/`, `examples/physics_examples/grmhd/` — the skeletons of
  axes 3 and 4.
