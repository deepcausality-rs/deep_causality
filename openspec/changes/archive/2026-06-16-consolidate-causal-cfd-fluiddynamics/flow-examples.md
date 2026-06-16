# Flow DSL — comprehensive grounding against all examples and regimes

This is the **review artifact** the `cfd-crate.md` workflow calls for: the Flow facade
expressed against *every* validation example and *every* solver regime, so the B5
implementation builds the right abstractions, not just the trivial periodic case. It is a
design sketch (illustrative API), grounded in the exact construction calls each example uses.

## 1. The three solver kinds

The six examples decompose into three solver *shapes*, all producing a common `Report<R>`
and all lowering onto `CausalFlow`:

- **March solver** (DEC incompressible) — mesh + zones + seed + march + observe. Covers
  `dec_taylor_green_re1600`, `dec_lid_cavity_re1000`, `dec_cylinder_wake`,
  `dec_cylinder_validation`. March styles: fixed-step, until-developed/steady, and the
  uncertain causal-monad `bind` march. Supports `.couple` for between-step **multiphysics**
  (thermo × fluid × stress) and the Phase-2 counterfactual `.intervene` / `.continue_with`.
- **MMS verifier** (pointwise regime + autodiff) — a manufactured solution + a `FluidTheory`
  regime kernel; checks the kernel RHS against the exact `∂u/∂t` and an Rk4 amplitude decay.
  No DEC mesh/march. Covers `cfd_taylor_green`. Generic over the regime, so this is also the
  **all-regimes** coverage (Incompressible / Euler / Stokes / Compressible).
- **Operator-accuracy study** — a (possibly graded) mesh + DEC operators (`d`, `i_X`, `δd`)
  vs an analytic reference, swept over resolutions for convergence orders. No march, no
  regime kernel. Covers `dec_graded_mms`.

`Solver` is the seam (static dispatch): `fn solve(self) -> Result<Report<R>, PhysicsError>`.
A new solver kind is a new impl, not a change to the DSL core.

## 2. The Flow abstraction surface (covers every axis)

Each abstraction maps to the exact library calls the examples use; the owned spec is
materialized inside `run()` so no borrow escapes (design D2).

### Mesh — geometry + periodicity + spacing + immersion
```rust
Mesh::<3, R>::periodic_cube(n)                 // LatticeComplex::cubic_torus(n) + CubicalReggeGeometry::unit()
Mesh::<2, R>::box(shape)                        // new(shape, [false,false]) + uniform(h)
Mesh::<2, R>::channel(shape)                    // new(shape, [true,false])  + uniform(h)
Mesh::<2, R>::torus(n)                          // square_torus(n) + unit
    .spacing(h)                                 // uniform(h) (default unit)
    .graded(Grading::cosine(axis, amplitude))   // from_edge_lengths(per-edge ℓ(pos))
    .immersed(Body::disk(center, radius).merge_floor(f))   // cut-cell registry + with_cut_cells
```
`Grading` is an enum (e.g. `Cosine{axis,amplitude}`) so meshes stay `Clone`/static — no boxed
closures. `materialize()` builds the lattice, edge lengths, the Regge metric, the cut-cell
registry (`CutCellRegistry::from_primitive(...).with_cell_merging(f)` → `with_cut_cells`), and
`Manifold::from_cubical_with_metric`.

### Body — immersed cut-cell primitive
```rust
Body::disk(center: [R;2], radius: R).merge_floor(f)   // Primitive::ball + CutCellRegistry
```

### Zones — the existing static `BoundaryZone` tuple, with named-axis conveniences
```rust
.moving_wall(MovingWall::lid(velocity))                       // with_moving_wall(1,true,velocity)
.zones((Inflow::west(u), Outflow::east(), SlipWall::top(), SlipWall::bottom()))  // with_zones tuple
.uncertain_inflow(UncertainInflow::top(sensor_stream))        // the causal-monad inflow zone
```
The conveniences desugar to the existing `Inflow::new(axis,far,u)` / `Outflow::new` /
`SlipWall::new` / `with_moving_wall` calls and the archived `BoundaryZone` cons-tuple.

### Seed — named initial conditions (enum, static)
```rust
Seed::rest()                                                  // zeros
Seed::taylor_green_vortex()                                   // 3D: u=sin x cos y cos z, v=-cos x sin y cos z, w=0; k=2π/n
Seed::uniform_x(u).perturb(Blob::gauss(center, eps, sigma))   // uniform stream + transverse blob
```
Each builds the vertex vector field for the materialized manifold and calls
`seed_from_vertex_vectors`.

### Theory / regime — the FluidTheory selector
```rust
DecNs::config()...build()                                     // DEC incompressible marcher (B2, done)
Mms::regime(Regime::Incompressible | Euler | Stokes | Compressible)   // pointwise FluidTheory for verification
OperatorStudy::convective() | ::viscous()                    // DEC operator accuracy
```

