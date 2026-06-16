# B1 Flow DSL + `CfdConfigBuilder` redesign

Concrete design for the configuration/composition split (option **B1**: caller owns the
geometry, Flow borrows it), sized to the capability level needed to rewrite the five
non-cylinder-validation CFD examples bit-identically.

## 0. Principles

1. **Configuration ≠ composition.** `CfdConfigBuilder` builds *configuration* (parameter
   bundles, one per solver / per parameterized coupling, plus a per-solver-kind container that
   bundles the scenario). The **Flow DSL** *composes the workflow* — it wires those configs onto
   a geometry, couples physics/solvers, and runs.
2. **B1 geometry-as-input.** The **manifold (geometry) is owned by the caller** and lent to the
   pipeline as `&'m Manifold`. This is the established borrow-based solver (`DecNsSolver<'m>`)
   reused as-is: zero runtime cost, no `unsafe`, bit-identical numerics. The pay-off is that a
   proven solver/config **transposes across geometries** — materialize a different geometry,
   `.on(&that)`, reuse the same config — and an expensive geometry is **materialized once, run
   many** (counterfactuals / sweeps).
3. **D2 at the seam.** Nothing self-referential: the borrowed manifold lives in the caller; the
   solver + marching state live inside the pipeline bound to `'m` and never escape it.
4. **The DSL owns the workflow, not every diagnostic.** Standard observations are DSL-provided;
   bespoke diagnostics (Ghia RMSE, streamfunction vortex centers, graded-MMS norms,
   edge-probe Strouhal) are computed by the example **from the raw marched state the Report
   exposes** — which *guarantees* bit-identical output (same formulas, same numbers).

## 1. Layered architecture

```
Layer 1  CfdConfigBuilder ─ configuration (owned, validated at build(), no borrows)
   ├─ sub-configs:   dec_ns() → DecNsConfig
   │                 thermal_relax(), viscosity_arrhenius() → coupling stage configs
   │                 mms() → MmsConfig            operator_study() → OperatorConfig
   └─ containers:    march(name) → MarchConfig   (bundles mesh+seed+solver+stop+observe+zones+couple)
                     uncertain_march(name) → UncertainMarchConfig

Layer 2  Flow ─ workflow composition (borrows the caller's geometry, runs, returns Report)
   Flow::march(&config).on(&manifold).mesh().solver().seed().march().observe().run()? → Report
   Flow::verify_mms(&config).run()?            (no geometry; pointwise)
   Flow::operator_study(&config).run()?        (materializes its own swept geometries)
```

## 2. `CfdConfigBuilder` surface

### 2.1 Solver sub-config (today's `DecNs::config()`, re-homed)
```rust
let ns = CfdConfigBuilder::dec_ns()           // typestate: viscosity → time_step → ready
    .viscosity(nu).time_step(dt)
    .cfl_factors(0.9, 0.9).warm_start()        // optional knobs
    .build()?;                                 // → DecNsConfig<R>
```

### 2.2 Coupling sub-configs (parameterized stages get dedicated builders)
```rust
let thermo    = CfdConfigBuilder::thermal_relax().rate(2.0).wall_temperature(800.0).build();
let arrhenius = CfdConfigBuilder::viscosity_arrhenius().nu_ref(nu0).t_ref(300.0).beta(-1.5).build();
```

### 2.3 March container (bundles the scenario; `build()` validates required pieces)
```rust
let config = CfdConfigBuilder::march::<3, FloatType>("tgv-re1600")
    .mesh(Mesh::periodic_cube(n))              // domain spec (materialized later via .on)
    .solver(ns)                                // DecNsConfig (required)
    .seed(Seed::TaylorGreenVortex)
    .march_for(steps)                          // or .march_until_steady(tol, max)
    .observe(Observe::default().kinetic_energy())
    .couple(Coupling::between_steps().then(thermo).then(arrhenius).build())   // optional
    .coupled_scalar("temperature", 300.0)      // optional
    .build()?;                                 // → MarchConfig<D,R,Z,C>
```

