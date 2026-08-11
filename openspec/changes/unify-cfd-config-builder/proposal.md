## Why

`fluiddynamics-dsl` requires that "a single `CfdConfigBuilder` entry SHALL start each owned,
validated configuration". The crate has nine configuration families and `CfdConfigBuilder` starts
four of them. `QttMarchConfig` and `CompressibleMarchConfig` are started by their own public
builders, `DuctConfig` and `BlendedMapConfig` by positional constructors with six and seven
arguments, and `DecNsConfig` has two public entries (`CfdConfigBuilder::dec_ns` and
`DecNs::config`). Six further configuration types are public structs with public fields, built as
struct literals at 35 sites, which the codebase convention on field visibility does not permit.

The result is that a reader of one CFD case cannot predict the shape of the next one, and the spec
statement above is false as written.

## What Changes

Delivered in three tranches, each independently green.

**Tranche A — single entry (re-routing; no numerics change).**

- Add `CfdConfigBuilder::qtt_march`, `CfdConfigBuilder::compressible_march`, and
  `CfdConfigBuilder::duct`, each taking the case name, matching `march` / `verify` /
  `uncertain_march`.
- **BREAKING** `QttMarchConfigBuilder::new`, its `Default` impl, `CompressibleMarchConfigBuilder::new`
  and its `Default` impl become crate-private; the `.name(..)` method is removed from both and the
  case name becomes required (the `"qtt_march"` / `"compressible_march"` default names are dropped).
- **BREAKING** Add `DuctConfigBuilder`; `DuctConfig::new` becomes crate-private and its validation
  moves to `build()`. `DuctConfig` gains a case name, so a duct `Report` carries it instead of the
  hardcoded `"duct_march"`.
- **BREAKING** `DecNs::config` becomes crate-private and `DecNs` is no longer exported, leaving
  `CfdConfigBuilder::dec_ns` as the only entry to the DEC solver config.

**Tranche B — absorb the struct-literal configs.**

- **BREAKING** `DuctInlet` and `DuctStop` are absorbed into `DuctConfigBuilder::inlet(p0, t0)` and
  `DuctConfigBuilder::stop(max_steps, residual_tol)`.
- **BREAKING** `ReferenceScales` is absorbed into
  `CompressibleMarchConfigBuilder::reference(t_ref, n_ref, u_ref)`.
- **BREAKING** `PlumeImprint` (8 public fields) and `PlumeNozzle` (10 public fields) gain validated
  constructors with private fields and getters. `PlumeNozzle` validates the Cordell jet-gamma
  envelope its docstring currently states in prose.
- `AtmosphereRow` stays a public data record: it is a table row, and 11 of its sites are literal
  table data.

**Tranche C — remaining bundles and the raw-solver harnesses.**

- **BREAKING** `BlendedMapConfig::new` (7 positional arguments) is replaced by `BlendedMapConfig`'s
  own fluent builder. It stays out of `CfdConfigBuilder`, which starts solver and case
  configurations, one level above a coordinate map.
- `DescentSchedule` and `ForcingRegion` stay value types, peer to `Mesh`, `Body`, `Seed` and
  `Observe`. Both already validate at construction and expose fluent overrides.
- Eight binaries drive solvers directly with no configuration layer. Ruled case by case on
  configuration size: `dec_cylinder_verification` (512 lines, 9 environment knobs, a hand-built
  duplicate of the `MarchConfig` family) is lifted onto `CfdConfigBuilder::march`; the other seven
  keep direct solver construction and are recorded as a named exemption class with the reason for
  each.
- Lifting the cylinder harness adds two library capabilities it needs: a symmetry-breaking seed
  variant, and a pressure/friction split on the drag observable.

## Capabilities

### New Capabilities

- `cfd-config-entry`: the crosscutting configuration contract — the complete `CfdConfigBuilder`
  entry set, the rule that no configuration type exposes a public bypass constructor or public
  fields, the value-type boundary (what stays a `Mesh`-peer rather than becoming an entry), and the
  named exemption class for solver-level verification harnesses.

### Modified Capabilities

- `fluiddynamics-dsl`: the configuration/composition requirement enumerates the complete entry set
  rather than three examples.
- `qtt-flow`: the QTT config is started by `CfdConfigBuilder::qtt_march` with a required name. Also
  corrects a standing drift — the spec requires a `CfdFlow::qtt_march(&config)` entry that the code
  replaced with the unified `CfdFlow::march` + `MarchDispatch`.
- `duct-march`: the duct config is builder-built, carries a case name, and its report is named. Also
  corrects a standing drift — the spec requires a `CfdFlow::duct_march` entry that the code replaced
  with the unified `CfdFlow::march`.
- `compressible-flow-host`: the compressible config is started by
  `CfdConfigBuilder::compressible_march`, and the reference scales are set through a builder method.
- `body-fitted-qtt-coordinate`: the blended-map configuration is builder-built and validated.
- `surface-force-diagnostic`: the drag observable can emit the pressure and friction contributions
  as separate series.
- `dec-ns-validation`: the isolated-cylinder harness is configured through the config layer, with
  its per-step diagnostics on the existing `run_with` hook.

## Impact

**Library (`deep_causality_cfd/src`)** — 15 files. New `types/flow_config/duct_builder.rs`; changes
to `cfd_config_builder.rs`, `qtt_march_config.rs`, `compressible_march_config.rs`, `duct_config.rs`,
`flow_config/mod.rs`, `lib.rs`, `solvers/dec/dec_config/mod.rs`, `solvers/mod.rs`,
`types/flow/duct_march_run.rs`, `types/flow/march_run.rs`, `types/flow_config/seed.rs`,
`types/flow_config/observe.rs`, `types/flow/retropulsion.rs`, `coordinate/blended.rs`.

**Call sites** — about 145 across 30 files: 15 test files, 6 verification/study binaries, and 3
example files. All library changes are source-breaking; the crate is `publish = false`, so the blast
radius is the workspace.

**Numerical results** — none of the three tranches changes numerics. Every touched verification
binary and example must reproduce its recorded numbers, which is the acceptance gate for each
tranche.

**Documentation** — `deep_causality_cfd/README.md` and two verification READMEs.
