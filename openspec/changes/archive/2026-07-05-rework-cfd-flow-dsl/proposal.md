## Why

The CfdFlow DSL today owns only the middle of a study's data path: it marches, but the
schedule-in, rows-out, gate, and verdict stages are hand-stitched imperative glue in
each example's `main.rs`, split across `deep_causality_cfd`, `deep_causality_file`, and
`deep_causality_core`. The result reads as a sequencer, not a language: the table writer
is hardcoded to `f64` in violation of the precision-as-a-parameter rule, column names
are repeated away from the fields they describe, the five march families each have a
separate entry point, and the counterfactual, ensemble, coupling, and audit constructs
the plasma-blackout examples need have no vocabulary at all. The design is fully worked
out and its three novel type constructs are proven to compose (feasibility spike, PASS);
the window to land a no-compromise rework is now, while `deep_causality_cfd` is still
`publish = false` and the entire surface can be rebuilt with zero external breakage.

## What Changes

- **Typed table IO becomes precision-generic and schema-driven** (`deep_causality_file`):
  a `TableScalar` codec bounding both reader and writer so `read(write(t)) == t` at every
  precision (`f64`/`f32`/`Float106`); `TableRow`/`FromTableRow` putting the column schema
  on the row struct; `NumericTable::column(name)`; `read_rows`/`write_rows`. **BREAKING**:
  the writer stops taking `NumericTable<f64>` (source-compatible for `f64` callers).
- **A two-level study grammar** (`deep_causality_cfd`): the campaign phase family
  (`StudyDef → Cases → Prepared/Counterfactual/ForkStudy → Configured/Branched → Marched
  → Swept → Judged`) over a `StudyEffect` carrier that hides its haft HKT witness (the CDL
  pattern), with verbs `read`/`matrix`/`cases`, `prepare`/`case`/`baseline`/`fork`/
  `ensemble`/`sweep`, `couple`/`march`/`march_for`/`continue_for`, `reduce`/`reduce_all`/
  `reduce_ensemble`, `record`/`refine`, `gates`, `inspect`, `verdict`. Mis-ordered
  programs do not compile.
- **Counterfactuals become first-class in both forms**: origin (`baseline`/`alternate`)
  and event-fork (`fork`/`branch`/`continue_for`), both guaranteeing the
  `!!ContextAlternation!!` provenance marker.
- **The gate builder**: `GateSeq<Row>` gating sequences as named, reusable, row-typed
  values (higher-ranked gate `fn`s over a borrowed `StudyView`); `Verdict` with `merge`;
  `StudyError`.
- **One trajectory march verb**: `Marchable` unifies the five march entries; `MarchState`
  unifies pause-export, `from`-resume, and checksummed snapshot; the named-stage builder
  `.couple/.from/.until/.run/.run_for` replaces positional `run_until`.
- **The audit log** (`AuditLog`/`LogSink`, `save_log` verb): the effect log written
  stepwise to disk, one thread one file under fan-out, abort-tail preserved.
- **Single entry point**: `CfdFlow::study(..)` and `CfdFlow::march(..)` only. **BREAKING**,
  retired: `qtt_march`/`compressible_march`/`duct_march`/`uncertain_march` entries,
  positional `run_until`, `carry_field`, the eager `Gates` builder, `write_xy_csv`,
  `Report::write_series_csv`, the `write_csv` re-export, `fail`.
- **Complete migration**: remove the `examples/avionics_examples` workspace exclusion and
  migrate all seven safety examples plus the two verification-harness probe traces to the
  new DSL, reproducing every recorded `output.txt` identically.

## Capabilities

### New Capabilities
- `cfd-study-grammar`: the campaign-level study language — phase typestates, the case
  binders, sweep/march/reduce, record/refine, the `GateSeq` gate builder, `Verdict`,
  `StudyError`, and the `CfdFlow::study` entry.
- `cfd-trajectory-march`: the reworked trajectory level — `Marchable` single march verb,
  the named-stage builder (`couple`/`from`/`until`/`run`/`run_for`), `MarchState`
  (one state, two transports), and the fork (`fork`/`continue_with`/`continue_branches`).
- `cfd-counterfactual`: the two counterfactual forms (origin `baseline`/`alternate`,
  event `fork`/`branch`/`continue_for`), the alternation-marker guarantee, `ensemble`
  realizations, and `reduce_all`/`reduce_ensemble`.
- `cfd-audit-log`: `AuditLog`/`LogSink`, the `save_log` verb, stepwise flushing, the
  one-thread-one-file fan-out policy, and the abort-tail guarantee.

### Modified Capabilities
- `typed-table-io`: the writer becomes precision-generic (`TableScalar`), `TableRow`/
  `FromTableRow` and `read_rows`/`write_rows` are added, and `NumericTable::column` is
  added. This is the only existing capability whose requirements change.

## Impact

- **Crates**: `deep_causality_file` (table IO surface), `deep_causality_cfd` (the whole
  DSL). No new external dependency; `deep_causality_cfd` already depends on
  `deep_causality_file`. Both are `publish = false` or additive-for-`f64`, so no external
  breakage.
- **Cargo.toml**: the `examples/avionics_examples` entry moves from `exclude` back into
  the workspace `members`.
- **Examples**: nozzle, VIV, placard, plasma-blackout corridor, plasma-blackout weather,
  the stagnation-line study, and `compressible_carrier_timing` are rewritten to the
  grammar; the cylinder-wake and lid-cavity verification harnesses move their probe
  traces to `write_rows`.
- **Gate**: the compile spike validating the three novel type constructs has already
  passed (see `openspec/notes/cfd-dsl/04-dsl-feasibility.md`); it is recorded as task
  group 0, satisfied.
- **Acceptance**: each verb carries a four-link chain — stated guarantee, enforcing test
  (behavioral or `compile_fail`), shipped example, doc page (`03-dsl-acceptance.md`).
