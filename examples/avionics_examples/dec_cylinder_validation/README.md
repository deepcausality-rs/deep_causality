# Isolated-cylinder validation — CFD Stage 4 (tasks D2/D3)

Flow past an isolated circular cylinder, assembled from the full Stage-4 boundary-zone stack. It
measures the shedding **Strouhal** number and the **drag coefficient** against the Williamson lineage
and Lehmkuhl et al. (2013).

The domain is external flow: **west `Inflow`** (uniform `U`), **east `Outflow`** (pressure-reference,
zero-gradient), **far-field `SlipWall`** top and bottom (so the lateral boundaries do not confine the
wake), and the **immersed cut cylinder**. The cylinder comes from `CutCellRegistry::from_primitive`,
which gives exact clipped volumes and apertures and an auto-derived no-slip set on the solid-incident
edges. All zones compose through `with_zones`.

## What it reports

The harness streams a CSV (`step, t, max_speed, interior_div, v_probe`) and prints two summary lines
to stderr.

- **Strouhal** `St = f·D/U` from the wake probe's mean-crossing rate. Williamson gives
  `St(Re=100) ≈ 0.164`.
- **Drag** `C_d = F_x / (½ U² D)`, the **cycle mean** over the developed (second-half) window, split
  into the **pressure** force (`pressure_surface_force` over the static pressure from
  `pressure_diagnostic`) and the **viscous (friction)** force (`viscous_surface_force`); the lift
  `C_l` and the `C_d` swing are reported alongside. Lehmkuhl gives `C_d(Re=100) ≈ 1.33` with friction
  near 35 %.

## Symmetry breaking

The discretisation, geometry, and inflow are all top-to-bottom symmetric. A symmetric march therefore
converges to the steady symmetric wake and never sheds, even though that wake is linearly unstable at
`Re ≥ ~47`. To break the symmetry, the harness seeds a uniform stream plus a small single-signed
transverse-velocity blob one diameter behind the cylinder. The seed projection makes it
divergence-free, and it tips the flow off the symmetric branch so the von-Kármán instability can grow.

## Running

```text
cargo run --release -p avionics_examples --example dec_cylinder_validation
```

The swept parameters are read from the environment, so the Re-ladder and grid-refinement runs need no
recompile. Defaults are shown in parentheses.

| Variable      | Meaning                                  | Default |
|---------------|------------------------------------------|---------|
| `RE_D`        | Reynolds number `U·D/ν`                  | `100`   |
| `CELLS_PER_D` | grid resolution (cells across one D)     | `12`    |
| `LX_D`        | streamwise domain extent (diameters)     | `12`    |
| `LY_D`        | cross-stream domain extent (diameters)   | `6`     |
| `STEPS`       | number of time steps                     | `1500`  |
| `MERGE`       | cut-cell volume-fraction merge floor     | `0.25`  |
| `CFL`         | advective CFL number, `dt = CFL·h/U`     | `0.4`   |
| `CG_TOL`      | projection CG tolerance (unset = machine-eps) | unset |
| `CG_MAX_ITER` | projection CG iteration budget           | `30·(nx+ny)` |

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
- **smaller `LX_D` / `LY_D`** cuts the cell count directly (mild confinement shifts `St` a little).
- **`MERGE=0.5`** improves the cut-cell conditioning, so each CG solve converges in fewer iterations.

A fast shedding probe (minutes, not hours) starts at the coarsest grid that might shed:

```text
CELLS_PER_D=16 LX_D=16 LY_D=8 STEPS=1600 CFL=0.4 CG_TOL=1e-6 \
  cargo run --release -p avionics_examples --example dec_cylinder_validation > re100_16.csv
```

```text
# Reference-quality Re=100 (resolves the boundary layer; long, so run it in a pinned terminal):
CELLS_PER_D=32 LX_D=24 LY_D=12 STEPS=12000 \
  cargo run --release -p avionics_examples --example dec_cylinder_validation > re100.csv

# Re-ladder rung, e.g. Re=200:
RE_D=200 CELLS_PER_D=32 LX_D=24 LY_D=12 STEPS=12000 \
  cargo run --release -p avionics_examples --example dec_cylinder_validation > re200.csv
```

## A note on Resolution

The composed primitive stack is correct. The march is stable and **interior-divergence-free to
≈ 1e-15** at every resolution; the global residual is just the open-boundary inlet flux. The shedding
instability, though, is resolution-gated. At `CELLS_PER_D ≤ 12` the `Re=100` boundary layer (thickness
`~D/√Re ≈ 0.1 D`, about one cell) is under-resolved, so the discrete scheme is effectively sub-critical
and the flow settles to the steady symmetric solution as the trigger's perturbation decays. A developed
von-Kármán street, and a reference-quality `St` or `C_d`, needs roughly 24 to 40 cells/D and a long run.
That cost is real, the "real compute time" the change's `tasks.md` D2/D3 flags. The default config is an
affordable smoke run that exercises the full path and reports the steady-flow drag.

## A note on `--features parallel`

The `parallel` feature rayon-parallelises the DEC **operator loops** (de Rham, sharp, wedge, interior
product). It does not touch the CG projection solves, which dominate the per-step cost here. On small
and medium grids it actually slows the run down, because the threads oversubscribe on short loops. It
helps only on large grids where the operator loops are substantial. Prefer the default serial build
unless you are running a large `CELLS_PER_D` and have measured a speedup.
