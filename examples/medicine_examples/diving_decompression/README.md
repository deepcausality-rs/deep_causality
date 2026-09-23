# SCUBA Decompression Planner

This example plans a dive with the Bühlmann ZH-L16C algorithm over sixteen tissue compartments,
so the diver surfaces while the dissolved nitrogen stays in solution, and tracks the CNS oxygen
clock alongside.

```bash
cargo run -p medicine_examples --example diving_decompression
```

## What the run prints

A dive table from 10 m to 50 m, then one dive that exceeds its limit so the planner has a
schedule to produce.

Each table row holds the bottom for that depth's no-decompression limit, capped at twenty minutes
so every row runs in the same handful of milliseconds. The rows from 10 m to 30 m therefore hold
the bottom for twenty minutes and print the longer limit beside them; from 35 m down the limit is
under the cap and the row holds it in full.

```text
Controlling compartment at the bottom
  #4   half-time  18.5 min   ceiling 7.1 m

Decompression schedule
  mandatory stop    4 min @ 9 m
  safety stop       3 min @ 5 m
```

Fifty minutes at thirty metres is twice the limit for that depth. Compartment 4 governs the
ascent, as physiology predicts: over a fifty-minute exposure the fast compartments have saturated
and a mid-speed one becomes binding. The stop at 9 m is held, two
minutes at a time, until the ceiling has risen past 6 m, where the next step lands. Below the
clearance depth of 6 m the ascent runs to the safety stop, and from there to the surface.

## The physics

**Tissue loading, the Schreiner equation.** A compartment approaches the inspired pressure
exponentially, at its own rate.

```text
p(t) = p_inspired + (p_initial − p_inspired)·e^(−kt)      k = ln2 / half_time
```

**The ascent ceiling.** Bühlmann's M-value line gives the tension a compartment tolerates at
ambient pressure `P` as `P/b + a`. A gradient factor admits only `gf` of the gap between the
ambient pressure and that line:

```text
tension = P + gf·(P/b + a − P)
```

Solving for `P` gives the shallowest pressure the compartment allows; ten metres of seawater
is one bar. `a` sets the intercept and `b` the slope, so changing either published coefficient
moves the ceiling and can change which compartment governs.

**The CNS oxygen clock.** Oxygen partial pressure rises with depth, and the NOAA table gives a
tolerated exposure per pressure. Below 1.0 bar the clock does not run.

## What the code demonstrates

Four categorical operations carry the program, and all four are in `main.rs`:

| Operation | Pairs or reduces | For |
|---|---|---|
| `try_step` | diver state to the next one | the dive as a chain of four phases: descend, hold, ascend to the safety stop, surface |
| `zip_with` | tension against its own half-time | loading all sixteen compartments |
| `zip_with` | tension against its M-value coefficients | the ceiling each compartment imposes |
| `fold` | sixteen ceilings to the binding one | the compartment that governs the ascent |

Each compartment carries its own constants, so every compartment computation is a **pairing**.
`ZipTensorWitness::zip_with` walks two tensors slot by slot: the laws in `model.rs` are written
once for one compartment, with no compartment index, and the witness applies them to all sixteen.

A fifth abstraction is the tangent functor. A dive computer watches the gas-loading rate `dp/dt`,
and here it comes from one evaluation over `Dual`. The run prints it beside the analytic rate
`k·(p_inspired − p)` as a check; the two agree to the precision of the working scalar.

## Precision is a parameter

One alias in `main.rs` sets the working scalar for the whole program.

```rust
pub type FloatType = Float106;
```

Every constant in the model is declared at that type through `const_scalar_from_int!` and
`const_scalar_from_float!`, so the compiler resolves them against the alias and no conversion runs
at any call site. Switching the alias re-declares all of them.

The alias defaults to `Float106` rather than `f64` so that a hard-coded `f64` fails to compile.
While the alias *is* `f64` the two types coincide and such a line goes unnoticed.

The autodiff agreement shows what changes with the scalar:

| Scalar | agreement | ceiling | stop at 9 m |
|---|---|---|---|
| `BFloat16` | 9.8e-4 | 6.9 m | 2 min |
| `f32` | 0.0 | 7.1 m | 4 min |
| `f64` | 2.8e-17 | 7.1 m | 4 min |
| `Float106` | 1.5e-33 | 7.1 m | 4 min |  <- the default

All four reach the same dive plan: one mandatory stop at 9 m, then the safety stop. `BFloat16`
carries two significant decimal digits, reads the ceiling at 6.9 m and clears the stop after one
two-minute hold; the three wider scalars read 7.1 m and hold it for two.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the constants `main` uses, the phase chain, and the two witness operations |
| `model.rs` | the Bühlmann constants, the state types, the physiology, and the differentiable curve |
| `utils_print.rs` | the presentation, and the only `lower` calls in the example |

## Scope

This example demonstrates the algorithm; it is not a dive computer. It uses a single gradient
factor rather than a GF-low to GF-high gradient across the ascent, takes its no-decompression
limits from a coarse depth table rather than from the compartment model, and plans a single dive
with no repetitive-dive residual nitrogen. Do not dive it.

## Adaptation

- **Nitrox.** Change `F_N2` and `F_O2`; the gas laws read them directly.
- **A GF gradient.** Add a `GF_LOW` constant and interpolate toward `GF_HIGH` by depth inside the
  ascent phase, so the ceiling tightens as the diver comes up.
- **Limits from the model.** Replace the depth table by searching for the bottom time at which the
  ceiling first reaches the surface.
