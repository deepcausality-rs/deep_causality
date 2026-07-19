# DSL review: the five everyday examples drafted in today's CfdFlow

Date: 2026-07-03. Status: **shipped**; S1 to S5 landed with examples 1 to 3 through
`openspec/changes/add-cfd-study-dsl-and-examples/` (one scope correction recorded in that
change's design D3: `run_owned` is MarchPipeline-only, because the uncertain config carries no
mesh). Related:
[common-examples](../cfd-examples/common-examples.md), [cfd-roadmap](../../cfd-plasma-retropulsion/cfd-roadmap.md).

Method, per the review brief: draft all five everyday examples in the **current** DSL first,
mark the friction each draft exposes, then derive the smallest syntax program that removes it.
Every proposed addition is verified against all five mocks at the end. Changing existing
syntax was on the table if it cut complexity or verbosity; the review's conclusion is that
**no breaking change is needed**: five additive pieces cover everything, and one of them
(gates) pays for itself again if the three existing gate implementations migrate.

## The five drafts in the current DSL

Drafts are condensed to the load-bearing lines; error handling per the house `fail` helper.
Precision as a parameter is upheld throughout: every draft computes in the example's
`FloatType` alias, never a raw `f64`; the only `f64` is the display boundary the result-table
writer embodies (exact literals lift in, values downcast out at write time, per the house
convention).

### 1. Nozzle operating map (propulsion)

```rust
// TODAY: no duct path exists in CfdFlow. The draft falls back to the raw solver.
let mut rows = Vec::new();
for &p_back in read_table::<FloatType>("back_pressures.csv").run()?.rows()[0].iter() {
    let mut solver = CompressibleEuler1d::new(/* grid, gamma, area profile by hand */)?;
    let mut state = seed_state(/* inlet stagnation conditions */);
    for _ in 0..STEPS {
        state = solver.step(&state, p_back)?;             // raw stepping, no Report
    }
    let shock_x = find_shock(&state);                     // hand-rolled scan
    let cf = thrust_coefficient(&state);                  // hand-rolled integral
    rows.push(vec![p_back, shock_x, cf]);
}
// gates: hand-rolled println/bool pattern, copied from print_utils a fourth time
```

Friction: the whole example bypasses the DSL (no config description, no `Report`, no
observables), the sweep is a hand loop, the gate block is the fourth copy of the same
fifteen lines, and the result table is hand-assembled.

### 2. Vortex-induced-vibration margin check (structures)

```rust
let mut rows = Vec::new();
for &airspeed in AIRSPEEDS {
    let case = config::build_march_config(airspeed)?;      // CfdConfigBuilder::march(...)
    let manifold = case.materialize()?;                    // B1 two-step, every iteration
    let report = CfdFlow::march(&case).on(&manifold).run()?;
    let lift = report.series("lift").expect("observed");
    let f_shed = dominant_frequency(lift, dt)?;
    rows.push(vec![airspeed, f_shed, STRUCT_HZ / f_shed]);
}
// gates + table: hand-rolled again
```

Friction: the march itself is clean; the sweep loop, the per-iteration
`materialize`/`.on` pair, the gate block, and the table assembly are ceremony.

### 3. Flight-envelope placard table (loads)

```rust
let matrix = read_table::<FloatType>("mach_alt_matrix.csv").run()?;   // group 1, exists
let mut rows = Vec::new();
for row in matrix.rows() {
    let (mach, alt) = (row[0], row[1]);
    let (n_inf, t_inf) = atmosphere_at(alt);
    let post = FittedNormalShock::new(GAMMA_EFF)?.post_shock(t_inf, n_inf, mach)?;
    let q = dynamic_pressure(n_inf, mach, t_inf);
    let qdot = sutton_graves(/* ... */);
    rows.push(vec![mach, alt, q, post.t2, qdot]);
}
// gates + table: hand-rolled again
```

