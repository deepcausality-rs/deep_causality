## 1. `Arrow` trait + category combinators in `deep_causality_haft`

- [x] 1.1 Added `src/arrow/` (one combinator struct per file). `pub trait Arrow { type In; type Out; fn run(&self, input: Self::In) -> Self::Out; }` with `#[diagnostic::on_unimplemented]` and the fluent provided methods `compose`/`first`/`second`/`split`/`fanout`. The trait lives in `arrow/mod.rs` (not `arrow/arrow.rs`, which would trip `clippy::module_inception`).
- [x] 1.2 `Id<A>` (identity arrow, with `Default`) and `Lift<A, B, F>` (lift `F: Fn(A) -> B`, carrying `A`/`B` via `PhantomData` to avoid E0207 — **`Lift`, not `Pure`**, since `haft` already exports a `Pure` trait); each `impl Arrow`. `Lift` is the value-level counterpart of `FnMorphism::apply`; `Compose` supplies the composition `Morphism` deferred to this stage.
- [x] 1.3 `Compose<F, G>` with `impl<F: Arrow, G: Arrow<In = F::Out>> Arrow for Compose<F, G>` (`run` = `g.run(f.run(x))`).
- [x] 1.4 Re-exported `Arrow`, `Id`, `Lift`, `Compose`, `First`, `Second`, `Split`, `Fanout`, `ArrowBuilder`, `arrow` from `src/lib.rs` (`mod arrow;` kept private so the `arrow` fn and `arrow` module do not clash).

## 2. Strength / monoidal-product combinators

- [x] 2.1 `First<F, C>` and `Second<F, C>` (act on one component of a pair; the other passes through).
- [x] 2.2 `Split<F, G>` (`***`): `(F::In, G::In) → (F::Out, G::Out)`.
- [x] 2.3 `Fanout<F, G>` (`&&&`): `F::In → (F::Out, G::Out)` with `F::In = G::In` and `In: Clone` (bound on the `fanout` method + the `Fanout` impl, not the trait).

## 3. Arrow builder (hides the machinery)

- [x] 3.1 `ArrowBuilder<S: Arrow>` + entry point `arrow(f)` lifting `F: Fn(A) -> B` into `ArrowBuilder<Lift<A, B, F>>`, and `ArrowBuilder::new` to wrap an existing arrow. Methods: `then` (compose), `then_fn` (lift a raw closure then compose), `par` (split), `fanout`, terminals `build()` and `run(input)`. Threads the growing arrow type through `Self`. (`then_fn`'s return is a `type ThenFn<…>` alias to satisfy `clippy::type_complexity`.)
- [x] 3.2 `#[diagnostic::on_unimplemented]` on `Arrow` for legible errors. Categorical names (`compose`/`split`/`fanout`) on `Arrow`; friendly aliases (`then`/`par`/`fanout`) on the builder. No sealing needed.
- [x] 3.3 Documented (module + builder docs) that this is the carrier-free generic builder; the causal process builder over `PropagatingEffect`/`PropagatingProcess` is `causal-arrow-cdl-unification` (note `causal-process-builder.md`).

## 4. Tests (arrow laws + multi-input witness + builder)

- [x] 4.1 Category laws: `Lift`/`run` equals the function; `compose` runs left-to-right; `Id` is the left and right unit; composition is associative.
- [x] 4.2 Product laws: `first`/`second` pass-through; `split` parallel on a pair; `fanout` feeds one input to two arrows; `*** = first >>> second` decomposition.
- [x] 4.3 Multi-input witness: `normal.split(anomalous).compose(combine)` over two cohorts; the captured structural parameter (`bias`) appears in neither `In` nor `Out`. `tests/algebra/arrow_tests.rs` (Bazel `glob`).
- [x] 4.4 Builder: `arrow(f).then_fn(g).par(h).build()`/`.run(x)` equals the explicit `Lift`/`compose`/`split` construction, with no combinator-struct or `Morphism` name in the pipeline expression; `build()` yields a reusable, further-composable `Arrow`.

## 5. Verification

- [x] 5.1 `cargo test -p deep_causality_haft` — 176 + doctests pass, 0 failed (13 new arrow tests).
- [x] 5.2 `cargo fmt` + `cargo clippy -p deep_causality_haft --all-targets` — 0 warnings/errors, no `#[allow(...)]`; no `dyn`/macros/external deps introduced.
- [x] 5.3 Purely additive: `Morphism`/`Endomorphism` unchanged; existing-file diffs limited to `lib.rs` `mod`/`pub use` + `tests/algebra/mod.rs` registration. Commit message prepared; not committed (owner commits).
