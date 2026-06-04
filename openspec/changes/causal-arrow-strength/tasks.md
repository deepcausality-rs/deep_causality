## 1. `Arrow` trait + category combinators in `deep_causality_haft`

- [ ] 1.1 Add an `arrow` module (folder `src/arrow/` with the trait + one struct per file, mirroring the type-per-module convention). Define `pub trait Arrow { type In; type Out; fn run(&self, input: Self::In) -> Self::Out; }` with `#[diagnostic::on_unimplemented]`, and the fluent provided methods `compose`/`first`/`second`/`split`/`fanout` returning the combinator structs.
- [ ] 1.2 `Id<A>` (identity arrow) and `Pure<A, B, F>` (lift `F: Fn(A) -> B`, carrying `A`/`B` via `PhantomData` to avoid E0207); each `impl Arrow`. Document that `Pure` subsumes `FnMorphism::apply` and `Compose` supplies the composition `Morphism` deferred to this stage.
- [ ] 1.3 `Compose<F, G>` with `impl<F: Arrow, G: Arrow<In = F::Out>> Arrow for Compose<F, G>` (`run` = `g.run(f.run(x))`).
- [ ] 1.4 Register the modules in `src/arrow/mod.rs` (or `src/traits/`), re-export `Arrow`, `Id`, `Pure`, `Compose` (and the product structs) from `src/lib.rs`.

## 2. Strength / monoidal-product combinators

- [ ] 2.1 `First<F, C>` and `Second<F, C>` (act on one component of a pair; the other passes through).
- [ ] 2.2 `Split<F, G>` (`***`): `(F::In, G::In) → (F::Out, G::Out)`.
- [ ] 2.3 `Fanout<F, G>` (`&&&`): `F::In → (F::Out, G::Out)` with `F::In = G::In` and `In: Clone` (bound only on the `fanout` method / this impl).

## 3. Arrow builder (hides the machinery)

- [ ] 3.1 Add `ArrowBuilder<S: Arrow>` plus the entry point `arrow(f)` lifting `F: Fn(A) -> B` into `ArrowBuilder<Pure<A, B, F>>`. Methods: `then`/`then_fn` (alias of `compose`), `par` (alias of `split`), `fanout`, terminal `build()` (yield the composed `Arrow`) and `run(input)` (apply). Threads the growing arrow type through `Self`.
- [ ] 3.2 Add `#[diagnostic::on_unimplemented]` to `Arrow` (and seal where it improves errors) so a mis-typed chain is legible. Keep the categorical names on `Arrow`; the friendly aliases on the builder.
- [ ] 3.3 Document that this is the carrier-free generic builder; the causal process builder over `PropagatingEffect`/`PropagatingProcess` is `causal-arrow-cdl-unification` (note `causal-process-builder.md`).

## 4. Tests (arrow laws + multi-input witness + builder)

- [ ] 4.1 Category laws: `Pure`/`run` equals the underlying function; `compose` runs left-to-right; `Id` is the left and right unit; composition is associative (same result for `(f≫g)≫h` and `f≫(g≫h)`).
- [ ] 4.2 Product laws: `first` passes the second component through; `split` runs both arrows on a pair; `fanout` feeds one input to two arrows; `*** = first >>> second` (decomposition law).
- [ ] 4.3 Multi-input witness: build `(InA, InB) → Out` via `a.split(b).compose(combine)`; assert the structural parameter the arrows close over appears in neither `In` nor `Out` (it is captured, not flowing). Register tests; Bazel uses `glob`.
- [ ] 4.4 Builder: `arrow(f).then(g).par(h).build()` (and `.run(input)`) produces the same result as the explicit `Pure`/`compose`/`split` construction, with no combinator-struct or `Morphism` name in the test's pipeline expression; `build()` yields a reusable, further-composable `Arrow`.

## 5. Verification

- [ ] 5.1 `cargo build -p deep_causality_haft && cargo test -p deep_causality_haft`.
- [ ] 5.2 `cargo fmt` + `cargo clippy -p deep_causality_haft --all-targets` — 0 warnings/errors, no `#[allow(...)]`; confirm no `dyn`/macros/external deps introduced.
- [ ] 5.3 Purely additive: `Morphism`/`Endomorphism` unchanged; existing-file diffs limited to `mod`/`pub use` lines. Prepare a commit message; do not commit (owner commits).