### March — the three styles (lower onto CausalFlow)
```rust
.march_for(Steps(n))                       // iterate_n          (TG, lid, validation)
.march_until(Developed::after(k), max)     // iterate_until      (validation window, lid steady)
.march_uncertain(sensor_stream)            // bind(inflow_march_step) over PropagatingProcess
```

### Couple — between-step multiphysics (the `.couple` seam)
`.couple` registers a between-step physics pipeline evaluated once per timestep *around* the
CFD step, lowering onto the `CausalFlow` `bind`/`bind_or_error` passthrough (as in
`multi_physics_pipeline`). A `Coupling<R>` is a statically-composed pipeline of `PhysicsStage<R>`
values over the error algebra — a stage, a sub-process, and the whole coupling are all
first-class `Coupling` values, built in their own modules and wired with `then` / `compose`
(the `flight_envelope_monitor` / `sensor_processing` module discipline applied to physics). The
solver reads its ambient (ν, freestream) from the per-step `Ambient` channel
(`DecNs::config()…ambient_from_context()`), so a coupling stage can drive `ν(T)` between steps.
```rust
.couple(Coupling::between_steps()
    .then(Thermo::advect_diffuse().prandtl(0.71))     // T advected by u + diffused
    .then(Solid::conduction().wall_temperature(350.0)) // conduction into the immersed body
    .then(Properties::viscosity_arrhenius())           // ν(T) → Ambient  (closes thermo → fluid)
    .then(Stress::thermal_von_mises()                  // = the triple_hkt_stress_field walk
            .material(Material::steel()).yield_stress(250e6))
    .build())
```
A new coupled physics is a small `PhysicsStage` impl, not a change to the DSL core. Errors
propagate across the holistic coupling automatically (a stage failure short-circuits the step).

### Observe — every diagnostic the examples compute
```rust
Observe::kinetic_energy().dissipation_rate()        // dec_kinetic_energy / volume; −dE*/dt*
Observe::centerline_profiles().vs(Ghia1982::re1000())  // edge-cochain centerline + Ghia RMSE
Observe::vortex_centers()                            // streamfunction extrema
Observe::strouhal(Probe::velocity_y_at(point))       // crossing-rate of developed wake; St=fD/U
Observe::drag().every(k).in_developed_window()       // pressure_diagnostic + pressure/viscous_surface_force + force_coefficient
Observe::interior_divergence()                       // codifferential over interior cells
Observe::convergence_orders()                        // log2 error ratios across resolutions
Observe::mms_error()                                 // kernel-RHS vs exact ∂u/∂t; amplitude decay error
```

## 3. Worked Flow sketches — all six examples + the regime showcase

### dec_taylor_green_re1600 (March; periodic 3D)
```rust
Flow::march::<FloatType>("tgv-re1600")
    .mesh(Mesh::periodic_cube(n))
    .solver(DecNs::config().reynolds(1600.0).cfl(0.2).spectral_diffusion().build()?)
    .seed(Seed::taylor_green_vortex())
    .march_for(Time::convective(10.0))
    .observe(Observe::kinetic_energy().dissipation_rate())
    .run()?
```

### dec_lid_cavity_re1000 (March; walls + moving wall; →steady; Ghia)
```rust
Flow::march::<FloatType>("cavity-re1000")
    .mesh(Mesh::box([n, n]).spacing(h))
    .solver(DecNs::config().reynolds(1000.0).cfl(0.45).build()?)
    .moving_wall(MovingWall::lid([1.0, 0.0]))
    .seed(Seed::rest())
    .march_until(Steady::div_below(1e-10), max_steps)
    .observe(Observe::centerline_profiles().vs(Ghia1982::re1000()).vortex_centers())
    .run()?
```

### dec_cylinder_wake (March; cut-cell + uncertain inflow; causal-monad march)
```rust
Flow::march::<FloatType>("cylinder-wake")
    .mesh(Mesh::channel([nx, NY]).spacing(h).immersed(Body::disk(center, radius).merge_floor(0.25)))
    .solver(DecNs::config().reynolds(100.0).cfl_diffusive(0.2).build()?)
    .uncertain_inflow(UncertainInflow::top(sensor_stream))     // lowers onto bind(inflow_march_step)
    .seed(Seed::rest())
    .march_uncertain(STEPS)
    .observe(Observe::kinetic_energy().interior_divergence().strouhal(Probe::velocity_y_at(wake)))
    .run()?
```

### dec_cylinder_validation (March; open zones + cut-cell; drag/lift)
```rust
Flow::march::<FloatType>("cylinder-re100")
    .mesh(Mesh::box([nx, ny]).spacing(h).immersed(Body::disk(center, 0.5).merge_floor(0.25)))
    .solver(DecNs::config().reynolds(100.0).cfl(0.4).no_slip(NoSlip::ApertureResolved)
            .cg(Cg::tol(1e-6).warm_start()).build()?)
    .zones((Inflow::west(1.0), Outflow::east(), SlipWall::top(), SlipWall::bottom()))
    .seed(Seed::uniform_x(1.0).perturb(Blob::gauss_behind_body(0.3, 0.75)))
    .march_until(Developed::after(steps / 2), steps)
    .observe(Observe::drag().every(steps / 80).in_developed_window()
             .and(Observe::strouhal(Probe::velocity_y_at_wake())))
    .run()?
```

