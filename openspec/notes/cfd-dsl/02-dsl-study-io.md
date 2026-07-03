[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Proposal: The CfdFlow Language, Complete Rework

STATUS: proposed, awaiting review. Revision 4: rev 3 reviewed against the two
plasma-blackout examples; five gaps found and closed (section 1a). Revision 4.1 adds
the implementation substrate: the CDL effect pattern over haft HKT witnesses, hidden
behind the facade (section 3, "The substrate"). The table schema pieces (P1 to P4) are
accepted; the two-level structure and single entry point of rev 3 stand.

## 0. Design Rules, Fixed by Review

1. **One entry point.** The language is `CfdFlow`. A campaign enters through
   `CfdFlow::study(..)`; a trajectory enters through `CfdFlow::march(..)`. No second
   facade type, no free-standing constructors.
2. **No process concerns in the language.** `verdict()` returns a `Verdict` value. The
   DSL never exits, never prints to stderr, never touches the process.
3. **Syntax is semantics.** Every verb's guarantee is stated and holds exactly.
4. **Types own the order.** A verb exists only on the phase where it is meaningful; a
   mis-ordered program does not compile.
5. **Complete migration.** Every march family, every example, every legacy IO surface
   moves in this change.

## 1. The Language Has Two Levels, Connected by Three Laws

**The trajectory level**: one flight, one duct, one wake. Configure, couple the
physics, march, pause at events, fork, resume, report.

**The campaign level**: a family of cases. Load the matrix, bind worlds, sweep, reduce
to rows, record, gate, verdict.

The laws:

- **A report reduces to a row.** The trajectory's output is the campaign's data point.
- **A fork's branches are a study's cases.** A counterfactual sweep is a campaign whose
  fork point is a paused trajectory and whose case axis is the command distinguishing
  the branches.
- **A counterfactual is an alternation with a witness.** Every counterfactual world,
  whether alternated at the origin or forked at an event, carries the
  `!!ContextAlternation!!` provenance marker naming what it is a counterfactual of. The
  grammar guarantees the marker; a gate can demand it.

### 1a. What the review of the two plasma-blackout examples found

Rev 3 was checked verb-by-verb against `plasma_blackout_corridor/main.rs` and
`plasma_blackout_weather/main.rs`. Five constructs those programs use had no grammar
representation:

1. **The coupling seam was invisible.** Every coupled march passes a multiphysics stack
   (`world::corridor_coupling(bias, draw)`): flow, reacting plasma, regime
   classification, navigation, and the cybernetic envelope, composed as data. Rev 3
   buried it inside an opaque `run_until` argument. The seam is the multi-physics
   heart of the crate and becomes the `couple` verb at both levels.
2. **Only one of the two counterfactual forms existed.** The corridor forks at a
   flow-resolved event (shared history, O(1) copy-on-write). The weather table
   alternates whole atmospheres at the origin (shared baseline description, flown from
   scratch). Rev 3 had the fork form only; the origin form (`alternate_context` before
   the run) is now the `baseline`/`alternate` pair.
3. **The ensemble axis was missing.** The weather campaign flies every condition
   `MC_DRAWS` times with deterministic receiver-noise realizations and reduces each
   condition's draw set to mean and scatter; today that is a hand-rolled
   `flat_map` over `(condition, draw)` tuples. It becomes the `ensemble` verb.
4. **Cross-case reduction was missing.** The corridor's aim point derives from the
   ballistic branch's terminal state, then every branch scores against it: a reduction
   that must see all reports at once. `reduce` is per-case; `reduce_all` is the
   collective form.
5. **Mixed programs need one report.** The corridor is a trajectory program with an
   embedded campaign; its gates read leg witnesses, branch rows, compression, and
   provenance together. `Verdict::merge` composes reports, so a mixed program still
   ends in one verdict.

## 2. Control Flow: Where Loops and Branches Live

Normative; a construct must not appear at two levels.

| Control flow | Owner | Syntax | Semantics |
|---|---|---|---|
| Time/space stepping | solver | `MarchStop { max_steps, residual_tol }` | innermost loop; bounded, stop-criterion typed, never user-written |
| Event-bounded legs | trajectory | chained `.from(state).until(event)` | a fold over pauses; each leg starts from the carried state of the last |
| Origin counterfactuals | campaign | `.baseline(w).alternate(f)` | each case a world alternated from one declared baseline, marker guaranteed, flown from scratch |
| Event-fork counterfactuals | campaign | `.fork(&pause).branch(f).continue_for(n)` | branches share history to the pause bit-identically, O(1) copy-on-write, flown concurrently |
| The case sweep | campaign | `.sweep(f)` / `.march()` / `.continue_for(n)` | the data-parallel loop; order-preserving, first-error-wins |
| Ensemble realizations | campaign | `.ensemble(n)` | each case flown `n` times, the draw index threaded to the coupling; the reduction sees the draw set |
| Refinement rounds | campaign | `.refine(f)` then re-bind and re-run | study-level feedback; each round explicit, all rounds retained for the gates |
| Regime transitions | physics stages | `RegimeClassify` and friends inside the coupled step | data-driven switching inside the march; model content, not control syntax |
| Solver-family dispatch | types | the config type, via `Marchable` | which solver marches is decided at compile time |

There is deliberately no general `loop` or `if` verb at the campaign level. A study
needing arbitrary control flow between phases is two studies, and Rust composes them.

## 3. The Campaign Level

### Entry

```rust
impl CfdFlow {
    /// Open a named study. The name titles the verdict report.
    pub fn study(title: &str) -> StudyDef;
}

impl StudyDef {
    /// Audit this study: attach a stepwise disk sink to the effect log (section
    /// "The audit log"). Console behavior is unchanged; declared before the cases so
    /// the whole study is on the record.
    pub fn save_log(self, path: impl AsRef<Path>) -> StudyDef;
    /// The typed test matrix: one case per file row, columns matched to the schema by name.
    pub fn matrix<T: FromTableRow>(self, path: impl AsRef<Path>) -> Cases<T>;
    /// Read one named column as the case axis.
    pub fn read<R: TableScalar>(self, path: impl AsRef<Path>, column: &str) -> Cases<R>;
    /// An in-memory case list.
    pub fn cases<T>(self, cases: Vec<T>) -> Cases<T>;
}
```

### The substrate: plain phases inside one effect (the CDL pattern)

The Causal Discovery Language (`deep_causality_discovery`) already ships the
implementation pattern this grammar needs, proven in-repo, and the study grammar
adopts it:

- **Phases are plain data.** CDL's typestates (`SurdConfigured<T>` through
  `WithAnalysis<T>`) are dumb structs carrying exactly what exists at that point, the
  run config traveling inside the state rather than in a separate object. The study
  phases below are built the same way.
