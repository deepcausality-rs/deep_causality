> Three tranches. Each group ends green: `cargo build -p deep_causality_cfd`,
> `bazel test //deep_causality_cfd/...`, `make format && make fix`. Group boundaries are commit
> boundaries.

## 1. Tranche A — library entries

- [ ] 1.1 `src/types/flow_config/qtt_march_config.rs`: change `new()` to `pub(crate) fn new(name: impl Into<String>)`, delete the `Default` impl and the `.name(..)` method, change the field to `name: String`, drop the `"qtt_march"` default in `build()`
- [ ] 1.2 `src/types/flow_config/compressible_march_config.rs`: same treatment on `CompressibleMarchConfigBuilder` — `pub(crate) fn new(name)`, delete `Default` and `.name(..)`, drop the `"compressible_march"` default
- [ ] 1.3 New `src/types/flow_config/duct_builder.rs`: `DuctConfigBuilder<R>` with `pub(crate) fn new(name)`, `profile`, `inlet`, `gamma`, `back_pressure`, `cells`, `stop`, and `build()` carrying the validation moved verbatim from `DuctConfig::new`
- [ ] 1.4 `src/types/flow_config/duct_config.rs`: add the `name` field and its getter, make `DuctConfig::new` a `pub(crate)` field constructor with the validation removed
- [ ] 1.5 `src/types/flow/duct_march_run.rs:309`: label the report with `config.name()` instead of the literal `"duct_march"`
- [ ] 1.6 `src/solvers/dec/dec_config/mod.rs:33`: make `DecNs::config` `pub(crate)`; drop `DecNs` from `src/solvers/mod.rs` and `src/lib.rs:146`
- [ ] 1.7 `src/types/flow_config/cfd_config_builder.rs`: add `qtt_march::<R>(name)`, `compressible_march::<R>(name)`, `duct::<R>(name)`, each documented against the family it starts
- [ ] 1.8 `src/types/flow_config/mod.rs` + `src/lib.rs`: register `duct_builder`, export `DuctConfigBuilder`
- [ ] 1.9 `src/types/flow_config/march_builder.rs:17`: rewrite the paragraph that intra-doc-links `crate::DecNs::config` to name `CfdConfigBuilder::dec_ns`
- [ ] 1.10 Verify: `cargo doc -p deep_causality_cfd` emits no broken intra-doc link

## 2. Tranche A — test call sites

- [ ] 2.1 `tests/types/flow/compressible_march_run/mod.rs` (L65, L112) — the shared fixture four sibling files depend on; do first
- [ ] 2.2 `tests/types/flow/compressible_march_run/{carrier,forcing,imprint}_tests.rs` (L92 / L25, L207 / L55)
- [ ] 2.3 `tests/types/flow/{march_state,coupled_march}_tests.rs` (L46 / L45) and `tests/types/flow_config/compressible_march_config_tests.rs` (L108, L120, L142)
- [ ] 2.4 `tests/types/flow_config/qtt_march_config_tests.rs` — 11 sites (L20, 51, 58, 73, 85, 97, 106, 116, 132, 150, 170)
- [ ] 2.5 `tests/types/flow/qtt_march_run_tests.rs` — 8 sites (L42, 129, 202, 268, 340, 448, 538, 687) and `tests/types/flow/qtt_march_pause_tests.rs:28`
- [ ] 2.6 Split `tests/types/flow_config/duct_config_tests.rs`: the 15 rejection tests move to a new `tests/types/flow_config/duct_builder_tests.rs` asserting the same errors from `build()`; the remaining accessor tests re-route through the entry
- [ ] 2.7 Register `duct_builder_tests` in `tests/types/flow_config/mod.rs` (the Bazel `*_tests.rs` glob needs no `BUILD.bazel` edit)
- [ ] 2.8 `tests/types/flow/duct_march_tests.rs:20`, `tests/types/flow/study_grammar_tests.rs:141`, `tests/traits/marchable_tests.rs` (L20 duct, L60 QTT)
- [ ] 2.9 `tests/solvers/dec_config/dec_config_tests.rs` — 8 sites (L19, 32, 61, 78, 88, 102, 117, 129) from `DecNs::config()` to `CfdConfigBuilder::dec_ns()`
- [ ] 2.10 `tests/types/flow_config/cfd_config_builder_tests.rs`: add coverage for `qtt_march`, `compressible_march`, and `duct` — the built config carries the entry-supplied name, and `build()` rejects each missing required section

## 3. Tranche A — binaries and examples

- [ ] 3.1 `verification/qtt_taylor_green_verification/config.rs:102`, `verification/qtt_cylinder_verification/config.rs:143`, `verification/qtt_park2t_blackout/config.rs:106`
- [ ] 3.2 `studies/qtt_rank_plume/main.rs:435`
- [ ] 3.3 `examples/avionics_examples/src/shared/world.rs` L96 (`descent_world_with`) and L469 (`terminal_descent_world`)
- [ ] 3.4 `examples/avionics_examples/cfd/nozzle_operating_map/model_config.rs:21`

## 4. Tranche A — acceptance

- [ ] 4.1 `cargo build -p deep_causality_cfd && cargo build -p avionics_examples`
- [ ] 4.2 `bazel test //deep_causality_cfd/...` green
- [ ] 4.3 Run `qtt_taylor_green_verification`, `qtt_cylinder_verification`, `qtt_park2t_blackout`, `nozzle_operating_map`, `plasma_blackout_corridor`; diff each against its pre-change output — identical numbers
- [ ] 4.4 `make format && make fix`; commit Tranche A

## 5. Tranche B — absorb the struct-literal configs

