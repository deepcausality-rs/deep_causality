## ADDED Requirements

### Requirement: Value-level Arrow with total composition

`deep_causality_haft` SHALL provide a value-level `Arrow` trait — `type In`, `type Out`, and `fn run(&self, input: Self::In) -> Self::Out` — together with concrete combinator types that make composition total under static dispatch (no `dyn`/trait objects, no macros). It SHALL provide `Id<A>` (the identity arrow), `Pure<A, B, F>` lifting any `Fn(A) -> B` into an arrow, and `Compose<F, G>` implementing `f >>> g` (run `g` on the output of `f`), available as a fluent method `Arrow::compose`. Each combinator SHALL itself implement `Arrow`, so composites compose. The trait and combinator types SHALL be re-exported from the crate root.

#### Scenario: Lift and run a function

- **WHEN** a function `f: Fn(A) -> B` is lifted with `Pure` and `run` on input `a`
- **THEN** the result equals `f(a)`

#### Scenario: Composition runs left-to-right and is total

- **WHEN** arrows `f: A → B` and `g: B → C` are composed as `f.compose(g)` and run on `a`
- **THEN** the result equals `g(f(a))`, and the composite is itself an `Arrow<In = A, Out = C>` that can be composed further

#### Scenario: Identity is the unit of composition

- **WHEN** `Id` is composed before or after an arrow `f`
- **THEN** `Id.compose(f)` and `f.compose(Id)` both run identically to `f` (the category identity laws)

#### Scenario: Composition is associative

- **WHEN** three arrows are composed as `(f.compose(g)).compose(h)` and as `f.compose(g.compose(h))`
- **THEN** both run to the same result for every input

### Requirement: Monoidal product (strength) on arrows

`deep_causality_haft` SHALL provide the strong-profunctor / Hughes-Arrow product combinators as concrete `Arrow`-implementing types, available as fluent methods: `First<F>` (`A → B` becomes `(A, C) → (B, C)`), `Second<F>` (`(C, A) → (C, B)`), `Split<F, G>` / `***` (`(A, C) → (B, D)` from `A → B` and `C → D`), and `Fanout<F, G>` / `&&&` (`A → (B, C)` from `A → B` and `A → C`, requiring `In: Clone`). These give arrows the monoidal product that the causal monad's `bind` cannot express.

#### Scenario: First acts on the first component only

- **WHEN** `f.first::<C>()` runs on `(a, c)`
- **THEN** the result is `(f.run(a), c)` — the second component passes through unchanged

#### Scenario: Split runs two arrows in parallel on a pair

- **WHEN** `f.split(g)` runs on `(a, c)`
- **THEN** the result is `(f.run(a), g.run(c))`

#### Scenario: Fanout feeds one input to two arrows

- **WHEN** `f.fanout(g)` runs on `x` (with `In: Clone`)
- **THEN** the result is `(f.run(x), g.run(x))`

#### Scenario: The product decomposes via first and second

- **WHEN** `f.split(g)` is compared to `f.first().compose(g.second())` on the same `(a, c)`
- **THEN** both run to `(f.run(a), g.run(c))` (the `*** = first >>> second` law)

### Requirement: Multi-input arrows keep static structure as a parameter

The combinators SHALL be sufficient to express a multi-input operator over two aligned inputs — `(InA, InB) → Out` built by `arrow_a.split(arrow_b)` followed by a combiner — without that operator being a Kleisli arrow. Any static structure an arrow depends on (a graph, a lattice, a configuration) SHALL be a captured **parameter** of the arrow value, never a flowing `In`/`Out` component, so that the operator remains Arrow-but-not-Kleisli.

#### Scenario: Two aligned inputs combine into one arrow

- **WHEN** two arrows over distinct inputs are combined with `split` and composed with a combiner arrow
- **THEN** the result is a single `Arrow` from the input pair to the combined output, and the structural parameter it closes over does not appear in `In` or `Out`

### Requirement: A builder hides the arrow machinery behind a fluent surface

`deep_causality_haft` SHALL provide a fluent **arrow builder** that lets a user construct a composed arrow as a left-to-right chain without naming the combinator types (`Compose`, `Split`, …) or the witness-level `Morphism`. It SHALL provide an entry point that lifts a function into a builder, chaining methods that desugar to the `Arrow` combinators — sequential (`then`, an alias of `compose`), parallel-product (`par`, an alias of `split`; `fanout`) — and a terminal that either yields the composed `Arrow` value (`build`) or applies it to an input (`run`). The builder SHALL thread the growing arrow type through `Self` (the typestate/witness-camouflage pattern the CDL builder already uses), and `Arrow` SHALL carry `#[diagnostic::on_unimplemented]` so a mis-typed chain produces a legible error. The categorical method names SHALL remain available on the `Arrow` trait; the builder provides the friendlier aliases. The builder SHALL preserve the parameter-vs-payload separation: structure an arrow depends on is captured as a parameter of the arrow value, not introduced as a flowing builder step.

This change provides the **carrier-free generic** arrow builder only. The causal process builder over `PropagatingEffect`/`PropagatingProcess` (which additionally hides the causal monad) is out of scope here and belongs to a later change that owns that carrier.

#### Scenario: A fluent chain builds a composed arrow without naming combinator types

- **WHEN** a user writes `arrow(f).then(g).par(h).build()` (or `.run(input)`)
- **THEN** the result is the same arrow as the explicit `Pure::new(f).compose(g).split(h)` construction, and the user's code names no combinator struct (`Compose`/`Split`/`Pure`) nor the witness `Morphism`

#### Scenario: The builder yields a reusable arrow

- **WHEN** `build()` is called at the end of a chain
- **THEN** it returns a value implementing `Arrow` that can be `run` repeatedly and composed further
