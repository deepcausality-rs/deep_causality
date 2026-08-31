# DEC-Native 3D Taylor–Green at Re 1600

The smooth 3D Taylor–Green vortex transitions toward turbulence, and the kinetic-energy
dissipation-rate curve `−dE*/dt*` against the published DNS reference data is the standard
structure-preservation test a new solver is judged by.

Where the sibling [mms_taylor_green_verification](../mms_taylor_green_verification/README.md) example *verifies the
pointwise right-hand side* by manufactured solutions, this example *runs the actual solver*: the
periodic DEC-native incompressible Navier–Stokes march from
`deep_causality_cfd::solvers::dec`. Three DeepCausality abstractions appear together:

- **The DEC solver.** Velocity is an edge 1-form on a cubical torus for the entire solve. Each
  `Rk4` stage evaluates the Leray-projected rate `P(−i_u ω − ν Δ_dR u♭)` — the projector *is* the
  incompressibility equation, so the march never approximates divergence-freeness; it enforces it
  to CG tolerance at every stage. The marching state is the `SolenoidalField` type-state, which
  only a projection can construct: an unprojected field cannot be time-stepped, by construction.
- **The Flow DSL.** The case is declared as a `MarchConfig` through `CfdConfigBuilder` in
  `config.rs`, then composed and run by `CfdFlow::march(&config).on(&manifold).run()`. Seeding and
  marching happen inside that run, not as user-written stages. The run returns
  `Result<Report<FloatType>, PhysicsError>`; any solver failure aborts the march and surfaces as
  that error, which `main` prints with its stage context before exiting non-zero.
- **Precision as a parameter.** The `deep_causality_cfd` config and flow types are generic over
  `R: CfdScalar`; the single `FloatType` alias in `main.rs` picks the `R` for the lattice metric,
  the de Rham seeding, every projection solve, the `Rk4` march, and the energy series alike.
  Values are cast to `f64` only at the display boundary, for CSV presentation.

## The case

The classic Taylor–Green initial field on a periodic `[0, n]³` lattice (unit spacing, wavenumber
`k = 2π/n` playing the role of the unit mode):

```
u =  sin kx · cos ky · cos kz
v = −cos kx · sin ky · cos kz
w =  0
Re = U·L/ν = 1600,  U = 1,  L = 1/k   ⇒   ν = 1/(k·Re)
```

The field is smooth at `t = 0`; vortex stretching steepens it until dissipation peaks near
`t* ≈ 9` (in convective units `t* = t·k·U`). The shape of the `−dE*/dt*` curve — slow start,
steep rise, peak, decay — is the recognizable signature reviewers compare against the published
DNS reference (see `openspec/notes/archive/cfd/references.md`).

## How it works

**Stage 1, seed.** The analytic field is sampled at the lattice vertices (the trigonometry runs at
the working precision), pushed through the de Rham map onto the edges, and projected once at
`t = 0` — a sampled field is divergence-free analytically, not discretely. The flow carries the
projected edge cochain.

**Stage 2, march.** The state re-enters the `SolenoidalField` type-state through its only door
(a near-free projection: the cochain is already solenoidal), then `solver.step` advances it to the
horizon while the energy per volume and the dissipation rate `−dE*/dt*` are collected at every
step, all at the working precision.

## Running it

```sh
cargo run --release -p deep_causality_cfd --example dec_taylor_green_re1600_verification [grid] [t_star_max]

# Multi-core: enable the Rayon feature (forwarded through physics to the
# topology crate's DEC operator loops and CG matvecs):
cargo run --release -p deep_causality_cfd --features parallel \
    --example dec_taylor_green_re1600_verification [grid] [t_star_max]
```

`grid` defaults to 16 — a smoke-scale run that completes in seconds. The reporting resolutions
from the Stage 1 roadmap are 64–128, which take minutes to hours of unpreconditioned CG time
(four CG solves per step is the price of marching the exactly-projected dynamics). Output is CSV
on stdout:

```
t_star,kinetic_energy_per_vol,dissipation_rate
0.0000,0.12024247,0.00000000
0.0785,0.12020751,0.00044514
0.1571,0.12017253,0.00044534
...
```

with a human-readable summary on stderr, so `> curve.csv` captures a clean plot input. The summary
reports the final `E*/E0` and the largest dissipation over the sampled horizon with its time. That
maximum is only the curve's peak when the curve turned over before the horizon; the summary says
which case it is. At the default 16³ / `t*_max = 10` it has not turned over, so the reported figure
is the last sample, still rising.

CI runs this harness as a fast one (`.github/workflows/cfd_verification.yml`), and it exits
non-zero on a broken gate. The two gates are tripwires on the discretisation's own invariants:
kinetic energy never rises step to step, and the horizon ends below `E0`. The dissipation curve
itself is **not** gated. At the default 16³ it is grossly under-resolved, so the CSV is for
plotting against the DNS reference by eye, not for a pass/fail claim.

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the `FloatType` alias, argument parsing, geometry materialization, and the `CfdFlow::march` run. |
| `config.rs` | The case parameters (`RE`, `CFL_DT`, and the derived `wavenumber`/`viscosity`/`dt_star`/`volume`/`build_steps`), the `ft` precision lift, and `build_march_config`, which assembles the `MarchConfig` through `CfdConfigBuilder`. |
| `print_utils.rs` | The CSV artifact and the stderr summary (the one place values are cast to `f64`), plus `verify`: the two structure-preservation gates `main` exits non-zero on. |

