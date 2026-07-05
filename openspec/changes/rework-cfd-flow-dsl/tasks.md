## 0. Feasibility gate (satisfied before implementation)

- [x] 0.1 Compile spike validating the three novel type constructs (Marchable GAT, ForkStudy borrow through scoped_map, higher-ranked GateSeq) against the real generic arities — PASS (openspec/notes/cfd-dsl/04-dsl-feasibility.md)
- [x] 0.2 Negative probes confirming the forbidden programs are rejected (verdict-before-gates, gates-before-sweep, wrong-row GateSeq) — PASS
- [x] 0.3 Fold the three forced refinements (F1 GateSeq<Row>+HRTB, F2 Marchable+Coupled, F3 ForkStudy threads pause params) back into the design note

## 1. Table IO precision codec (deep_causality_file)

- [x] 1.1 Add the `TableScalar` trait with cell encode/parse; implement for `f64`, `f32`, `Float106` (hi|lo pair cell); plain-decimal fallback via exact `f64`
- [x] 1.2 Make the writer precision-generic (`WriteTable<R: TableScalar>`), retiring the `NumericTable<f64>` hardcoding; swap the reader bound to `TableScalar`
- [x] 1.3 Add `TableRow` / `FromTableRow` traits (SCHEMA, cells, from_cells) and the `read_rows` / `write_rows` verbs, with header-name matching and schema-order delivery
- [x] 1.4 Add `NumericTable::column(name) -> Result`, naming absent columns
- [x] 1.5 Tests: `read(write(t)) == t` per scalar (f64/f32/Float106); read_rows column reorder + missing-column error; column() absent-name error; register in mod tree and tests/BUILD.bazel

## 2. Trajectory foundation (deep_causality_cfd)

- [x] 2.1 Add the `Marchable` trait (`march()` one-shot; GAT-`Pipeline` unification of `CfdFlow::march` deferred to the group-6 retirement pass per the additive-now decision); implement for `DuctConfig`, `MarchConfig`, `QttMarchConfig`; tests prove each equals its canonical entry
- [x] 2.2 Coupled march path — design refinement: the real coupled `run_until` needs an initial field the spike's `Coupled<C,S>=(config,stack)` could not carry, so the coupled seam is the named-stage builder's `ReadyMarch` (config+stack+field) rather than a bare `Coupled: Marchable`. `.couple(stack).from(field).run_for(n)` is the coupled one-shot the campaign lowers onto; the campaign's coupled `march`/`march_for` (group 4) calls it. No separate `Coupled` wrapper; `CompressibleMarchConfig`/`UncertainMarchConfig` reach the report through the builder.
- [ ] 2.3 Route `CfdFlow::march` through `Marchable`; retire `qtt_march`/`compressible_march`/`duct_march`/`uncertain_march`
- [x] 2.4 Add `MarchState` (wraps the coupled field + step, unifying pause export / resume input / checksummed snapshot); `CompressiblePause::state()` export; `save`/`load` disk transport; test proves disk-resume == in-memory-resume bit-identical. (`from(state)` resume verb lands with the named-stage builder in 2.5; `carry_field` is example-only and retires in the group-7 migration)
- [x] 2.5 Add the named-stage march builder: `CompressibleMarchRun::couple(stack)` → `CoupledMarch` (`trigger`/`kappa` optional stages) → `from(state)`/`from_field(..)` → `ReadyMarch` → `until(event)`/`run()`/`run_for(n)`. Tests prove each terminal equals the positional `run_until`/`run_coupled`/`march_with`. (`alternate` = existing `alternate_context`; `save_log` lands in group 5; positional `run_until` retires in the group-6 pass)
- [x] 2.6 Add singular `continue_with(world, steps)` on the pause beside the batch `continue_branches` (forks, alternates, continues; marker carried); test proves it equals the one-world batch
- [ ] 2.7 Tests: disk-resume == in-memory-resume bit-identical; fork history-sharing; march-verb dispatch across families; register in mod tree and tests/BUILD.bazel

## 3. Study effect carrier (deep_causality_cfd)

- [x] 3.1 Add `StudyError` (verb-tagged via `in_stage`) wrapping `PhysicsError` and `DataLoadingError` with `From` impls; Display names the verb, `source()` chains the data cause
- [x] 3.2 Add the `StudyEffect<T>` carrier and its `StudyEffectWitness` (haft Functor/Applicative/Monad/Pure in sibling modules, CDL pattern); error short-circuit + warning accumulation; ergonomic `pure`/`and_then`/`map`/`warn`/`from_result`/`into_parts`
- [x] 3.3 Add `StudyWarning` (Data/Case/Generic) + `StudyWarningLog` (LogAddEntry/LogAppend/LogSize/LogEffect) — the non-fatal channel for force_load overrides, clamped candidates, solver fallbacks
- [x] 3.4 Tests: monad laws (identity/associativity + witness Functor/Applicative/Monad), error short-circuit keeps its verb tag, warnings accumulate in order; registered in mod tree (types_flow Bazel suite auto-globs)

