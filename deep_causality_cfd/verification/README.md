# CFD Verification 

Runnable **verification** programs for the CFD stack — the DEC-native solver and the
quantized-tensor-train (QTT) solver — each driven through the `CfdFlow` DSL. *Verification* here is the broad sense: a run is checked against either an **internal
consistency** invariant (a property the discretization must preserve at any grid/precision — energy
decay, incompressibility, observed convergence order) **or** a **published reference** result
(analytic solutions and benchmark papers).

```bash
cargo run --release -p deep_causality_cfd --example <name>_verification
```

## Convention: self-verifying, exit nonzero on break

Every example **self-verifies** and **exits with a nonzero status** the moment its invariant or
reference check fails — so the suite is usable as a gate, not just a demo. What each one checks, and
how it fails, is in the per-example sections below.

**CI runs this suite.** `.github/workflows/cfd_verification.yml` executes the ten `FAST_HARNESSES` on
every pull request and the three `SLOW_HARNESSES` nightly, failing the build on a non-zero exit. A
fourteenth, `qtt_cylinder_verification`, sits in `OFFLINE_HARNESSES` and no job runs it. A completeness
check asserts that every `[[example]]` declared under `verification/` in `Cargo.toml` appears in one of
those three lists, so a newly added harness cannot silently never run. Until this workflow existed,
`cargo test` *compiled* these binaries and never ran them, which meant every quantitative claim on this
page was unenforced.

## Convention: every gate declares its evidence class

Each gate line carries one of two labels, so a `[PASS]` can be read correctly:

| Label | Meaning |
|---|---|
| `[reference]` | The bound comes from an analytic solution or a published external value, cited at the definition site. Clearing it is evidence **about the physics**. |
| `[tripwire]` | The bound is pinned from this code's own prior output. Clearing it is evidence of **non-regression only**, and carries no claim of external accuracy. |

```text
  [PASS] [reference] density  L1 error = 0.0175          <- vs the exact Riemann solution
  [PASS] [tripwire] St 0.1710 in [0.152, 0.19]           <- pinned from a prior run
```

Unlabelled defaults to `tripwire`: claiming agreement with an external reference requires positive
evidence, so the weaker class is the safe one. A tripwire is never presented as validation against a
reference — several bounds here are honestly pinned (the `qtt_ramc_stagline` ±0.70-decade band says
so in its own gate text, and the lid-cavity RMSE bounds carry headroom from their pinning run), and
the label makes that machine-visible rather than something a reader has to find in prose.

Precision is a parameter: each example fixes a `FloatType` alias (`f32` / `f64` / `Float106`) and runs
the whole computation at that precision, downcasting to `f64` only at the display boundary. All numbers
below were measured at **`f64`** on an Apple M3 Max (release build). Runtimes are wall-clock at the
stated configuration and scale strongly with grid size and step count.

## Convention: `baseline.txt` is a complete run

Each harness directory carries a `baseline.txt` — the captured output of a full run, **stdout and
stderr together**, so it holds both the reported quantities and the gate block. A baseline must reach
the harness's terminal summary; a truncated or aborted run is not committed, because it silently
removes the reference a reader compares against.

Regenerate with:

```bash
cargo run --release -p deep_causality_cfd --example <name> > <name>/baseline.txt 2>&1
```

Two properties are load-bearing:

- **The header must describe the run.** The grid, horizon and step count in the baseline are the
  configuration whose numbers this page reports for that harness.
- **A failing baseline is committed as failing.** `qtt_cylinder_verification`'s committed baseline is
  from its old `L = 5` configuration (`exit 1`, two `NOT CONVERGING` ladder verdicts). Since
  `close-qtt-solver-envelope` the harness runs at `L = 8` with a wall-error-target `η` — physically
  correct, but a single march is ~17 min and the full harness ~4-9 hours, so it is **offline / manual,
  not in CI** (see the OFFLINE / MANUAL note in `.github/workflows/cfd_verification.yml`). The `L = 8`
  baseline is therefore **pending an offline regeneration**; the `L = 5` artifact is retained as the
  last completed run rather than replaced by a fabricated one. The gate stays red for a
  solver-performance reason, not a parameter choice.

Where a harness's default mode has no gates and its gated mode is a subcommand (the lid-driven
cavity), the baseline carries both, under a labelled separator.

## Summary

The **Measured** and **Reference** columns hold the compared values; **Divergence** is their exact
difference. Measured at `f64` on an Apple M3 Max (release).

