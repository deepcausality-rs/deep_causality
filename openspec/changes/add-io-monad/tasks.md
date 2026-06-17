# Tasks

## Layer 1 — `deep_causality_haft` (abstraction, no_std, no dyn)

- [ ] Add `src/io/mod.rs`: the `IoAction` trait (`Output`, `Error`, `run`, and provided `map` /
      `and_then` / `map_err`) plus free `pure` / `fail` constructors.
- [ ] Add one combinator type per file: `src/io/io_pure.rs`, `io_fail.rs`, `io_map.rs`,
      `io_and_then.rs`, `io_map_err.rs`, each implementing `IoAction`.
- [ ] Declare `mod io;` and re-export `IoAction`, the combinator structs, and `io_pure`/`io_fail`
      from `src/lib.rs`.
- [ ] Confirm the module is `no_std`-safe (no `alloc` required for the combinators) and `dyn`-free.
- [ ] Tests under `tests/io/` (mirror `src`, `_tests` suffix), registered in each `mod.rs` and in
      `tests/BUILD.bazel`: construction performs no effect, `run` semantics, monad laws (left/right
      identity, associativity) using a pure in-memory action, `map`/`map_err`/`and_then` behavior.

## Layer 2 — `deep_causality_core` (file specialization, std-gated)

- [ ] Add `CausalityErrorEnum::IoError(String)` in `src/errors/causality_error.rs`; extend its
      `Display`.
- [ ] Add `src/types/io/`: `read_text.rs`, `write_text.rs`, `write_csv.rs`, `read_csv.rs` — concrete
      `IoAction`s with `Error = CausalityError`, all `#[cfg(feature = "std")]`. Map `std::io::Error`
      to `CausalityError::IoError`.
- [ ] `write_csv` emits `header.join(",")` then each pre-rendered `row.join(",")`, `'\n'`-terminated.
- [ ] Add `src/types/causal_flow/io.rs`: the generic bridge `CausalFlow::source(io)` (Output → value)
      and value-preserving `CausalFlow::commit(|v| io)` (run `IoAction<Output=()>`, value passes
      through, append `EffectLog` entry); then the format-qualified verbs as thin wrappers —
      `CausalFlow::read_text_from` / `read_csv_from` (read constructors) and `flow.write_text_to` /
      `write_csv_to` (value-preserving write steps). Register the module in `causal_flow/mod.rs`.
- [ ] Re-export the file actions and the new error variant from `src/lib.rs`.
- [ ] Add `tempfile` as a dev-dependency.
- [ ] Tests under `tests/types/io/` and `tests/types/causal_flow/io_tests.rs`: golden-bytes
      `write_csv`, `read_text` round-trip, failure → `IoError`, short-circuit on first failure,
      a read constructor produces the value, a write step preserves the carried value (`CausalFlow<V>`
      stays `CausalFlow<V>`), `commit`/`source` Ok/Err/audit-entry, std-gating. Register in
      `tests/BUILD.bazel`.

## Layer 3 — `deep_causality_cfd` (application, std-gated)

- [ ] Add `src/types/flow/io.rs`: `Report::write_series_csv(path, labels)` and a free `(x, y)` series
      writer, both returning `impl IoAction<Output = (), Error = CausalityError>` and building on the
      core actions. Register in `flow/mod.rs`; re-export from `lib.rs`.
- [ ] Capture the current `dec_cylinder_wake` CSV output as a byte baseline.
- [ ] Port `examples/avionics_examples/dec_cylinder_wake/main.rs`: collect each step's row as
      pre-formatted strings (identical specifiers), then write the wake CSV via a single
      `write_csv_to` step at the edge. Verify byte-for-byte equality against the baseline
      (`cargo run` + diff).
- [ ] Port `dec_lid_cavity_re1000`'s raw centerline writes onto `write_csv_to`; verify bit-identical.
- [ ] Add `tempfile` as a dev-dependency; tests for the CSV helpers.

## Cross-cutting

- [ ] Each new module opts into `[lints] workspace = true`.
- [ ] `make format && make fix`; `cargo build`/`cargo test` for the three crates; `make build` /
      `make test` once all three have changed.
- [ ] 100% coverage of added code; no prelude; no macros in `src`; static dispatch only.
- [ ] Prepare a commit message and ask the user to commit (do not commit).