- **One effect carrier owns the channels.** CDL wraps every phase in
  `CdlEffect<T> { inner: Result<T, CdlError>, warnings: CdlWarningLog }`. The study
  grammar mirrors it as `StudyEffect<T>` over `StudyError` and a study warning log:
  errors short-circuit, warnings accumulate, both travel invisibly.
- **The HKT witness stays behind the facade.** `CdlEffectWitness<E, W>` fixes the
  error and warning types and implements haft's `Functor`/`Applicative`/`Monad`; the
  fluent surface delegates to it, and no witness type appears in any signature a user
  reads. `StudyEffectWitness` does the same, so the grammar gets lawful monadic
  composition without exposing one HKT bound.
- **Every verb is implemented twice, once real and once fluent.** CDL puts the logic
  on `CDL<State>` (returning `CdlEffect<CDL<Next>>`) and a one-line forwarding impl on
  `CdlEffect<CDL<State>>` via `and_then`; that is why a pipeline chains
  `.load(..).clean(..).discover(..)` without a `bind` in sight. The study verbs use
  the identical double impl, which keeps `main` one expression while the effect
  threads underneath.
- **Precedent for the converging binders.** CDL runs two compile-time-isolated
  sub-pipelines (SURD, BRCD) that converge on one `WithAnalysis` tail; crossing them
  is a compile error. The three case binders (`case`, `baseline`/`alternate`,
  `fork`/`branch`) converging on `Swept` are the same shape.

The warning channel is a genuine addition, not plumbing: non-fatal diagnostics get a
first-class home. A `force_load` snapshot override's warnings, a clamped fine-sweep
candidate duplicating at the span edge, a solver fallback: each accumulates in the
effect and renders in the `Verdict` report. Warnings never gate; gates gate.

### Phases

Every phase rides inside the `StudyEffect` carrier and short-circuits; the first
failure is tagged with its verb and surfaces at `verdict()`, warnings alongside.