| Example | Quantity verified | Measured | Reference | Divergence | Config | Runtime (seq) |
|---|---|---|---|---|---|---|
| `mms_taylor_green_verification` | RHS residual; amplitude error | 1.1e-16; 6.7e-16 | 0 (analytic) | ≈ machine-ε (~0 %) | default | ~1 s |
| `dec_graded_mms_verification` | observed order (finest pair) | 1.98–2.00 | 2.00 | ≤ 0.02 (< 1 %) | 8²–64² | ~1 s |
| `dec_taylor_green_re1600_verification` | peak dissipation ε; energy invariant | 0.0025 (E\*/E0 0.893, monotone) | ≈ 0.0124 (DNS) | **−80 %** (16³ under-resolved); invariant PASS | 16³, t\*=10 | <1 s |
| `dec_lid_cavity_re1000_verification` | primary vortex (x, y); centerline RMSE | (0.5312, 0.5625); RMSE 0.0617 | Ghia (0.5313, 0.5625) | Δ ≈ (1e-4, 0) — **primary vortex matches to 1e-4 in x, exactly in y** | 65², t=100 (the no-argument default, not the `trend` rung) | ~20 min |
| `dec_cylinder_wake_verification` | max divergence residual; log count | 3.3e-15; 80 | 0; 80 (= 2×40) | ≈ machine-ε; exact | 2000 steps, 93×32 | ~155 s |
| `dec_cylinder_verification` | Strouhal St; drag C_d | 0.171; 1.345 | 0.164; 1.32–1.36 | **+4.3 %**; **−1.1 %** (inside band; friction share ~13 % vs the reference's ~25 %) | 96², Re=100, 1500 steps | ~510 s |
| `qtt_taylor_green_verification` | TG decay error (32²); observed order; convection | 5.3e-5; 2.02–2.18; 3.2e-3 | 0 (analytic); 2.00; 0 (analytic) | converges 2nd-order; **+9 %** order; conv ≈ 0.6 % | 8²–32², t=0.2 | <1 s |
| `qtt_cylinder_verification` ⚠ *(offline)* | drag convergence vs bond; no-slip interior; **η and mask-smoothing ladders** | env resolved at `L = 8` (η from 2.5 % wall-error target); acceptance run pending offline | — | penalization layer now resolved (`√(ην) ≈ dx`); gate red on **solver cost**, not parameters | 256², bond [24, 48] + 2 ladders | **~4-9 h (offline)** |
| `qtt_park2t_blackout` | 6 LER coupling gates (stability, kernel vs an independent sub-stepped integration, RH band, lag + rate grounding, path-dependence, n_e>0); peak `n_e` reported beside them | all 6 PASS; peak `n_e` 1.000e22 m⁻³ (α saturated at 1); ω_p 5.6e12 ≫ band | RAM-C II ≈ 1e19 m⁻³ (order-of-magnitude flight anchor) | **+3.0 dec** — Saha saturates at α = 1 at the perfect-gas `γ = 1.4` post-shock temperature. **Internal-invariant scope**: the six gates check the LER coupling, not agreement with RAM-C (see the Tier-A disclaimers) | 32², 40 steps, γ = 1.4 | ~4 s |
| `qtt_sod` | Sod shock tube vs exact Riemann (L1 of ρ/u/p) | 0.018 / 0.027 / 0.015 | < 0.03 (1st-order global Lax–Friedrichs: Rusanov with a global wave-speed estimate) | p\*=0.303 (exact), fan+contact+shock correct | 512 cells, t=0.2 | ~1 s |
| `qtt_ramc_stagline` | peak electron density `n_e` / blackout onset | 5.31e17 (Park-2T controller); 2.25e19 (uncalibrated network) | ~1e19 (RAM-C II, order-of-mag) | **−1.27 dec** Park-2T (reported, not re-admitted); **+0.35 dec** network (earned band ±0.70) | stagnation line, γ = 1.1 | <0.1 s |
| `qtt_blunt_body_2d` | rank lever: bow-shock χ, fitted vs Cartesian capture | fitted 3→5; capture 16→61 | structural (fitted χ bounded, ≤ +1 per refinement) | fitted flat 3→5; capture ≈ side/2 (16, 32, 61), not ~√side | 2^5–2^7 | <1 s |
| `qtt_reentry_3d` | rank lever: 3-D forebody χ (wake out-of-scope) | fitted 2→4; Cartesian 10→59; wake 41 | structural (`qtt_rank_3d` bound) | fitted plateau; capture grows | 2^3–2^5 | <1 s |

> ⚠ **`qtt_cylinder_verification` is known-failing and runs nightly, not per-PR.** Its two parameter
> ladders gate and both report `NOT CONVERGING`. This is a correct measurement, not a regression:
>
> - **η ladder** (0.128 → 0.008): `C_d` = 17.39, 24.02, 26.25, 23.76, 21.40 — it rises, peaks, then
>   falls. There is no `η → 0` limit, and that limit is what licenses calling the penalization
>   integral a drag at all (Angot, Bruneau & Fabrie 1999, `O(η^{3/4})`).
> - **Smoothing ladder** (0.5 → 4 cells): `C_d` = 7.70 … 47.27, a **6.1×** span driven by a purely
>   numerical mask width.
>
> Root cause: the physical Brinkman layer `√(ην) = 0.144·dx` is ~7× thinner than one cell, and the
> resolution criterion `η ≥ dx²/ν = 0.771` is violated 48× by the configured `η = 0.016`. The grid
> therefore resolves the mask smoothing skirt, not the penalization layer — which is why the reported
> force tracks the smoothing width rather than η.
>
> **What this retires.** The former headline for this harness — "the convergence trend is the
> verification result" — refers to the *bond* ladder, i.e. saturation of the tensor-train compression.
> That says nothing about whether the compressed quantity is a drag. Read the absolute `C_d ≈ 23.8`
> as a property of this configuration's blur width, not of a cylinder.
>
> Resolution (choose η from a wall-error target; refine to resolve the layer) is a solver and cost
> change tracked as Phase 2 item 10 of [`AUDIT-REPORT.md`](../../openspec/audits/cfd_audit/AUDIT-REPORT.md).
> The gate is kept rather than silenced: widening the bound until it passes is exactly the
> back-fitting this suite is being cleaned of.

**Validation scope labels.** The QTT reacting/compressible gates verify at four distinct tiers. Read each
gate for what it actually proves. **Analytic** (`qtt_sod` vs the exact Riemann solution) is rigorous, the
only quantitative-accuracy gate. **Flight-data, order-of-magnitude** is the `qtt_ramc_stagline` network
prediction: its renewal arm lands +0.35 dec of the RAM-C II peak `n_e`, inside the ±0.70 chemistry-spread
band. The Park-2T closed-form controller in the same harness lands −1.27 dec below the anchor after the
`fix-ramc-vibrational-relaxation-pair` reduced-mass correction; that offset is **reported**, not presented
as agreement (the former +0.0-dec headline was an artifact of an invalid `μ = 7.0`). **Structural /
rank-lever** is `qtt_blunt_body_2d` and `qtt_reentry_3d`: the body-fitted coordinate *bounds* χ where the
Cartesian capture grows with resolution, so these gate **rank**, not physical accuracy. Neither harness
measures a √side law. `qtt_blunt_body_2d`'s capture grows roughly linearly, χ ≈ side/2 (16, 32, 61 at
side 32, 64, 128; √side would give ≈ 6, 8, 11), and `qtt_reentry_3d`'s grows 10 → 59 over side 8 → 32.
The √side law is measured in `studies/qtt_rank_3d`: χ ~ side^0.53 over 16³ → 128³. The **dynamic marched** rank
growth (flux-through-front) and the **wake** are *reported, never asserted*; bounding the marched χ needs
re-pinning and an exact-RH interface (design D9), the named open remainder.

