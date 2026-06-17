## Why

The CfdFlow DSL composes a simulation declaratively but its examples still perform side-effecting
file output imperatively — `dec_lid_cavity_re1000` writes its two `cavity_centerline_*.csv` files with
raw `File::create`/`writeln!`. The CFD consolidation explicitly deferred file I/O to "an IO monad" so
that reading and writing files becomes a composable, lazily-evaluated effect that the DSL can carry
and that runs only at the program edge. No IO effect exists anywhere in the workspace today.

## What Changes

- Introduce a general, lazy `Io<T>` effect in the core layer: a deferred description of an IO
  computation that performs **no** side effects until an explicit `run()` at the program edge.
- Functor/monad surface: `pure`, `map`, `and_then`, and `sequence` (for a list of actions), so IO
  steps compose without running.
- A minimal file-action set: `write_text(path, contents)`, `write_csv(path, header, rows)`,
  `read_text(path)` — enough to retire the cavity CSV writes and to read input data later.
- Errors thread through the existing `CausalityError` algebra (an IO failure becomes a
  `CausalityError`), so an `Io<T>` pipeline short-circuits like the rest of the causal monad and
  interoperates with `PropagatingEffect` / `CausalFlow`.
- File IO is **std-only**; the effect type and its pure combinators are available without `std`, but
  the file actions and `run()` are `#[cfg(feature = "std")]`-gated.
- First consumer: migrate `dec_lid_cavity_re1000`'s two `cavity_centerline_*.csv` writes onto
  `Io::write_csv`, executed via a single `run()` — output **bit-identical** to the current writes.

## Capabilities

### New Capabilities
- `io-monad`: a lazy `Io<T>` effect for deferred, composable file input/output that runs only at the
  program edge, propagates failures as `CausalityError`, and composes with the causal-monad / Flow
  DSLs.

### Modified Capabilities
<!-- None: this adds a new capability; existing specs' requirements are unchanged. The CfdFlow DSL
     consumes Io but its requirement set does not change. -->

## Impact

- **New code**: an `io` module in `deep_causality_core` (placement decided in design.md — core vs
  `deep_causality_haft`), with the `Io<T>` type, combinators, file actions, and `run()`.
- **Dependencies**: no new external crates; reuses `CausalityError`. File actions use `std::fs`/`std::io`.
- **Consumers**: `deep_causality_cfd` may expose a `Report`→CSV-rows helper that feeds `Io::write_csv`;
  `dec_lid_cavity_re1000` migrates its file writes. Other examples that write to stdout are unaffected.
- **Feature flags**: the `std` feature gates the file actions and `run()`; pure combinators are
  no_std-safe.
