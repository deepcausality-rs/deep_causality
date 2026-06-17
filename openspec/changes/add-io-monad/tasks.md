# Tasks

## Layer 1 — `deep_causality_haft` (abstraction, no_std, no dyn)

- [x] Add `src/io/mod.rs`: the `IoAction` trait (`Output`, `Error`, `run`, and provided `map` /
      `and_then` / `map_err`) plus free `pure` / `fail` constructors.
- [x] Add one combinator type per file: `src/io/io_pure.rs`, `io_fail.rs`, `io_map.rs`,
      `io_and_then.rs`, `io_map_err.rs`, each implementing `IoAction`.
- [x] Declare `mod io;` and re-export `IoAction`, the combinator structs, and `io_pure`/`io_fail`
      from `src/lib.rs`.
- [x] Confirm the module is `no_std`-safe (uses only `core`) and `dyn`-free.
- [x] Tests in `tests/algebra/io_tests.rs` (alongside `arrow_tests.rs`), registered in
      `tests/algebra/mod.rs` (auto-globbed by the existing `algebra` Bazel suite): construction
      performs no effect, `run` semantics, monad laws (left/right identity, associativity) via a pure
      in-memory action, `map`/`map_err`/`and_then`, short-circuit. **13 tests, green.**

## Layer 2 — `deep_causality_core` (file specialization, std-gated)

- [x] Add `CausalityErrorEnum::IoError(String)` in `src/errors/causality_error.rs` (the `Display`
      impl delegates to `{:?}`, so no separate change needed).
- [x] Add `src/types/io/`: `read_text.rs`, `write_text.rs`, `write_csv.rs`, `read_csv.rs` — concrete
      `IoAction`s with `Error = CausalityError`, gated `#[cfg(feature = "std")]`. `std::io::Error` →
      `CausalityError::IoError` via `io_error`.
- [x] `write_csv` emits `header.join(",")` then each pre-rendered `row.join(",")`, `'\n'`-terminated.
- [x] Add `src/types/causal_flow/io.rs`: generic `CausalFlow::source(io)` (Output → value) and
      value-preserving `CausalFlow::commit(|v| io)` (run `IoAction<Output=()>`, value passes through,
      append `EffectLog` entry); format-qualified verbs `read_text_from` / `read_csv_from` (read
      constructors) and `write_text_to` / `write_csv_to` (value-preserving write steps). Registered in
      `causal_flow/mod.rs`.
- [x] Re-export the file actions and the new error variant from `src/lib.rs`.
- [x] Tests in `tests/types/io/io_tests.rs` and `tests/types/causal_flow/io_tests.rs`: golden-bytes
      `write_csv`, `read_text`/`read_csv` round-trip, failure → `IoError`, no-effect-until-run, a read
      constructor produces the value, a write step preserves `CausalFlow<V>`, `commit`/`source`
      Ok/Err/audit-entry. Registered in `tests/types/mod.rs`, `tests/types/causal_flow/mod.rs`, and a
      new `types_io` Bazel suite. **18 tests, green.**
- [~] `tempfile` dev-dependency — **not added**; tests use `std::env::temp_dir()` + unique names
      instead (core has zero dev-deps and AGENTS.md says avoid external crates). Deviation from the
      proposal, intentional.

## Layer 3 — `deep_causality_cfd` (application, std-gated)

- [x] Add `src/types/flow/io.rs`: `Report::write_series_csv(path, labels)` and free
      `write_xy_csv(path, header, series)`, both returning `WriteCsv` and built on core `write_csv`.
      Registered in `flow/mod.rs`; `write_xy_csv` + `IoAction` re-exported from `lib.rs`.
- [x] Port `examples/avionics_examples/dec_cylinder_wake/main.rs`: after the march, write the full
      wake-probe series to `cylinder_wake.csv` via a deferred `write_xy_csv` action executed by one
      `run` at the edge. **Verified by running** (`cargo run --release`): 2000 samples + header.
- [~] Bit-identical `write_csv_to` migration of the 6-column stdout stream — **superseded**: the
      example writes the full probe series via `write_xy_csv` instead (the prior output was stdout
      diagnostics, not a file; reproducing it verbatim was not meaningful). Spec updated to match.
- [ ] Port `dec_lid_cavity_re1000`'s raw centerline writes onto `write_csv` — **not done** (the user
      scoped this task to `dec_cylinder_wake`). The `cavity_centerline_*.csv` writers remain raw.
- [x] CSV-helper tests in `tests/types/flow/io_tests.rs` (golden `write_xy_csv`, report-backed
      `write_series_csv`, ragged columns). **4 tests, green.** No `tempfile` dep (temp_dir as above).

## Cross-cutting

- [x] New modules inherit the crate-level `[lints] workspace = true` (no per-module opt-in needed).
- [x] `cargo fmt` + `cargo clippy` on the three crates: clean (the only remaining warning is the
      pre-existing `functor/mod.rs` unused-import, out of scope). `cargo build`/`cargo test` green for
      all three crates (haft, core, cfd) plus the example. `make build` / `make test` (full repo) not
      yet run.
- [x] No prelude; no macros in `src`; static dispatch only; no `dyn`.
- [ ] Prepare a commit message and ask the user to commit (do not commit).
