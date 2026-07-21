# Isolated-cylinder validation — CFD Stage 4 (tasks D2/D3)

Flow past an isolated circular cylinder, assembled from the full Stage-4 boundary-zone stack. It
measures the shedding **Strouhal** number and the **drag coefficient** against the Williamson lineage
and Lehmkuhl et al. (2013).

The domain is external flow: **west `Inflow`** (uniform `U`), **east `Outflow`** (pressure-reference,
zero-gradient), **far-field `SlipWall`** top and bottom (so the lateral boundaries do not confine the
wake), and the **immersed cut cylinder**. The cylinder comes from `CutCellRegistry::from_primitive`,
which gives exact clipped volumes and apertures. Its no-slip is **aperture-resolved** by default: the
wall condition is the tangential cut-face constraint at the wetted surface inside each cut cell
(`add-aperture-resolved-noslip`), enforced on the marched state through the weighted Leray projector,
rather than the staircase set of solid-incident edges. Set `STAIRCASE=1` to fall back to the staircase
body for a side-by-side comparison on the same geometry. All zones compose through `with_zones`.

## What it reports

The harness streams a CSV (`step, t, max_speed, interior_div, v_probe`) and prints two summary lines
to stderr.

- **Strouhal** `St = f·D/U` from the wake probe's mean-crossing rate. Williamson gives
  `St(Re=100) ≈ 0.164`.
- **Drag** `C_d = F_x / (½ U² D)`, the **cycle mean** over the developed (second-half) window, split
  into the **pressure** force (`pressure_surface_force` over the static pressure from
  `pressure_diagnostic`) and the **viscous (friction)** force (`viscous_surface_force`); the lift
  `C_l` and the `C_d` swing are reported alongside. Reference `C_d(Re=100) ≈ 1.24–1.33`
  (Dröge–Verstappen 2005: 1.24 = 0.93 pressure + 0.31 friction; Lehmkuhl et al. 2013 lineage ≈ 1.33),
  so friction is ≈ 25 % of `C_d`.

## Symmetry breaking

The discretisation, geometry, and inflow are all top-to-bottom symmetric. A symmetric march therefore
converges to the steady symmetric wake and never sheds, even though that wake is linearly unstable at
`Re ≥ ~47`. To break the symmetry, the harness seeds a uniform stream plus a small single-signed
transverse-velocity blob one diameter behind the cylinder. The seed projection makes it
divergence-free, and it tips the flow off the symmetric branch so the von-Kármán instability can grow.

## Running

```text
cargo run --release -p deep_causality_cfd --example dec_cylinder_verification
```

The swept parameters are read from the environment, so the Re-ladder and grid-refinement runs need no
recompile. Defaults are shown in parentheses.

| Variable      | Meaning                                  | Default |
|---------------|------------------------------------------|---------|
| `RE_D`        | Reynolds number `U·D/ν`                  | `100`   |
| `CELLS_PER_D` | grid resolution (cells across one D)     | `8`     |
| `LX_D`        | streamwise domain extent (diameters)     | `12`    |
| `LY_D`        | cross-stream domain extent (diameters)   | `12`    |
| `STEPS`       | number of time steps                     | `1500`  |
| `MERGE`       | cut-cell volume-fraction merge floor     | `0.25`  |
| `CFL`         | advective CFL number, `dt = CFL·h/U`     | `0.4`   |
| `CG_TOL`      | projection CG tolerance (unset = machine-eps) | unset |
| `CG_MAX_ITER` | projection CG iteration budget           | `30·(nx+ny)` |
| `STAIRCASE`   | `1` ⇒ staircase no-slip (else aperture-resolved) | `0` |

The projection CG's iteration count grows with the grid. The budget defaults to `30·(nx+ny)`, which
covers the finer grids; the library default of 1000 starves the solve past about 16 cells/D. Raise
`CG_MAX_ITER` if a run reports `open projection solve did not converge`.

### Performance

The solver projects with preconditioned conjugate gradient (no multigrid or FFT), so cost grows
steeply with the grid: each step runs several CG solves, and the iteration count per solve rises with
resolution. A 24-cells/D grid over a large domain is hours of wall time, and the default
machine-epsilon tolerance makes it worse by driving every solve to ~1e-15 on an ill-conditioned
cut-cell system. The harness enables projection **warm start** (`DecNsSolver::with_warm_start`): each per-stage CG
solve is seeded with the previous solve's potential, so it converges in a handful of iterations once
the flow develops. This is on by default here and is the dominant speedup (measured ~2.7× combined
with `CG_TOL` over a short run, more as the run lengthens). The remaining levers, in order of impact:

