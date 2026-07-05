## Context

The full design is worked out across four notes under `openspec/notes/cfd-dsl/`:
`01-dsl-review.md` (the shipped S1–S5 study primitives), `02-dsl-study-io.md` (the
complete language, rev 4.1, the normative reference for every verb and phase),
`03-dsl-acceptance.md` (the per-verb acceptance chain), and `04-dsl-feasibility.md` (the
compile spike, PASS). This document records the architecture-level decisions and their
rationale; the exact signatures live in `02-dsl-study-io.md` and are not duplicated here.

Current state: the CfdFlow DSL marches but does not own its data path. Five separate
march entries (`march`/`qtt_march`/`compressible_march`/`duct_march`/`uncertain_march`),
a positional five-argument `run_until`, a table writer hardcoded to `f64`, four IO
surfaces across three crates, and no vocabulary for the counterfactual, ensemble,
coupling, or audit constructs the plasma-blackout examples already use by hand. The crate
is `publish = false`, so the surface can be rebuilt without external breakage; that window
closes at first publication.

## Goals / Non-Goals

**Goals:**
- One entry point (`CfdFlow::study`, `CfdFlow::march`) and a two-level grammar where a
  mis-ordered program does not compile.
- Precision as a parameter end to end: no `f64` in the data path except at a `Display`
  boundary; the table codec round-trips `f64`/`f32`/`Float106` bit-exactly.
- First-class counterfactuals (both forms), ensembles, the multiphysics coupling seam,
  reusable gating sequences, and a crash-durable audit log.
- Complete migration: all seven safety examples and the two verification harnesses on the
  grammar, recorded outputs reproduced identically, `examples/avionics_examples` back in
  the workspace.

**Non-Goals:**
- Field export (VTK/CGNS) stays on the roadmap; the audit log and typed tables are the
  only new outputs.
- No general `loop`/`if` verb at the campaign level; arbitrary control flow between phases
  is two studies composed in Rust.
- No unification with `CausalArrow`/`CausalFlow` in this change; the verbs are Kleisli
  arrows, so that unification stays open but nothing here waits for it.
- No `fsync`-per-entry by default (power-loss durability is an opt-in flag).

## Decisions

**D1. The substrate is the CDL pattern (plain phases inside one effect).** Each phase is
a plain typestate struct; all phases ride inside a `StudyEffect<T>` carrier whose haft HKT
witness (`StudyEffectWitness`, hidden) supplies lawful `Functor`/`Applicative`/`Monad`;
every verb ships the CDL double impl (logic on the phase, one-line fluent forwarding on
the effect). *Why over a hand-rolled builder:* `deep_causality_discovery` already ships
this exact architecture compiled and tested, so the substrate carries no risk, and it buys
the warning channel for free (non-fatal diagnostics accumulate and render in the
`Verdict`). *Alternative rejected:* a bespoke `Result`-threading builder — reinvents CDL
without its proof and without the witness-hidden monad laws.

**D2. `GateSeq<Row>` with higher-ranked gate `fn`s, not `GateSeq<StudyView<'static,Row>>`.**
Feasibility F1: a `'static` view cannot borrow the run's rows. The sequence is
parameterized by the row; each check is `for<'a> fn(&StudyView<'a, Row>) -> (bool,String)`.
*Why `fn` pointers over boxed closures:* static dispatch, no `dyn`, and it matches the
house convention that gate checks are free functions in `model.rs`. Sequences are
row-typed, so misplacing one does not compile (verified in the spike).

**D3. `Marchable` spans the uncoupled families plus a `Coupled<C,S>` wrapper.** Feasibility
F2: the coupling stage is a run-time argument to `run_until`, absent from the config type.
So `Marchable` is implemented by the three uncoupled configs directly, and `.couple(stack)`
produces `Coupled<C,S>: Marchable` running the fixed-horizon `run_until` path. One march
verb covers both. *Alternative rejected:* two march verbs (coupled/uncoupled) — leaks the
coupling distinction into the surface the two-level split already hides.

**D4. `MarchState` is one type with two transports.** What `pause.state()` exports, what
`from` accepts, and what the checksummed snapshot stores are unified. `pack_resume` already
covers three-quarters (fields, scalars, nav engine, log, step for disk); the remaining
quarter is the owned in-memory form. *Why:* the "pause it, resume it next line or next
week, bit-identically" guarantee needs a single type or it is three shapes that can drift.

