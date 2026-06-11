# DEC-Native 3D Taylor–Green at Re 1600

The smooth 3D Taylor–Green vortex transitions toward turbulence, and the kinetic-energy
dissipation-rate curve `−dE*/dt*` against the published DNS reference data is the standard
structure-preservation test a new solver is judged by.

Where the sibling [cfd_taylor_green](../cfd_taylor_green/README.md) example *verifies the
pointwise right-hand side* by manufactured solutions, this example *runs the actual solver*: the
periodic DEC-native incompressible Navier–Stokes march from
`deep_causality_physics::theories::fluid_dynamics::dec`. Three DeepCausality abstractions appear
together:

- **The DEC solver.** Velocity is an edge 1-form on a cubical torus for the entire solve. Each
  `Rk4` stage evaluates the Leray-projected rate `P(−i_u ω − ν Δ_dR u♭)` — the projector *is* the
  incompressibility equation, so the march never approximates divergence-freeness; it enforces it
  to CG tolerance at every stage. The marching state is the `SolenoidalField` type-state, which
  only a projection can construct: an unprojected field cannot be time-stepped, by construction.
- **The causal flow.** The program is a two-stage `CausalFlow` chain — seed, then march — where
  each stage is a plain `Value -> Result<U, CausalityError>` composed with `try_step`. A CG
  failure or CFL violation short-circuits the chain through the error channel with the failing
  step in the message.
- **Precision as a parameter.** Every model struct is generic over `R: RealField`; the single
  `FloatType` alias in `main.rs` selects the precision for the lattice metric, the de Rham
  seeding, every projection CG solve, the `Rk4` march, and the energy series alike. Values are
  cast to `f64` only at the display boundary, for CSV presentation.

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
DNS reference (see `openspec/notes/cfd/references.md`).

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
cargo run --release -p avionics_examples --example dec_taylor_green_re1600 [grid] [t_star_max]

# Multi-core: enable the Rayon feature (forwarded through physics to the
# topology crate's DEC operator loops and CG matvecs):
cargo run --release -p avionics_examples --features parallel \
    --example dec_taylor_green_re1600 [grid] [t_star_max]
```

`grid` defaults to 16 — a smoke-scale run that completes in seconds. The reporting resolutions
from the Stage 1 roadmap are 64–128, which take minutes to hours of unpreconditioned CG time
(four CG solves per step is the price of marching the exactly-projected dynamics). Output is CSV
on stdout:

```
t_star,kinetic_energy_per_vol,dissipation_rate
0.0000,0.10669417,0.00000000
0.1571,0.10662775,0.00042287
...
```

with a human-readable summary (final `E*/E0`, peak dissipation and its time) on stderr, so
`> curve.csv` captures a clean plot input.

This is a code example, not a test host: CI never executes it, and correctness is gated by the
solver's own validation ladder in `deep_causality_physics` (Taylor–Green convergence tables,
inviscid invariants, the double shear layer).

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the `FloatType` alias, argument parsing, and the `CausalFlow` chain that sequences seed and march. |
| `model.rs` | The precision-generic model: the lattice manifold, the solver configuration (`ν` from Re at `R`), the two flow stages, and the `Sample<R>`/`Report<R>` carriers. |
| `print_utils.rs` | Presentation only: the CSV artifact and the stderr summary (the one place values are cast to `f64`). |

## Precision as a parameter

Change one alias in `main.rs` and the whole pipeline — metric, seeding, every CG solve, the
march, the energy series — re-runs at that precision:

```rust
pub type FloatType = f64; // try f32, or Float106 (also add `use deep_causality_num::Float106;`)
```

Exact `f64` specifications (`Re`, the CFL step, π) lift once into `R` through the `flt!` macro in
`model.rs` and never come back down; every computed quantity stays at `R`. The macro routes
through `FromPrimitive` rather than `From<f64>` so the same call sites serve `f32`, `f64`, and
`Float106` alike (std has no `f32: From<f64>`).

## Performance

The solver is tracked by the criterion benchmark in `deep_causality_physics`
(`benches/dec_solver_benchmark.rs`), measuring the rate assembly, one Leray
projection, and the full projected step on this example's workload at f64:

```sh
cargo bench -p deep_causality_physics --bench dec_solver_benchmark
cargo bench -p deep_causality_physics --bench dec_solver_benchmark --features parallel
```

Final numbers (Apple Silicon, release):

| Grid | Component | Sequential | Parallel | Speedup |
| --- | --- | ---: | ---: | ---: |
| 16³ | rate assembly (`−i_u ω − νΔu♭`) | 3.7 ms | 2.3 ms | 1.6× |
| 16³ | Leray projection (one CG solve) | 0.76 ms | 0.81 ms | ≈1× |
| 16³ | full step (4 projected stages + CFL) | 36 ms | 32 ms | 1.1× |
| 32³ | rate assembly | 29.4 ms | 11.7 ms | 2.5× |
| 32³ | Leray projection | 5.9 ms | 6.4 ms | ≈1× |
| 32³ | full step | 429 ms | 388 ms | 1.1× |

How to read the table:

- **The per-cell operator loops parallelize well.** Wedge, interior product,
  de Rham, and sharp fan out over Rayon and carry the rate assembly to 2.5×
  at 32³, growing with the grid.
- **The CG solves are the step's floor and are memory-bound.** A CSR matvec
  row is a handful of multiply-adds, so parallel dispatch per CG iteration
  only pays at large systems; the library thresholds it to engage at
  64³-scale grids and above. At the 64–128³ reporting resolutions — where
  the runtime actually hurts — both the operator loops and the matvecs run
  parallel. Below the thresholds the parallel build falls back to serial
  loops, so small runs pay no fork-join overhead.
- These numbers already include the serial-side optimizations the benchmark
  drove (memoized boundary/coboundary matrices, cache-preserving lattice
  clones, arithmetic cell indexing in place of per-call hash maps, and
  `_of` operator variants that eliminated every per-evaluation scratch
  manifold): the 32³ step measured 850 ms when first benchmarked, 388 ms
  now. The remaining cost is genuine unpreconditioned CG arithmetic — a
  preconditioner is the designated next performance follow-up.

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
