# Lid-Driven Cavity at Re 1000 — DEC-Native, Wall-Bounded

The square cavity with a moving lid is the canonical wall-bounded
incompressible benchmark: a unit box, three no-slip walls, and a lid
sliding at constant speed. At Re 1000 the steady state carries a primary
vortex near the center and counter-rotating eddies in the bottom corners,
tabulated in the reference every cavity solver is compared against:

> Ghia, U., Ghia, K. N., Shin, C. T. (1982). *High-Re solutions for
> incompressible flow using the Navier–Stokes equations and a multigrid
> method.* J. Comput. Phys. 48, 387–411.

## What this example demonstrates

The wall substrate of the DEC Navier–Stokes solver, working together:

- **No-slip walls** — all four boundaries are walls; tangential edge
  coefficients are pinned to zero, and the lid (the y-max face) carries
  the tangential velocity `U = 1` through `DecNsSolver::with_moving_wall`.
- **The constrained Leray projector** — every stage rate and the step
  re-entry project onto the *intersection* of the divergence-free and
  no-slip subspaces (the M-orthogonal intersection projection), so both
  invariants hold exactly at every step boundary. The masked grade-0
  solve runs Jacobi-preconditioned CG.
- **The boundary-corrected Hodge star** — the clipped dual volumes at the
  walls (faces ½, corners ¼ in 2D) that make the wall operators
  M-symmetric.

## Usage

```text
cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification [grid] [t_end]
cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification trend
```

- `grid` defaults to **65** (minutes of runtime, clear vortex structure).
- The **reporting resolution is 129** — Ghia's own grid — with
  `t_end ≥ 150` for the fully developed corner eddies (hours of
  Jacobi-PCG time; the spectral preconditioning of the masked solve is
  the documented escalation if this becomes routine).
- A 33²/`t_end = 20` smoke run takes ~15 s.
- **`trend`** runs the refinement-trend verification: 17² → 33² at the
  time-converged horizon (`t = 60`), gated (RMSE 0.32 / 0.20, strict
  decrease), nonzero exit on violation (~1 min). It lives here rather
  than in the test suite by design: tests stay fast; verification runs
  as long as it needs.

## Output

- `cavity_centerline_u.csv` — `u` along the vertical centerline: the 17
  Ghia stations with reference values and differences, then the full
  computed profile.
- `cavity_centerline_v.csv` — `v` along the horizontal centerline, same
  layout.
- stdout — run header, centerline RMSE against the pooled Ghia tables,
  and the detected vortex centers (streamfunction extrema; ψ is
  integrated up the columns with ψ = 0 on the walls) against Ghia's
  node-snapped values:

```text
vortex,x,y,psi,ghia_x,ghia_y
primary,0.5938,0.6250,-6.0997e-2,0.5313,0.5625
bottom-left,unresolved,unresolved,,0.0859,0.0781
bottom-right,0.8750,0.1562,+4.3673e-4,0.8594,0.1094
```

(33², t = 20 shown: the primary vortex and the bottom-right eddy are
already in place; the weak bottom-left eddy separates at finer
resolution and longer horizons.)

The CI gate for this case is the coarse rung in
`deep_causality_physics/tests/theories/fluid_dynamics/dec/cavity_tests.rs`
(17² → 33² centerline RMSE 0.2523 → 0.2156, pinned with headroom at the
fast t = 10 spin-up — tests stay quick by design; the time-converged
values 0.252 → 0.133 belong here); this example produces the
full-resolution artifacts — deliberately the same artifacts a
CFD-challenge entry needs, so the comparison tooling exists once.