Friction: correctly needs no march and no manifold; the DSL is not in the way. What repeats
is, again, gates and table assembly. Conclusion: the placard example needs library
conveniences, not flow syntax.

### 4. Cooling-channel pressure-drop sizing (thermal)

```rust
for &flow_rate in read_table::<FloatType>("flow_rates.csv").run()?.rows()[0].iter() {
    let case = config::channel_config(flow_rate)?;         // dec_ns + box mesh + walls
    let manifold = case.materialize()?;
    let report = CfdFlow::march(&case).on(&manifold).run()?;
    let dp = pressure_drop(report.final_field().expect("field"), N, H);
    let f_measured = friction_factor(dp, flow_rate);
    rows.push(vec![flow_rate, dp, f_measured, 64.0 / reynolds(flow_rate)]);
}
// gates + table: hand-rolled again
```

Friction: identical shape to draft 2. The pattern is now unmistakable: *sweep, extract,
gate, table* is the everyday program, and only "extract" has DSL support today.

### 5. Wind-tunnel data reduction with error bars (test)

```rust
let trace = read_sensor_trace::<FloatType>("tunnel_run_042.csv").run()?;   // group 1, exists
for draw in 0..DRAWS {
    seed_sampler(draw as u64);
    let case = config::uncertain_inflow_config(&trace)?;   // UncertainMarchConfig
    let manifold = case.materialize()?;
    let report = CfdFlow::uncertain_march(&case).on(&manifold).run()?;
    forces.push(extract_forces(&report));
}
let (mean, sd) = mean_sd(&forces);                         // hand statistics
// significance: hand-rolled sigma rule (the weather example's fourth copy)
```

Friction: the uncertain march is clean; the draw loop is a sweep; the statistics and the
significance gate are hand-rolled. The statistics half is roadmap item 3 (`Uncertain<T>`
combinator) and stays out of this review's scope; the sweep and gate halves are the same
friction as drafts 1 to 4.

## What the drafts expose, ranked by repetition

1. **Gates**: hand-rolled in every draft, already copied three times in the tree (corridor
   `utils_print`, stagline `print_utils`, weather `main`). Eight copies after group 2/3.
2. **Sweeps**: every draft is a loop over configs collecting per-case rows; the weather
   example already hand-rolls the concurrent version with `scoped_map`.
3. **Table assembly**: every draft ends by hand-building rows for the group-1 writer.
4. **The B1 pair** (`materialize` + `.on`) inside sweep bodies: correct for geometry reuse,
   ceremony when each case owns a fresh grid.
5. **The nozzle has no DSL path at all**: the only draft that falls out of the DSL entirely.

## The syntax program (all additive, verified below)

**S1: `sweep` — the parameter-study combinator** (`deep_causality_cfd::types::flow`).
Order-preserving, `Result`-collecting map over a slice of case inputs, riding
`deep_causality_par::scoped_map` under the `parallel` feature, bit-identical to sequential:

```rust
let rows: Vec<[FloatType; 3]> = sweep(&airspeeds, |&v| {
    let case = config::build_march_config(v)?;
    let report = CfdFlow::march(&case).run_owned()?;      // S4 below
    let f_shed = dominant_frequency(report.series("lift").expect("observed"), dt)?;
    Ok([v, f_shed, STRUCT_HZ / f_shed])
})?;
```

Signature: `sweep<T, U, E>(items: &[T], f: impl Fn(&T) -> Result<U, E> + Sync) -> Result<Vec<U>, E>`.
Output-agnostic on purpose: a row array, a `Report`, or a force struct all work.

**S2: `Gates` — the acceptance-gate builder** (same module as the existing gate culture):

```rust
let ok = Gates::new("nozzle operating map")
    .gate("chokes past the critical ratio", choked_ok, detail_1)
    .gate("shock position matches 1-D theory", pos_ok, detail_2)
    .finish();                    // prints the [PASS]/[FAIL] lines, returns bool
if !ok { std::process::exit(1); }
```