| Phase | What exists | Verbs |
|---|---|---|
| `Cases<T>` | the typed case axis | `prepare`, `case`, `baseline`, `fork`, `ensemble`, `sweep`, `inspect` |
| `Prepared<T, A>` | cases plus shared apparatus | `sweep`, `inspect` |
| `Counterfactual<T>` | cases plus the declared baseline world | `alternate` |
| `ForkStudy<'p, T>` | cases plus the shared fork point | `branch` |
| `Configured<T, C>` | one bound world/config per case | `couple`, `march`, `march_for`, `inspect` |
| `Branched<'p, T>` | one branch world per case at the fork | `continue_for` |
| `Marched<T, C, R>` | one report per case (or per case and draw) | `reduce`, `reduce_all`, `reduce_ensemble`, `inspect` |
| `Swept<Row, Rig>` | the reduced rows, rig retained | `record`, `refine`, `gates`, `inspect` |
| `Judged<Row, Rig>` | rows plus accumulated gating sequences | `gates`, `inspect`, `verdict` |

### The case binders (three, each with its provenance semantics)

```rust
impl<T> Cases<T> {
    /// Plain binding: one owned solver config per case (the model_config seam).
    pub fn case<C>(self, f: impl Fn(&T) -> Result<C, PhysicsError>) -> Configured<T, C>;

    /// Declare the validated origin world. The study's counterfactuals alternate from it.
    pub fn baseline<W>(self, origin: impl FnOnce() -> Result<W, PhysicsError>) -> Counterfactual<T>;

    /// Declare the shared fork point: a paused trajectory this study's cases continue from.
    pub fn fork<'p>(self, pause: &'p CompressiblePause<'_, R, S>) -> ForkStudy<'p, T>;
}

impl<T> Counterfactual<T> {
    /// Bind each case to a world alternated from the baseline. Guarantee: every bound
    /// world carries the alternation marker naming the baseline; the baseline itself
    /// (if present in the case axis) binds unmarked.
    pub fn alternate<W>(self, f: impl Fn(&T) -> Result<W, PhysicsError>) -> Configured<T, W>;
}

impl<'p, T> ForkStudy<'p, T> {
    /// Bind each case to a branch world at the fork. Guarantee: every branch resumes
    /// the pause's state bit-identically, alternated and marked, sharing it copy-on-write.
    pub fn branch<W>(self, f: impl Fn(&T) -> Result<W, PhysicsError>) -> Branched<'p, T>;
}
```

### The multiphysics seam and the runs

```rust
impl<T, C> Configured<T, C> {
    /// Attach the coupled-physics stack per case and draw: flow, reacting plasma,
    /// regime classification, navigation, envelope control, composed as data. The
    /// draw index is 0 unless `ensemble` raised the multiplicity.
    pub fn couple<S>(self, f: impl Fn(&T, usize) -> S) -> Configured<T, Coupled<C, S>>;

    /// March every case to its stop criterion (C: Marchable selects the solver).
    pub fn march(self) -> Marched<T, C, R>;
    /// March every case a fixed horizon (the weather table's form).
    pub fn march_for(self, steps: usize) -> Marched<T, C, R>;
}

impl<'p, T> Branched<'p, T> {
    /// Fly every branch from the shared fork, concurrently, reports in case order.
    pub fn continue_for(self, steps: usize) -> Marched<T, BranchWorld, R>;
}

impl<T> Cases<T> {
    /// Raise the sweep multiplicity: each case flies `draws` times; the draw index
    /// reaches the coupling; reductions see the whole draw set per case.
    pub fn ensemble(self, draws: usize) -> Cases<T>;   // recorded on the phase, applied at march
}
```

### The reductions

```rust
impl<T, C, R> Marched<T, C, R> {
    /// One report, one row.
    pub fn reduce<Row>(self, f: impl Fn(&CaseRun<'_, T, C, R>) -> Result<Row, PhysicsError>)
        -> Swept<Row, ()>;

    /// The collective form: sees every case's run at once, returns one row per case in
    /// order. For cross-case references (the corridor's shared aim point from the
    /// ballistic branch).
    pub fn reduce_all<Row>(self, f: impl FnOnce(&[CaseRun<'_, T, C, R>]) -> Result<Vec<Row>, PhysicsError>)
        -> Swept<Row, ()>;

    /// The ensemble form: one case and its draw set in, one row out (means, scatters,
    /// worst draws computed where the data is).
    pub fn reduce_ensemble<Row>(self, f: impl Fn(&T, &[Report<R>]) -> Result<Row, PhysicsError>)
        -> Swept<Row, ()>;
}
```