### 2.4 MMS sub-config — built-in regimes **or** a caller-supplied manufactured solution
```rust
// built-in (today): regime + nu + rho
let m = CfdConfigBuilder::mms::<R>("tgv-mms").regime(Regime::Incompressible).viscosity(0.1).density(1.0).build()?;

// bring-your-own manufactured solution (cfd_taylor_green): autodiff fields + amplitude march
let m = CfdConfigBuilder::mms::<FloatType>("tgv-mms")
    .manufactured(my_tg)                       // impl Manufactured<R> (provides u, ∇u, ∇²u, ∇p, exact ∂u/∂t)
    .sample_at([0.7, 1.1, 0.0], 0.0)
    .amplitude_march(Rk4Steps { dt: 0.005, steps: 200 }, my_rate)   // optional kernel-in-loop march
    .build()?;
```

### 2.5 Operator-study sub-config
```rust
let op = CfdConfigBuilder::operator_study::<FloatType>("graded-mms")
    .mesh_family(|n, amp| Mesh::torus(n).graded(Grading::cosine(1, amp)))  // graded geometry family
    .resolutions([8, 16, 32, 64])
    .amplitudes([0.0, 0.1, 0.2, 0.3])
    .operators([Operator::Convective, Operator::Viscous])
    .norms([Norm::Max, Norm::L2])
    .build()?;
```

## 3. Flow DSL surface (B1)

### 3.1 Typestate pipeline
`Flow::march(&config)` returns a pipeline holding `&'c MarchConfig` + per-step override slots.
`.on(&manifold)` enters `'m`. The no-arg stages **resolve** their sub-config from the container (or
apply an override); the typestate enforces order and completeness; the terminal `.run()` performs
the borrow-bound materialize→seed→march→observe in one body, returning an owned `Report`.

```rust
let manifold = config.materialize()?;          // caller owns the geometry (B1)
let report = Flow::march(&config)
    .on(&manifold)
    .solver()                  // or .solver_with_config(&other_ns)  — counterfactual override
    .seed()                    // or .seed_with_config(Seed::UniformX { speed: 1.0 })
    .march()                   // or .march_with(MarchStop::Steady { tol, max })
    .observe()                 // or .observe_with_config(Observe::default().drag(u))
    .on_step(|s| { … })        // optional per-step hook (progress / streamed CSV)
    .run()?;
```

`.mesh()` is implicit in `.on(&manifold)` (the geometry *is* the materialized mesh); kept as a
no-op stage only if we want the literal `.mesh()` token for symmetry — otherwise dropped.

### 3.2 Per-step hook (NEW — required by cavity + cylinder_wake)
`.on_step(FnMut(&StepView<R>))` is called after every projected step with a cheap read-only view
(step index, time, the edge cochain, helpers for energy/divergence/max-speed/probe-edge). This is
how the cavity streams `# t = … (step/total)` and the wake streams its per-step CSV — output the
old code produced *inside* the march loop.

### 3.3 Report exposes the raw final state (NEW — required by cavity + wake)
```rust
report.series("kinetic_energy")    // standard observations (as today)
report.final_field()               // &[R] — the final edge cochain (velocity 1-form)
report.manifold()                  // &Manifold — for bespoke diagnostics
```
With these, the example computes centerline / streamfunction / Ghia RMSE / edge-probe Strouhal
**exactly as the original did**, off the same final cochain ⇒ bit-identical guaranteed.

## 4. Capability matrix (the five in-scope examples)

| Example | Workflow used | Standard obs | Bespoke (from raw state) | NEW capability needed |
|---|---|---|---|---|
| **dec_taylor_green_re1600** | `Flow::march` periodic cube, TG seed, fixed steps | kinetic_energy | t*, E/vol, dissipation (post-proc of energy series) | B1 structure + `CfdConfigBuilder` |
| **dec_lid_cavity_re1000** | `Flow::march` box, lid, fixed steps | — | centerline, Ghia RMSE, ψ vortex centers (from `final_field`) | per-step hook; `final_field()`; (file write → IO monad) |
| **dec_graded_mms** | `Flow::operator_study` graded torus | convergence orders | — (table from study output) | graded mesh; convective+viscous; max+L2 norms; amplitude sweep |
| **cfd_taylor_green** | `Flow::verify_mms` BYO-manufactured | mms_error | — | `Manufactured` trait (autodiff fields); `sample_at`; `amplitude_march` (kernel-in-loop Rk4) |
| **dec_cylinder_wake** | `Flow::march`+uncertain inflow, cut-cell body | — | edge-probe series, Strouhal (from per-step hook / `final_field`) | `uncertain_march` (PropagatingProcess + inflow_march_step); cut-cell body (have) |

