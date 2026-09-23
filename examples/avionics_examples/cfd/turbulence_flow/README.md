# Turbulence predictability: the forecast horizon of a chaotic flow

This example measures how far ahead a turbulent flow can be forecast, and shows that arithmetic
precision moves that horizon. Aircraft must cope with turbulence (atmospheric convection, thermals,
storm cells, wake and clear-air turbulence) for structural loads, ride quality, and control.
Turbulent flow is chaotic, so its forecast has a hard predictability horizon: beyond some lead time
the prediction is worthless however good the model, because the flow amplifies the smallest error
exponentially.

The testbed is the Lorenz system, Saltzman and Lorenz's three-mode truncation of Rayleigh-Bénard
convection and the original model of atmospheric convective turbulence. It runs in a few lines yet
carries the property that caps every turbulence forecast: exponential growth of any perturbation,
at a rate set by the leading Lyapunov exponent `λ ≈ 0.906`.

## Why this matters in avionics

Turbulence forecasting is a chaotic-prediction problem, in the same family as weather. The flow
has a finite predictability window, and any system that consumes a turbulence forecast (gust-load
alleviation, route planning around convection, ride control) must know how long that window is and
what sets it. The size of the smallest error in the computation sets it. A finer step shrinks
truncation error, but underneath lies the irreducible floor of floating-point roundoff. A chaotic
flow magnifies that floor exponentially, so arithmetic precision caps the achievable forecast
horizon, and raising precision buys lead time.

## What caps the forecast, in one equation

Run the same `Rk4` scheme at the same step `dt` at two precisions. The two computations solve an
identical discrete map and differ only by roundoff, of size machine epsilon `ε`. Because the scheme
and step match, the truncation error is common to both and cancels in their difference, so the
state-space distance between them is the roundoff growth alone. That seed of size `ε` grows like
`e^{λ t}` and reaches the separation `L` at which the forecast is declared lost at

```
t_horizon ≈ ln(L / ε) / λ
```

This example sets `L = 1`, the unit separation threshold in `main.rs`, and `Report::horizon_law`
evaluates the law at that value: `t ≈ −ln(ε)/λ`. The choice is conservative. The attractor spans
tens of state-space units (the divergence table below reaches `3.3e1`), so a forecast that has
drifted one unit counts as lost while it still looks like the right flow. Declaring the loss at the
attractor scale instead would raise every horizon by `ln(L)/λ`, about 2.5 time units per decade,
and would not change their order.

Past that time the forecast still shows a plausible turbulent state on the same attractor, but no
longer the trajectory that follows from the initial condition. The horizon grows linearly in the
number of correct digits, so each step up in precision extends the trustworthy window by a fixed
amount.

## Running it

```sh
cargo run --release -p avionics_examples --example turbulence_flow
```

The divergence table and horizon summary from a run, verbatim (state-space distance to the Float106
forecast; truncation cancels, so this is roundoff growth):

```
     t    |  f32 vs F106  |  f64 vs F106
  --------+---------------+--------------
     5.0  |     2.69e-5   |    2.31e-14
    10.0  |     1.48e-4   |    1.09e-13
    15.0  |     4.49e-3   |    4.62e-12
    20.0  |     1.94e-1   |    1.96e-10
    25.0  |      1.99e1   |     4.01e-8
    30.0  |      1.75e1   |     2.97e-6
    35.0  |      2.49e1   |     2.21e-3
    40.0  |      2.69e0   |     4.84e-3
    45.0  |      1.71e1   |      2.00e0
    50.0  |      1.17e1   |      1.55e1
    55.0  |      3.31e1   |      1.67e1

Forecast horizon (lead time before the flow state is off by one state-space unit):
  f32        t ≈ 21.5
  f64        t ≈ 44.5
  Float106   beyond T=60 here; the law below puts it near t ≈ 81
```

The full run is committed as `output.txt`; rerun it to diff.

## Reading it

**An f64 forecast of this flow is trustworthy to about `t ≈ 44`,** then becomes fiction. Its
divergence starts near `1e-14` (the `~2e-16` roundoff seed, already amplified a little) and climbs
by roughly `e^{λ t}`. It crosses the unit threshold at `t = 44.5` and saturates at attractor scale,
order ten units, a few time units after that (`1.55e1` at `t = 50`). No smaller `dt` helps,
because roundoff sets the limit.

**f32 fails far sooner, at `t ≈ 21`,** because its seed (`~1e-7`) is nine orders larger.
**Float106 reaches `t ≈ 81`** by the same law, roughly double f64. The measured spacing between the
horizons, about 23 time units per ~16 digits, matches `ln(1/ε)/λ`: precision buys forecast lead
time linearly.

```
    f32  (ε≈1.2e-7 ):  t ≈ 17.6
    f64  (ε≈2.2e-16):  t ≈ 39.8
    F106 (ε≈1.0e-32):  t ≈ 81.3
```

Past a forecast's own horizon the numbers fluctuate, because the distance then measures the gap
between two unrelated points on the attractor; only the first crossing of the threshold carries
meaning. The growth is bursty rather than smooth, since the local stretching rate varies along
the orbit, and `λ` is the long-run average.

## The precision angle

For most CFD the precision floor is irrelevant: discretization and modeling error dwarf `f64`
roundoff, which is why production solvers run in `f64` or even `f32`. Chaotic flow, turbulence
above all, is the exception. There the flow amplifies roundoff without bound, precision caps the
reliable horizon, and `f64` cannot reach the long-range forecast a wider type can. Raising precision
is the established recipe for trustworthy chaotic-flow trajectories (Liao's Clean Numerical
Simulation uses hundreds of digits for converged turbulence references). This example applies that
idea in miniature: `Float106`, reached by changing a type parameter, roughly doubles the horizon
`f64` can certify.

## How it is built

The example combines three DeepCausality pieces with little glue:

- **The Arrow calculus.** `Rk4` is the integration operator; a forecast is one `iterate_n` call.
- **Precision as a parameter.** The rate field and the march are written once over the `Scalar`
  bound and instantiated at `f32`, `f64`, and `Float106`. One flow model, three precisions.
- **The causal monad.** `PropagatingEffect` sequences *simulate* then *analyse*, short-circuiting
  through the error channel if a trajectory leaves the finite range.

| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the monadic *simulate → analyse* pipeline. |
| `model.rs` | The scalar-generic `Vec3`, the convective rate field, the `Rk4` march, the cross-precision distance and horizon helpers, and the report types. |
| `print_utils.rs` | Presentation only: the divergence table and the horizon summary. |

The model layer holds all the physics: a three-line rate field and a one-line march, written once
and run at three precisions. Everything else is measurement and presentation.