- [ ] 5.1 `DuctConfigBuilder`: replace the `DuctInlet` / `DuctStop` arguments with `inlet(p0, t0)` and `stop(max_steps, residual_tol)`; leave both types exported and unused pending the deletion decision recorded in `design.md`
- [ ] 5.2 `CompressibleMarchConfigBuilder`: add `reference(t_ref, n_ref, u_ref)` replacing the `ReferenceScales` literal; make the type's fields private with getters
- [ ] 5.3 `src/types/flow_config/compressible_march_config.rs`: give `PlumeImprint` a validated constructor with private fields and getters
- [ ] 5.4 `src/types/flow/retropulsion.rs:316`: give `PlumeNozzle` a validated constructor with private fields and getters; validate the documented Cordell jet-gamma envelope `[1.2, 1.4]`
- [ ] 5.5 Update the 24 literal sites: `tests/types/flow/compressible_march_run/{mod,imprint}_tests.rs`, `tests/types/flow/{coupled_march,march_state,retropulsion}_tests.rs`, `tests/types/flow_config/{compressible_march_config,duct_config}_tests.rs`, `tests/types/flow/{duct_march,study_grammar}_tests.rs`, `tests/traits/marchable_tests.rs`, `studies/qtt_rank_plume/main.rs`, `examples/avionics_examples/src/shared/world.rs`, `examples/avionics_examples/cfd/nozzle_operating_map/model_config.rs`
- [ ] 5.6 Add rejection tests for the `PlumeNozzle` envelope and the `PlumeImprint` constructor

## 6. Tranche B — acceptance

- [ ] 6.1 `bazel test //deep_causality_cfd/...` green
- [ ] 6.2 Re-run the same five binaries as 4.3; identical numbers
- [ ] 6.3 `make format && make fix`; commit Tranche B

## 7. Tranche C — blended-map builder

- [ ] 7.1 `src/coordinate/blended.rs`: add the fluent builder (lattice, radial range, angular range, `λ`), move validation into `build()`, make `BlendedMapConfig::new` crate-private
- [ ] 7.2 Update the 7 sites: `tests/coordinate/blended_tests.rs` (L23, 131, 152), `verification/qtt_blunt_body_2d/main.rs` (L80, 94), `studies/qtt_rank_plume/main.rs:592`, `studies/qtt_blend_metric/main.rs:69`
- [ ] 7.3 Add rejection tests for a non-positive radial/angular range and a `λ` outside `[0, 1]`

## 8. Tranche C — exemption declarations

- [ ] 8.1 Add the exemption paragraph (subject + reason) to the module docs of `verification/qtt_sod/main.rs`, `verification/qtt_reentry_3d/main.rs`, `verification/dec_graded_mms_verification/main.rs`, `verification/dec_wall_heat_flux_verification/main.rs`, `studies/compressible_carrier_timing/main.rs`
- [ ] 8.2 `studies/srp_momentum_jet/main.rs`: note in the module docs that its configuration lives in `config.rs` under the config/execution separation, and that the marcher construction is exempt as a study of the marched field

## 9. Tranche C — cylinder-harness library support

- [ ] 9.1 `src/types/flow_config/seed.rs`: add the perturbed uniform-stream variant (offset, width, amplitude), keeping `Seed` `Copy`; apply through the existing seed projection
- [ ] 9.2 Test the perturbed seed: the field is divergence-free to the same tolerance as `UniformX`, and a symmetric cylinder case sheds where the unperturbed one does not
- [ ] 9.3 `src/types/flow_config/observe.rs` + `src/types/flow/march_run.rs`: add the drag-split opt-in emitting the pressure and friction series from the existing `surface_force_coefficients` integrations; default off
- [ ] 9.4 Test the split: the two series sum to the combined drag at every step; the split is absent by default; requesting it without an immersed body returns the existing missing-body error

## 10. Tranche C — lift the cylinder harness

- [ ] 10.1 Record the pre-change output of `dec_cylinder_verification` at default knobs (`St`, `C_d`, pressure/friction split, divergence) as the acceptance baseline
- [ ] 10.2 Rewrite its setup onto `CfdConfigBuilder::march` — mesh (box domain + immersed disk + merge floor), solver (viscosity, `dt`, CG options, warm start, staircase toggle), zone tuple, perturbed seed, observables (probe + drag split) — with the environment knobs feeding the configuration values
- [ ] 10.3 Move the per-step force sampling onto `CfdFlow::march(&config).on(&manifold).run_with(..)`; delete the hand-built lattice, registry, manifold, and `DecNsSolver` construction
- [ ] 10.4 Verify against 10.1: `St`, `C_d`, and the split match to the printed precision
- [ ] 10.5 Verify the gates still fail loudly: a forced solver error exits non-zero without reporting truncated-series values

## 11. Tranche C — acceptance and documentation

- [ ] 11.1 `bazel test //deep_causality_cfd/...` green; `make build && make test`
- [ ] 11.2 Re-run every touched binary and example; identical numbers
- [ ] 11.3 Update `deep_causality_cfd/README.md` and the `dec_taylor_green_re1600_verification` / `qtt_taylor_green_verification` READMEs to the new entries
- [ ] 11.4 `make format && make fix`; commit Tranche C
- [ ] 11.5 Confirm every entry in the `cfd-config-entry` table exists and no public bypass remains: `grep -rn "QttMarchConfigBuilder::new\|CompressibleMarchConfigBuilder::new\|DuctConfig::new\|DecNs::config\|BlendedMapConfig::new" --include='*.rs' deep_causality_cfd examples` returns only `src/` definition sites