## 5. NEW DSL capabilities (each grounded in an example)

1. **B1 config/runner**: `CfdConfigBuilder` (sub-configs + containers) and the
   `Flow::march(&config).on(&manifold)…run()` pipeline with `_with_config` overrides. *(all)*
2. **Per-step hook** `.on_step(FnMut(&StepView))`. *(cavity progress; wake per-step CSV)*
3. **Raw state in Report** `final_field()` + `manifold()`. *(cavity centerline/ψ; wake probe)*
4. **Graded mesh** `Mesh::torus(n).graded(Grading::cosine(axis, amp))` → a `PerEdge`
   `CubicalReggeGeometry`. *(graded_mms)*
5. **MMS — bring-your-own manufactured solution**: a `Manufactured<R>` trait (the example
   supplies `u, ∇u, ∇²u, ∇p` — via the autodiff tangent functor — and the exact `∂u/∂t`), plus
   `.sample_at(point, t)` and `.amplitude_march(Rk4Steps, rate)` (kernel-in-the-loop). The DSL runs
   the regime kernel residual and the Rk4 amplitude march. *(cfd_taylor_green)*
6. **Operator study — full**: `Operator::{Convective, Viscous}`, `Norm::{Max, L2}`, a graded
   `mesh_family`, and an amplitude sweep. The driver reproduces the example's
   midpoint-evaluation + ℓ-factor normalization + unit-carrier `d`. *(graded_mms)*
7. **Uncertain-inflow march**: `Flow::uncertain_march(&config)` wrapping the physics
   `PropagatingProcess`/`inflow_march_step`/`UncertainInflowZone` machinery, with the per-step
   hook streaming the probe. *(cylinder_wake — see determinism caveat below)*

## 6. What stays in the example (bespoke post-processing, off `final_field()`)

- Cavity: `centerline_profiles`, `interp_profile`, `centerline_rmse` vs the Ghia tables,
  streamfunction `ψ` integration + `extremum` vortex finding, `trend` mode (two marches + gates).
- Wake: edge-probe series, up-crossing Strouhal.
- These read the raw cochain the DSL exposes — identical formulas ⇒ identical bytes.

## 7. Caveats / deferred

- **File I/O** (cavity `cavity_centerline_*.csv`; cylinder_validation `re100_16_*`) → the
  follow-up **IO-monad** spec. The DSL covers the *computation*; only the file writes are deferred.
- **`dec_cylinder_wake` determinism**: `Uncertain::sample()` draws a **random** index via
  `deep_causality_rand::rng()`. If that RNG is not fixed-seed-stable across runs, the wake is
  nondeterministic and bit-identical is impossible — to be verified by a double-run diff before
  committing to its migration. If nondeterministic, the wake migrates structurally (same workflow)
  but is verified by *invariant* checks (divergence bound, Strouhal in range), not a byte diff.

## 8. Build phases (each ends green + bit-identical where applicable)

- **P1** `CfdConfigBuilder` + `MarchConfig` container + `Flow::march(&config).on(&m)…run()` (B1)
  with `_with_config` overrides and `.on_step`; `Report::final_field()/manifold()`. Re-home
  `DecNs::config()`. Update lib exports + the ~14 cfd tests. Re-verify **dec_taylor_green_re1600**.
- **P2** Graded mesh + full `operator_study` (convective+viscous, max+L2, amplitude sweep).
  Rewrite **dec_graded_mms**; diff.
- **P3** `Manufactured` trait + `sample_at` + `amplitude_march`. Rewrite **cfd_taylor_green**; diff.
- **P4** `uncertain_march`. Verify wake determinism; rewrite **dec_cylinder_wake**; diff or
  invariant-gate.
- **P5** (separate) **dec_lid_cavity_re1000** compute via Flow (+per-step hook + `final_field`);
  its CSV writes wait on the IO-monad spec.
- **P6** Update `fluiddynamics-dsl` spec; deferred-deletion cleanup of physics fluid theories.
