# Cut-cell cylinder wake — CFD Stage 4 (Groups C + D)

Flow past a circular cylinder built as an **immersed cut-cell body** (Group D), driven by a
**sensor-fed uncertain inflow** through a **causal-monad march** (Group C). It exercises the
full Stage-4 stack end to end:

- the cylinder is a `CutCellRegistry` from the analytic disk primitive
  (`CutCellRegistry::from_primitive`, A4) — exact clipped volumes + apertures, not a staircase;
- the cut **Hodge star** (B5) makes every operator (compiled stencils, constrained Leray
  projection, codifferential) see the partial cells transparently;
- the immersed **no-slip / no-penetration** condition (B4) pins the body's edges through the
  existing constrained projector;
- the channel is driven by a **moving top wall whose velocity is a `MaybeUncertain<f64>`
  sensor stream** (Group C): each step the reading is presence-gated and collapsed to a scalar
  inflow; a **dropout** falls back to the last-good value via a Pearl `do(...)` intervention,
  recorded in the `EffectLog`.

```bash
cargo run --release -p deep_causality_cfd --example dec_cylinder_wake_verification
```

It streams a CSV to stdout (`step, t, kinetic_energy, max_speed, div_residual, v_probe`) and
prints the case setup, the `EffectLog` dropout count, and a shedding-Strouhal estimate to stderr.
The full per-step wake-probe series is written to `cylinder_wake.csv` through the IO effect.

## What it verifies (exit nonzero on break)

Two **internal-consistency** gates, checked in `main.rs` after the march:

- **incompressibility** — the sampled max divergence residual stays below `1e-6`, i.e. the
  constrained Leray projector holds the field divergence-free through the cut cells;
- **effect accounting** — the `EffectLog` holds exactly `2 × dropouts` entries (a fallback and
  an intervention per sensor dropout).

Neither gate is a published-reference comparison; see the section below for why.

## Measured (f64, 2000 steps, 93×32, ~155 s)

| Quantity | Measured | Expected | Verdict |
|---|---|---|---|
| max divergence residual | 3.334e-15 | < 1e-6 | PASS (machine precision) |
| `EffectLog` entries | 80 | 80 = 2 × 40 dropouts | PASS (exact) |

The probe settles to a steady `v ≈ 2.70e-2` and the run reports **no clear shedding** in the
developed signal at this configuration — the confined periodic-x channel at 25 % blockage
damps the street the isolated case sheds. Strouhal is therefore reported, never gated; the
isolated-cylinder Strouhal and drag live in `dec_cylinder_verification`.

## Files

- `main.rs` — the `CfdFlow::march` drive loop, diagnostics, and the two gates;
- `config.rs` — case + sensor parameters, the cut-cell geometry, and the `UncertainMarchConfig`;
- `print_utils.rs` — diagnostic rendering, the Strouhal estimate, and the CSV write;
- `baseline.txt` / `cli_output.txt` — recorded stdout stream and stderr report;
- `cylinder_wake.csv` — recorded wake-probe series.

The run is **bit-identical across invocations**: the SPRT presence gate is seeded, the QMC
collapse is deterministic by construction, and the cut-cell registry is built with
deterministic ordering.

## The causal-monad march (Group C)

The solver is **stateless and portable** (`step(&self, field)`); the **state lives in the
monad**. Each step is the `inflow_march_step` bind stage over a
`PropagatingProcess<f64, InflowMarchState, InflowContext>`: it presence-gates the sensor sample
(`MaybeUncertain::lift_to_uncertain`), collapses a present reading to a prescribed wall velocity
(`expected_value`), reconfigures the boundary through the **existing** moving-wall lift, and
marches — **the uncertain types never enter the solver core, and the solver is unchanged**. On a
dropout the last-good value is substituted through `intervene` (a logged value alternation). The
example drives the march one bind at a time so the wake probe can be streamed;
`deep_causality_physics::march_inflow` packages the identical stage as a `CausalFlow::iterate_n`
loop.

## What this harness is — and is not

The DEC solver has **no inflow/outflow surface**; the sensor drives a **prescribed moving wall**
(a Dirichlet boundary the solver already supports), confined in a **periodic-x channel**
(periodic-x, wall-y) containing the cylinder. This sheds a von-Kármán street and is a faithful
exercise of the cut-cell + uncertain-zone machinery.

The quantitative **isolated-cylinder Reynolds ladder** against Lehmkuhl et al. (2013) and the
Williamson lineage (tasks D2/D3 — Strouhal and drag over Re 100–3900) needs a true
inflow/outflow surface. It is **not** claimed here; the printed Strouhal is for the
confined/periodic case and is a qualitative shedding check, not a reference comparison.

## Small-cell stabilization (B1/B2)

In a finite-volume cut-cell solver, arbitrarily small cut cells collapse the explicit time
step — the canonical hazard that needs Berger–Helzel cell-merging or Colella–Graves–Modiano
flux-redistribution. **This formulation does not have that problem:** the cut Hodge star is a
*consistent metric clip*, so the codifferential `δ = M⁻¹ ∂ M` cancels it across grades and the
explicit march is inherently small-cell-stable (a sliver vertex `s0 ≈ ε` is fed by sliver
edges `s1 ≈ ε`, so operator entries are `s1/s0 ≈ O(1)`). See the change's design D4 and
`deep_causality_physics` `cut_cell_wiring_tests::tiny_cut_cells_are_inherently_small_cell_stable`.

The selected stabilizer is **cell-merging** (`CutCellRegistry::with_cell_merging`, a
volume-fraction floor on the cut star — flux-redistribution does not fit the projected-rate
formulation). It is engaged here only to tighten the masked-CG projection conditioning on
sliver cells, not as a CFL guard.

## Cheap CI regressions (D4)

The fast, no-march regressions that gate the substrate live in the crate test suites, not in
this example (per the tests-fast / examples-verify split):

- **cut-geometry exactness** — the clipped fluid volumes of a disk sum to the exact
  `domain − π r²` (`deep_causality_topology` `cut_cell::consistency_tests`), alongside the
  per-primitive exactness at f64 and Float106 (`cut_cell::intersection_tests`);
- **axis-aligned consistency** — an empty registry's cut star is byte-equal to the Stage-3
  star, and an empty-registry march is bit-identical to the plain solver
  (`cut_cell::cut_star_tests`; physics `cut_cell_wiring_tests`);
- **immersed no-slip** — an immersed solid block pins its edges to zero and the flow stays
  divergence-free (physics `cut_cell_wiring_tests`).

The **small-cell stability** rung (a deliberately tiny cut marches without CFL blow-up under
the chosen stabilizer) lands with the B1–B3 stabilizer selection.
