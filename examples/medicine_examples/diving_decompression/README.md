# SCUBA Decompression Planner

A dive plan answers one question: how does a diver surface while the dissolved nitrogen stays in
solution? This example answers it with the Bühlmann ZH-L16C algorithm over sixteen tissue
compartments, and tracks the CNS oxygen clock alongside.

```bash
cargo run -p medicine_examples --example diving_decompression
```

## What the run prints

A dive table from 10 m to 50 m, each depth planned at its no-decompression limit, then one dive
that deliberately exceeds its limit so the planner has a schedule to produce.

```text
Controlling compartment at the bottom
  #4   half-time  18.5 min   ceiling 7.2 m

Decompression schedule
  mandatory stop    2 min @ 9 m
  safety stop       3 min @ 5 m
```

Fifty minutes at thirty metres is twice the limit for that depth. Compartment 4 governs the
ascent, which is the physiologically right answer: over a fifty-minute exposure the fast
compartments have saturated and a mid-speed one becomes binding.

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

Solving for `P` gives the shallowest pressure the compartment allows, and ten metres of seawater
is one bar. Both published coefficients are load-bearing: `a` sets the intercept and `b` sets the
slope, and changing either moves the ceiling and can change which compartment governs.

**The CNS oxygen clock.** Oxygen partial pressure rises with depth, and the NOAA table gives a
tolerated exposure per pressure. Below 1.0 bar the clock does not run.

## What the code demonstrates

Four categorical operations carry the program, and all four are in `main.rs`:

| Operation | Pairs or reduces | For |
|---|---|---|
| `try_step` | diver state to the next one | the dive as a chain of four phases |
| `zip_with` | tension against its own half-time | loading all sixteen compartments |
| `zip_with` | tension against its M-value coefficients | the ceiling each compartment imposes |
| `fold` | sixteen ceilings to the binding one | the compartment that governs the ascent |

Each compartment carries its own constants, so every compartment computation is a **pairing**.
`ZipTensorWitness::zip_with` walks two tensors slot by slot, so the laws in `model.rs` are written
once for one compartment and the witness applies them to all sixteen. No compartment index appears
in either law.

A fifth abstraction is the tangent functor. The gas-loading rate `dp/dt` is what a dive computer
watches, and it comes from one evaluation over `Dual`. The run prints it beside the analytic rate
`k·(p_inspired − p)` as a check; the two agree to the precision of the working scalar.

## Precision is a parameter

One alias in `main.rs` sets the working scalar for the whole program.

```rust
pub type FloatType = Float106;
```

Every constant in the model is declared at that type through `const_scalar_from_int!` and
`const_scalar_from_float!`, so the compiler resolves them against the alias and no conversion runs
at any call site. Switching the alias re-declares all of them.

It sits at `Float106` rather than `f64` on purpose. A hard-coded `f64` anywhere in the program is
invisible while the alias *is* `f64`, because the two types coincide and everything compiles. At
`Float106` the same line is a compile error. The default is a canary.

The autodiff agreement is the clearest reading of what changes:

| Scalar | agreement | ceiling |
|---|---|---|
| `BFloat16` | 9.8e-4 | 7.1 m |
| `f32` | 0.0 | 7.2 m |
| `f64` | 2.8e-17 | 7.2 m |
| `Float106` | 1.5e-33 | 7.2 m |  <- the default

All four reach the same dive plan and the same single mandatory stop. Only the precision moves.

## Files

| File | Holds |
|---|---|
| `main.rs` | the alias, the constants `main` uses, the phase chain, and the two witness operations |
| `model.rs` | the Bühlmann constants, the state types, the physiology, and the differentiable curve |
| `utils_print.rs` | the presentation, and the only `lower` calls in the example |

## Scope

This is a demonstration of the algorithm, not a dive computer. It uses a single gradient factor
rather than a GF-low to GF-high gradient across the ascent, its no-decompression limits come from
a coarse depth table rather than from the compartment model itself, and it plans a single dive
with no repetitive-dive residual nitrogen. Do not dive it.

## Adaptation

- **Nitrox.** Change `F_N2` and `F_O2`; the gas laws read them directly.
- **A GF gradient.** Add a `GF_LOW` constant and interpolate toward `GF_HIGH` by depth inside the
  ascent phase, so the ceiling tightens as the diver comes up.
- **Limits from the model.** Replace the depth table by searching for the bottom time at which the
  ceiling first reaches the surface.