**D5. `Verdict` is a value; the DSL never touches the process.** `verdict()` returns
`Result<Verdict, StudyError>`; `Verdict` implements `Display` and `passed()`, and `merge`
composes a mixed program's campaign and trajectory verdicts into one. The caller's `main`
maps to exit codes. *Why:* a language that calls `std::process::exit` cannot be embedded,
tested cleanly, or composed.

**D6. The audit log is one-thread-one-file under fan-out.** `save_log(path)` attaches a
stepwise-flushed sink; under any concurrent fan-out each branch thread writes its own file
named by numbered round and case, and the main file records spawn and rejoin. *Why over a
single interleaved file:* auditing one branch is reading one file, and the per-file
abort-tail names the casualty; a merged stream shreds each branch's story across threads.
*Alternative rejected:* contiguous case-blocks in one file (the prior draft) — still
serializes concurrent writers and loses the per-branch abort-tail.

**D7. One write verb.** `record` over `write_rows` is the language's only write;
`write_xy_csv` and `Report::write_series_csv` retire (an `(x,y)` probe trace is a
two-column `TableRow`, which gains units and the exact codec). Core's `write_csv` stays as
core's low-level action, no longer CFD-facing.

## Risks / Trade-offs

- [`ForkStudy` must thread the pause's `'c, R, S, M` (feasibility F3)] → the borrow itself
  is proven by the shipped `continue_branches`; the type-parameter threading is mechanical.
  If the four parameters make the phase type unwieldy, hide them behind a `Forkable` bound.
- [Typestate compiler errors name `Judged<PlacardRow, Shock>` at a confused engineer] →
  `#[must_use]` on every phase, every verb's doc comment written as the fix, and a review
  pass reading each forbidden program's actual error (acceptance §1.5).
- [`Float106` bit-exact round-trip through a text cell] → the `hi|lo` pair encoding is exact
  by construction with a plain-decimal fallback; tested by `read(write(t)) == t` per scalar.
- [Migration reproduces recorded outputs] → each example is diffed against its shipped
  `output.txt`; a mismatch is a migration bug, not a new baseline. The corridor is the acid
  test — if it needs an escape hatch the grammar lacks, the gap returns to the design.
- [Large surface retired at once] → contained: the crate is `publish = false`, the writer
  change is source-compatible for `f64`, and every retirement lands as reviewable diff with
  call sites migrated in the same change (never silent deletion; golden rules).

## Migration Plan

1. **Task group 0 (done):** the compile spike validating the three novel constructs — PASS
   (`04-dsl-feasibility.md`). Gate satisfied before any implementation task.
2. `deep_causality_file`: `TableScalar`, generic writer, `TableRow`/`FromTableRow`,
   `column`, `read_rows`/`write_rows`. Existing `f64` callers unbroken.
3. `deep_causality_cfd` foundation: `Marchable`, `MarchState`, the named-stage march
   builder, `StudyError`, the `StudyEffect` carrier and witness.
4. `deep_causality_cfd` campaign: the phase family, the case binders, sweep/march/reduce,
   record/refine, `GateSeq`, `Verdict`, `save_log`/`AuditLog`.
5. Retire the old surface (five march entries, positional `run_until`, `carry_field`, eager
   `Gates`, `write_xy_csv`, `write_series_csv`, `write_csv` re-export, `fail`) with call
   sites migrated in the same diff.
6. Remove the `examples/avionics_examples` exclusion in `Cargo.toml`; migrate the seven
   examples and two verification harnesses; diff each against its recorded output.
7. Sync the four new capability specs and the `typed-table-io` delta.

Rollback: the change is one reviewable diff on a feature branch; the crate is unpublished,
so rollback is dropping the branch. No external consumer is affected.

## Open Questions

- `ForkStudy` parameter threading versus a `Forkable` trait bound — decided at
  implementation time against whichever keeps the phase signature readable (D3/F3).
- Whether `reduce_all` and `reduce_ensemble` share one internal collector or stay two verbs
  — surface stays two verbs regardless; internal sharing is an implementation detail.
