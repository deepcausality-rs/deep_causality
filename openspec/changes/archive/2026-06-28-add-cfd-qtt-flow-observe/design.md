## Context

`add-cfd-qtt-incompressible-2d` delivered `QttIncompressible2d` (a `Marcher` over `(u, v)` velocity
trains) and `QttProjector2d`. This change makes that solver **composable** through the CfdFlow DSL and
**observable**, so its rollout produces the same owned `Report<R>` the DEC march does and downstream
physics can read time series and the final fields. It adds glue, not numerics.

The DSL already has a proven shape (`flow/march_run.rs`): a borrowed config → a runnable pipeline →
`run` / `run_with(hook)` → an owned `Report<R>`; observables sampled each step into a private `Series`
accumulator; a cheap per-step `StepView` for the hook. But that pipeline is hardwired to the DEC solver:
its state is `SolenoidalField<R>`, it samples DEC diagnostics (`dec_kinetic_energy`, …), and it needs a
`Manifold` + `CutCellRegistry`. The QTT solver has none of those.

## Goals / Non-Goals

Goals: a parallel QTT pipeline on `CfdFlow` that drives `QttIncompressible2d` over a fixed/steady horizon
and returns an owned `Report<R>`; TT-native observable extraction (energy, divergence, max speed, bond
dimension, final fields); a per-step hook; counterfactual overrides — all reusing `Report<R>` /
`MarchStop<R>`.

Non-Goals (next change): immersed-body BCs and the body-surface observables (drag/lift/heat flux); the
Gap-2 ionization physics (`n_e`); 3-D; the full flagship step [4]. No change to the DEC `march` path.

## Decisions

- **Parallel pipeline, not a retrofit (sibling, per gap note §4).** Add `CfdFlow::qtt_march(&config) ->
  QttMarchRun<'_, R>` alongside `CfdFlow::march`. `QttMarchRun` borrows the config and owns the run
  (design D2: borrows never escape; only the `Report<R>` does). There is **no `.on(manifold)` stage** —
  the QTT solver carries no borrowed geometry; the power-of-two grid lives in the config. Conflating the
  two pipelines (forcing `SolenoidalField` onto the QTT state) is explicitly rejected.

- **Config-driven seed, materialized at build time.** `QttMarchConfigBuilder` evaluates a seed closure
  `Fn(R, R) -> (R, R)` (or the analytic Taylor–Green vortex) over the grid **once**, storing the owned
  `(u0, v0): (CausalTensor<R>, CausalTensor<R>)` in the config — so `run` is argument-free and the config
  is the single source of truth (mirrors `Seed` in the DEC path, but holds owned fields rather than a
  Copy enum because a velocity field is data, not an analytic tag). The builder validates the grid is
  `2^Lx × 2^Ly` and the seed fields match it.

- **TT-native observables first; dequantize only where unavoidable.** The headline metrics are computed
  on the trains directly — kinetic energy `½(‖u‖² + ‖v‖²)` from `TensorTrain::norm`, divergence residual
  `‖∇·u‖` from `QttProjector2d::divergence(u, v)` then `norm`, and the **maximum bond dimension** from the
  cores (`max c.shape()[2]`), the compression/rank metric every QTT-CFD reference reports. Only
  `max_speed` needs the pointwise field, so it dequantizes and takes `max √(u² + v²)`. Observable
  extraction lands in `solvers/qtt/observe.rs` as free functions over `(&CausalTensorTrain, …)`, so they
  are testable without the DSL.

- **Reuse `Report<R>` and `MarchStop<R>`.** The same owned `Report` shared by the DEC march / MMS /
  operator-study carries the QTT series (`kinetic_energy`, `divergence`, `max_speed`, `bond`). The final
  `(u, v)` are dequantized at the end; `u` goes to `Report::set_final_field` (mirroring the DEC velocity
  cochain) and `v` to a `final_v` series, so the existing `Report` API needs no change. `MarchStop<R>` is
  reused verbatim: `Fixed(n)` and `Steady { tol, max_steps }` (the steady test is the kinetic-energy
  delta, computed from the TT norm — no manifold needed).

- **`QttObserve` fluent set.** Mirrors `Observe`: `kinetic_energy()`, `divergence()`, `max_speed()`,
  `bond()` — each a `bool` toggle (no immersed-body / probe / centerline options, which need a body this
  change does not encode). `Default` collects nothing.

- **`QttStepView` for the hook.** A cheap read-only view exposing `step()` / `time()` / the `(u, v)`
  train references plus the same TT-native diagnostics (`kinetic_energy`, `divergence`, `max_bond`) — for
  progress lines and streamed per-step diagnostics, identical in spirit to `StepView`.

- **Module placement.** Observables in `solvers/qtt/observe.rs`; the DSL pipeline in
  `types/flow/qtt_march_run.rs` with `CfdFlow::qtt_march` on `flow/cfd_flow.rs`; the config in
  `types/flow_config/qtt_march_config.rs`. Re-exported from the crate root beside the existing flow
  types.

## Risks / Trade-offs

- **No numerics added — wiring must be transparent.** The DSL path must reproduce the direct
  `QttIncompressible2d::run` result bit-for-bit (same seed, same steps, same trunc). A test pins this, so
  the wiring can never silently diverge from the verified solver.
- **`Report` carries two fields awkwardly.** `Report` was shaped for one velocity cochain; the QTT field
  is a pair. Storing `u` as `final_field` and `v` as a `final_v` series is a small asymmetry, chosen over
  widening the shared `Report` API (which the DEC path and MMS/operator-study also use). Documented on
  the `qtt_march` output.
- **Steady-state on kinetic energy only.** The QTT steady stop reuses the DEC heuristic (energy-delta <
  tol). Adequate for the decaying/periodic Tier-A cases; richer stopping (residual-based) is deferred.
- **Power-of-two grid.** Inherited constraint, re-validated at config-build time.
- **Scope discipline.** Periodic, no walls, no body-surface observables. Drag / heat flux / `n_e` ride on
  the immersed-body encoding (§3.4) and the Gap-2 physics — the next changes.