## 4. Campaign phase family (deep_causality_cfd)

- [x] 4.1a Uncoupled phase types (`StudyDef`, `Cases`, `Prepared`, `Configured`, `Marched`, `Swept`, `Judged`) + `CfdFlow::study` entry — done (src/types/flow/study/)
- [x] 4.2a Case-axis verbs (`read`/`matrix`/`cases`) + uncoupled binders (`prepare`/`case`) — done
- [x] 4.3a Uncoupled `march` + the `sweep` (Cases::sweep, Prepared::sweep; order-preserving, first-error-wins over scoped_map) — done
- [x] 4.4a `reduce` (per-case via `CaseRun`), `reduce_all` (cross-case, sequential), `record`, `inspect` (all phases) — done
- [x] 4.5 Add `StudyView`, `CaseRun`, `Verdict` (`passed`/`Display`/`warnings`/`merge`, `GateOutcome`) — done
- [x] 4.6a Uncoupled grammar tests: pointwise happy path, gate-fail preserves table, sweep short-circuit names the verb, sequence merge, march path reduce, read/matrix/prepare, reduce_all cross-case — done (8 tests, parallel==serial via sweep)

**SEQUENCING DECISION (recorded):** the campaign counterfactual verbs are moved into group 7,
built alongside the examples that drive their exact shape. Each is welded to a migration-time
design decision that cannot be resolved without its concrete consumer:
- [x] 4.7a Carrier context-lifetime refinement: `continue_with`/`continue_branches` relaxed to a fresh world lifetime (was pinned to the pause's config `'c`, blocking a campaign `branch` from continuing worlds it owns). Done by factoring the continued-march segment into a lifetime-free `run_continued_segment(cfg: &M::Config, …)` free fn shared by `CarrierFork::continue_march` (borrowed override) and `CarrierPause::continue_with` (short-lived owned world); `CarrierFork`/`alternate_context` untouched (zero blast radius). Audit trail proven byte-identical — the pre-existing `continue_branches_matches_the_manual_fork_chain` / `continue_with_matches_the_single_world_batch` tests (bit-match vs `fork().alternate_context().continue_march()`) still pass.
- [x] 4.7b `ForkStudy`/`Branched` phases + `fork`/`branch`/`continue_for` (the corridor's counterfactual core). `fork(&CompressiblePause)` declares the shared fork point; `branch(f)` binds one owned world per case (first-error-wins, config/execution seam); `continue_for(n)` lowers onto `continue_branches` (concurrent under `parallel`) and lands on `Marched` so `reduce` reads each branch through a `CaseRun`. Behavioral test `campaign_fork_branch_continue_for_reduces_branches_in_case_order` (case-order alignment + `!!ContextAlternation!!` marker per branch). 655 tests + 6 doctests green, clippy clean.
- [ ] 4.7c `Counterfactual` phase + `baseline` — the origin-form counterfactual; lands with the weather group (4.8), where `baseline`/`alternate` are the pair and neither is useful alone.
- [ ] 4.8 (→ group 7, plasma-blackout-weather) `alternate`/`couple`(campaign)/`march_for`/`ensemble`/`reduce_ensemble`. Requires the coupled-campaign initial-field source (F2: the fresh `CoupledField` lives in neither config nor stack) and the draw-index threading, both defined by the weather example.
- [x] 4.9 `refine` — the `Refining`/`RefineBranched`/`RefineMarched` mini-family: `refine(&pause, f)` re-attaches the fork point `reduce_all` dropped, derives the next round's cases from the rows so far, and threads the finished round into the next `Swept`'s `rounds` (gates read `view.rows()` for the fine round, `view.rounds()` for the coarse). Public `StudyView::of(&rows)` added for trajectory-level leg gates checked outside a phase (`leg_gates().check(&StudyView::of(&legs))`). Behavioral test `refine_reforks_the_same_onset_and_carries_the_prior_round`. 656 tests green, clippy clean. (Example consumer — the corridor two-stage sweep — lands in 7.6.)

## 5. Gate builder and audit log (deep_causality_cfd)

- [x] 5.1 `GateSeq<Row>` (HRTB gate fns), `new`/`gate`/`check`; `Swept::gates` and `Judged::gates` wired — done in group 4. (The eager `Gates` builder retires in the group-6 pass.)
- [x] 5.4 `compile_fail` doctests on `CfdFlow::study` for the phase discipline: verdict-before-gates, gates-before-sweep, record-before-sweep, reduce-before-march, wrong-row GateSeq — all correctly refuse to compile; plus the happy-path doctest.
- [ ] 5.2 (→ group 7) `AuditLog`/`LogSink` + `save_log` verb — sequenced with the coupled examples. Its stepwise flush hooks the coupled march loop's per-step `EffectLog`, and its value (rich provenance) exists only on the corridor/weather coupled runs; a skeleton without that integration is non-functional.
- [ ] 5.3 (→ group 7) the one-thread-one-file fan-out policy — inseparable from the counterfactual fan-out (corridor fork, weather ensemble), which itself lands in group 7 (tasks 4.7/4.8). Abort-tail + one-file-per-branch tests land with it.

## 6. Retire the legacy IO surface

- [x] 6.1a IO writers retired: `write_xy_csv`, `Report::write_series_csv`, the `write_csv` re-export gone; `io.rs`/`io_tests.rs` deleted; cfd now re-exports the file-crate typed table surface (`write_rows`/`read_rows`/`NumericTable`/`TableRow`/etc.) so a program imports table IO from one crate.
- [x] 6.2 Verification probe traces migrated: cylinder-wake → `write_rows` over a two-column `ProbeRow`; lid-cavity centerline (a heterogeneous formatted report, not a typed numeric table) imports core's `write_csv` directly per design D7.
- [x] 6.3a `MarchDispatch` GAT trait added (all 5 config families open their pipeline); `CfdFlow::march` unified to dispatch over it — one trajectory entry for every family. Existing `CfdFlow::march(&march_config)` sites source-compatible.
- [x] 6.3b Migrated 64 march-entry call sites (60 cfd tests + 4 harnesses) to `CfdFlow::march`; localized `fail` into each of 11 harnesses (9 single-file `fn fail`, 2 multi-file `pub(crate) fn fail` + `crate::fail`) — via two subagents over disjoint file sets, both green.
- [x] 6.4 Removed the 4 old march entries (`qtt_march`/`compressible_march`/`duct_march`/`uncertain_march`) and `pub fn fail` from the cfd public surface; repointed the removed-method doc links to `CfdFlow::march`. Final gate: cfd clippy clean (all targets), 654 tests + 6 doctests pass, all verification harnesses build. (`CfdConfigBuilder::uncertain_march` — a config builder, not a march entry — correctly kept.)

## 7. Migrate the examples

- [x] 7.1 Removed the `examples/avionics_examples` exclusion in root `Cargo.toml`; the shared lib builds green in the workspace again.
- [x] 7.2 Migrated nozzle_operating_map to the grammar (the exemplar): `CfdFlow::study.read.case.march.reduce.record.gates.verdict`; config in `model_config::duct_case(&FloatType)`, reduce `model::map_row(&CaseRun)`, `MapRow: TableRow`, gating sequence `model::nozzle_gates()`, `main -> ExitCode` mapping the `Verdict`. Runs green (4/4 gates, exit 0); numbers match the recorded map (shock 0.656→0.930, M_exit 2.12, first critical 0.937, exit shock 0.513); output.txt re-recorded; clippy clean.
- [x] 7.3 Migrated viv_resonance_margin: example-local `WakeCase: Marchable` (carries config + dt), `.case.march.reduce`, `MarginRow: TableRow`, gating sequence `model::viv_gates()`. Green — St 0.1818–0.1909 in band, margin 0.236, 3/3 gates pass, exit 0; output.txt re-recorded; clippy clean.
- [x] 7.4 Migrated flight_envelope_placard: the pointwise path — `.matrix::<FlightPoint>` (FromTableRow) + `.prepare(shock).sweep(placard_point)` (no march), `PlacardRow: TableRow`, gating sequence `model::placard_gates()`. Default green (max q 23.7 kPa, max T0 1502 K, 2/2 pass, exit 0); the exceeds-matrix correctly FAILs naming the offender (q 85.1 kPa) and exits 1; output.txt re-recorded; clippy clean.
- [ ] 7.5 Migrate plasma_blackout_weather (baseline/alternate/ensemble/couple/reduce_ensemble); diff against recorded output.txt
- [ ] 7.6 Migrate plasma_blackout_corridor (fork/branch/continue_for/reduce_all/refine + leg gates via GateSeq::check + Verdict::merge); diff against recorded output.txt — the acid test
- [ ] 7.7 Migrate the stagnation-line study and compressible_carrier_timing; diff against recorded outputs

## 8. Finalize

- [ ] 8.1 `make format && make fix` clean across deep_causality_file and deep_causality_cfd; no `#[allow]`, no new `unsafe`, no `dyn`/macros in lib code
- [ ] 8.2 Full test suites green (file, cfd) plus every migrated example running with byte-identical recorded output
- [ ] 8.3 Confirm the per-verb acceptance chain (guarantee + test + example + doc page) for each new verb; note any gaps for the docs stage
- [ ] 8.4 Prepare the commit message; ask the user to commit