**Internal-invariant only** is the fourth tier, and `qtt_park2t_blackout` is its sole member. Its six gates
test the LER coupling itself: relaxation stability at `τ = Δt/1000`, the closed-form kernel against an
independently sub-stepped integration of `dx/dt = (x_eq − x)/τ`, a wide `10⁴–10⁵ K` Rankine–Hugoniot band,
that the ionization lag is real and `τ_ion` varies with `T`, counterfactual path-dependence, and `n_e > 0`.
Not one of them compares a marched quantity against flight data. The harness runs on the incompressible
rollout at perfect-gas `γ = 1.4`, the Saha surrogate saturates at `α = 1`, and its peak `n_e = 1.000e22 m⁻³`
therefore sits **+3.0 decades** above the RAM-C II anchor. That offset is a statement of scope, not a
result. `qtt_ramc_stagline` is the physical-accuracy arm: it runs the effective `γ = 1.1`, carries the exact
Rankine–Hugoniot post-shock state instead of a reconstruction, and gates against the anchor. The two
harnesses therefore disagree on `γ` (1.4 against 1.1) and their `n_e` figures are not comparable. Only the
`γ = 1.1` choice is justified at its definition site, as an effective-γ closure for strongly dissociated
air; the `γ = 1.4` in `qtt_park2t_blackout/config.rs` carries no such note, and raising it to 1.1 would be
a behaviour change, not a documentation one.