### The gate builder: gating sequences as values

Gates are not chained onto the study one by one. A gating sequence is built once, as a
named value with the standard Rust builder pattern, and the study takes only that type.
The sequence lives in `model.rs`, which makes it the documented, reviewable definition
of what this study (or this phase of a complex workflow) must satisfy; a second
sequence for a later stage is a second value:

```rust
/// A named, ordered gating sequence over a subject `S`: what one judgment pass checks.
/// Checks are plain `fn` pointers (static dispatch, no boxing); gate checks are free
/// functions in `model.rs` by convention, so the sequence is a list of named functions.
pub struct GateSeq<S> { /* title, Vec<(&'static str, fn(&S) -> (bool, String))> */ }

impl<S> GateSeq<S> {
    pub fn new(title: &str) -> Self;
    pub fn gate(self, label: &'static str, check: fn(&S) -> (bool, String)) -> Self;
    /// Run the sequence against a subject and return the report. Usable directly by
    /// trajectory-level programs (the corridor's leg gates); the campaign's `gates`
    /// verb lowers onto this.
    pub fn check(&self, subject: &S) -> Verdict;
}
```

The campaign fixes the subject to the study view, and judgment takes sequences whole:

```rust
impl<Row, Rig> Swept<Row, Rig> {
    /// Typed table out; schema from the row type; precision from its TableScalar.
    /// Recording precedes judgment by construction.
    pub fn record(self, path: impl AsRef<Path>) -> Self where Row: TableRow;
    /// Next refinement round from the rows so far; the rig (fork point, apparatus)
    /// carries over; all rounds stay readable in the gates.
    pub fn refine<T2>(self, f: impl FnOnce(&[Row]) -> Result<Vec<T2>, PhysicsError>) -> /* re-bind phase */;
    /// Insert a whole gating sequence. The sequence is typed by the row, so a
    /// sequence built for another study's rows does not compile here.
    pub fn gates(self, seq: GateSeq<StudyView<'static, Row>>) -> Judged<Row, Rig>;
}

impl<Row, Rig> Judged<Row, Rig> {
    /// A complex workflow inserts its later gating sequences the same way.
    pub fn gates(self, seq: GateSeq<StudyView<'static, Row>>) -> Self;
    /// Terminal. No printing, no exit. A carried error names the verb that failed.
    pub fn verdict(self) -> Result<Verdict, StudyError>;
}

impl Verdict {
    pub fn passed(&self) -> bool;
    /// Compose reports: a mixed program (trajectory gates + campaign gates) ends in one.
    pub fn merge(self, other: Verdict) -> Verdict;
    /// The accumulated non-fatal diagnostics from the study's effect channel
    /// (force_load overrides, clamped candidates, solver fallbacks). Rendered by
    /// Display; never part of pass/fail.
    pub fn warnings(&self) -> &[StudyWarning];
}
```

The shipped `Gates` runtime builder (eager, printing, bool-returning) retires into
`GateSeq`: same build-up pattern, but the sequence is a first-class value, the checks
are deferred until `check`/`verdict`, and the result is a `Verdict` instead of a
printed side effect.

### The audit log: provenance to disk (`save_log`)

The effect log rides the coupled field through the entire dynamic process: regime
transitions, navigation-mode changes, carrier rebuilds, bounded corrections,
alternation markers. For audited runs that record must survive the process. The
custom log type:

```rust
/// The effect log plus an optional stepwise disk sink. Without a sink this is
/// exactly today's in-memory log with console rendering; nothing changes. With a
/// sink attached, every entry is appended to the file and flushed the moment it is
/// recorded.
pub struct AuditLog { /* entries: EffectLog, sink: Option<LogSink> */ }
```

One verb at each level, taking the path:

```rust
CfdFlow::study("weather-dispersion table")
    .save_log(&audit_path())        // the whole campaign on the record
    ...

CfdFlow::march(&nominal)
    .save_log(&audit_path())        // one trajectory on the record
    .couple(..).from(..).until(..)
```

The guarantees:

- **Console unchanged.** `save_log` attaches the sink and changes nothing else; a
  study without the verb behaves exactly as today.
