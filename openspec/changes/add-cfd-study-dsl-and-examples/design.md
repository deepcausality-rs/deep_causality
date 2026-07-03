# Design: add-cfd-study-dsl-and-examples

## Context

The DSL review drafted five planned examples in the current syntax and derived five additive
pieces (S1 `sweep`, S2 `Gates`, S3 `from_columns`, S4 `run_owned`, S5 `duct_march`), verified
against all five drafts. This change implements S1 through S5 and the three examples that
exercise them hardest: the nozzle operating map (the one program the DSL cannot express
today), the vortex-shedding margin check (the sweep-plus-frequency path), and the
flight-envelope placard table (the no-march path that keeps `sweep` output-generic). Group 1
of the example plan (table IO, traces, snapshots) shipped in `add-cfd-file-io` and is
consumed here.

## Goals / Non-Goals

**Goals:**

- The syntax program exactly as reviewed: additive, no breaking change, each piece verified
  against the three shipped examples plus the two drafted ones.
- Every example self-verifies with gates whose bands come from closed forms or stated
  placards, computes in its `FloatType` alias, and writes its result table through the
  group-1 writer.
- Practitioner-grade errors on every new surface, per the open Group-1 audit item: a wrong
  usage produces a message naming the artifact and the fix, tested per example.
- READMEs in the established convention (what it does, how to run, what happens, gates,
  where things live), prose per the writing guides.

**Non-Goals:**

- Examples 4 and 5 of the plan (the channel sizing and tunnel data reduction); example 5
  waits on roadmap item 3 by design.
- Migrating the three existing gate blocks (optional, mechanical, separately committable).
- Any turbulence content: the VIV example runs at laminar-wake Reynolds numbers and says so.
- The `Uncertain<T>` statistics combinator (roadmap item 3).

## Decisions

**D1: `sweep` is output-generic and lives in the flow module.**
`sweep<T, U, E>(items: &[T], f: impl Fn(&T) -> Result<U, E> + Sync) -> Result<Vec<U>, E>`,
riding `deep_causality_par::scoped_map` under the `parallel` feature and a plain map without
it, order-preserving and bit-identical either way, first error wins. Output-generic is
load-bearing: the placard example returns row arrays with no march, the VIV example returns
rows extracted from reports, and a `Report`-typed signature would exclude the first. `U` and
`E` carry no bounds beyond `Send` where the parallel feature demands it.

**D2: `Gates` prints what the hand-rolled blocks print, and only that.**
`Gates::new(title)` then `.gate(label, pass, detail)` accumulating, `.finish() -> bool`
printing the `[PASS]`/`[FAIL]` lines and the closing verdict line. No process exit, no
colors, no timing: the exact behavior of the three existing blocks, so a later migration is
a mechanical substitution. Details are `String`s the caller formats; the builder never
formats numbers itself, which keeps precision display at the caller's boundary.

**D3: `run_owned` materializes internally and drops the geometry after the run.**
On `MarchPipeline` (and the uncertain pipeline): `run_owned(self) -> Result<Report<R>, PhysicsError>`
is `let m = case.materialize()?; self.on(&m).run()`. The B1 borrow form stays the primary
API; `run_owned` exists for sweep bodies where each case owns a fresh grid, and its
docstring says when NOT to use it (geometry reuse across runs).

**D4: the duct march follows the established config-then-flow shape.**
`DuctConfig` is an owned description: an area profile (a `NumericTable` of x versus area or
an analytic profile variant), inlet stagnation state, back pressure, grid resolution, and a
stop condition. `CfdFlow::duct_march(&config)` lowers onto the existing 1-D compressible
Euler solver and returns the standard `Report` with series `"x"`, `"mach_profile"`,
`"pressure_profile"`, plus scalars `"shock_position"` and `"thrust_coefficient"`. No new
solver numerics: the runner is a driver over `CompressibleEuler1d`, marched to a quasi-steady
state under the stop condition, with the shock located from the pressure profile's steepest
gradient and the thrust coefficient integrated from the exit state.

**D5: the analytic gates get cited kernels, not example-local formulas.**
The area-Mach relation and the isentropic pressure, temperature, and density ratios land in
`deep_causality_physics/src/kernels/` as pointwise kernels with full citations (Anderson,
Modern Compressible Flow; source PDF in `deep_causality_physics/papers/` per house rule).
Normal-shock relations already exist through the fitted Rankine-Hugoniot machinery and are
reused, not duplicated. Float literals appear only as cited constants and in tests.

**D6: the VIV example reuses the validated cylinder configuration.**
The wake march reuses the DEC cylinder setup the verification ladder already validates
(Strouhal against published references), swept over airspeed with `sweep` + `run_owned`,
shedding frequency extracted with the existing `dominant_frequency`, margin gated against a
stated structural frequency constant. The example states its Reynolds-number range plainly
and claims nothing beyond the laminar-wake regime the solver validates in.

**D7: the placard example is pointwise on purpose.**
It reads the Mach-altitude matrix through the group-1 reader, computes per point with
existing kernels (Rankine-Hugoniot post-shock state, Sutton-Graves heating) plus dynamic
pressure, and writes one table. It demonstrates that the study shape needs no manifold and
no march; `sweep` over matrix rows is the only DSL piece it touches besides `Gates` and the
writer.

**D8: naming and prose.**
The examples are named for what they compute (`nozzle_operating_map`, `viv_resonance_margin`,
`flight_envelope_placard`). Artifacts and READMEs state what each example does and how; no
marketing labels. READMEs follow the established section convention and the writing guides
(varied sentence rhythm, no em dashes in body text, no banned phrases). All example code
computes in the example's `FloatType` alias; `f64` appears only at the table-writing display
boundary.

## Risks / Trade-offs

- **The duct runner's quasi-steady march may converge slowly near the exactly-choked
  point.** Mitigated: the sweep's back-pressure grid brackets rather than lands on the
  critical ratio, and the stop condition caps steps with a residual gate, so a slow point
  fails loudly instead of hanging.
- **Shock location from a gradient scan is resolution-dependent.** Accepted and gated: the
  gate compares against the analytic position within a band derived from the grid spacing,
  and the band's derivation is written next to the constant.
- **`sweep` under the parallel feature runs user closures concurrently.** The bound is
  `Sync` on the closure and `Send` on outputs; the docstring states that side effects in the
  closure (printing, file writes) will interleave and belong after the sweep.
- **Three examples in one change is wide.** Accepted: they share the new primitives, and
  landing them together is the verification of the syntax program; the tasks are ordered so
  the primitives are green before any example starts.