Exactly the fifteen lines every example hand-rolls, once. Process exit stays with the
caller (a library type never exits). The three existing implementations migrate
mechanically when touched; migration is optional because S2 is additive.

**S3: `NumericTable::from_columns`** (`deep_causality_file`). Collapse the two-step
column-vector construction:

```rust
let table = NumericTable::from_columns(
    [("airspeed", "m/s"), ("f_shed", "Hz"), ("margin", "-")],
    rows,
)?;
write_table(path, table).run()?;
```

**S4: `run_owned` — one-shot geometry** (`MarchPipeline`, and the uncertain twin). When the
case is not reusing a caller-owned manifold, materialize internally:

```rust
CfdFlow::march(&case).run_owned()?          // today: case.materialize()? + .on(&m).run()?
```

The B1 borrow form stays untouched for geometry reuse (the cavity trend loop keeps it).

**S5: `CfdFlow::duct_march` — the missing 1-D compressible path.** An owned `DuctConfig`
(area profile as a table or closure, inlet stagnation state, back pressure, stop condition)
lowered onto `CompressibleEuler1d`, returning the standard `Report` with duct observables
(`"mach_profile"`, `"pressure_profile"`, `"shock_position"`, `"thrust_coefficient"`). This
is the one moderate item: a new config container, a runner, and observables, following the
existing multi-entry pattern (`march`, `qtt_march`, `compressible_march`, `uncertain_march`
already coexist). Without it the nozzle example cannot be expressed in the DSL at all.

## Verification of the program against all five mocks

| | 1 nozzle | 2 VIV | 3 placard | 4 channel | 5 tunnel |
|---|---|---|---|---|---|
| S1 `sweep` | over back pressures | over airspeeds | over matrix rows | over flow rates | over noise draws |
| S2 `Gates` | 4 gates | 3 gates | 2 gates | 3 gates | 3 gates |
| S3 `from_columns` | map table | margin table | placard table | sizing table | forces table |
| S4 `run_owned` | n/a (duct runner) | per-case grid | n/a (no march) | per-case grid | per-case grid |
| S5 `duct_march` | **required** | n/a | n/a | n/a | n/a |

Checks worth recording. S1 is output-generic, so draft 3 (no march, plain closures) and
draft 5 (force structs, not rows) both fit; a `Report`-typed sweep would have failed both,
which is why the signature stays generic. S4 is deliberately absent from draft 3 (nothing to
materialize) and draft 1 (the duct runner owns its 1-D grid internally), present in the
three manifold-carrying drafts. S5 is single-purpose by design; the brief's "obviously not
applicable but verify anyway" applies: drafts 2 to 5 have no duct, and forcing them through
a duct entry would be shape-wrong, so its non-applicability is correct rather than a gap.
The `Report` type needed no change: `series` plus the existing observables cover every
extraction in the five drafts.

**After-shape of a whole everyday example** (draft 4 rewritten): read the flow-rate table
(group 1), one `sweep` body of five lines, `Gates` with three entries, `from_columns` plus
`write_table`. About 35 lines of program against roughly 90 in the current-syntax draft,
with no new concepts beyond the five above.

## Breaking-change assessment

None required. The review looked for existing syntax whose replacement would cut
complexity: the `CfdConfigBuilder` ceremony is already the config-as-data seam and earns
its lines; the B1 `materialize`/`.on` pair is load-bearing for geometry reuse and S4 covers
the one-shot case additively; `Report::series` fits every extraction. The only rewrite
worth doing to existing code is migrating the three hand-rolled gate blocks to S2, which is
optional, mechanical, and strictly deleting duplicated lines.

## Suggested sequencing

S3 lands with any next file-crate touch (one constructor). S1, S2, and S4 are one small
change together (they are the shared skeleton of all five examples). S5 is its own change
and gates only example 1; examples 2 to 5 can proceed the moment S1/S2/S4 exist. Then the
group 2/3 specs can reference this note instead of re-deriving the shapes.