- **Stepwise, not end-of-run.** Each entry is appended and flushed as it is
  recorded. The abort-tail property follows: when the process dies for any reason,
  the file ends at the last event before the abort, and the tail tells the engineer
  what went wrong. (Flush-per-entry survives a process abort or panic; surviving
  power loss needs fsync-per-entry, available as an option and off by default.)
- **Complete at completion.** A finished run's file is the entire effect log, closed
  by the verdict summary, so the disk record and the in-memory log render
  identically.
- **One thread, one file.** The main log file is the main process and is the single
  source of truth up to any concurrent fan-out. When the process fans out (a
  counterfactual fork, a case sweep, an ensemble), every concurrently running branch
  gets its own file, exclusively written by its own thread, per-entry stepwise like
  the main file. Branch files derive their names from the main path plus the
  fan-out's enumeration and the case's own name; a dynamic process can fork more
  than once, so rounds are numbered:

  ```
  corridor.audit.log                                  // the main process
  corridor.audit.counterfactual-1.bank_15_deg.log     // coarse round, one branch
  corridor.audit.counterfactual-2.fine_bank_07.log    // fine round, one branch
  weather.audit.sweep-1.polar_winter.draw-3.log       // a swept ensemble member
  ```

  The main file records the fan-out event naming every branch file it spawned, and
  the rejoin naming each branch's outcome. No file is ever written by two threads,
  no lines interleave, and auditing a single branch is reading a single file, which
  simplifies concurrency debugging by a wide margin. The abort-tail property holds
  per file: a branch that dies leaves its own file ending at its last event, and the
  main file names which branch died.
- **Loud on failure.** A sink write error fails the run at that point with the IO
  error; an audited run that can no longer be audited does not silently continue.

### What no longer compiles

- `record`/`gates` before rows exist; `verdict` without a gating sequence; `reduce`
  without a march; `march` without a binder; recording without a schema (all as rev 3).
- Inserting the wrong gating sequence: `GateSeq` is typed by the study's row, so
  sequence A built for one workflow cannot be placed into a study whose rows it does
  not understand.
- `branch` without a declared fork point: `branch` exists only on `ForkStudy`.
- `alternate` without a declared baseline: `alternate` exists only on `Counterfactual`.
- `continue_for` on origin-form cases: only `Branched` has it; origin worlds march,
  fork branches continue.
- `reduce_ensemble` on a study that never called `ensemble` (draw multiplicity is in
  the phase type).

## 4. The Trajectory Level, Reworked

### One march verb, named stages instead of positional arguments

Today's `run_until(coupling, field, trigger, t0, predicate)` carries five positional
arguments; the redesign names them, and the trigger and start time fold into the
descriptions they belong to (the trigger into the coupling stack, the clock into the
state):

```rust
CfdFlow::march(&world)                  // C: Marchable picks the solver family
    .alternate(&other_world)            // optional: the origin-form counterfactual swap
    .save_log(path)                     // optional: the audit sink, stepwise to disk
    .couple(stack)                      // the multiphysics seam (coupled families)
    .from(state)                        // MarchState::fresh() | pause.state() | loaded
    .until(event)?                      // -> CompressiblePause: the leg's end
    // or .run()? / .run_for(steps)?    // -> Report: terminal forms
```

`Marchable` (unchanged from rev 3) collapses the five named entries into `march`;
example-local case types implement it by delegation.

### One state, two transports

`MarchState<R>` unifies what a pause exports (`pause.state()`), what `from` accepts,
and what the checksummed snapshot stores; it carries fields, scalars, ambient,
navigation engine, provenance log, step index, and clock. What you pause is what you
resume, next line or next week; `carry_field` retires into it.

### The fork

```rust
impl CompressiblePause<'_, R, S> {
    pub fn fork(&self) -> CompressibleFork<'_, R>;
}
impl CompressibleFork<'_, R> {
    /// One branch world, one continued report; O(1) copy-on-write divergence.
    pub fn continue_with(&self, world: &W, steps: usize) -> Result<Report<R>, PhysicsError>;
    /// The batch form (concurrent, world order) remains.
    pub fn continue_branches(..) -> ..;
}
```

The campaign's `fork`/`branch`/`continue_for` lowers onto exactly this.

## 5. The Foundation (accepted in review, restated)

- **P1** `TableScalar`: one codec bounding reader and writer; `f64`/`f32` shortest
  round-trip, `Float106` as the exact `hi|lo` pair cell with plain-decimal fallback.
