## Context

`deep_causality_core` already provides the causal-effect monads (`PropagatingEffect`,
`PropagatingProcess`, `CausalFlow`) and the `CausalityError` algebra, but nothing models *side
effects*. The CFD examples write CSV files imperatively (`File::create`/`writeln!`), which the
consolidation flagged as the one part of an otherwise declarative pipeline that escapes the DSL. We
want a lazy IO effect — a *description* of file work that runs only at the edge — that reuses the
existing error algebra and composes with the causal monads. The crate is `no_std`-capable; file IO is
inherently `std`.

## Goals / Non-Goals

**Goals:**
- A lazy `Io<T>`: constructing or composing it performs **no** side effect; effects happen only at an
  explicit `run()` call.
- A small, monadic surface (`pure`, `map`, `and_then`, `sequence`) so multi-file output composes into
  one runnable program.
- A minimal file-action set sufficient to retire the cavity CSV writes: `write_text`, `write_csv`,
  `read_text`.
- Failures surface as `CausalityError` so an `Io` pipeline short-circuits and interoperates with the
  rest of the causal stack.
- `no_std`-safe pure combinators; `std`-gated file actions and `run()`.

**Non-Goals:**
- Async IO, sockets, process spawning, or a general effect system beyond files.
- Capturing stdout/stderr (examples that `println!` are out of scope).
- A typed CSV schema / serde layer — `write_csv` takes pre-rendered rows.
- Parallel/concurrent IO.

## Decisions

### D1 — Placement: `deep_causality_core`
`Io<T>` lives in `deep_causality_core` (`src/types/io/`), next to the other effect monads it mirrors
and whose `CausalityError` it reuses. `deep_causality_haft` holds higher-order *trait* abstractions
(HKT witnesses), not concrete effect runtimes, so it is the wrong home. Core already gates `std`
features, so the file actions fit its existing feature story.

### D2 — Representation: an eager-built action tree, lazily run
`Io<T>` is an owned value describing the computation, not a boxed `FnOnce`. A small internal enum
captures the leaves (`Pure(T)`, `WriteText{path,contents}`, `WriteCsv{path,header,rows}`,
`ReadText{path}`) and the combinators (`Map`, `AndThen`). `run(self) -> Result<T, CausalityError>`
interprets the tree once, performing the side effects in order. This keeps `Io<T>` `Clone`/`Debug`
where the payload allows, avoids boxed closures in the common path, and makes the "no effect before
`run`" property structural rather than convention. (`AndThen` needs a stored continuation; it is the
one boxed-closure case, `std`/`alloc`-gated.)

### D3 — Error channel: `CausalityError`
Every fallible action maps its `std::io::Error` into `CausalityError` (the existing
`CausalityErrorEnum::Custom`/IO variant), so `Io` short-circuits identically to `PropagatingEffect`
and an `Io` result drops cleanly into a `CausalFlow` step. No new error type.

### D4 — Action set (v1)
- `Io::pure(value)` — lift a value, no effect.
- `Io::write_text(path, contents)` — write a UTF-8 file, yielding `()`.
- `Io::write_csv(path, header, rows)` — write `header` then each pre-rendered row line, yielding `()`.
  Rows are `Vec<String>` (already formatted by the caller), so byte output is fully caller-controlled
  (needed for bit-identical migration).
- `Io::read_text(path)` — read a file to `String`.
- `Io::sequence(actions)` — run a `Vec<Io<()>>` in order, yielding `()`.

### D5 — Integration with the Flow DSL / Report
The CFD side stays thin: a corpus helper renders a `Report` (or example-computed profile) into CSV
`header` + `rows`, and the example builds an `Io` program from those and calls `run()` at the end of
`main`. No change to `CfdFlow::run` — the DSL still returns an owned `Report`; IO is a separate,
explicit edge stage, preserving "borrows never escape run" (D2 of the Flow design).

### D6 — Std boundary
The `Io<T>` type, `pure`, `map`, `and_then`, and `sequence` are available without `std`. The file
*actions* (`write_text`/`write_csv`/`read_text`) and `run()`'s file interpretation are
`#[cfg(feature = "std")]`. Under `no_std` an `Io` can still be *described* over `pure`/`map` but not
file-run.

## Risks / Trade-offs

- **Boxed continuation for `and_then`.** Monadic bind needs a stored closure (`Box<dyn FnOnce>`),
  the one allocation/dynamic-dispatch point. Acceptable: IO is edge-only and not hot; the action
  leaves stay closure-free. Documented; `alloc`-gated.
- **Not a full effect system.** Files only. Mitigation: the action enum is extensible; new leaves add
  without changing the combinator surface (open/closed).
- **Bit-identical migration constraint.** `write_csv` must reproduce the example's exact bytes. Taking
  pre-rendered `Vec<String>` rows (caller formats with the same `{:.6}`/`{:+.6}` specifiers) keeps the
  output byte-for-byte; the migration is verified by diffing the cavity CSVs against the current ones.
- **Laziness vs. ergonomics.** An action tree is slightly more machinery than a `FnOnce` thunk, but it
  buys `Debug`/structural guarantees and avoids capturing the world in a closure.