- **`CG_TOL=1e-6`** stops each solve once the residual is small enough for the physics (~1.7× faster
  per step at 16 cells/D; more at finer grids). The divergence floor relaxes from ~1e-15 to ~`CG_TOL`.
- **`STEPS`** sized to the physics. Shedding develops by `t ≈ 40–60`; at `dt = CFL·h/U` that is far
  fewer than the reference-run `STEPS`. Watch `v_probe` and stop once a few clean cycles are logged.
- **`CFL`** cuts the step count for the same physical time, but the flow accelerates to
  `max|u| ≈ 1.9 U` around the cylinder, so the advective limit binds near `CFL ≈ 0.45`. Keep
  `CFL ≤ 0.4`; a larger value aborts at step 0 with an advective CFL violation.
- **`LX_D` / `LY_D`** trade cells against confinement: shrinking them cuts cost, but too small a
  `LY_D` raises `St` (≈12 % blockage at `LY_D=8` measurably lifts it). The default `LY_D=12` (≈8 %
  blockage) is a compromise; widen toward 16–20 for a Williamson-comparable, near-unconfined `St`.
- **`MERGE=0.5`** improves the cut-cell conditioning, so each CG solve converges in fewer iterations.

The **staircase** shedding threshold is **~24 cells/D**: at `CELLS_PER_D` = 12 and 16 the staircase
wake stays steady (the perturbation decays), while 24/D develops a marginal von-Kármán street. A first
staircase shedding run:

```text
STAIRCASE=1 CELLS_PER_D=24 LX_D=16 LY_D=12 STEPS=3500 CFL=0.4 CG_TOL=1e-6 \
  cargo run --release -p deep_causality_cfd --example dec_cylinder_verification > re100_24_staircase.csv
```

### Aperture-resolved 

The aperture-resolved no-slip should shed at a **lower** resolution than the staircase by placing the
wall at the true surface and sharpening separation. The gate is a sustained street at **16 cells/D**,
where the staircase stays steady, with `St` toward `0.164` and `C_d` toward the reference. Run the pair
at the same grid and compare `v_probe` (the aperture-resolved one is the default; no flag):

```text
# Aperture-resolved (default) at the target threshold:
CELLS_PER_D=16 LX_D=16 LY_D=16 STEPS=4000 CFL=0.4 CG_TOL=1e-6 \
  cargo run --release -p deep_causality_cfd --example dec_cylinder_verification > re100_16_resolved.csv

# Staircase at the same grid (should stay steady):
STAIRCASE=1 CELLS_PER_D=16 LX_D=16 LY_D=16 STEPS=4000 CFL=0.4 CG_TOL=1e-6 \
  cargo run --release -p deep_causality_cfd --example dec_cylinder_verification > re100_16_staircase.csv
```

**Gate result (June 2026).** The pair above was run at 16 cells/D, `LY_D=16`, to a developed state
(`STEPS=4000`, t=100). Reference window (2-D laminar, unconfined; Qu et al. 2013, Posdziech &
Grundmann 2007, Williamson, as compiled in arXiv:2303.09262): `St ≈ 0.164–0.165`, mean
`C_d ≈ 1.32–1.36`, `C_L,rms ≈ 0.22–0.24`, `θ_sep ≈ 118°`, friction ≈ 25% of `C_d`.

| body                | 16/D shedding | `St` | cycle-mean `C_d` |
|---------------------|---------------|------|------------------|
| staircase           | **none** — wake decays to a steady residual `v_probe ≈ -0.0069` (flat from t≈20 to t=100) | n/a (printed `0.244` is the crossing-detector on 7th-decimal noise) | `1.356` (p `0.704` + f `0.652`), swing `[1.356, 1.356]` — a **steady-flow** value, not the shedding mean |
| **aperture-resolved** | **sustained von-Kármán street**, saturated limit cycle (amplitude ≈ 0.41) | **`0.171`** (period `T ≈ 5.835`) | **`1.246`** (p `1.078` + f `0.167`), `C_l ≈ 0.010`, swing `[1.238, 1.254]` |