- **P2** `TableRow` / `FromTableRow`: the schema on the row struct in `model.rs`.
- **P3** File verbs in `deep_causality_file`; the campaign's `matrix`/`read`/`record`
  are their only example-facing form.
- **P4** `NumericTable::column(name) -> Result`, naming absent columns.
- **P5** `StudyError` (verb-tagged) wrapping `PhysicsError` and `DataLoadingError`.

## 6. The Programs, After

Caller-side process mapping, once per example: print the merged `Verdict`, exit 1 on
`!passed()`, exit 2 on `Err`.

### 6.1 The nozzle operating map

The gating sequence is a named value in `model.rs`, the documented definition of what
this study must satisfy:

```rust
// model.rs
pub fn nozzle_gates() -> GateSeq<StudyView<'static, MapRow>> {
    GateSeq::new("nozzle operating map")
        .gate("choking", gate_choking)
        .gate("shock position", gate_shock_position)
        .gate("shock-free profiles", gate_area_mach)
        .gate("physical thrust", gate_thrust)
}
```

and the study places it whole:

```rust
CfdFlow::study("nozzle operating map")
    .read::<FloatType>(&input_path_from_args(), "p_back_over_p0")
    .inspect(utils_print::intro)
    .case(model_config::duct_case)
    .march()
    .reduce(model::map_row)
    .inspect(utils_print::rows)
    .record(&out_path())
    .gates(model::nozzle_gates())
    .verdict()
```

### 6.2 The vortex-shedding resonance margin

The gating sequence in `model.rs`:

```rust
pub fn viv_gates() -> GateSeq<StudyView<'static, MarginRow>> {
    GateSeq::new("vortex-shedding resonance margin")
        .gate("strouhal band", gate_strouhal_band)
        .gate("finite margins", gate_finite_margins)
}
```

The study, with `model_config::wake_case` binding the example-local `Marchable`
(`WakeCase` carrying its `dt`):

```rust
CfdFlow::study("vortex-shedding resonance margin")
    .read::<FloatType>(&model::example_file("airspeeds.csv"), "airspeed")
    .inspect(utils_print::intro)
    .case(model_config::wake_case)
    .march()
    .reduce(model::margin_row)
    .inspect(utils_print::rows)
    .record(&model::example_file("viv_resonance_margin.csv"))
    .gates(model::viv_gates())
    .verdict()
```

### 6.3 The flight-envelope placard

The gating sequence in `model.rs`:

```rust
pub fn placard_gates() -> GateSeq<StudyView<'static, PlacardRow>> {
    GateSeq::new("flight envelope placard")
        .gate("q-max placard", gate_q_max)
        .gate("stagnation temperature", gate_stagnation_temperature)
}
```

The pointwise study:

```rust
CfdFlow::study("flight envelope placard")
    .matrix::<FlightPoint>(&model_config::matrix_path())
    .inspect(utils_print::intro)
    .prepare(model_config::shock_model)
    .sweep(model::placard_point)
    .inspect(utils_print::rows)
    .record(&model_config::table_path())
    .gates(model::placard_gates())
    .verdict()
```

### 6.4 The weather-dispersion table (origin counterfactuals, ensemble, coupling)

```rust
CfdFlow::study("weather-dispersion table")
    .cases(constants::WEATHER.to_vec())
    .baseline(model::standard_day)           // the validated origin, built once
    .alternate(model::weather_world)         // five counterfactual atmospheres, marked
    .ensemble(constants::MC_DRAWS)           // deterministic receiver-noise draws
    .couple(|case, draw| world::corridor_coupling(model::bias_departure(case.d_temp), draw))
    .march_for(constants::STEPS)             // fixed horizon, concurrent over (case, draw)
    .reduce_ensemble(model::world_row)       // draw sets collapse to mean / scatter / worst
    .inspect(utils_print::rows)
    .record(&table_path())
    .gates(model::weather_gates())           // six gates, one named sequence in model.rs
    .verdict()
```

The gating sequence in `model.rs`, the certification checklist of the table as one
reviewable value:

```rust
pub fn weather_gates() -> GateSeq<StudyView<'static, WorldRow>> {
    GateSeq::new("weather-dispersion table")
        .gate("counterfactual audit trail", gate_markers)              // law 3, checked
        .gate("flow-resolved windows everywhere", gate_windows)
        .gate("weather moves the window", gate_window_spread)
        .gate("cold drift factor", gate_cold_drift)
        .gate("cold effect statistically resolved", gate_cold_sigma)   // above receiver noise
        .gate("every weather reacquires, every draw", gate_reacq)
}
```

