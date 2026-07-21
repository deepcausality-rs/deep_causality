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

**CI runs this suite.** `.github/workflows/cfd_verification.yml` executes the nine fast harnesses on
every pull request and the four slow ones nightly, failing the build on a non-zero exit. A
completeness check asserts that every `[[example]]` declared under `verification/` in `Cargo.toml`
appears in one of the two lists, so a newly added harness cannot silently never run. Until this
workflow existed, `cargo test` *compiled* these binaries and never ran them, which meant every
quantitative claim on this page was unenforced.

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
- **A failing baseline is committed as failing.** `qtt_cylinder_verification`'s baseline records
  `exit 1` and its two `NOT CONVERGING` ladder verdicts. Replacing it with a stale passing artifact
  would hide a real measurement — see its known-failing note above.

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
| `dec_lid_cavity_re1000_verification` | primary vortex (x, y); centerline RMSE | (0.5312, 0.5625); RMSE 0.0617 | Ghia (0.5313, 0.5625) | Δ ≈ (1e-4, 0) — **primary vortex matches to 1e-4 in x, exactly in y** | 65², t=100 (default) | ~20 min |
| `dec_cylinder_wake_verification` | max divergence residual; log count | 3.3e-15; 80 | 0; 80 (= 2×40) | ≈ machine-ε; exact | 2000 steps, 93×32 | ~155 s |
| `dec_cylinder_verification` | Strouhal St; drag C_d | 0.171; 1.345 | 0.164; 1.24–1.33 | **+4.3 %**; **+1.1 %** (over band top) | 96², Re=100, 1500 steps | ~510 s |
| `qtt_taylor_green_verification` | TG decay error (32²); observed order; convection | 5.3e-5; 2.02–2.18; 3.2e-3 | 0 (analytic); 2.00; 0 (analytic) | converges 2nd-order; **+9 %** order; conv ≈ 0.6 % | 8²–32², t=0.2 | <1 s |
| `qtt_cylinder_verification` ⚠ | drag convergence vs bond; no-slip interior; **η and mask-smoothing ladders** | ΔC_d 1.9e-11; max\|u\| 4.2e-2; **neither ladder converges** | 0 (converged); 0 (no-slip) | bond converges to machine-ε; **4 %** of free-stream; **C_d has no η→0 limit and moves 6.1× with smoothing width** | 32², 4 bond caps + 2 ladders | ~37 s |
| `qtt_park2t_blackout` | 6 LER gates (stability, exactness, RH band, lag+Saha, path-dependence, n_e>0) | all 6 PASS; ω_p 5.6e12 ≫ band | all gates pass | Gap-2 Tier-A verified (cross-refs, Tier-A disclaimers) | 32², 40 steps | ~1 s |
| `qtt_sod` | Sod shock tube vs exact Riemann (L1 of ρ/u/p) | 0.018 / 0.027 / 0.015 | < 0.03 (1st-order Rusanov) | p\*=0.303 (exact), fan+contact+shock correct | 512 cells, t=0.2 | ~1 s |
| `qtt_ramc_stagline` | peak electron density `n_e` / blackout onset | 1.085e19 (calibrated Park-2T); 2.991e19 (uncalibrated network) | ~1e19 (RAM-C II, order-of-mag) | **+0.0 dec** calibrated; **+0.48 dec** prediction (earned band ±0.70) | stagnation line | ~1 s |
| `qtt_blunt_body_2d` | rank lever: bow-shock χ, fitted vs Cartesian capture | fitted 3→5; capture 16→61 | structural (no √side growth, fitted) | fitted bounded; capture ~√side | 2^5–2^7 | ~2 s |
| `qtt_reentry_3d` | rank lever: 3-D forebody χ (wake out-of-scope) | fitted 2→4; Cartesian 10→59; wake 41 | structural (`qtt_rank_3d` bound) | fitted plateau; capture grows | 2^3–2^5 | ~3 s |

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
> change tracked as Phase 2 item 10 of [`AUDIT-REPORT.md`](../../openspec/notes/cfd_audit/AUDIT-REPORT.md).
> The gate is kept rather than silenced: widening the bound until it passes is exactly the
> back-fitting this suite is being cleaned of.

**Validation scope labels.** The QTT compressible gates verify at three distinct tiers — read each gate for
what it actually proves: **analytic** (`qtt_sod` vs the exact Riemann solution — rigorous, the only
quantitative-accuracy gate); **flight-data, order-of-magnitude** (`qtt_ramc_stagline` peak `n_e` vs RAM-C II;
the Apollo re-entry dwell window is the corridor-time anchor, not a per-point accuracy claim); and
**structural / rank-lever** (`qtt_blunt_body_2d`, `qtt_reentry_3d` — the body-fitted coordinate *bounds* χ
where the Cartesian capture grows ~√side; these gate **rank**, not physical accuracy). The **dynamic marched**
rank growth (flux-through-front) and the **wake** are *reported, never asserted* — bounding the marched χ
needs re-pinning + an exact-RH interface (design D9), the named open remainder.

