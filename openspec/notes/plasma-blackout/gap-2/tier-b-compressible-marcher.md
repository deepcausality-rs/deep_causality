<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Gap 2, Tier-B — the compressible shock-capturing QTT marcher (measured)

**What this is.** The Tier-B half of [Gap 2](gap-two-reacting-plasma.md): turning the **incompressible**
QTT marcher into a **compressible, shock-bearing** one that can carry a real Mach-25 reentry flowfield.
This note differs from the rest of the plasma-blackout notes in one way — its central claims are now
**measured**, not argued. Four self-verifying rank studies in `deep_causality_cfd/studies/` settle the
load-bearing assumption (is the flowfield low tensor-train rank?) and the lever that controls it.

Honesty convention: **[holds]**, **[holds under precondition]**, **[open]**, **[measured]**.

---

## 1. Verdict (measured)

The Tier-B thesis — "micrometre-over-metre resolution for free because the flowfield is low tensor-train
rank" (corridor §3.3) — is **half-true, and the false half is fixable by construction**:

- **A shock is NOT intrinsically high rank; a *coordinate-misaligned curved surface* is.** The rank driver
  is **alignment with the grid axes**, not sharpness or curvature per se. **[measured]**
- **Captured on a fixed Cartesian QTT grid, a realistically-formed 3-D curved shock has bond dimension
  `χ ~ √side`, growing without bound in resolution.** A body-fitted (shock-aligned) coordinate holds the
  same shock at **`χ ~ O(10)`, constant in resolution.** **[measured]**
- **The lever is coordinate alignment, not artificial viscosity.** Thickening does not remove the
  curvature that sets the rank, and naive over-thickening is **diffusion-CFL-unstable** (it blows up to
  full rank) — so stable dissipation needs an **implicit / IMEX** step. **[measured]**

**Bottom line:** a 3-D Tier-B compressible QTT marcher is tractable **only** with a **shock-aligned /
body-fitted coordinate** plus an **implicit/IMEX** time step. Capturing the curved shock on a Cartesian
QTT grid keeps *storage* sub-dense but gives back most of the compression win exactly where it is needed.
This is the **singularity-confinement** reformulation (§3), the spatial dual of the LER pattern that
resolved the Tier-A source stiffness.

---

## 1a. Build status (Stages 0–6 built, 2026-06-30)

The `add-cfd-compressible-qtt-marcher` change is staged 0–6. **Stages 0–6 are now built and gated.** The
genuinely-irreducible remainders are two named, defensible open nodes: **(i)** bounding the *dynamic marched*
rank of a flux-through-front (needs re-pinning + an exact-RH interface, design D9 / `qtt_repin_marcher`), and
**(ii)** a **3-D body-fitted `MetricProvider`** (the 3-D marcher is Cartesian-capture so far) — plus the
standing out-of-scope **wake** (turbulence). The six ARIZ resolutions
([4](gap-two-resolution-4-body-fit-parameter.md)–[9](gap-two-resolution-9-moment-closure-turbulence.md)) that
de-risked the make-or-break nodes are now realized in code.

**Built & verified (code, exit-nonzero gates):**
- **Stage 0** — 3-D QTT codec + `gradient_{x,y,z}` / `laplacian_3d` / divergence MPOs (7 gates).
- **Stage 1** — body-fitted polar coordinate, low-rank Jacobian, chain-rule physical gradient, **rank-lever
  gate** (fitted bond bounded `O(10)` while the Cartesian control grows) (5 gates).
- **Stage 2** — conservative compressible Euler (ideal gas + Rusanov), **Sod exact-Riemann gate** passing
  (density/velocity/pressure L1 within tol).
- **Stage 3** — IMEX split-acoustic integrator (`AcousticImex1d`). The stiff constant-coefficient core is
  advanced by its **closed-form low-rank inverse** (`AcousticCoreInverse`), realizing the D10 ideal the
  `qtt_acoustic_precond` study validated densely — **no AMEn-convergence gamble**, free-stream-exact, **exact
  at all N**. Gates: D10 gate 1 (`A₀A₀⁻¹ = I` to round-off, bond ≤16 flat L8→L10), stability beyond the
  explicit acoustic-diffusion limit, conservation, positivity.
- **Stage 4** — RAM-C stagnation line (`FittedNormalShock` + reused Tier-A LER), peak `n_e` vs RAM-C II
  within ~1 decade (`verification/qtt_ramc_stagline`, exit 0).