Today's hand-rolled `flat_map` over `(condition, draw)`, the manual regrouping into
draw sets, the inline closure-`gate` block, and the baseline-versus-alternated `if` on
the run builder are all grammar now. The wall-clock gate stays with the caller (it
times the whole program, which the study cannot see).

### 6.5 The plasma-blackout corridor (both levels, both counterfactual forms' sibling)

This is the two-sequence workflow the gate builder exists for: the campaign sequence
judges the branch study, the leg sequence judges the flown trajectory, and each is one
named value in `model.rs`:

```rust
pub fn corridor_gates() -> GateSeq<StudyView<'static, BranchRow>> {
    GateSeq::new("bank-angle corridor")
        .gate("steering beats ballistic", gate_steering)
        .gate("fine at least coarse", gate_refinement)     // reads view.rounds()
}

pub fn leg_gates() -> GateSeq<LegSet> {
    GateSeq::new("corridor legs")
        .gate("flow-resolved onset", gate_onset)
        .gate("peak passage in envelope", gate_peak)
        .gate("flow-resolved exit", gate_exit)
        .gate("reacquisition collapses drift", gate_reacquisition)
        .gate("tensor compression holds", gate_compression)
        .gate("provenance complete", gate_provenance)
}
```

The program:

```rust
// The trunk: trajectory level, to the flow-resolved onset.
let onset = CfdFlow::march(&nominal)
    .couple(world::corridor_coupling(1.0, 0))
    .from(MarchState::fresh())
    .until(model::blackout_onset)?;
let leg1 = model::snapshot("descent to blackout onset", &onset);

// The bank-command study: campaign level, event-fork counterfactuals, two rounds.
let corridor = CfdFlow::study("bank-angle corridor")
    .cases(constants::BANK_ANGLES_DEG.to_vec())
    .fork(&onset)                            // the shared flow-resolved fork point
    .branch(model::bank_world)               // one alternated world per command, marked
    .continue_for(constants::BRANCH_STEPS)   // concurrent, copy-on-write
    .reduce_all(model::score_branches)       // aim point from the ballistic branch first
    .inspect(utils_print::coarse_branches)
    .refine(model::fine_candidates)          // 0.5-deg bracket around the winner
    .branch(model::bank_world)
    .continue_for(constants::BRANCH_STEPS)
    .reduce_all(model::score_branches)       // same aim point: rounds stay comparable
    .inspect(utils_print::fine_branches)
    .record(&corridor_table_path())
    .gates(model::corridor_gates())          // steering beats ballistic; fine at least coarse
    .verdict()?;

// The committed world flies the diagnostic legs: trajectory level again.
let committed = corridor_committed_world(&corridor);
let peak = CfdFlow::march(&nominal).alternate(committed)
    .couple(world::corridor_coupling(1.0, 0))
    .from(onset.state())
    .until(model::peak_passage_61km)?;
let exit_pause = CfdFlow::march(&nominal).alternate(committed)
    .couple(world::corridor_coupling(1.0, 0))
    .from(peak.state())
    .until(model::link_recovered)?;
let reacq = CfdFlow::march(&nominal).alternate(committed)
    .couple(world::corridor_coupling(1.0, 0))
    .from(exit_pause.state())
    .run_for(constants::REACQ_STEPS)?;

// One report: the campaign verdict merged with the leg gating sequence, applied to
// the leg witnesses through the same GateSeq machinery (trajectory-level check()).
let legs = model::LegSet::new(&leg1, &peak, &exit_pause, &reacq);
let verdict = corridor.merge(model::leg_gates().check(&legs));
```

### 6.6 The verification studies

MMS and operator convergence: cases are the resolution ladder, `case` binds a
`VerifyConfig`, `reduce` reads error norms, the observed-order gate reads the whole row
set. Their bespoke driver loops migrate onto the grammar.

## 7. Migration, Complete

