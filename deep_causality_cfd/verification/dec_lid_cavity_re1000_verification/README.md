# Lid-Driven Cavity at Re 1000 — DEC-Native, Wall-Bounded

This example runs the lid-driven cavity at Re 1000 on the DEC
Navier–Stokes solver and compares it with Ghia et al. The case is the
standard wall-bounded incompressible benchmark: a unit box, three no-slip
walls, and a lid sliding at constant speed. At Re 1000 the steady state
carries a primary vortex near the center and counter-rotating eddies in
the bottom corners, tabulated in:

> Ghia, U., Ghia, K. N., Shin, C. T. (1982). *High-Re solutions for
> incompressible flow using the Navier–Stokes equations and a multigrid
> method.* J. Comput. Phys. 48, 387–411.

## What this example demonstrates

The example exercises three wall components of the DEC Navier–Stokes solver together:

- **No-slip walls.** All four boundaries are walls. Tangential edge
  coefficients are pinned to zero, and the lid (the y-max face) carries
  the tangential velocity `U = 1` through `DecNsSolver::with_moving_wall`.
- **The constrained Leray projector.** Every stage rate and the step
  re-entry project onto the *intersection* of the divergence-free and
  no-slip subspaces (the M-orthogonal intersection projection), so both
  invariants hold exactly at every step boundary. The masked grade-0
  solve runs Jacobi-preconditioned CG.
- **The boundary-corrected Hodge star.** The clipped dual volumes at the
  walls (faces ½, corners ¼ in 2D) make the wall operators M-symmetric.

## Usage

```text
cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification [grid] [t_end]
cargo run --release -p deep_causality_cfd --example dec_lid_cavity_re1000_verification trend
```

- `grid` defaults to **65** (minutes of runtime, clear vortex structure).
- The **reporting resolution is 129** (Ghia's own grid), with
  `t_end ≥ 150` for fully developed corner eddies. That run takes hours
  of Jacobi-PCG time; spectral preconditioning of the masked solve is
  the documented escalation if it becomes routine.
- A 33²/`t_end = 20` smoke run takes ~15 s.
- **`trend`** runs the refinement-trend verification: 17² → 33² at the
  time-converged horizon (`t = 60`), gated (RMSE 0.32 / 0.20, strict
  decrease) and exits nonzero on violation (~1 min). It lives here
  rather than in the test suite so tests stay fast.

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
in place; the weak bottom-left eddy separates at finer resolution and
longer horizons.)

The CI gate for this case is `coarse_cavity_gates_against_ghia` in
`deep_causality_cfd/tests/solvers/dec/cavity_tests.rs`. It marches the
17² cavity to t = 10 and asserts a pooled centerline RMSE below 0.32
(0.2523 measured). No CI test runs the 33² rung. This example produces
the full-resolution artifacts, the same ones a CFD-challenge entry needs.