## Precision as a parameter

Change one alias in `main.rs` and the whole pipeline — metric, seeding, every CG solve, the
march, the energy series — re-runs at that precision:

```rust
pub type FloatType = f64; // try f32, or Float106 (also add `use deep_causality_num::Float106;`)
```

Exact `f64` specifications (`Re`, the CFL step, π) lift once into `FloatType` through the `ft`
function in `config.rs` and never come back down; every computed quantity stays at that precision.
`ft` routes through `FromPrimitive::from_f64` rather than `From<f64>`, so the same call sites serve
`f32`, `f64`, and `Float106` alike (std has no `f32: From<f64>`).

## Performance

The solver is tracked by the criterion benchmark in `deep_causality_physics`
(`benches/dec_solver_benchmark.rs`), measuring the rate assembly, one Leray
projection, and the full projected step on this example's workload at f64:

```sh
cargo bench -p deep_causality_physics --bench dec_solver_benchmark
cargo bench -p deep_causality_physics --bench dec_solver_benchmark --features parallel
```

Final numbers (Apple Silicon, release). The Leray projection runs the
spectral (FFT) grade-0 Poisson solve from `deep_causality_fft`, and the rate assembly streams through the compiled  DEC stencil tables; flat folded-coefficient gathers instead of CSR traversal and per-cell index arithmetic:

| Grid | Component | Sequential | Parallel |
| --- | --- | ---: | ---: |
| 16³ | rate assembly (`−i_u ω − νΔu♭`, stencils) | 0.29 ms | — |
| 16³ | Leray projection (spectral) | 0.11 ms | — |
| 16³ | full step (4 projected stages + CFL) | 2.0 ms | — |
| 32³ | rate assembly (stencils) | 2.4 ms | 1.9 ms |
| 32³ | rate assembly (generic operators, baseline) | 41.6 ms | — |
| 32³ | Leray projection (spectral) | 0.93 ms | 1.1 ms |
| 32³ | full step | 16.7 ms | 14.5 ms |
| 64³ | rate assembly (stencils) | 20.2 ms | 8.3 ms |
| 64³ | rate assembly (generic operators, baseline) | 354 ms | — |
| 64³ | Leray projection (spectral) | 9.3 ms | 6.8 ms |
| 64³ | full step | 144 ms | 72 ms |

How to read the table:

- **The compiled stencils removed the operator-loop cost.** The rate
  assembly dropped 12× against the in-run generic baseline (the ≥ 2×
  spec gate for the default switch); every gather index and folded
  coefficient (incidence signs × Hodge factors × transport weights ×
  cup signs) is precompiled once per manifold.
- **The step history** (32³, serial): 850 ms when first benchmarked →
  388 ms after the allocation/memoization pass → 137 ms with the spectral
  projection → 30.3 ms with the stencil pipeline → **16.7 ms** after
  memoizing the diagonal Hodge star (the projection's `δω`/`dφ` were
  rebuilding the star matrices on every call) — 51× cumulative.
  Equivalence to the generic operators is pinned at ≤ 100·ε by CI on
  every lattice/metric/precision combination.
- **Spectral diffusion is available but opt-in**
  (`with_spectral_diffusion()`): at 32³ it measures 4.1 ms — slightly
  slower than the stencil viscous passes, so the stencil path stays the
  default; the option matters at larger grids and lower Re.
- The remaining step cost is the four projections plus integrator
  plumbing; parallel gains at 32³ are modest because most passes sit
  under the fan-out thresholds — 64³ engages them (rate 20.2 → 8.3 ms,
  step 144 → 72 ms). The projection is now genuinely FFT-bound (the
  N log N work), with `δω`/`dφ` reduced to plain sparse matvecs against
  memoized matrices. The 64³ full step at 72 ms parallel puts the
  Re-1600 dissipation curve (Stage 1's exit artifact) in overnight
  reach at 128³.
- Wall-bounded and mixed-periodicity uniform boxes now solve their
  grade-0 Poisson problems **directly via DCT-I/DFT** (the
  `neumann-poisson` capability); no-slip flows run the constrained
  projector through Jacobi-preconditioned CG (see the
  `dec_lid_cavity_re1000` example). Plain CG remains only for per-edge
  metrics and degenerate extents.

## Notes for the curious

- The projector sits *inside* the `Rk4` stages, not after them. The post-step (Chorin) placement
  was measured during development to bleed 5–20% of the inviscid energy over `T = 10`, halving
  with `dt` — the textbook first-order splitting dissipation. Marching `∂u♭/∂t = P(rhs)` directly
  removes the splitting error entirely; the solver's inviscid tests then conserve energy and
  helicity to the spatial-residue level.
- The CFL guard enforces both the advective limit `dt ≤ C·dx/max|u|` and the diffusive limit
  `dt ≤ C·dx²/(2Dν)` after every step; at Re 1600 the advective limit governs.
- The opt-in pressure diagnostic (`solver.pressure_diagnostic`) recovers both the Bernoulli and
  static pressure 0-forms from one extra CG solve — not used here, but one call away for a
  pressure-field visualization.
- The solver's performance is tracked by `deep_causality_physics`'s
  `dec_solver_benchmark` (criterion): `cargo bench -p deep_causality_physics
  --bench dec_solver_benchmark [--features parallel]` measures the rate
  assembly, one Leray projection, and the full step at 16³ and 32³.