| Piece | Change |
|---|---|
| `deep_causality_file` | `TableScalar`; generic writer; `TableRow`/`FromTableRow`; `column()`; `read_rows`/`write_rows` |
| `deep_causality_cfd` new | the phase family (section 3), `GateSeq`, `StudyView`, `CaseRun`, `Verdict` (with `merge`), `StudyError`, `Marchable`, `MarchState`, `AuditLog`/`LogSink` with the `save_log` verb, the named-stage march builder |
| `CfdFlow` | gains `study`; `march` becomes the single trajectory verb |
| Retired in this change | `qtt_march`/`compressible_march`/`duct_march`/`uncertain_march`; positional `run_until`; `carry_field`; the eager `Gates` builder (into `GateSeq`); `write_xy_csv`; `Report::write_series_csv`; the `write_csv` re-export; `fail`; direct example table IO |
| Examples migrated now | nozzle, VIV, placard, plasma-blackout corridor, plasma-blackout weather, the stagnation-line study, `compressible_carrier_timing` |
| Verification harness output | the two `print_utils.rs` probe traces (cylinder wake, lid cavity) move from `write_xy_csv`/`write_series_csv` to `write_rows` over two-column row types |
| Verification drivers | MMS / operator-study loops onto the grammar |
| Specs | delta on `typed-table-io`; new `cfd-study-grammar`; deltas on march capabilities for `Marchable`/`MarchState`/named stages |

Retired means removed from the public surface with all call sites migrated in the same
change; per the golden rules the removals land as reviewable diff, never as silent
deletions.

## 8. Decisions Taken

- Entry: `CfdFlow::study(..)` and `CfdFlow::march(..)` only.
- `verdict()` returns `Result<Verdict, StudyError>`; `Verdict::merge` composes mixed
  programs; `Display` renders; the DSL never exits or prints.
- Counterfactuals are first-class in both forms: `baseline`/`alternate` (origin) and
  `fork`/`branch`/`continue_for` (event), both marker-guaranteed (law 3).
- The multiphysics stack is the `couple` verb at both levels; the blackout trigger
  folds into the coupling description, the clock into `MarchState`.
- Ensembles are a case-axis multiplier (`ensemble(n)`), reduced by `reduce_ensemble`.
- Cross-case reductions are `reduce_all`, order-preserving, one row per case.
- **Gating sequences are values.** `GateSeq<S>` is built once with the standard
  builder pattern, named, and documented in `model.rs`; the study's `gates` verb takes
  only that type. A complex workflow defines sequence A and sequence B as two values
  and places each at its stage. Checks are plain `fn` pointers (static dispatch, no
  `dyn`); sequences are row-typed, so misplacement does not compile. Gate checks
  receive `&StudyView<Row>` (rows, rounds, case count, title); trajectory programs run
  the same sequences directly via `check(&subject)`. The eager printing `Gates`
  builder retires into it.
- `Float106` cells: the `hi|lo` pair encoding.
- **One write.** The language's only write is `record` (campaign) over `write_rows`
  (file layer). `write_xy_csv` and `Report::write_series_csv` are ten-line string
  wrappers over core's `write_csv` and both retire; an `(x, y)` probe trace is a
  two-column `TableRow`, which also gains it units and the exact-precision codec that
  the `Display`-stringifying path loses. Core's `write_csv` remains what it is, the
  core crate's low-level string file action, no longer re-exported or documented as
  CFD-facing.
- `FromTableRow` ships now; the placard consumes it.
- **The audit log is a verb, and it is stepwise.** `save_log(path)` at either level
  attaches a disk sink to the effect log; entries append and flush as they are
  recorded (the abort-tail property: a dead process's file ends at the event that
  preceded the death), the completed file equals the rendered in-memory log plus the
  verdict summary, and a sink failure fails the run loudly. Console behavior without
  the verb is unchanged.
- **One thread, one audit file.** Under any concurrent fan-out (counterfactual fork,
  sweep, ensemble) each branch thread exclusively writes its own file, named from
  the main path plus the numbered fan-out and the case name
  (`corridor.audit.counterfactual-1.bank_15_deg.log`). The main file is the single
  source of truth up to the fan-out, records the spawn naming every branch file, and
  records the rejoin with each branch's outcome. Abort-tail holds per file.
- **The substrate is the CDL pattern.** Plain typestate phases wrapped in one
  `StudyEffect` carrier whose haft HKT witness (`StudyEffectWitness`) provides lawful
  `Functor`/`Applicative`/`Monad` composition; witness types never appear in
  user-facing signatures; every verb ships the CDL double impl (logic on the phase,
  one-line fluent forwarding on the effect). This buys the warning channel for free:
  non-fatal diagnostics (force_load overrides, clamped candidates, solver fallbacks)
  accumulate and render in the `Verdict`. Later unification under the Causal Arrow
  thesis remains open and nothing waits for it.