Reference papers per example are in the sections below and the [References](#references). The cavity
centerline RMSE (0.137) is itself a deviation-from-Ghia measure (no single reference value), so its
divergence is shown via the primary-vortex offset. `mms` and the `wake` divergence residual verify
against the *ideal* (analytic 0 / exact incompressibility), so their reference is 0.

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

**Measured (f64, default 65² grid, t=100, ~20 min — the configuration `baseline.txt` records).**
Centerline **RMSE 0.0617** vs Ghia; primary vortex at **(0.5312, 0.5625)** vs Ghia **(0.5313, 0.5625)**
— matching to **1e-4 in x and exactly in y**; bottom-right eddy at (0.8594, 0.1094), exactly Ghia's
value; bottom-left at (0.0781, 0.0781) vs (0.0859, 0.0781), one cell out in x. The reporting
**129²/t≥150** (hours, Ghia's own grid) tightens further.

*This row previously reported the coarse `33²/t=40` trend rung (RMSE 0.137, vortex (0.563, 0.594), a
"≈ 6 % of span" offset) while every other row reported its default — an inconsistent basis that
**understated** the solver by more than a factor of two on the RMSE. Those coarse numbers are still
what the `trend` subcommand's pinned bounds are set against; they are a refinement-trend rung, not
this harness's headline result.*

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
**80 = 2 × 40** dropouts — both **PASS**. Strouhal is reported but disclaimed (confined/periodic, not
the isolated-cylinder reference — see `dec_cylinder_verification` for that). The full wake-probe series
is written to `cylinder_wake.csv` via the IO effect.

**Reference.** None quantitative (internal-consistency exercise).

---

## `dec_cylinder_verification` — isolated cylinder (D2/D3 validation)

**Verifies.** Flow past an *isolated* circular cylinder (Inflow / Outflow / far-field SlipWall + the
immersed cut cylinder) against published laminar benchmarks: the shedding **Strouhal** `St = f·D/U`
(Williamson) and the cycle-mean **drag coefficient** `C_d` with its pressure/friction split
(Dröge–Verstappen; Lehmkuhl lineage). Case parameters (`RE_D`, `CELLS_PER_D`, `LX_D`, `LY_D`, `STEPS`,
`CFL`) are environment-overridable for the Reynolds ladder and grid refinement.

**Self-check.** Four gates, exit nonzero on break: shedding was detected at all, the developed window
produced a cycle-mean drag, and `St` and `C_d` each sit inside a pinned band. A solver error now
**exits 1** rather than breaking the march and reporting `St`/`C_d` from the truncated series — the
harness previously contained no assertion and no `process::exit` and returned 0 in that case,
contradicting the suite convention above.

All four bounds are `[tripwire]`, not `[reference]`, and deliberately so: at the affordable 8 cells/D
default the measured values sit **outside** the published bands (below), so gating against Williamson
and Dröge–Verstappen directly would fail a correctly-working solver. The published values print beside
each measurement with the offset, so the gap stays visible and a `[PASS]` is never read as agreement.
The `St`/`C_d` bands describe `Re = 100` only; a Reynolds-ladder run prints `[SKIP]` for them rather
than passing against a band that does not describe it.

**Measured (f64, default: Re=100, 96² @ 8 cells/D, 12×12 D domain, 1500 steps, ~510 s ≈ 8.5 min).**
- **St ≈ 0.1710** vs Williamson Re=100 **≈ 0.164** → **+4.3 %**.
- **C_d ≈ 1.345** vs reference band **1.24–1.33** → **+1.1 %** above the top of the band (pressure
  1.173 + friction 0.172; `C_l ≈ −0.007`, C_d swing [1.338, 1.353]).
- Friction fraction ≈ 13 % vs the reference ≈ 25 % — under-resolved skin friction at 8 cells/D; a finer
  grid (16–32 cells/D) and longer run bring both St and the friction split toward the references.

**Reference.** Williamson (1996); Dröge & Verstappen (2005); Lehmkuhl, Rodríguez, Borrell & Oliva
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
compression (bond vs. dense) is reported. Driven through `CfdFlow::qtt_march`.

**Self-check.** `verify()` gates all four and **exits nonzero** on any break (error not converging,
order < 1.8, convection wrong/zero, or divergence above 1e-6).

**Measured (f64, 8²–32², t=0.2, <1 s).** Error `9.8e-4 → 2.4e-4 → 5.3e-5` (N=8→16→32), observed order
**2.02 → 2.18** — clean 2nd-order convergence to the analytic decay; finest-grid error **5.3e-5**.
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
analysis (immersed body + surface observables). Driven through `CfdFlow::qtt_march`.

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
  flow past arbitrarily shaped objects.* Int. J. Numer. Methods Fluids **47**, 979–985.
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

> Divergence figures are single-machine measurements at the **affordable default** configuration; they
> are dominated by spatial resolution, not the discretization's asymptotic accuracy. Reference-grid
> runs (finer grids, longer horizons — noted per example) tighten every figure. Re-measure on the
> target hardware.