Reference papers per example are in the sections below and the [References](#references). The cavity
centerline RMSE (**0.0617**, the 65²/t=100 default the row reports) is itself a deviation-from-Ghia
measure (no single reference value), so its divergence is shown via the primary-vortex offset. `mms` and
the `wake` divergence residual verify against the *ideal* (analytic 0 / exact incompressibility), so their
reference is 0.

---

## `mms_taylor_green_verification` — Method of Manufactured Solutions

**Verifies.** The incompressible Navier–Stokes right-hand-side kernel against the **closed-form**
Taylor–Green vortex: fed the exact spatial derivatives (via the tangent functor — autodiff, no finite
differences), a correct kernel must return the exact `du/dt`, and a correct `Rk4` march must track the
exact amplitude decay `a(t) = exp(−2νt)`.

**Self-check.** Stage-1 residual and stage-2 amplitude error are reported; they sit at machine
precision. (The example prints the residual; treat a residual far above ε as a regression.)

**Measured (f64, default, ~1 s).** Stage-1 max abs error **1.11e-16**; stage-2 amplitude error
**6.66e-16** — i.e. essentially **0 %** divergence, at the floor of the working precision.

**Precision reached.** f32 ≈ 3e-8, f64 ≈ 1e-16, `Float106` ≈ 8e-33 (the residual tracks ε of the
alias).

**Reference.** Taylor & Green (1937); MMS methodology: Roache (2002), Salari & Knupp (2000).

---

## `dec_graded_mms_verification` — graded-metric operator MMS

**Verifies.** That the two operators of the incompressible march — the convective interior product
`i_X ω` (Cartan magic formula) and the viscous Hodge Laplacian `δd` — retain **second-order accuracy**
on smoothly graded meshes, measured in both max- and L2-norms over a `8²→64²` refinement ladder at
grading amplitudes 0.0–0.3. The reference is the theoretically expected order **2**.

**Self-check.** Observed orders are tabulated; the anticipated result is order ≈ 2 at every grading.

**Measured (f64, default, ~1 s).** Finest-pair observed order **1.98–2.00** (both operators, both
norms); at strong grading the coarse-pair order dips to ~1.7 but recovers to ~2.0 as the mesh refines
— i.e. **< 1 %** from order 2 at the finest pair. Error magnitudes at 64²: ≈ 5e-3 (convective),
≈ 8e-4 (viscous). Divergence-freeness is exact at any grading (combinatorial).

**Reference.** DEC: Hirani (2003); Desbrun, Hirani, Leok & Marsden (2005). Regge metric: Regge (1961).

---

## `dec_taylor_green_re1600_verification` — 3D Taylor–Green at Re 1600

**Verifies.** *Internal consistency* (structure preservation): the unforced viscous TGV can only lose
kinetic energy, so the projected DEC march must keep energy monotonically non-increasing with
`E*(T) < E*(0)`. *Published reference* (informational): the kinetic-energy dissipation-rate curve
`−dE*/dt*` against the high-Re DNS.

**Self-check.** `verify()` gates the energy-monotonicity invariant and **exits nonzero** on any
spurious energy production. (No DNS data is needed for the gate; the DNS comparison is the CSV the run
emits.)

**Measured (f64, 16³ smoke grid, t\*=10, <1 s).** Invariant **PASS** (`E*/E0 = 0.8929`, monotone).
Peak dissipation **≈ 0.0025** vs the DNS reference peak **≈ 0.0124** near t\*≈9 — **~80 % below**,
because 16³ is grossly under-resolved (it cannot represent the small-scale dissipation peak). The
reporting resolutions **64³–128³** close this gap; raise the grid arg to approach the DNS curve.

**Reference.** van Rees, Leonard, Pullin & Koumoutsakos (2011); Brachet et al. (1983); 1st
International Workshop on High-Order CFD Methods (2012), case C3.5.

---

## `dec_lid_cavity_re1000_verification` — lid-driven cavity at Re 1000

**Verifies.** Centerline velocity profiles and the streamfunction **vortex centers** against the Ghia,
Ghia & Shin (1982) tables (pooled centerline RMSE + primary/corner-eddy locations). The `trend`
subcommand is the gated mode: it runs a `17²→33²` refinement at time-converged horizons and **exits
nonzero** unless the RMSE both clears a pinned bound and strictly decreases under refinement.

**Self-check.** `cargo run … --example dec_lid_cavity_re1000_verification trend` (exit nonzero on a
broken refinement trend). The default single run reports the RMSE and vortex table.

**Measured (f64, default 65² grid, t=100, ~20 min — the configuration the summary row reports and the
first half of `baseline.txt` records; invoke it with no argument).** Centerline **RMSE 0.0617** vs Ghia;
primary vortex at **(0.5312, 0.5625)** vs Ghia **(0.5313, 0.5625)** — matching to **1e-4 in x and exactly
in y**; bottom-right eddy at (0.8594, 0.1094), exactly Ghia's value; bottom-left at (0.0781, 0.0781) vs
(0.0859, 0.0781), one cell out in x. The reporting **129²/t≥150** (hours, Ghia's own grid) tightens
further.

**Measured (gated `trend` mode, `… --example dec_lid_cavity_re1000_verification trend`, ~1 min — the
second half of `baseline.txt`, under its labelled separator).** 17²/t=60 RMSE **0.2369**, 33²/t=60 RMSE
**0.1309**; all three tripwires PASS (pinned bounds 0.32 and 0.20, strict-decrease margin 0.04). These are
refinement-trend rungs at a coarser grid and a shorter horizon than the default, so they are not
comparable to the 0.0617 above and are not this harness's headline result.

*This row previously reported a coarse `33²` rung (RMSE 0.137, vortex (0.563, 0.594)) while every other
row reported its default — an inconsistent basis that **understated** the solver by more than a factor of
two on the RMSE. It also described that vortex offset as "≈ 6 % of span"; the actual offset is
**3.1 % per axis**, which is one cell at 33² (Δx = Δy ≈ 0.0317 against a cell width of 1/32 = 0.03125).*

**Reference.** Ghia, Ghia & Shin (1982).

---

## `dec_cylinder_wake_verification` — cut-cell cylinder wake (sensor-fed uncertain inflow)

**Verifies.** *Internal consistency only.* This is a confined, periodic-x harness (a prescribed moving
wall, not a true inflow/outflow surface) exercising the cut-cell + uncertain-zone machinery; it makes
**no quantitative reference claim** for the shedding Strouhal. The gate is (a) incompressibility — the
constrained Leray projector keeps the divergence residual at machine precision — and (b) exact
`EffectLog` accounting: every sensor dropout records its fallback + intervention pair.

**Self-check.** Gates max divergence `< 1e-6` and `log_entries == 2 × dropouts`; **exits nonzero** on
break.

**Measured (f64, 2000 steps, 93×32, ~155 s).** Max divergence **3.33e-15** (machine precision); log
**80 = 2 × 40** dropouts — both **PASS**. Strouhal is attempted and reports **no clear shedding**: the
confined periodic-x channel at 25 % blockage (`config::BLOCKAGE = 0.25`) damps the von-Kármán street at
this configuration, so the probe settles to a steady `v ≈ 2.70e-2` instead of oscillating. The estimate
is printed, never gated; `dec_cylinder_verification` is the harness that sheds. The full wake-probe
series is written to `cylinder_wake.csv` via the IO effect.

**Reference.** None quantitative (internal-consistency exercise).

---

## `dec_cylinder_verification` — isolated cylinder (D2/D3 validation)

**Verifies.** Flow past an *isolated* circular cylinder (Inflow / Outflow / far-field SlipWall + the
immersed cut cylinder) against published laminar benchmarks: the shedding **Strouhal** `St = f·D/U`
(Williamson) and the cycle-mean **drag coefficient** `C_d` with its pressure/friction split. The `C_d`
reference is the 2-D unconfined laminar consensus band **1.32–1.36** (Qu et al. 2013, Posdziech &
Grundmann 2007, Williamson, as compiled in arXiv:2303.09262); `main.rs` carries it as
`CD_REFERENCE_BAND`. Dröge & Verstappen (2005) is the secondary reference, for the split only:
their cut-cell `C_d = 1.24 = 0.93` pressure `+ 0.31` friction, i.e. friction ≈ 25 %. That 1.24 is a
single low-side cut-cell datum, no longer the band's lower edge. Case parameters (`RE_D`,
`CELLS_PER_D`, `LX_D`, `LY_D`, `STEPS`, `CFL`) are environment-overridable for the Reynolds ladder and
grid refinement.

**Self-check.** Four gates, exit nonzero on break: shedding was detected at all, the developed window
produced a cycle-mean drag, and `St` and `C_d` each sit inside a pinned band. A solver error now
**exits 1** rather than breaking the march and reporting `St`/`C_d` from the truncated series — the
harness previously contained no assertion and no `process::exit` and returned 0 in that case,
contradicting the suite convention above.

All four bounds are `[tripwire]`, not `[reference]`, and deliberately so. At the affordable 8 cells/D
default `St` sits **outside** the published value, and `C_d` sits **inside** the published band for the
wrong reason: friction is ≈ 13 % of `C_d` against the reference's ≈ 25 %, so the total agrees by
cancellation. Gating against the published values directly would fail a correctly-working solver on
`St` and would reward that cancellation on `C_d`. The published values print beside
each measurement with the offset, so the gap stays visible and a `[PASS]` is never read as agreement.
The `St`/`C_d` bands describe `Re = 100` only; a Reynolds-ladder run prints `[SKIP]` for them rather
than passing against a band that does not describe it.

**Measured (f64, default: Re=100, 96² @ 8 cells/D, 12×12 D domain, 1500 steps, ~510 s ≈ 8.5 min).**
- **St ≈ 0.1710** vs Williamson Re=100 **≈ 0.164** → **+4.3 %**.
- **C_d ≈ 1.345** vs the reference band **1.32–1.36** → **inside the band**, 1.1 % below its top
  (pressure 1.173 + friction 0.172; `C_l ≈ −0.007`, C_d swing [1.338, 1.353]).
- Friction fraction ≈ 13 % vs the reference ≈ 25 %: skin friction is under-resolved at 8 cells/D. The
  total `C_d` therefore lands in the band by cancellation of an over-predicted pressure force against
  an under-predicted friction force, not because both parts are right. A finer grid (16–32 cells/D)
  and a longer run bring both St and the friction split toward the references.

**Reference.** Williamson (1996); the 2-D laminar consensus compilation arXiv:2303.09262 for the `C_d`
band; Dröge & Verstappen (2005) for the pressure/friction split; Lehmkuhl, Rodríguez, Borrell & Oliva
(2013).

---

## `qtt_taylor_green_verification` — quantized-tensor-train 2-D Taylor–Green

**Verifies.** The `QttIncompressible2d` solver — a 2-D incompressible flowfield that evolves entirely
as a **tensor train** — against the closed-form 2-D Taylor–Green vortex (Taylor & Green 1937),
`u = −cos x sin y`, `v = sin x cos y`, decaying as `e^{−2νt}`. Four gates: (1) the final-field error
vs. the analytic decay **strictly decreases under refinement** to a pinned bound at ~2nd order;
(2) the nonlinear convection `u·∇u` matches the closed form `−½ sin 2x` — checked **directly**, because
single-mode TG's convective term is a pure gradient the projection removes, so the marched decay alone
cannot test it; (3) the post-projection divergence stays at the projection floor; (4) the MPS
compression (bond vs. dense) is reported. Driven through `CfdFlow::march`.

**Self-check.** `verify()` gates all four and **exits nonzero** on any break (error not converging,
order < 1.8, convection wrong/zero, or divergence above 1e-6).

**Measured (f64, 8²–32², t=0.2, <1 s).** Error `9.8e-4 → 2.4e-4 → 5.3e-5` (N=8→16→32), observed order
**2.02 → 2.18**: 2nd-order in space (centered FD + spectral projection), first-order in time (explicit
Euler at fixed `dt`), so the ladder measures the spatial order; finest default-grid error **5.3e-5**.
The fixed-`dt` Euler error is a temporal floor of opposite sign, so extending the ladder is **not**
free: the signed error crosses zero near N=64–128 (`+5.9e-6 → −5.9e-6`), making the N=64 order of 3.16
a cancellation artifact and collapsing the order to 0.02 by N=128. **`max_level = 5` is the maximum
usable length**; the documented `max_level 7` fails the order gate. See the harness README.
Convection vs the closed form **3.2e-3** (≈ 0.6 % of the 0.5 signal) — the nonlinear term is real and
correct. Divergence **~1e-14** (the spectral Leray projection is exact to machine precision). Bond `= N`
on this smooth field → `N×` compression that grows with resolution.

**Reference.** Taylor & Green (1937); the MPS-CFD method: Peddinti et al. (2024), Gourianov et al.
(2022).

---

## `qtt_cylinder_verification` — immersed cylinder by Brinkman penalization (tensor-train)

**Verifies.** The immersed-body QTT solver (`QttImmersed2d`): a cylinder in a periodic free-stream
enforced by **Brinkman volume penalization** (a smoothed mask, no cut cells), with drag read as a
**tensor-train contraction** of the mask with the velocity deficit. Closes Gap 1 of the plasma-blackout
analysis (immersed body + surface observables). Driven through `CfdFlow::march`.

**Self-check.** Three gates, **exit nonzero** on break: (a) no-slip — interior `max|u|` at the
penalization floor; (b) accuracy-vs-bond — the drag coefficient **converges** as the round bond cap rises;
(c) physical drag — positive and finite.

**Measured (f64, 32², 4 bond caps, ~1 s).** `C_d` settles `24.05 → 23.76 → 23.7577 → 23.7577`, with the
successive change collapsing `2.9e-1 → 7.2e-3 → 1.9e-11` and divergence dropping `3.8e-1 → 5.5e-14` as the
bond cap rises — the headline accuracy-vs-bond trade-off. Interior `max|u| ≈ 4.2e-2` vs free-stream `1.0`
(no-slip). The **absolute** `C_d ≈ 23.8` is *not* the isolated-cylinder value (DEC `≈ 1.345`): it is
inflated by ~30 % blockage, the smoothing-skirt penalization-force definition, and the transient — so the
**convergence trend** is the verification result, with the DEC `C_d` a disclaimed cross-reference.

**Reference.** Angot, Bruneau & Fabrie (1999) — volume penalization; Peddinti et al. (2024) — MPS
immersed objects; DEC cross-reference `dec_cylinder_verification`.

---

## `qtt_park2t_blackout` — Park two-temperature blackout coupling (Tier-A)

**Verifies.** *Internal consistency only.* The Lagging-Equilibrium Relaxation (LER) coupling hosted in
the QTT march (`QttMarchRun::run_coupled`) over a Brinkman-penalized blunt forebody: a
`RecoveryTemperatureStage` rebuilds `T_tr` from a mandatory Rankine–Hugoniot normal-shock jump plus
`½|u|²/c_p`, an `IonizationStage` relaxes the carried ionization fraction toward the Park-2T Saha
surrogate, and an `EosStage` closes the pressure. A `BlackoutTrigger` maps peak `n_e` to the plasma
frequency and compares it to the configured comms band. The gates test the coupling machinery. **None of
them compares a marched quantity against RAM-C II or any other flight measurement.**

**Self-check.** Six gates, exit nonzero on break. (i) The relaxation stays bounded and monotone at
`τ = Δt/1000`, where a single explicit Euler rate step overshoots the equilibrium by ~10³× (the gate
demands at least 100×). (ii) `ler_step` agrees with a 10⁶-substep forward-Euler integration of
`dx/dt = (x_eq − x)/τ` to `1e-6` relative. That is the one `[reference]`-class gate here, because the
integration is a genuinely separate derivation; the tolerance is sized from the reference's own
truncation error, `a²/2N ≈ 4.5e-8`, not from the measurement. (iii) The RH jump lands `T_post` inside
`(1e4, 1e5) K`. (iv) The Arrhenius rate rises with `T` and the lagged `α` sits below `α_eq`. (v) Two
temperature histories reaching the same target carry different `α`. (vi) Marched `n_e > 0`. Gates (i) and
(iii)–(vi) are `[tripwire]`.

**Measured (f64, 32², 40 steps, M = 25, γ = 1.4, n_tot = 1e22 m⁻³, ~4 s).** All six PASS. Peak
`n_e = 1.000e22 m⁻³`, i.e. `α = 1` exactly — the Saha surrogate **saturates** at the frozen RH post-shock
temperature, so the harness is measuring a ceiling, not a prediction. Peak `ω_p = 5.641e12 rad/s` against
the 9.4e9 rad/s comms band; blackout dwell 1.6e-1 s. Against the RAM-C II ≈ 1e19 m⁻³ anchor that peak is
**+3.0 decades high**. The over-prediction has three named parts: the incompressible carrier (`T_tr` is a
reconstruction, not a transported post-shock path), perfect-gas `γ = 1.4` (which ignores the dissociation
and vibration that absorb post-shock energy), and `T_ve = T_e` lumping (~2×, Farbar–Boyd–Martin 2013). The
operator split is first-order Lie. `qtt_ramc_stagline` is the harness that carries the physical claim.

**Reference.** None quantitative. Reported as cross-references only: RAM-C II (Grantham 1970);
Park (1990, 1993); Millikan & White (1963); Farbar, Boyd & Martin (2013); Aiken, Carter & Boyd (2025).

---

## `qtt_sod` — Sod shock tube against the exact Riemann solution

**Verifies.** `CompressibleEuler1d`, the 1-D conservative compressible Euler marcher that carries
`U = (ρ, ρu, ρE)` as three tensor trains, against the **exact Riemann solution** on the classic Sod data
(`ρ,u,p`: `1,0,1` | `0.125,0,0.1`, `γ = 1.4`). The Rusanov update rearranges into a conservative central
flux difference plus a scalar artificial viscosity, `dU/dt = −∂ₓF + ½·s_max·Δx·∂²ₓU`, so it is assembled
from the `gradient` / `laplacian` MPOs and recompressed each step. The wave speed `s_max = max(|u| + c)`
is taken over the whole state, so this is the Rusanov/local-Lax–Friedrichs *form* driven by a **global**
speed estimate. The nonlinear flux and EOS are evaluated pointwise (dequantize → compute → requantize);
the rank-preserving TT-cross form is the named large-`L` upgrade. This is the **only quantitative
physical-accuracy gate** in the QTT compressible set.

**Self-check.** L1 error of density, velocity and pressure against the exact solution over `|x| ≤ 0.5`,
each below the recorded tolerance `0.03`; exit nonzero on break. All three are `[reference]` class. The
domain is the wide `[−1, 1]` so the periodic boundary-jump waves stay outside the measurement window.

**Measured (f64, 512 cells, CFL 0.4, t = 0.2, ~1 s).** Density **0.0175**, velocity **0.0274**, pressure
**0.0151** — all PASS. The star pressure comes out `p* = 0.3031`, the canonical Sod value, and the left
expansion fan, the contact and the right shock are at the correct positions and speeds. First-order
Rusanov smears the contact, which is what the L1 bound accommodates: the bound is on mean accuracy, not on
sharpness at the discontinuity. Companion unit tests in `tests/solvers/qtt/compressible_tests.rs` gate
conservation of `∫ρ`, `∫ρu`, `∫ρE` and free-stream preservation.

**Reference.** Sod (1978); the exact solver follows Toro, ch. 4.

---

## `qtt_ramc_stagline` — RAM-C II stagnation line (Tier-B Stage 4)

**Verifies.** The stagnation streamline treated as a 1-D **fitted interface**: the freestream crosses the
bow shock and the exact Rankine–Hugoniot jump sets the post-shock state, so no flux is marched *through*
the front and each side stays smooth and `O(1)` rank. `T₂` is real transported energy, which retires the
Tier-A recovery-temperature reconstruction. The smooth post-shock relaxation zone then drives two
independent ionization paths against the RAM-C II peak-`n_e` anchor: the closed-form Park-2T controller,
and an **uncalibrated** three-channel RP-1232 finite-rate network with no Saha target anywhere in it.

**Self-check.** Seven gates, exit nonzero on break: `T₂` in the ~10⁴ K band; peak `n_e` matches the
corrected Park-2T value; blackout onset (`ω_p >` comms band); the relaxation profile stays `O(1)` rank;
the network prediction sits inside its ±0.70-decade earned band; electron impact is a refinement rather
than the driver; the carried sheath arm self-limits at or below the renewal arm. All seven print as
`[tripwire]`, the two anchored ones included: the ±0.70-decade band is pinned from this harness's own
measurement, as its own gate text says, not derived from the flight data's stated uncertainty.

**Measured (f64, M = 25, effective γ = 1.1, T∞ = 250 K, ~0.01 s).** All seven PASS. Exact RH post-shock
state: `T₂ = 8044 K`, `ρ₂/ρ₁ = 20.349`, `u₂/u₁ = 0.049`, `p₂/p₁ = 6.547e2`, post-shock
`n_tot = 2.645e22 m⁻³`; relaxation-profile bond **2**. Park-2T controller: `α = 2.007e-5`, peak
`n_e = 5.310e17 m⁻³`, **−1.27 decades** below the 1e19 anchor, `ω_p = 4.111e10 rad/s`. Uncalibrated
network: channel 1 plus the lagged atom pool `1.887e19` (+0.28 dec), full network `2.251e19` (+0.35 dec).
Sheath-renewal A/B: renewal `2.251e19` (+0.35 dec, kept), carried `1.768e18` (−0.75 dec, self-limiting).

The −1.27-decade controller offset is **reported, not re-admitted**: it followed the
`fix-ramc-vibrational-relaxation-pair` correction of the reduced mass from an invalid `μ = 7.0` (the N–N
atomic pair, which has no vibrational mode) to the N₂–N₂ `μ = 14.007`, which lengthens `τ_vt` about 1.9×
and cools the Park rate-controlling temperature `Tₐ = √(T_tr·T_ve)`. The band was not widened to restore
the old +0.0-decade headline. Read the single-pair figure as a lower bound: the bath also holds lighter
partners whose shorter `τ_vt` a mixture-weighted closure would recover. Open levers: `T_e = T_ve` lumping
(~2×), the single associative-ionization channel, and the ~2–5× Millikan–White chemistry-model spread.
Note the effective `γ = 1.1`, an engineering closure for strongly dissociated hypersonic air. Its
definition site records that perfect-gas `γ = 1.4` over-predicts `T₂` at ≈30 000 K, because it ignores the
dissociation and vibration that absorb the post-shock energy. `qtt_park2t_blackout` runs at that
perfect-gas value.

**Reference.** Grantham (1970), NASA TN D-6062 — the RAM-C II peak-`n_e` anchor; Gupta, Yos, Thompson &
Lee (1990), NASA RP-1232 — the Table II rate pairs; Park (1990, 1993); Millikan & White (1963).

---

## `qtt_blunt_body_2d` — 2-D bow-shock rank lever (Tier-B Stage 5)

**Verifies.** *Structural only: this gate bounds **rank**, not physical accuracy.* A blunt-body bow shock
stands off the nose at a constant *physical* radius `R`. In a body-fitted coordinate that surface is a
line `η = const`, a step in one axis, so its quantized-tensor-train bond `χ` is small and
resolution-independent. Sampled on a Cartesian lattice the identical physical shock is curved on the
grid, so `χ` grows with resolution. `BlendedMap` carries both coordinates as one blend parameter (`λ = 1`
body-fitted polar fan, `λ = 0` Cartesian capture) and the same `CompressibleMarcher2d` runs over both
through the `MetricProvider` seam, so the coordinate is the only variable.

**Self-check.** Two gates, exit nonzero on break: **BB-A**, the fitted `χ` stays at 12 or below and grows
by at most 1 per refinement (no `√side` growth); **BB-B**, the Cartesian capture `χ` grows with resolution
and overtakes the fitted bond by at least 2×. Both are `[tripwire]`.

**Measured (f64, `2^5 → 2^7` ladder, smoothed compression `ρ: 1 → 1.8`, `p: 1 → 3` at `R = 1.5`,
quantized at tolerance 1e-8, <1 s).** Fitted `χ` = 3, 4, 5; Cartesian capture `χ` = 16, 32, 61. Both PASS.
Fitted is flat; the capture cost grows roughly linearly in side, `χ ≈ side/2`, not as `√side`. At these
three resolutions `√side` would give ≈ 6, 8, 11.

**Reported, never asserted.** The **dynamic marched** rank is the open remainder: a plain
flux-through-front marcher injects angular structure across the captured front and grows `χ` to **64**
over 6 steps, even in the fitted coordinate. Bounding it needs re-pinning plus an exact-RH interface with
no flux marched across the front. That is design D9 and the `qtt_repin_marcher` study. The harness prints
the datapoint and never gates on it.

**Reference.** None. The quantitative accuracy gate for this solver is `qtt_sod`, against the exact
Riemann solution. The 3-D form of the same lever is `qtt_reentry_3d`.

---

## `qtt_reentry_3d` — 3-D forebody sheath rank lever (Tier-B Stage 6)

**Verifies.** *Structural only: this gate bounds **rank**, not physical accuracy.* The 3-D form of the
`qtt_blunt_body_2d` lever, on the crate's serial `x`-`y`-`z` codec (`quantize_3d`). A smoothed step at the
standoff radius `R = 1.5` is sampled three ways and quantized at tolerance 1e-8: **fitted** (the shell as
a function of the radial index alone), **Cartesian** (the same physical shell on a `[−2, 2]³` lattice,
curved on the grid), and **wake** (two off-axis lobes downstream, a multi-feature structure no single
fitted coordinate aligns).

**Self-check.** Two gates, exit nonzero on break: **RE-A**, the fitted forebody `χ` stays at 8 or below
with a flat high-resolution tail (last refinement adds at most 1); **RE-B**, the Cartesian capture `χ`
grows with resolution and overtakes the fitted bond by at least 2×. Both are `[tripwire]`.

**Measured (f64, `2^3 → 2^5` ladder, <1 s).** Fitted `χ` = 2, 4, 4; Cartesian `χ` = 10, 30, 59. Both PASS.
Fitted plateaus; the capture cost grows, following the `qtt_rank_3d` upper bound.

**Reported, never asserted.** The **wake** is out of scope by design (a separated, unsteady wake needs
turbulence): `χ = 41` at `2^5`, comparable to the Cartesian capture and un-fittable by construction. It is
printed as a datapoint for the standing `qtt_rank_3d` question and never gated (design D9). The dynamic
marched forebody is likewise reported: there is no 3-D body-fit metric yet, so the marcher runs Cartesian
and grows `χ` to **16** over 6 steps.

**Reference.** None. See `qtt_sod` for the quantitative accuracy gate on this solver.

---

## References

- **Taylor, G. I. & Green, A. E.** (1937). *Mechanism of the production of small eddies from large
  ones.* Proc. R. Soc. Lond. A **158**, 499–521.
- **Peddinti, R. D., Pisoni, S., Marini, A., Lott, P., Argentieri, H., Tiunov, E. & Aolita, L.** (2024).
  *A quantum-inspired framework for computational fluid dynamics.* Commun. Phys. **7**, 135.
- **Gourianov, N., Lubasch, M., Dolgov, S., van den Berg, Q. Y., Babaee, H., Givi, P., Kiffner, M. &
  Jaksch, D.** (2022). *A quantum-inspired approach to exploit turbulence structures.* Nat. Comput.
  Sci. **2**, 30–37.
- **Angot, P., Bruneau, C.-H. & Fabrie, P.** (1999). *A penalization method to take into account obstacles
  in incompressible viscous flows.* Numer. Math. **81**, 497–520.
- **Brachet, M. E., Meiron, D. I., Orszag, S. A., Nickel, B. G., Morf, R. H. & Frisch, U.** (1983).
  *Small-scale structure of the Taylor–Green vortex.* J. Fluid Mech. **130**, 411–452.
- **van Rees, W. M., Leonard, A., Pullin, D. I. & Koumoutsakos, P.** (2011). *A comparison of vortex
  and pseudo-spectral methods for the simulation of periodic vortical flows at high Reynolds numbers.*
  J. Comput. Phys. **230**, 2794–2805.
- **1st International Workshop on High-Order CFD Methods** (2012). Case C3.5 — Taylor–Green vortex.
- **Ghia, U., Ghia, K. N. & Shin, C. T.** (1982). *High-Re solutions for incompressible flow using the
  Navier–Stokes equations and a multigrid method.* J. Comput. Phys. **48**, 387–411.
- **Williamson, C. H. K.** (1996). *Vortex dynamics in the cylinder wake.* Annu. Rev. Fluid Mech.
  **28**, 477–539.
- **Dröge, M. & Verstappen, R.** (2005). *A new symmetry-preserving Cartesian-grid method for computing
  flow past arbitrarily shaped objects.* Int. J. Numer. Methods Fluids **47**, 979–985. Table II is the
  secondary reference for the cylinder pressure/friction split, `C_d = 1.24 = 0.93 + 0.31`.
- **arXiv:2303.09262** — the compilation the 2-D unconfined laminar band `C_d(Re=100) ≈ 1.32–1.36` is
  taken from (Qu et al. 2013, Posdziech & Grundmann 2007 and Williamson, as gathered there), cited at
  `verification/dec_cylinder_verification/main.rs`. **Authors, title and venue are not recorded anywhere
  in this repository**, so they are omitted here rather than supplied from recall.
- **Lehmkuhl, O., Rodríguez, I., Borrell, R. & Oliva, A.** (2013). *Low-frequency unsteadiness in the
  vortex formation region of a circular cylinder.* Phys. Fluids **25**, 085109.
- **Roache, P. J.** (2002). *Code verification by the method of manufactured solutions.* J. Fluids Eng.
  **124**(1), 4–10.
- **Salari, K. & Knupp, P.** (2000). *Code verification by the method of manufactured solutions.*
  Sandia National Laboratories, SAND2000-1444.
- **Hirani, A. N.** (2003). *Discrete Exterior Calculus.* PhD thesis, California Institute of
  Technology.
- **Desbrun, M., Hirani, A. N., Leok, M. & Marsden, J. E.** (2005). *Discrete Exterior Calculus.*
  arXiv:math/0508341.
- **Regge, T.** (1961). *General relativity without coordinates.* Nuovo Cimento **19**, 558–571.
- **Grantham, W. L.** (1970). *Flight Results of a 25,000-Foot-Per-Second Reentry Experiment Using
  Microwave Reflectometers To Measure Plasma Electron Density and Standoff Distance.* NASA TN D-6062,
  NASA Langley. The RAM-C II peak-`n_e` anchor. PDF: `deep_causality_physics/papers/ram_c_ii_nasa_tn_d6062.pdf`.
- **Gupta, R. N., Yos, J. M., Thompson, R. A. & Lee, K.-P.** (1990). *A Review of Reaction Rates and
  Thermodynamic and Transport Properties for an 11-Species Air Model for Chemical and Thermal
  Nonequilibrium Calculations to 30000 K.* NASA RP-1232. Table II supplies the finite-rate network. PDF:
  `deep_causality_physics/papers/gupta_1990_nasa_rp1232.pdf`.
- **Park, C.** (1990). *Nonequilibrium Hypersonic Aerothermodynamics.* Wiley. The two-temperature model.
- **Park, C.** (1993). *Review of Chemical-Kinetic Problems of Future NASA Missions, I: Earth Entries.*
  J. Thermophys. Heat Transfer **7**(3), 385.
- **Millikan, R. C. & White, D. R.** (1963). *Systematics of Vibrational Relaxation.* J. Chem. Phys.
  **39**, 3209. The `τ_vt` correlation behind the lagging `T_ve`.
- **Aiken, Carter & Boyd** (2025). Plasma Sources Sci. Technol. **34**. Cited at
  `deep_causality_physics/src/kernels/hypersonic/ionization.rs` for RAM-C sitting in the mixed
  associative + electron-impact ionization band. **Title, initials and page are not recorded anywhere in
  this repository**, so they are omitted here rather than supplied from recall.
- **Farbar, Boyd & Martin** (2013). Cited across the plasma-blackout corridor for the ~2× peak-`n_e`
  over-prediction of `T_ve = T_e` lumping versus a separate electron-translational energy equation.
  **Initials, title and venue are not recorded anywhere in this repository**; fill them in from the
  published record before this page is used as a citation source.
- **Sod, G. A.** (1978). The shock-tube problem, cited at `verification/qtt_sod/print_utils.rs`.
  **Title, venue, volume and pages are not recorded anywhere in this repository**, so they are omitted
  here rather than supplied from recall.
- **Toro, E. F.** *Riemann Solvers and Numerical Methods for Fluid Dynamics*, ch. 4 — the construction the
  harness's exact Riemann solver follows. **Edition, publisher and year are not recorded anywhere in this
  repository.**

> Divergence figures are single-machine measurements at the **affordable default** configuration; they
> are dominated by spatial resolution, not the discretization's asymptotic accuracy. Reference-grid
> runs (finer grids, longer horizons — noted per example) tighten every figure. Re-measure on the
> target hardware.
