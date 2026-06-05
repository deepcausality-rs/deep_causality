# Turbulence predictability: the forecast horizon of a chaotic flow

An avionics example about turbulence, and specifically about how far ahead a turbulent flow can be
forecast at all. Aircraft must cope with turbulence (atmospheric convection, thermals, storm cells,
wake and clear-air turbulence) for structural loads, ride quality, and control. Turbulent flow is
chaotic, so a forecast of it has a hard predictability horizon: beyond some lead time the prediction
is worthless however good the model, because the flow amplifies the smallest error exponentially.
This example finds that horizon and shows the one knob that moves it.

The testbed is the Lorenz system, Saltzman and Lorenz's three-mode truncation of Rayleigh-Bénard
convection. It is the original model of atmospheric convective turbulence and the birthplace of
chaos theory, small enough to run in a few lines yet carrying the property that caps every
turbulence forecast: exponential growth of any perturbation, at a rate set by the leading Lyapunov
exponent `λ ≈ 0.906`.

## Why this matters in avionics

Turbulence forecasting is a chaotic-prediction problem, in the same family as weather. You cannot
run it arbitrarily far ahead; the flow has a finite predictability window, and engineering anything
that consumes a turbulence forecast (gust-load alleviation, route planning around convection, ride
control) means knowing how long that window is and what sets it. The answer is uncomfortable: the
horizon is governed by the size of the smallest error in the computation. Truncation error you can
shrink with a finer step, but underneath it lies the irreducible floor of floating-point roundoff.
A chaotic flow magnifies that floor exponentially, so the achievable forecast horizon is capped by
arithmetic precision. Precision is not a detail here; it is the lever that buys lead time.

## What caps the forecast, in one equation

Run the same `Rk4` scheme at the same step `dt` at two precisions. The two computations solve an
identical discrete map and differ only by roundoff, of size machine epsilon `ε`. Because the scheme
and step match, the truncation error is common to both and cancels in their difference, so the
state-space distance between them is the roundoff growth alone. That seed of size `ε` grows like
`e^{λ t}` and reaches the scale of the attractor (`L ≈ 10`) at

```
t_horizon ≈ ln(L / ε) / λ
```

Past that time the forecast has lost every correct digit. It is still a plausible turbulent state,
but no longer the one that follows from the initial condition. The horizon grows linearly in the
number of correct digits, so each step up in precision extends the trustworthy window by a fixed
amount.

## Running it

```sh
cargo run -p avionics_examples --example turbulence_flow
```

Sample output (state-space distance to the Float106 forecast; truncation cancels, so this is
roundoff growth):

```
     t    |  f32 vs F106  |  f64 vs F106
  --------+---------------+--------------
     5.0  |     2.69e-5   |    2.31e-14
    10.0  |     1.48e-4   |    1.09e-13
    20.0  |     1.94e-1   |    1.96e-10
    25.0  |      1.99e1   |     4.01e-8
    40.0  |      2.69e0   |     4.84e-3
    45.0  |      1.71e1   |      2.00e0

Forecast horizon (lead time before the state is off by one state-space unit):
  f32        t ≈ 21.5
  f64        t ≈ 44.5
  Float106   beyond T=60 here; the law puts it near t ≈ 81
```

## Reading it

**An f64 forecast of this flow is trustworthy to about `t ≈ 44`,** then becomes fiction. Its
divergence starts near `1e-14` (the `~2e-16` roundoff seed, already amplified a little) and climbs
by roughly `e^{λ t}` until it saturates around the attractor diameter near `t ≈ 45`. No smaller
`dt` helps; the wall is roundoff, not truncation.

**f32 fails far sooner, at `t ≈ 21`,** because its seed (`~1e-7`) is nine orders larger.
**Float106 reaches `t ≈ 81`** by the same law, roughly double f64. The measured spacing between the
horizons, about 23 time units per ~16 digits, matches `ln(1/ε)/λ`: precision buys forecast lead
time linearly.

```
    f32  (ε≈1.2e-7 ):  t ≈ 17.6
    f64  (ε≈2.2e-16):  t ≈ 39.8
    F106 (ε≈1.0e-32):  t ≈ 81.3
```

Past a forecast's own horizon the numbers fluctuate, because the distance is then just the gap
between two unrelated points on the attractor; only the first crossing of the threshold is
meaningful. The growth is bursty rather than smooth, since the local stretching rate varies along
the orbit, and `λ` is the long-run average.

## The precision angle

For most CFD the precision floor is irrelevant: discretization and modeling error dwarf `f64`
roundoff, which is why production solvers are `f64` or even `f32`. Chaotic flow is the exception,
and turbulence is the headline case. There, roundoff is amplified without bound, the reliable
horizon is capped by precision, and `f64` simply cannot reach the long-range forecast that a wider
type can. Extending the horizon by raising precision is the established recipe for trustworthy
chaotic-flow trajectories (Liao's Clean Numerical Simulation, which uses hundreds of digits for
converged turbulence references). This example is that idea in miniature: `Float106`, reached by
changing a type parameter, roughly doubles the horizon `f64` can certify.

## How it is built

The whole example is three DeepCausality pillars and almost no glue:

- **The Arrow calculus.** `Rk4` is the integration operator; a forecast is one `iterate_n` call.
- **Precision as a parameter.** The rate field and the march are written once over the `Scalar`
  bound and instantiated at `f32`, `f64`, and `Float106`. One flow model, three precisions.
- **The causal monad.** `PropagatingEffect` sequences *simulate* then *analyse*, short-circuiting
  through the error channel if a trajectory leaves the finite range.

| File | Responsibility |
| --- | --- |
| `main.rs` | The workflow: the monadic *simulate → analyse* pipeline and the report types. |
| `model.rs` | The scalar-generic `Vec3`, the convective rate field, the `Rk4` march, and the cross-precision distance and horizon helpers. |
| `print_utils.rs` | Presentation only: the divergence table and the horizon summary. |

The model layer is the whole of the physics: a three-line rate field and a one-line march, written
once and run at three precisions. Everything else is measurement and presentation.