- **Stage 5** — 2-D body-fitted compressible marcher (`CompressibleMarcher2d`), IMEX (explicit convection +
  implicit acoustic dissipation via the closed-form 2-D ADI inverse), implements `Marcher`, runs over the
  `MetricProvider` seam; the continuous body-fit blend (`BlendedMap`, the `λ` dial). **Rank-lever gate**
  (`verification/qtt_blunt_body_2d`, exit 0): static χ bounded fitted (3→5) vs Cartesian capture (16→61).
- **Stage 6** — 3-D compressible marcher (`CompressibleMarcher3d`, 5 conserved vars, closed-form 3-D ADI
  inverse), implements `Marcher`. **Forebody rank-lever gate** (`verification/qtt_reentry_3d`, exit 0):
  body-fitted forebody χ bounded (2→4) vs Cartesian (10→59); the **wake** reported out-of-scope.

**Open remainders (named, defensible) and the de-risking resolutions:**

| Node (was) | Resolution | New status |
|---|---|---|
| "Structured/body-fitted only sacrifices generality" | [Res 4](gap-two-resolution-4-body-fit-parameter.md) (D8) | body-fit is a **`MetricProvider` blend parameter `λ`**, general across structured geometries. **[measured, `qtt_blend_metric`]** the blend stays valid (`det J` one sign) and `λ` dials bond 114→5 |
| **Make-or-break:** does the static rank lever survive *marching*? | [Res 5](gap-two-resolution-5-dynamic-rank-lever.md) (D9) | **feedback-pinned** map ⇒ rank `O(1)` **by construction**. **[measured, `qtt_rank_fitted_dynamic` + `qtt_repin_marcher`]** aligned front bounded (7/7, flat); a *static* fitted chart grows 25→35, and **re-pin alone does not fix it** (still 25→35 — the driver is Cartesian flux *through* the curved front). Coordinate-aligned transport on a tracked interface holds bond 8, flat. So Stage 4 = **re-pin + exact-RH interface**, not flux-through-front |
| **Make-or-break:** does AMEn converge on the acoustic operator? | [Res 6](gap-two-resolution-6-implicit-acoustics.md) (D10) | **split** operator + **closed-form constant-coefficient inverse**; AMEn (if used) is preconditioned to `I + small`. **[measured, `qtt_acoustic_precond`]** core inverts at bond 8 (flat L8→L10); smooth-interior `ρ=0.59<1`, captured jump `ρ=0.87→1` |
| "shock-fitting coupled to a QTT bulk is unprecedented" | [Res 5](gap-two-resolution-5-dynamic-rank-lever.md) | fitting **is** the dynamic `MetricProvider` — not a bolt-on; the bulk runs *in* the fitted coordinate; **[de-risked]** |
| "the *wake* is the residual rank unknown" | [Res 8](gap-two-resolution-8-spectral-pinning.md) | **spectral pinning** (DLRA/`tdvp`): pin the discontinuity geometrically, carry the rest spectrally; extends closure to the **transitional near-wake**; out-of-scope boundary becomes a **measurable `K(t)` tripwire**; **[lever found]** |
| "the wake needs turbulence (a non-goal)" | [Res 9](gap-two-resolution-9-moment-closure-turbulence.md) | turbulence becomes a **modeled** region — RANS/moment closure (LER stages + low-rank eddy-viscosity MPO) delivering the **mean** `n_e`; **[reframed]** |

**The residual, now narrowed.** After the resolutions the genuinely-irreducible unknown is no longer "the
flowfield may not be low-rank" — it is two named, defensible caveats: **(i) instantaneous turbulent fine
structure** (high-rank, but *never needed* for the mean `n_e` that drives blackout) and **(ii) RANS-closure
fidelity** (the standard hypersonic-CFD modeling caveat). Everything between Stage 2 and the RAM-C milestone
is now engineering with a gate, not open research.

**Next deliverables (post-Stage-6):** (i) a **3-D body-fitted `MetricProvider`** so the 3-D marcher runs in
the fitted coordinate (the §1-in-3-D piece); (ii) **re-pinning + exact-RH interface** to bound the dynamic
marched χ of a flux-through-front (D9); (iii) `CfdFlow`/`QttMarchRun` wiring for the compressible marchers.
The RAM-C stagnation line (Stage 4) remains the honest first Tier-B validation point.

---

## 2. The empirical backbone — four rank studies

All in `deep_causality_cfd/studies/` (self-verifying, `cargo run --release -p deep_causality_cfd --example
<name>`). Measured at `f64`, Apple M3 Max, release.

