<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: The lazy graph is a value-level Arrow

`Uncertain<R>` SHALL implement `deep_causality_haft::Arrow` with the sample index as its input, and the graph's composition SHALL use haft's combinators rather than combinators private to this crate.

```rust
impl<R: RandScalar> Arrow for Uncertain<R> {
    type In = SampleIndex;
    type Out = Result<R, UncertainError>;
    fn run(&self, at: SampleIndex) -> Self::Out;
}
```

This is well-typed only because every leaf draw is addressed by seed, index and leaf ordinal:
evaluating the graph at an index is then a pure function, which is what `Arrow::run(&self, ..)`
requires. A graph that drew from ambient mutable state could not satisfy it.

`Arrow` is where haft puts a computation that stores functions. The container traits are for data
and deliberately carry no `'static`; a lazy graph is a program.

#### Scenario: A graph composes with a downstream operator

- **WHEN** an `Uncertain<R>` is composed with a further arrow using haft's `compose`
- **THEN** the composite is itself an `Arrow`, and running it at an index yields the downstream result for that index

#### Scenario: Composition is static

- **WHEN** the composed type is inspected
- **THEN** it is a concrete generic struct, and no trait object appears in its type

#### Scenario: Running at one index twice agrees

- **WHEN** the same graph is run at the same index twice under one session seed
- **THEN** both runs produce the same value, with nothing stored between them

### Requirement: The unreachable HKT node arms are removed

The node enum SHALL NOT retain a variant that no public constructor produces, and the crate SHALL contain no `dyn` in library code.

`PureOp`, `FmapOp`, `ApplyOp` and `BindOp` exist today and no builder feeds any of them. They were
an attempt to put the graph on the container surface; they ask for `Send + Sync + 'static` on a
stored closure, which the container traits withhold, so no implementation could ever have been
written against them. The Arrow instance is the composition surface they were reaching for.

Four of the crate's six `Arc<dyn ...>` sites are these arms. The remaining two are the
`FunctionOpF64` and `FunctionOpBool` arms, which take a function as a type parameter instead, as
`deep_causality_calculus` does for `Euler`, `Rk4` and `Diff`. AGENTS.md forbids `dyn` in everything
under `src/`.

#### Scenario: No unreachable variant remains

- **WHEN** the node enum is read after this change
- **THEN** every variant is produced by some public constructor or documented graph transformation

#### Scenario: The crate contains no trait object

- **WHEN** `deep_causality_uncertain/src` is searched for `dyn`
- **THEN** no occurrence remains

#### Scenario: The QMC guard still rejects data-dependent structure

- **WHEN** a graph whose structure depends on a sampled value is handed to the QMC pre-pass
- **THEN** it is rejected with an error naming the static-structure requirement, as before
