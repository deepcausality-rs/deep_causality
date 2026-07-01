## Why

Steps 4–5 (`add-cfd-qtt-incompressible-2d`) delivered a 2-D incompressible Navier–Stokes flowfield that
lives in, and evolves as, a tensor train — but it is reachable only by hand-driving `QttIncompressible2d`
directly. Nothing in the **CfdFlow DSL** can compose it, and there is no way to read observables off the
rollout: the marcher returns raw `(u, v)` trains, not a labeled `Report`. The flagship's step [4] — *MPS
flowfield → heat flux + drag + electron density* — needs both: a DSL seam that **composes** the QTT
solver the way `CfdFlow::march` composes the DEC solver, and **observable extraction** that turns the
tensor-train state into the time series and final fields downstream physics reads. Context:
`openspec/notes/plasma-blackout/gap-one-cfd-tensor-bridge.md` step 6.

The QTT solver is a deliberate **sibling** of the DEC solver, not a re-expression of it (gap note §4): its
state is `(CausalTensorTrain, CausalTensorTrain)`, it carries no manifold / cut-cell registry, and it runs
on a power-of-two periodic grid. So the wiring is a **parallel** pipeline that reuses the proven owned
value types (`Report<R>`, `MarchStop<R>`) — not a retrofit of `MarchRun`, which is hardwired to
`SolenoidalField` / `Manifold`.

## What Changes

- **QTT observable extraction** (`qtt-observe`, `solvers/qtt/observe.rs`): TT-native diagnostics computed
  directly on the velocity trains — kinetic energy `½(‖u‖² + ‖v‖²)` and divergence residual `‖∇·u‖` via
  the train `norm`/`inner` and the existing `QttProjector2d::divergence` (no dequantize), the maximum
  bond dimension (the compression / rank headline metric), and the maximum speed (dequantize + pointwise
  max). A fluent `QttObserve` set selects which series to collect.
- **CfdFlow DSL wiring** (`qtt-flow`): a parallel `CfdFlow::qtt_march(&config)` entry returning a
  `QttMarchRun` that drives `QttIncompressible2d` over a fixed or steady-state horizon, samples the
  enabled observables each step into the shared owned `Report<R>`, exposes the dequantized final `(u, v)`
  fields, and supports a per-step hook (`run_with`, a cheap `QttStepView`) and counterfactual overrides
  (`seed_with` / `march_with` / `observe_with`).
- **Config layer** (`flow_config/qtt_march_config.rs`): a `QttMarchConfig<R>` container + a
  `QttMarchConfigBuilder` that holds the grid (`Lx, Ly, dx, dy`), the solver params (`dt, ν, trunc`), the
  owned seed fields `(u0, v0)` (materialized from a closure over the grid, or the analytic Taylor–Green
  vortex), the `MarchStop<R>`, and the `QttObserve` set.
- **Validation**: the DSL path reproduces the direct-driver Taylor–Green result bit-for-bit (the wiring
  adds no numerics), the steady-state stop terminates on the kinetic-energy plateau, and the observable
  series match hand-computed references (energy decay, divergence ~ 0, bounded bond).
- Bound: real `R: CfdScalar + ConjugateScalar<Real = R>`. Purely additive; the DEC `CfdFlow::march` path
  and all existing types are unchanged.

### Non-Goals (explicit follow-on)
Immersed-body boundary conditions in QTT (§3.4) and the body-surface observables that ride them — drag /
lift / heat flux as boundary-fiber contractions; the Gap-2 ionization / reacting-flow surrogate (electron
density `n_e`); 3-D; and feeding the flagship's full step [4]. This change wires the *periodic* solver and
the observables computable without an immersed body — it is step 6 of the gap-one plan.

## Capabilities

### New Capabilities
- `qtt-observe`: tensor-train-native observable extraction (kinetic energy, divergence residual, max
  speed, bond dimension, final fields) for the QTT incompressible rollout.
- `qtt-flow`: CfdFlow DSL wiring for the QTT marcher — an owned config container/builder, the
  `qtt_march` pipeline with fixed/steady stop and a per-step hook, and the shared owned `Report`.
