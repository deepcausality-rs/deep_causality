# Avionics CFD Verification: Taylor–Green via Manufactured Solutions

A worked avionics example that verifies an incompressible-flow right-hand side with the **Method
of Manufactured Solutions (MMS)**, the standard way CFD solvers earn trust. It brings three
DeepCausality abstractions together on one physical problem:

- **The tangent functor** supplies the exact spatial derivatives the flow kernel needs. `gradient`
  produces the velocity Jacobian `∇u` and the pressure gradient `∇p`; nested duals produce the
  Laplacian `∇²u`. No finite differences appear anywhere.
- **The integration operator** (`Rk4`) marches the solution in time, with the Navier–Stokes kernel
  evaluated inside the loop.
- **The causal monad** (`PropagatingEffect`) sequences the two verification stages. Each stage
  binds onto the previous one, and a kernel failure short-circuits the chain through the effect's
  error channel.

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

**Stage 1, differentiate then kernel.** Each velocity component and the pressure are written once
as scalar-generic fields. `gradient` differentiates them at the sample point to assemble `∇u`,
`∇²u` and `∇p`, which feed `incompressible_ns_rhs_kernel`. The kernel's `∂u/∂t` is compared with
the exact `−2 ν u`.

**Stage 2, march and verify.** The field keeps its spatial shape and decays in amplitude `a(t)`;
velocity, `∇u` and `∇²u` scale with `a`, pressure with `a²`. The `Rk4` rate field reconstructs
those scaled inputs and calls the *same* kernel at every step, so the march exercises the full
pipeline rather than a closed form. The marched amplitude is compared with `exp(−2 ν t)`.

## Running it

```sh
cargo run --release -p deep_causality_cfd --example mms_taylor_green_verification
```

## File layout

| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the `FloatType` alias and the monadic pipeline that sequences the two stages. |
| `model.rs` | The Taylor–Green field equations, the tangent-functor plumbing (`∇u`, `∇²u`, `∇p`), the Navier–Stokes RHS wrapper, the amplitude-march rate, and the exact-solution references. |
| `print_utils.rs` | Presentation only: the report formatting. |

## Precision as a parameter

Change one alias in `main.rs` and the autodiff scalar, the kernel arithmetic, and the `Rk4`
accumulation all re-run at that precision:

```rust
pub type FloatType = f64; // try f32, or Float106 (also add `use deep_causality_num::Float106;`)
```

Physical constants (`ν`, `ρ`, the sample point) stay exact `f64` literals; they reach the
computation only through `from_f64`, which lifts them losslessly. Every *computed* quantity runs at
`FloatType`, including the transcendentals, so nothing is produced at f64 and then used at a wider
type.

The verification has two checks, and precision tells a different story in each.

**Stage 1 tracks machine epsilon.** The kernel is fed exact autodiff derivatives, so its residual
against `−2 ν u` is pure floating-point roundoff: about `3e-8` at f32, `1e-16` at f64, and `8e-33`
at Float106. Each value falls in lock-step with the precision, which is the evidence that the
arithmetic genuinely runs at the working type; an f64 downcast anywhere would pin all three near
`1e-16`.

**Stage 2 marches in time, and that is where precision matters most.** An `Rk4` step has global
error `~ dt⁴` (fourth order, so halving `dt` cuts the error 16-fold), down to a roundoff floor set by
the working type. Refining `dt` spends the first budget; widening `FloatType` lowers the second. To
watch both at once, march to a fixed `T = 1` over a power-of-two number of steps, so that `dt = 2⁻ᵏ`
is exactly representable and `steps · dt` is exactly `1` (the reason that matters comes just below):

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

**f32 and f64 are roundoff-bound from the very first row.** Even at 128 steps the Rk4 truncation
error is about `2.8e-16`; you can read it straight off the Float106 column, which lays the truncation
error bare. That is already far below the f32 floor (`~1e-7`) and sits right at the f64 floor
(`~2e-16`), so neither type can watch the integrator converge: refining `dt` only stirs the roundoff
floor. Worse, every extra step contributes its own roundoff, so past a few thousand steps the two
low-precision columns drift *upward*. f32 degrades from `~1e-7` to `1.4e-3` by 262144 steps; f64
wanders from `2e-16` up to `~7e-15`. At these precisions, more steps eventually buy less accuracy.

**Float106 is the only column that shows the method working.** Its roundoff floor (`~1e-32`) lies far
beneath the truncation error across the whole sweep, so truncation dominates the entire way down. The
error falls by exactly `16×` per halving of `dt`, the fourth-order signature, holding for thirteen
orders of magnitude from `2.8e-16` to `1.5e-29`. The true Float106 floor is still below that; the
final ratio (`17.3×` rather than `16×`) is roundoff just beginning to register near `~1e-29`. So
Float106 delivers roughly 29 verified digits here, about thirteen orders of magnitude below where
f64 stalls.

### Why dyadic time matters more as precision rises

The power-of-two step count is crititcal for Float106 precision. The march advances `steps` times by `dt` and lands at
time `steps · dt`, which must equal the `T` the reference uses. In floating point the two agree only
when `dt` is exactly representable, and `dt = T / steps` is exact precisely when `steps` is a power of
two (`dt = 2⁻ᵏ`). For an off-grid count such as the 200 steps of the default run, `dt = 0.005` is not
a binary fraction, `steps · dt` misses `1` by about `2e-17`, and at the decay rate `2 ν` that time
offset becomes a fixed `~1.9e-18` error in the amplitude.

Whether that offset matters depends entirely on the precision. At f32 (`~1e-7`) and f64 (`~1e-16`) it
is buried far below the roundoff floor and never shows. At Float106 it sits four orders of magnitude
*above* machine epsilon, so it stops being negligible and masquerades as a convergence floor: the
march appears to stall near `1.9e-18` when in truth it is still converging and only the reference time
is wrong. This is exactly the phantom floor an earlier draft of this file mistook for a limit of
`exp`. The closed-form `exp` is not the bottleneck; Float106's `exp` is accurate to its full ~32
digits (`exp(x)·exp(−x)` returns `1` to roughly `2e-32`). The reference *time* was the culprit.

The example guards against this directly: it evaluates the reference at `exp(−2 ν · t_final)` with
`t_final = dt · steps`, the instant the march truly reaches, so even the off-grid 200-step default
stays artifact-free. The power-of-two sweep above is the same idea made airtight, keeping the nominal
and reached times identical by construction, which is why the Float106 column converges cleanly
instead of stalling.

That is the through-line of the whole table. Raising precision does not simply lower the error floor;
it raises the standard everything else must meet. The reference value, the time grid, the
representability of the constants: details that f64 hides under its own roundoff become first-order
concerns at Float106. The example earns its 29 digits only because the time grid is exact and the
reference is read at the same instant the march arrives.

## Notes for the curious

- `Float106` is a double-double type with roughly 32 decimal digits. Some of its methods are
  inherent and shadow the corresponding traits, so the example reaches `from_f64`, `abs` and `max`
  through fully-qualified or `Scalar`-bounded helpers to stay precision-generic.
- The kernel is `deep_causality_physics::incompressible_ns_rhs_kernel`, which composes the
  convective, pressure and viscous sub-kernels. The wider fluid-dynamics set
  (vorticity, strain rate, Q-criterion, and related diagnostics) lives in the same crate.
