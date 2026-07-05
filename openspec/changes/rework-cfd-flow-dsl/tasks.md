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
- [ ] 2.2 Add the `Coupled<C, S>` wrapper and its `Marchable` impl (fixed-horizon run_until path); implement for `CompressibleMarchConfig`, `UncertainMarchConfig`
- [ ] 2.3 Route `CfdFlow::march` through `Marchable`; retire `qtt_march`/`compressible_march`/`duct_march`/`uncertain_march`
- [ ] 2.4 Add `MarchState` (fields, scalars, ambient, nav engine, log, step, clock); `pause.state()` export and `from(state)` resume; unify with the checksummed snapshot; retire `carry_field`
- [ ] 2.5 Add the named-stage march builder (`alternate`/`save_log`/`couple`/`from`/`until`/`run`/`run_for`); retire positional `run_until`
- [x] 2.6 Add singular `continue_with(world, steps)` on the pause beside the batch `continue_branches` (forks, alternates, continues; marker carried); test proves it equals the one-world batch
- [ ] 2.7 Tests: disk-resume == in-memory-resume bit-identical; fork history-sharing; march-verb dispatch across families; register in mod tree and tests/BUILD.bazel

## 3. Study effect carrier (deep_causality_cfd)

- [ ] 3.1 Add `StudyError` (verb-tagged) wrapping `PhysicsError` and `DataLoadingError` with `From` impls
- [ ] 3.2 Add the `StudyEffect<T>` carrier and its hidden `StudyEffectWitness` (haft Functor/Applicative/Monad, CDL pattern); error short-circuit + warning accumulation
- [ ] 3.3 Add `StudyWarning` and the warning channel plumbing (force_load overrides, clamped candidates, solver fallbacks)
- [ ] 3.4 Tests: monad-law smoke, error short-circuit tags the verb, warning accumulation reaches the effect; register in mod tree and tests/BUILD.bazel

## 4. Campaign phase family (deep_causality_cfd)

- [ ] 4.1 Add the phase types (`StudyDef`, `Cases`, `Prepared`, `Counterfactual`, `ForkStudy`, `Configured`, `Branched`, `Marched`, `Swept`, `Judged`) and the `CfdFlow::study` entry
- [ ] 4.2 Add the case-axis verbs (`read`/`matrix`/`cases`) and the binders (`prepare`/`case`/`baseline`/`fork`/`ensemble`)
- [ ] 4.3 Add `couple`/`march`/`march_for`/`continue_for` and the sweep (`sweep`, order-preserving, first-error-wins over scoped_map)
- [ ] 4.4 Add the reductions (`reduce`/`reduce_all`/`reduce_ensemble`), `record`, `refine`, and `inspect`
- [ ] 4.5 Add `StudyView`, `CaseRun`, `Verdict` (`passed`/`Display`/`warnings`/`merge`)
- [ ] 4.6 Tests: sweep order + first-error; reduce_all cross-case; ensemble draw-set reduction; parallel==serial results; register in mod tree and tests/BUILD.bazel

## 5. Gate builder and audit log (deep_causality_cfd)

- [ ] 5.1 Add `GateSeq<Row>` (HRTB gate fns), `new`/`gate`/`check`; wire `Swept::gates` and `Judged::gates`; retire the eager `Gates` builder
- [ ] 5.2 Add `AuditLog`/`LogSink` and the `save_log` verb (both levels); stepwise flush; loud on sink failure; optional fsync flag
- [ ] 5.3 Implement the one-thread-one-file fan-out policy (per-branch files named by round+case; main file records spawn/rejoin)
- [ ] 5.4 Tests: verdict-before-gates / gates-before-sweep / wrong-row GateSeq as `compile_fail` doctests; disk==memory; abort-tail via killed child process; one-file-per-branch; register in mod tree and tests/BUILD.bazel

## 6. Retire the legacy IO surface

- [ ] 6.1 Make `record`/`write_rows` the only write; retire `write_xy_csv`, `Report::write_series_csv`, the `write_csv` re-export, and `fail`
- [ ] 6.2 Migrate the two verification harness probe traces (cylinder wake, lid cavity) to `write_rows` over two-column TableRow types

## 7. Migrate the examples

- [ ] 7.1 Remove the `examples/avionics_examples` exclusion in root `Cargo.toml`; move it back into workspace members
- [ ] 7.2 Migrate nozzle_operating_map to the grammar; diff against recorded output.txt
- [ ] 7.3 Migrate viv_resonance_margin (example-local `WakeCase: Marchable`); diff against recorded output.txt
- [ ] 7.4 Migrate flight_envelope_placard (matrix/FromTableRow/prepare); diff against recorded output.txt
- [ ] 7.5 Migrate plasma_blackout_weather (baseline/alternate/ensemble/couple/reduce_ensemble); diff against recorded output.txt
- [ ] 7.6 Migrate plasma_blackout_corridor (fork/branch/continue_for/reduce_all/refine + leg gates via GateSeq::check + Verdict::merge); diff against recorded output.txt — the acid test
- [ ] 7.7 Migrate the stagnation-line study and compressible_carrier_timing; diff against recorded outputs

## 8. Finalize

- [ ] 8.1 `make format && make fix` clean across deep_causality_file and deep_causality_cfd; no `#[allow]`, no new `unsafe`, no `dyn`/macros in lib code
- [ ] 8.2 Full test suites green (file, cfd) plus every migrated example running with byte-identical recorded output
- [ ] 8.3 Confirm the per-verb acceptance chain (guarantee + test + example + doc page) for each new verb; note any gaps for the docs stage
- [ ] 8.4 Prepare the commit message; ask the user to commit
