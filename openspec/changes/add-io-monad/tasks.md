## 1. Core `Io<T>` effect (deep_causality_core)

- [ ] 1.1 Add `src/types/io/mod.rs`: the `Io<T>` type over an internal action/combinator enum (`Pure`, `Map`, `AndThen`, and the std-gated file leaves); register the module and re-export `Io` from `lib.rs`.
- [ ] 1.2 Implement the pure combinators (`pure`, `map`, `and_then`, `sequence`) — no_std-safe; `and_then`'s boxed continuation `alloc`-gated.
- [ ] 1.3 Implement `run(self) -> Result<T, CausalityError>` interpreting the action tree; map `std::io::Error` → `CausalityError`; short-circuit on first failure.
- [ ] 1.4 Implement the std-gated file actions: `write_text(path, contents)`, `write_csv(path, header, rows)`, `read_text(path)`.

## 2. Tests

- [ ] 2.1 Lazy property: constructing/composing actions writes nothing until `run()`.
- [ ] 2.2 Combinators: `pure`/`map`/`and_then`/`sequence` compose and run in order; `write_text` → `read_text` round-trips.
- [ ] 2.3 `write_csv` byte-exactness against a known header+rows fixture.
- [ ] 2.4 Failure short-circuits: a failing first action returns `Err(CausalityError)` and the second does not run.
- [ ] 2.5 no_std build: pure combinators compile without `std`; file actions are absent.

## 3. CFD consumer + migration

- [ ] 3.1 Add a `deep_causality_cfd` corpus helper that renders centerline rows (computed + Ghia reference + diff) into `write_csv`'s `header` + `Vec<String>` rows, reproducing the example's exact formatting.
- [ ] 3.2 Migrate `dec_lid_cavity_re1000`: replace the `File::create`/`writeln!` `write_centerline_csv` with an `Io` program (`write_csv` × 2, `run()` at the edge of `main`).
- [ ] 3.3 Verify the two `cavity_centerline_*.csv` outputs are **bit-identical** to the pre-migration files (A/B diff on grid 33²).

## 4. Gate

- [ ] 4.1 `openspec validate add-io-monad --strict`; `make format && make fix`; build + tests for `deep_causality_core` and `deep_causality_cfd`, both feature configs; prepare per-crate commit messages; archive the change.
