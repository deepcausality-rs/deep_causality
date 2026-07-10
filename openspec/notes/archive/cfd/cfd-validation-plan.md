# CFD validation plan — grid convergence + extra quantities (follow-up note)

_Written 2026-06-15 for a future session. Companion to `ctx/handover.md` and `cfd-crate.md`._

## Why this note exists

The Re=100 cylinder validation today is **one coarse point** (16 cells/D): aperture-resolved
`St ≈ 0.171`, mean `C_d ≈ 1.246` (pressure 1.078 + friction 0.167). That is enough to prove the
aperture-resolved no-slip works (it sheds where the staircase can't), but it is **not** a defensible
"competitive accuracy" claim. A credible claim needs (1) a grid-convergence study showing the error
shrinks at the expected order, and (2) more validation quantities than St and mean C_d. This note is
the plan; do it after (or alongside) the `deep_causality_cfd` migration so the harness changes land in
the new crate.

## Anchored reference values (Re=100, 2-D laminar, unconfined)

From the standard compilation (Qu et al. 2013; Posdziech & Grundmann 2007; Kravchenko; Park; Williamson
— table reproduced in arXiv:2303.09262):

| quantity | reference window | our 16/D |
|---|---|---|
| Strouhal `St` | **0.164–0.165** | 0.171 (+4%, mostly blockage) |
| mean `C_d` | **1.32–1.36** (Qu 1.319, P&G 1.325, Williamson 1.36) | 1.246 (**~6% low**) |
| `C_L,rms` | **0.22–0.24** (amplitude ≈ 0.33) | not measured (only mean `C_l ≈ 0.01`) |
| separation angle `θ_sep` | **~118°** (Qu 118.0°) | not measured |
| base-pressure `C_pb` | ≈ −0.70 (Qu) | not measured |
| friction fraction of `C_d` | **~25%** (Qu: pressure ≈75% of drag) | **13%** (the real weak point) |

Headline gaps to close: the **C_d under-prediction (~6%)** and the **friction/pressure split**
(13% vs 25%) — the integrated drag is close but for the wrong reason (pressure over, friction under),
which points at wall-shear under-resolution.

## Part A — grid-convergence study (the core deliverable)

Run the aperture-resolved cylinder at `Re=100` over a grid ladder, each to a developed limit cycle,
**at a fixed low-blockage domain** so resolution is the only varying axis:

- Resolutions: **16, 24, 32 cells/D** (add 48/D if compute allows — needed for a clean 3-grid
  Richardson triple with margin).
- Domain: `LY_D ≥ 24`, `LX_D ≥ 24` (blockage ≤ ~4%); keep it identical across the ladder.
- Refinement ratio `r = 1.5` (16→24→36) gives the cleanest Richardson triple; or `16/24/32`
  (non-constant `r`, use the generalized GCI formula).

For each grid record: `St`, mean `C_d` and its pressure/friction split, `C_L,rms`, `θ_sep`, `C_pb`.

Then per quantity `f`:
- **Observed order** `p = ln|(f₃−f₂)/(f₂−f₁)| / ln(r)` (constant-`r` triple).
- **Richardson extrapolation** to `h→0`: `f_h0 ≈ f₁ + (f₁−f₂)/(rᵖ−1)`.
- **Grid Convergence Index** (Roache) on the finest pair as the reported uncertainty band.
- Compare `f_h0` (not the coarse-grid value) to the reference window above.

**Pass criteria:**
- Monotone convergence (`p` positive and roughly constant; the scheme should be ~2nd order on smooth
  regions, lower near the cut wall).
- Extrapolated `St` within ~2% of 0.164–0.165; extrapolated mean `C_d` within ~3% of 1.32–1.35.
- **The friction fraction trends toward ~25% as the grid refines** — this is the decisive test of
  whether the 13% is a resolution artifact (expected) or a formulation bug (would not converge).

## Part B — blockage isolation (cheap, settles the St question)

At one fixed resolution (24/D), vary `LY_D ∈ {12, 16, 24, 32}`, measure `St`, fit `St(β)` vs blockage
`β = D/LY`, extrapolate to `β→0`. Confirms the raw +4% `St` is blockage (expected slope ~+0.01 per
0.1 β) rather than method error. One afternoon of runs; do this first — it is the cheapest way to
de-risk the `St` story.

## Part C — extra-quantity diagnostics (harness/code work)

These are missing today and are needed for B and the table above. Land them in the `deep_causality_cfd`
harness (generic over `R`, no f64 casts):

1. **`C_L,rms`** — record the full `C_l(t)` series over the developed window (auto-detected via the
   existing cycle/zero-crossing logic) and report rms. Today only an instantaneous/mean `C_l` prints.
2. **Separation angle `θ_sep`** — sample wall shear (the `viscous_surface_force` per-fragment traction,
   tangential component) around the cylinder by polar angle; `θ_sep` is the sign change of wall shear.
   The cut-cell fragments already carry the wall location + normal, so the machinery exists.
3. **Base-pressure `C_pb`** — pressure (`pressure_diagnostic`) at the rear stagnation fragment (θ=180°),
   nondimensionalized. Cheap once fragments are indexed by angle.
4. **Machine-readable run summary** — emit one JSON/CSV row per run (`{cells_per_d, ly_d, St, Cd, Cd_p,
   Cd_f, CLrms, theta_sep, Cpb, wall_clock}`) so a small convergence script can ingest the ladder
   without hand-parsing stderr. The Richardson/GCI reducer can be a tiny example or a `validation/`
   helper.

## Part D — likely accuracy levers (if convergence is slow or C_d stays low)

Ordered by expected impact on the C_d / friction gap:
- **Graded near-wall metric** (the `PerEdge` graded geometry already exists) to cluster cells at the
  cylinder surface — directly attacks the under-resolved wall shear that depresses friction.
- **The cut Hodge star in the projector metric** (Lever B, designed but not wired — the projector
  currently uses the plain star). Revisit only if the constraint-only wall under-performs after
  refinement.
- **A better projection preconditioner** than Jacobi-CG (multigrid / FFT on the unconstrained part)
  — this is mostly a *speed* lever but enables the finer grids that the convergence study needs.

## Cost / sequencing

16/D is already hours; 32/D is ~4× the cells and a smaller `dt` (advective CFL), so the ladder is a
multi-day pinned-terminal effort. Trim `STEPS` to the developed window per grid (16/D develops by
`t≈30`). Sequence: **B (cheap, blockage) → C (diagnostics) → A (the ladder) → D (levers as needed).**
This sits next to the deferred 3D high-Re Re-ladder (Re 200–3900) carved out of
`add-cut-cells-and-immersed-boundaries`; both are "real compute time" validation tracks.
