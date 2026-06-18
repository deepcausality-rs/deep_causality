## Why

The CfdFlow DSL composes a simulation declaratively, but its examples still perform side-effecting
file output imperatively. `dec_lid_cavity_re1000` writes its `cavity_centerline_*.csv` files with raw
`File::create`/`writeln!`, and `dec_cylinder_wake` streams its wake CSV through `println!` instead of
committing it as a composable effect. **No IO effect exists anywhere in the workspace today** — IO is
the only effect the functional core cannot yet describe.

Every other "effect" in DeepCausality (the `Error`/`Log`/`State` channels of
`CausalEffectPropagationProcess`, the `Effect3/4/5` type-encoded effect system) is *pure*: a fixed
type parameter threading **data** through a computation. None of them touch the outside world. IO is
categorically different — it executes a **real** side effect — and so needs its own abstraction: a
deferred description that runs only at the program edge and bridges into the existing error and audit
channels.

## What Changes

- **Layer 1 — `deep_causality_haft` (the abstraction).** Add a value-level `IoAction` trait and a set
  of concrete, defunctionalized combinator structs (`IoPure`, `IoFail`, `IoMap`, `IoAndThen`,
  `IoMapErr`), generic over the error type `E`. This is the **lazy IO monad**, realized exactly the
  way the `Arrow` algebra is realized: combinators return new concrete types, composition is total and
  monomorphized, **no `dyn`** (the workspace's test-enforced no-`dyn` rule). `map`/`and_then` compose
  without running; `run` is the single method that performs a side effect, at the edge.
- **Layer 2 — `deep_causality_core` (the file specialization).** Add std-gated file actions
  (`read_text`, `write_text`, `write_csv`, `read_csv`) that are concrete `IoAction`s fixing
  `Error = CausalityError`; a new `CausalityErrorEnum::IoError(String)` variant; and intent-named Flow
  verbs whose preposition names a file path — read **constructors** that produce the value
  (`CausalFlow::read_csv_from` / `read_text_from`) and value-**preserving** write steps that pass the
  value through (`flow.write_csv_to` / `write_text_to`) — over a generic `source` / `commit` bridge.
- **Layer 3 — `deep_causality_cfd` (the application).** Add std-gated CSV helpers
  (`Report::write_series_csv`, an `(x, y)` series writer) and port `dec_cylinder_wake`'s wake CSV onto
  a single value-preserving `write_csv_to` step — output **bit-identical** to the current `println!`
  stream.

## Capabilities

### New Capabilities
- `io-monad`: a lazy, `dyn`-free IO effect (`IoAction` + defunctionalized combinators) for deferred,
  composable file input/output that runs only at the program edge, propagates failures as
  `CausalityError`, records an audit entry in `EffectLog`, and composes with the causal-monad / Flow
  DSLs without a new error type.

### Modified Capabilities
<!-- None. This adds a new capability. The CfdFlow DSL consumes Io; its requirement set is unchanged. -->

## Impact

- **New code:** an `io` module in `deep_causality_haft` (trait + combinator structs, `no_std`-safe,
  no `dyn`); a `types/io` module + `causal_flow/io.rs` bridge + `IoError` variant in
  `deep_causality_core`; CSV helpers in `deep_causality_cfd`.
- **Dependencies:** no new runtime external crates; reuses `CausalityError`/`EffectLog` and
  `std::fs`/`std::io`. `tempfile` added as a **dev**-dependency to `core`/`cfd` for file-action tests
  (already a workspace dev-dep).
- **Consumers:** `dec_cylinder_wake` migrates its wake-CSV write; `dec_lid_cavity_re1000` migrates its
  centerline writes. Examples writing only to stdout are unaffected.
- **Feature flags:** the `std` feature gates the file actions and the filesystem effect of `run`; the
  `IoAction` trait and the pure combinators are `no_std`-safe.