### heated-cylinder multiphysics (March + `.couple`; thermo × fluid × stress) — showcase
The validation examples are single-physics; this showcase couples three physics around the CFD
step to demonstrate `.couple`. It composes onto any march case (here the cylinder geometry):
```rust
Flow::march::<FloatType>("heated-cylinder-multiphysics")
    .mesh(Mesh::box([nx, ny]).spacing(h)
          .immersed(Body::disk(center, 0.5).merge_floor(0.25).solid()))   // conducting body
    .solver(DecNs::config().reynolds(100.0).cfl(0.4).ambient_from_context().build()?)  // ν dynamic
    .zones((Inflow::west(1.0), Outflow::east(), SlipWall::top(), SlipWall::bottom()))
    .couple(Coupling::between_steps()
        .then(Thermo::advect_diffuse().prandtl(0.71))
        .then(Solid::conduction().wall_temperature(350.0))
        .then(Properties::viscosity_arrhenius())                 // ν(T) → Ambient
        .then(Stress::thermal_von_mises().material(Material::steel()).yield_stress(250e6)))
        .build())
    .seed(Seed::uniform_x(1.0).perturb(Blob::gauss_behind_body(0.3, 0.75)))
    .march_until(Developed::after(steps / 2), steps)
    .observe(Observe::drag().and(Observe::wall_heat_flux()).and(Observe::stress().safety_factor()))
    .run()?
```
The Phase-2 counterfactual `report.continue_with().intervene(Intervene::material(...))` branches
cheap scenarios on the developed coupled field (compute-once / report-many).

### cfd_taylor_green (MMS verify; pointwise regime + autodiff)
```rust
Flow::verify_mms::<FloatType>("tgv-mms")
    .regime(Regime::Incompressible)                  // FluidTheory pointwise kernel
    .manufactured(Manufactured::taylor_green(nu, rho))
    .sample_at([0.7, 1.1, 0.0], /*t0*/ 0.0)
    .amplitude_march(Rk4Steps { dt: 0.005, steps: 200 })   // vs exp(-2νt)
    .observe(Observe::mms_error())
    .run()?
```

### dec_graded_mms (Operator study; graded mesh; no march)
```rust
Flow::operator_study::<FloatType>("graded-mms")
    .mesh(|n, amp| Mesh::torus(n).graded(Grading::cosine(1, amp)))
    .sweep(Resolutions([8, 16, 32, 64]), Amplitudes([0.0, 0.1, 0.2, 0.3]))
    .operators([Operator::convective(), Operator::viscous()])   // i_X dω+d i_X ω ; δd f
    .observe(Observe::convergence_orders())
    .run()?
```

### Regime showcase (the "all regimes" coverage — MMS over every regime)
The validation examples only march incompressible; this showcase exercises the migrated
Euler / Stokes / Compressible regimes through the same MMS verifier:
```rust
for regime in [Regime::Incompressible, Regime::Euler, Regime::Stokes, Regime::Compressible] {
    let report = Flow::verify_mms::<FloatType>("regime-mms")
        .regime(regime)
        .manufactured(Manufactured::for_regime(regime, nu, rho))
        .sample_at(point, t0)
        .observe(Observe::mms_error())
        .run()?;
    assert!(report.kernel_error() < tol(regime));
}
```

## 4. Lowering and lifetimes (unchanged from D1/D2)

- `Flow::march/verify_mms/operator_study` build a **fully-owned** spec (no borrows).
- `.run()` materializes mesh → manifold → solver as locals, executes, and returns an owned
  `Report<R>`; borrows never escape (D2).
- The march lowers onto `CausalFlow`: `march_for`→`iterate_n`, `march_until`→`iterate_until`,
  `march_uncertain`→`bind(inflow_march_step)` over `PropagatingProcess` (the existing
  uncertain-inflow machinery). Multi-physics `.couple` and counterfactual `.intervene` /
  `.continue_with` (Phase 2) hang off the same march.

## 5. B5 implementation order (grounded by the above)

1. `Report<R>` + the `Solver` seam (3 impls share it).
2. `Mesh` (periodic_cube / box / channel / torus / graded / immersed) + `materialize`.
3. `Seed` (rest / taylor_green_vortex / uniform_x+blob).
4. `Flow::march` for the four DEC cases: fixed-step + until-developed first, then the
   uncertain `bind` march; zones via the existing tuple + conveniences.
5. `Observe` incrementally (energy → centerline/Ghia → strouhal/drag → divergence).
6. `Flow::verify_mms` (regime-generic) and `Flow::operator_study` for the two non-marching
   cases + the regime showcase.

Each step is verifiable by reproducing the corresponding example's reference numbers
(St / C_d / Ghia RMSE / dissipation curve / convergence orders).