| Study | Question | Result |
|---|---|---|
| `qtt_rank_study` | Is a flowfield low rank? (static) | **Alignment, not curvature, is the driver.** 1-D step is rank ≤ 2 at any position (cheap). 2-D: flat χ≈5, **curved bow χ≈151**, **straight 45° oblique χ≈394** (*worse than the curve*). Body-fitted (fn of r) collapses to **χ≈5** (~290× win). |
| `qtt_rank_dynamic` | Does a marcher keep it low rank? (linear) | **Rank-safe** for linear transport — fixed-tolerance rounding settles at the static rank, no runaway. Nonlinear steepening left open → next study. |
| `qtt_rank_nonlinear` | Does a *forming* shock stay low rank? (2-D Burgers) | 2-D curved shock rises **7→20** dynamically — the threat is **real**. **Thickening refuted as the lever** (curvature sets the rank; over-thickening is diffusion-CFL-unstable → full rank). |
| `qtt_rank_3d` | What is the **upper bound** in 3-D? (3-D Burgers, explicit Euler + central diff) | **`χ ~ side^0.53` (≈ √side), unbounded** — measured `45 → 135` over `16³ → 128³`. Flat / body-fitted stay **χ ~ 6 constant**. |

**Two cost facts that must not be conflated** (from `qtt_rank_3d`):

1. **QTT-vs-dense *storage*:** in 3-D, dense `~ side³` outruns `χ² ~ side^1.1`, so QTT storage **always wins
   asymptotically**. The `dense/QTT` ratio runs `0.08× (16³) → 0.92× (64³ break-even) → 2.74× (128³)` — the
   64³ "break-even" is a small-grid artifact, not a wall.
2. **The *solve* cost:** tensor-train ops are `O(χ²)–O(χ³)` per core. `χ ~ √side` means at a flight-relevant
   micrometre grid (`side ~ 10⁶`) a captured curved shock implies **`χ ~ thousands`** — bounded, but
   expensive enough to erase the practical advantage. The body-fitted shell holds `χ ~ O(10)` at **any**
   resolution. **That χ-gap — √side vs O(10) — is the Tier-B-deciding result, not storage-vs-dense.**

These are **lower bounds** on a live solver: a real marcher carries operator products *before* rounding,
and explicit central differencing adds dispersive-oscillation rank atop the irreducible curvature floor.

---

## 3. The reformulation — singularity confinement (dual of LER)

The Tier-A LER pattern confined *source* stiffness into a closed-form per-cell substep. Tier-B confines the
*spatial singularity* the same way. The false constraint to drop is **"the shock must be *captured* —
resolved as a steep gradient inside a fixed Cartesian QTT grid."**

> **Singularity confinement.** Keep the tensor-train field *smooth* (hence low-rank) by extracting every
> non-smooth or stiff feature into a separate low-dimensional carrier with its own analytic/implicit
> treatment: a scalar ODE (the LER source), a **fast-mode implicit solve** (acoustic IMEX), or a **fitted
> interface** in a shock-aligned coordinate (the shock).

For the shock specifically: a body-fitted / shock-adapted coordinate turns the curved bow shock into an
**axis-aligned coordinate surface**, replacing `χ ~ √side` with `χ ~ O(10)` (measured). The interface jump
is handled by **exact Rankine–Hugoniot** rather than a captured steep gradient — which also removes the
LeVeque–Yee wrong-shock-speed coupling caveat (C7) for the Tier-A reacting sources riding on top.

---

## 4. The capability gaps (updated by measurement)

| Cap | What | Status after the studies |
|---|---|---|
| **C1** | Conservative hyperbolic flux (Riemann/HLLC/Roe) as MPO/TT operators | **[open — engineering]**; on a *smooth* (fitted) field this reduces to flux-divergence via the existing stencil MPOs |
| **C2** | Shock-capturing limiter + artificial viscosity in TT | **[measured]** thickening is **not** the lever for a curved shock, and over-thickening is CFL-unstable → largely **dissolved** by fitting; only a residual rank-safety net remains |
| **C3** | Acoustic CFL → implicit / IMEX step | **[measured-necessary]** explicit dissipation is diffusion-CFL-limited; needs `solve::linear` (AMEn) for the fast mode; same operator-split shape as LER |
| **C4** | Nonlinear EOS `p(ρ,e,T_tr,T_ve,{Y_s})` via TT-cross | **[holds under precondition: smooth fields]** — which fitting provides |
| **C5** | Conservation- and positivity-preserving rounding | **[open]** RH at the fitted interface removes the wrong-shock-speed risk; carry conserved totals + rank-1 fixup; entropy/log variables for positivity |
| **C6** | Compressible BCs (characteristic inflow/outflow, thermal/catalytic wall) | **[open]** the fitted shock surface *is* the inflow (RH); the wall is a coordinate surface; outflow still characteristic |
| **C7** | Stiff source on a captured shock → wrong shock speed | **[dissolved by fitting]** no smeared shock; post-shock state exact via RH; the Tier-A LER reuse becomes caveat-free |
| **C8** | Dimensionality — axisymmetric / 3-D | **[measured-critical]** 3-D is where `χ ~ √side` bites; sphere-cone is naturally body-fitted/axisymmetric → fitting is also the C8 answer |

