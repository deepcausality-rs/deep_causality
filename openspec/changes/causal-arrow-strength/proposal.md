## Why

`causal-arrow-foundations` shipped `Morphism` (identity + application) but **deliberately omitted composition**: general arrow composition over capturing closures has no single concrete carrier under the repo's `unsafe_code = "forbid"` / no-`dyn` policy (closures are unnameable unique types; `Box<dyn Fn>` is a forbidden trait object). The Causal Arrow program cannot progress without it — composition (`>>>`) and the **monoidal product** (`first`/`***`, the strong-profunctor piece) are what let discovery operators be *wired together*, and the monoidal product is the one operation the causal monad's `bind` cannot express. It is the program's technical fulcrum: it is what makes multi-input operators (BRCD's two aligned data cohorts) expressible as a single composed arrow, which is precisely the Arrow-but-not-Kleisli structure the whole thesis turns on.

The blocker is real but has a known, fully-static resolution: realize composition and the product at the **value level**, where each combinator returns a *new concrete arrow type* (`Compose<F, G>`, `First<F>`, `Split<F, G>`, …) rather than trying to stuff a composed closure into one witness GAT. Composition becomes total, every combinator is a zero-cost generic struct, and no `dyn`/macros are involved.

## What Changes

- Add a value-level **`Arrow`** trait to `deep_causality_haft`: an arrow is a concrete value with `type In`, `type Out`, and `fn run(&self, In) -> Out`. This is the composable realization of the foundations `Morphism` interface (which stays as-is, the witness-level typeclass).
- Add the **category** combinators: `Id` (identity arrow) and `Compose<F, G>` (`f >>> g`), making composition **total** — the gap foundations left open. Provide `Lift<A, B, F>` to lift any `Fn(A) -> B` into an arrow.
- Add the **strength / monoidal-product** combinators (the strong-profunctor / Hughes-Arrow surface), each a generic struct implementing `Arrow`:
  - `First<F>`: `A → B` becomes `(A, C) → (B, C)`; `Second<F>`: `(C, A) → (C, B)`.
  - `Split<F, G>` (`***`): `(A, C) → (B, D)` from `A → B` and `C → D`.
  - `Fanout<F, G>` (`&&&`): `A → (B, C)` from `A → B` and `A → C` (input `Clone`).
- Expose them as fluent provided methods on `Arrow` (`.compose(g)`, `.first()`, `.split(g)`, `.fanout(g)`) so a pipeline reads as a left-to-right chain — the textual form of a string/wiring diagram.
- Add a thin **arrow builder** surface so the **entire categorical machinery is hidden behind a builder pattern**: an entry point (`arrow(f)`) plus friendly aliases (`.then(f)`, `.par(g)`, `.fanout(g)`, terminal `.run(input)` / `.build()`) over the `Arrow` algebra, with `#[diagnostic::on_unimplemented]` + sealing so the user writes a fluent chain and never names the combinator types (`Compose<Split<…>, …>`) or the witness `Morphism`. This is §8 of `causal-arrow-generalization.md` ("the builder *is* the syntax of the Arrow") and the ergonomic resolution of the value-`Arrow`/witness-`Morphism` split — both are desugared forms the user does not touch. The broader **causal process builder** that hides the monad + witness over the `PropagatingEffect`/`PropagatingProcess` carrier (extending today's CDL builder) is captured in `openspec/notes/arrow/causal-process-builder.md` and **deferred to `causal-arrow-cdl-unification`**, since it requires the §10 carrier rework; this change ships only the carrier-free *generic* arrow builder.
- Verify the **arrow laws** (category identity/associativity; the `first`/product exchange laws; `*** = first >>> second`) as tests, and witness the **multi-input** case (two aligned data cohorts combined by `***`, with static structure kept as a *parameter*, never a flowing input — preserving the non-Kleisli invariant).
- **Out of scope:** recasting SURD/BRCD as concrete arrows (that is `causal-arrow-cdl-unification`); `ArrowChoice` (`+++`/`|||`, sum types) and `ArrowLoop` (feedback) beyond what `CyberneticLoop` already covers; any change to `Morphism`/`Endomorphism`/`Dual` or to SURD/BRCD numerics.
- **No new external or numeric dependency.** Lift generic structs over the existing trait machinery; stays inside `unsafe_code = "forbid"`, static-dispatch-only, no macros in `src/`.

## Capabilities

### New Capabilities
- `arrow-strength`: a value-level `Arrow` algebra in `deep_causality_haft` — total composition (`Id`/`Compose`/`Lift`) plus the monoidal product (`First`/`Second`/`Split`/`Fanout`), realized as zero-cost generic combinator structs, with the arrow laws verified; **plus a fluent arrow builder that hides the combinator types and the witness machinery**, so the categorical algebra is the desugared form behind a user-facing builder. This is the strong category / Hughes Arrow that the witness-level `Morphism` could not express, the structural prerequisite for wiring multi-input discovery operators, and the ergonomic surface that makes it usable.

### Modified Capabilities
<!-- None. `morphism-algebra` is unchanged: the witness-level `Morphism` keeps its identity+application surface and its documented note that composition lives in the strength stage. `arrow-strength` fulfils that note additively at the value level; it does not alter `Morphism`/`Endomorphism`. -->

## Impact

- **New code, `deep_causality_haft`:** an `arrow` module under `src/traits/` (or `src/arrow/`) holding the `Arrow` trait and the combinator structs (`Id`, `Lift`, `Compose`, `First`, `Second`, `Split`, `Fanout`), re-exported from `src/lib.rs`; mirrored tests under `tests/`.
- **Relationship to `Morphism`:** `Lift::new(f)` lifts a plain function (subsuming `FnMorphism`'s apply role) and `Compose` makes composition total — so `arrow-strength` is the value-level layer that completes what `Morphism` started. `Morphism`/`Endomorphism` are untouched.
- **APIs:** one new trait + a handful of combinator structs, all additive. No existing signature changes.
- **Dependencies:** none added.
- **Consumers (later changes):** `causal-arrow-cdl-unification` recasts SURD/BRCD as concrete arrows wired with these combinators; the multi-input product is what makes BRCD's two-cohort `Data ⊗ Data` expressible as one arrow.
- **Verification:** arrow-law tests (category + product) and a multi-input witness, to 100% coverage of the new code.