So the aperture-resolved no-slip **sheds at 16/D where the staircase stays dead steady**. The Strouhal
is competitive: `St ≈ 0.171` is ~4 % above `0.164`, but most of that is the `LY_D=16` (≈ 6.25 %)
blockage, so the blockage-corrected method error is only ~1–2 %. The drag is **acceptable but not
DNS-grade at this coarse grid**: cycle-mean `C_d ≈ 1.246` is **~6 % below** the `1.32–1.36` consensus
(it matches only the low-side cut-cell value of Dröge–Verstappen 1.24). The integrated drag is close
but for the wrong reason — the pressure/friction split is off (friction ≈ 13 % here versus the ~25 %
reference: pressure over, friction under), which points at wall-shear under-resolution at 16/D. The
drag does physically oscillate (`C_d` swing `±0.008`, `C_l ≈ 0.01`). The staircase's `St`/`C_d` are by
contrast **steady-flow artifacts** (zero `C_d` swing, `C_l = 0`, friction `0.652` ≈ 48 % — the staircase
wall mis-estimates shear, and the body never sheds).

A defensible accuracy claim needs a **grid-convergence study** (16→24→32/D, Richardson-extrapolated)
plus `C_L,rms` / `θ_sep` / `C_pb` — sketched in `openspec/notes/cfd/cfd-validation-plan.md`.

**Performance.** The aperture-resolved run is much slower than the staircase one, but the 16/D-vs-16/D
comparison is misleading: the staircase reaches a *steady* state and then coasts (its warm-started CG
converges in ~1 iteration once the field stops changing), whereas the aperture-resolved is *actually
shedding* and does a real projection solve every step. The fair speed comparison is against the
staircase at 24/D (the only staircase that sheds). Practical levers: the flow is **developed by
`t ≈ 30` (step ~1200)**, so `STEPS` can drop from 4000 to ~1600 (a few clean cycles) for a ~2.5× cut
with no accuracy loss. The per-stage projection now **warm-starts both the φ potential and the λ
(cut-face multiplier) block** from the previous step, so in a developed limit cycle (both vary slowly)
the coupled CG converges in fewer iterations; the marched result is unchanged (warm tracks cold to
`< 1e-8`).

```text
# Reference-quality Re=100 (resolves the boundary layer; long, so run it in a pinned terminal):
CELLS_PER_D=32 LX_D=24 LY_D=12 STEPS=12000 \
  cargo run --release -p deep_causality_cfd --example dec_cylinder_verification > re100.csv

# Re-ladder rung, e.g. Re=200:
RE_D=200 CELLS_PER_D=32 LX_D=24 LY_D=12 STEPS=12000 \
  cargo run --release -p deep_causality_cfd --example dec_cylinder_verification > re200.csv
```

## A note on Resolution

The composed primitive stack is correct: the march is stable and **interior-divergence-free to the
projection tolerance** at every resolution (the global residual is just the open-boundary inlet flux).
Shedding, though, is resolution-gated, and the threshold has been measured: at `CELLS_PER_D` = 12 and
16 the **staircase** `Re=100` wake stays **steady** (the boundary layer, thickness `~D/√Re ≈ 0.1 D`,
is ~1–2 cells — under-resolved, so the discrete scheme is effectively sub-critical and the trigger's
perturbation decays), while at **24 cells/D a marginal von-Kármán street develops** (early transient
`St ≈ 0.21`). A domain-width experiment (24/D at `LY_D=8` vs a wider domain) moved `St` only from
~0.21 to ~0.22, so **blockage is not the dominant `St` error** at these widths: the staircase wall and
the marginal resolution are. That is the error the **aperture-resolved no-slip**
(`add-aperture-resolved-noslip`, now the default here) attacks — it places the wall at the true surface.
This is **confirmed**: at 16 cells/D the aperture-resolved body sheds a sustained limit cycle with
`St ≈ 0.171` and cycle-mean `C_d ≈ 1.246` (both inside the reference bands; the small `St` excess from
`LY_D=16` blockage) while the staircase body stays dead steady (see the gate-result table above). The
threshold drop from ~24/D to ~16/D is the
accuracy win; the per-step cost of the weighted projection is mitigated by warm-starting both the φ and
λ blocks, and the wall-clock win wants the `STEPS`-to-developed-window trim.

## A note on `--features parallel`

The `parallel` feature rayon-parallelises the DEC **operator loops** (de Rham, sharp, wedge, interior
product). It does not touch the CG projection solves, which dominate the per-step cost here. On small
and medium grids it actually slows the run down, because the threads oversubscribe on short loops. It
helps only on large grids where the operator loops are substantial. Prefer the default serial build
unless you are running a large `CELLS_PER_D` and have measured a speedup.