The single reformulation (fitting + IMEX + carry-invariants) discharges or reduces C2, C4, C5, C6, C7, C8
and is **mandated** by the C3/C8 measurements. C1 (a smooth conservative TT flux) is built (Stage 2, Sod-gated).
**Update (resolutions 4–9, §1a):** the two clauses formerly flagged here as *genuinely open research* are now
**de-risked**: "shock-fitting coupled to a QTT bulk is unprecedented" is resolved by making fitting the dynamic
`MetricProvider` the bulk runs in ([Res 5](gap-two-resolution-5-dynamic-rank-lever.md)); the *wake* residual now
has a lever — spectral pinning (DLRA/`tdvp`, [Res 8](gap-two-resolution-8-spectral-pinning.md)) plus a RANS
moment closure for the mean ([Res 9](gap-two-resolution-9-moment-closure-turbulence.md)), leaving only
instantaneous fine structure (not needed) + RANS fidelity as the standing caveat.

---

## 5. The mandate (what the spec must commit to)

1. **A shock-aligned / body-fitted coordinate is mandatory, not optional.** The corridor §3.3 "smooth
   analytic coordinate stretch through a low-rank Jacobian" is **elevated from a nicety to a hard
   requirement**: without it, 3-D `χ ~ √side` makes the solve impractical at flight resolution.
2. **Stable dissipation via an implicit / IMEX step** (gap C3) — not cranked artificial viscosity, which is
   CFL-unstable (measured).
3. **The interface jump is exact Rankine–Hugoniot**, keeping each side smooth (low-rank) and the Tier-A
   reacting sources caveat-free.

---

## 6. Smallest de-risking slice — the RAM-C stagnation line

Do **not** build the 3-D marcher first. Build the **RAM-C stagnation-line vertical slice**: a **quasi-1-D
fitted normal shock** along the stagnation streamline → exact RH post-shock state → the **Tier-A LER
chemistry/ionization** in the smooth post-shock relaxation zone → `n_e` → blackout. This is a 1-D fitted
shock (trivially low rank — `qtt_rank_study` showed 1-D is cheap) plus the **entire Tier-A stack reused
unchanged**, and it reproduces the canonical RAM-C peak electron density. It tests fitting, the IMEX step,
and the caveat-free LER reuse with no 2-D/3-D capturing. **[holds: buildable; the honest first Tier-B
deliverable]**

---

## 7. Fallback (cautioned by measurement)

If shock-fitting-in-QTT proves too hard, the fallback is **capture-and-thicken**: smooth the shock to a
fixed cell count + entropy variables + a conservation fixup. Post-Res-4 this is **not a separate code path** —
it is simply **`λ = 0`** on the `MetricProvider` blend (the Cartesian limit), so the marcher degrades to capture
continuously rather than by a branch. The studies still caution it: thickening does **not** remove the
curvature-driven `√side` rank growth, and over-thickening is diffusion-CFL-unstable — so the fallback still
needs an implicit step and still pays the `χ ~ √side` solve cost in 3-D. It is more robust to shock topology
(complex shock–shock interactions where fitting fails) but is **not** a rank fix. **[escape hatch — `λ=0`]**

---

## 8. Related

- `deep_causality_cfd/studies/` — the four rank studies that are this note's empirical backbone
  (`qtt_rank_study`, `qtt_rank_dynamic`, `qtt_rank_nonlinear`, `qtt_rank_3d`).
- [`gap-two-reacting-plasma.md`](gap-two-reacting-plasma.md) §4 — the compressible-marcher precondition this
  note measures; §1.4 — the LER pattern this reformulation is the spatial dual of.
- [`../gap-analysis.md`](../gap-analysis.md) §4 Gap 2, §5 — the Tier-B bullets this note quantifies.
- [`../plasma-blackout-corridor.md`](../plasma-blackout-corridor.md) §3.3 (resolution-via-rank), §6
  (shock-rank seam) — the claims now measured.
