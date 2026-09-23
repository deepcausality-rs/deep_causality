# Avionics CFD Verification: Taylor–Green via Manufactured Solutions

This example verifies the incompressible Navier–Stokes right-hand side with the **Method of
Manufactured Solutions (MMS)**, the standard trust check for CFD solvers. It combines three
DeepCausality abstractions on one physical problem:

- **The tangent functor** supplies the exact spatial derivatives the flow kernel needs. `gradient`
  produces the velocity Jacobian `∇u` and the pressure gradient `∇p`; nested duals produce the
  Laplacian `∇²u`. No finite differences appear.
- **The integration operator** (`Rk4`) marches the solution in time and evaluates the
  Navier–Stokes kernel inside the loop.
- **The Flow DSL.** `config.rs` declares the case through `CfdConfigBuilder::verify`, and
  `CfdFlow::verify(&config).run()` runs both verification stages. A kernel failure surfaces as a
  `PhysicsError`; `main` prints it and exits non-zero, as it does when either residual gate fails.

Precision is a parameter: a single type alias re-runs the whole computation at `f32`, `f64`, or
`Float106`.

## The manufactured solution

The [Taylor–Green vortex](https://en.wikipedia.org/wiki/Taylor%E2%80%93Green_vortex) is a
closed-form solution of the incompressible Navier–Stokes equations. In 2-D, embedded in 3-D with
`w = 0`:

```
u =  sin x · cos y · F(t)
v = −cos x · sin y · F(t)
w =  0
p =  (ρ/4)(cos 2x + cos 2y) · F(t)²
F(t) = exp(−2 ν t)
```

Because it is an exact solution, two facts must hold and can be checked numerically:

1. A correct kernel, fed the exact spatial derivatives, returns the exact time derivative. For
   Taylor–Green the convective and pressure terms cancel and only viscous diffusion survives, so
   `∂u/∂t = ν ∇²u = −2 ν u`.
2. A correct time march tracks the exact amplitude decay `a(t) = exp(−2 ν t)`.

The example checks both.

## How it works

**Stage 1, differentiate then kernel.** `deep_causality_cfd::TaylorGreen` writes each velocity
component and the pressure once as scalar-generic fields. `gradient` differentiates them at the
sample point to assemble `∇u`, `∇²u` and `∇p`, which feed `incompressible_ns_rhs`. The flow
compares the kernel's `∂u/∂t` with the exact `−2 ν u`.

**Stage 2, march and verify.** The field keeps its spatial shape and decays in amplitude `a(t)`;
velocity, `∇u` and `∇²u` scale with `a`, pressure with `a²`. The `Rk4` rate field reconstructs
those scaled inputs and calls the *same* kernel at every step, so the march exercises the full
pipeline rather than a closed form. The flow compares the marched amplitude with `exp(−2 ν t)`.

Both residuals must stay below `1e4 · ε` of `FloatType`; `print_utils::verify` checks the gates.

## Running it

```sh
cargo run --release -p deep_causality_cfd --example mms_taylor_green_verification
```

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the `FloatType` alias and the `CfdFlow::verify` run. |
| `config.rs` | The case constants (`NU`, `RHO`, `T0`, `DT`, `STEPS`), the `ft` precision lift, and `build_verify_config`, which assembles the `VerifyConfig` through `CfdConfigBuilder`. |
| `print_utils.rs` | The report rendering and `verify`, the two residual gates `main` exits non-zero on. |

The Taylor–Green field equations and their tangent-functor derivatives live in the crate
(`TaylorGreen`, `src/types/flow_config/manufactured.rs`); the stage logic and the amplitude-march
rate live in `src/types/flow/verify.rs`.

## Precision as a parameter

Change one alias in `main.rs` and the autodiff scalar, the kernel arithmetic, and the `Rk4`
accumulation all re-run at that precision:

```rust
pub type FloatType = f64; // try f32, or Float106 (also add `use deep_causality_num::Float106;`)
```

Physical constants (`ν`, `ρ`, the sample point) stay exact `f64` literals and reach the
computation only through `from_f64`, which lifts them losslessly. Every *computed* quantity,
transcendentals included, runs at `FloatType`, so nothing is produced at f64 and then used at a
wider type.

Precision affects the two checks differently.

**Stage 1 tracks machine epsilon.** The kernel is fed exact autodiff derivatives, so its residual
against `−2 ν u` is pure floating-point roundoff: about `3e-8` at f32, `1e-16` at f64, and `8e-33`
at Float106. Each value falls in lock-step with the precision, which shows the arithmetic runs at
the working type; an f64 downcast anywhere would pin all three near `1e-16`.

**Stage 2 marches in time, where precision matters most.** An `Rk4` step has global
error `~ dt⁴` (fourth order, so halving `dt` cuts the error 16-fold), down to a roundoff floor set by
the working type. Refining `dt` spends the first budget; widening `FloatType` lowers the second. To
watch both at once, march to a fixed `T = 1` over a power-of-two number of steps, so that `dt = 2⁻ᵏ`
is exactly representable and `steps · dt` is exactly `1` (the next section explains why):

| steps | `dt` | f32 error | f64 error | Float106 error |
| --- | --- | --- | --- | --- |
| 128 | 2⁻⁷ | 2.4e-7 | 2.2e-16 | 2.81e-16 |
| 256 | 2⁻⁸ | 3.6e-7 | 4.4e-16 | 1.76e-17 |
| 512 | 2⁻⁹ | 6.0e-8 | 6.7e-16 | 1.10e-18 |
| 1024 | 2⁻¹⁰ | 1.8e-7 | 4.4e-16 | 6.86e-20 |
| 2048 | 2⁻¹¹ | 1.8e-7 | 2.2e-16 | 4.29e-21 |
| 4096 | 2⁻¹² | 6.0e-8 | 2.2e-16 | 2.68e-22 |
| 8192 | 2⁻¹³ | 4.2e-7 | 1.3e-15 | 1.67e-23 |
| 16384 | 2⁻¹⁴ | 2.2e-6 | 4.3e-15 | 1.05e-24 |
| 32768 | 2⁻¹⁵ | 9.2e-6 | 1.6e-15 | 6.54e-26 |
| 65536 | 2⁻¹⁶ | 1.2e-4 | 6.9e-15 | 4.09e-27 |
| 131072 | 2⁻¹⁷ | 3.9e-4 | 6.6e-15 | 2.56e-28 |
| 262144 | 2⁻¹⁸ | 1.4e-3 | 2.2e-15 | 1.48e-29 |

### Reading the table

**f32 and f64 are roundoff-bound from the first row.** At 128 steps the Rk4 truncation error is
about `2.8e-16`, as the Float106 column shows. That lies far below the f32 floor (`~1e-7`) and at
the f64 floor (`~2e-16`), so neither type can show the integrator converging: refining `dt` only
stirs the roundoff floor. Every extra step adds its own roundoff, so past a few thousand steps the
two low-precision columns drift *upward*. f32 degrades from `~1e-7` to `1.4e-3` by 262144 steps;
f64 wanders from `2e-16` up to `~7e-15`. At these precisions, more steps eventually buy less
accuracy.

**Only the Float106 column shows the method working.** Its roundoff floor (`~1e-32`) lies far
beneath the truncation error across the whole sweep, so truncation dominates throughout. The error
falls by exactly `16×` per halving of `dt`, the fourth-order signature, for thirteen orders of
magnitude from `2.8e-16` to `1.5e-29`. The final ratio (`17.3×` rather than `16×`) is roundoff
beginning to register near `~1e-29`. Float106 delivers roughly 29 verified digits here, about
thirteen orders of magnitude below where f64 stalls.

### Why dyadic time matters more as precision rises

Float106 precision depends on the power-of-two step count. The march advances `steps` times by
`dt` and lands at time `steps · dt`, which must equal the `T` the reference uses. In floating point
the two agree only when `dt` is exactly representable, and `dt = T / steps` is exact precisely when
`steps` is a power of two (`dt = 2⁻ᵏ`). For an off-grid count such as the 200 steps of the default
run, `dt = 0.005` is not a binary fraction, `steps · dt` misses `1` by about `2e-17`, and at the
decay rate `2 ν` that time offset becomes a fixed `~1.9e-18` error in the amplitude.

At f32 (`~1e-7`) and f64 (`~1e-16`) this offset lies far below the roundoff floor and never shows.
At Float106 it sits four orders of magnitude *above* machine epsilon and masquerades as a
convergence floor: the march appears to stall near `1.9e-18` while it still converges and only the
reference time is wrong. The floor is easy to mistake for a limit of `exp`, yet Float106's `exp` is
accurate to its full ~32 digits (`exp(x)·exp(−x)` returns `1` to roughly `2e-32`). The reference
*time* causes it.

The example guards against this: it evaluates the reference at `exp(−2 ν · t_final)` with
`t_final = dt · steps`, the instant the march reaches, so even the off-grid 200-step default stays
artifact-free. The power-of-two sweep above keeps the nominal and reached times identical by
construction, which is why the Float106 column converges cleanly.

Raising precision lowers the error floor and also raises the standard everything else must meet.
The reference value, the time grid, the representability of the constants: details that f64 hides
under its own roundoff become first-order concerns at Float106. The example reaches 29 digits only
because the time grid is exact and the reference is read at the instant the march arrives.

## Notes for the curious

- `Float106` is a double-double type with roughly 32 decimal digits. `from_f64` is the
  `FromPrimitive` trait method.
- The kernel is `deep_causality_cfd::incompressible_ns_rhs`, which composes the convective,
  pressure and viscous sub-kernels from `deep_causality_physics`. The wider fluid-dynamics set
  (vorticity, strain rate, Q-criterion, and related diagnostics) lives in `deep_causality_physics`.
