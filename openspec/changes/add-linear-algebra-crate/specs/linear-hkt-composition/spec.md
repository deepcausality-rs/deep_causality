## ADDED Requirements

### Requirement: Every container type carries an HKT witness
`deep_causality_linear` SHALL provide a `deep_causality_haft` witness for each of its container types — the dense matrix, the vector, the bit-packed 𝔽₂ matrix and the sparse matrix — and each witness SHALL implement the same trait set the existing witnesses do.

Uniform composition across the mathematical crates is the reason `deep_causality_haft` exists, and
the containers this crate ships are the ones a caller would compose with `CausalTensor` and
`CausalTensorTrain`. `CsrMatrixWitness` (`sparse/src/extensions/ext_hkt.rs:19`) already implements
`HKT`, `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad`, `CoMonad` and `Adjunction`; a new
container that stopped short of that would be composable in some pipelines and not others, which is
worse than being uniformly absent.

#### Scenario: Each container has a witness
- **WHEN** the crate's public surface is enumerated
- **THEN** the dense matrix, the vector, the bit-packed 𝔽₂ matrix and the sparse matrix each name a witness type

#### Scenario: The witness surface matches the existing one
- **WHEN** a new witness is compared against `CsrMatrixWitness`
- **THEN** it implements the same trait set, or documents at that impl site which trait it cannot support and why

#### Scenario: The moved witness is unchanged
- **WHEN** `CsrMatrixWitness` is compared against its behaviour before the move
- **THEN** its trait impls and their results are identical

### Requirement: The HKT laws hold for every witness
Each witness SHALL satisfy the functor, applicative, monad and comonad laws its impls claim, and each law SHALL be exercised by a test.

An HKT impl that does not satisfy its laws is worse than no impl: it composes, and produces wrong
answers only when a caller relies on the law. The laws are cheap to state and cheap to check at
representative values, and the repository already does this for its existing witnesses.

#### Scenario: Functor identity and composition
- **WHEN** `fmap(id)` and `fmap(f ∘ g)` are applied to a container
- **THEN** the first is the identity and the second equals `fmap(f) ∘ fmap(g)`

#### Scenario: Monad left and right identity
- **WHEN** `bind(pure(a), f)` and `bind(m, pure)` are evaluated
- **THEN** they equal `f(a)` and `m` respectively

#### Scenario: Monad associativity
- **WHEN** `bind(bind(m, f), g)` and `bind(m, |x| bind(f(x), g))` are evaluated
- **THEN** they are equal

#### Scenario: Comonad extract and extend
- **WHEN** `extend(extract)` is applied to a container
- **THEN** the result equals the container

### Requirement: Shape is preserved by the mapping operations
`fmap` over any of the crate's containers SHALL preserve the container's shape, and SHALL NOT change a matrix's dimensions or a vector's length.

`fmap` is elementwise, so the shape is not its business. A sparse matrix adds a wrinkle worth stating
because it is the one that has bitten this pattern before: mapping a function that sends zero to a
non-zero value changes which entries are structurally present, and the impl has to decide whether it
maps the stored entries or the whole logical matrix.

#### Scenario: Dimensions survive a map
- **WHEN** `fmap` is applied to an `m × n` matrix
- **THEN** the result is `m × n`

#### Scenario: The sparse mapping rule is stated
- **WHEN** `fmap` is applied to a sparse matrix with a function that does not fix zero
- **THEN** the documented behaviour is what happens, and the documentation says which of the two it is

### Requirement: The crate composes with the neighbouring mathematical crates
A pipeline SHALL be able to move a value between this crate's containers and `deep_causality_tensor`'s through the HKT surface without a hand-written adapter.

This is what uniform composition means in practice, and the workspace has examples that already do
it across three crates (`examples/mathematics_examples/composable_multi_math/`). The conversions
between representations are specified in `linear-matrix-representations`; this requirement is that
the HKT surface does not become the place where composition stops.

#### Scenario: A cross-crate pipeline compiles
- **WHEN** a value is mapped, folded and bound across this crate's containers and `CausalTensor`
- **THEN** the pipeline compiles with no bespoke adapter

#### Scenario: The existing multi-math examples still run
- **WHEN** the examples under `examples/mathematics_examples/composable_multi_math/` are run after migration
- **THEN** each produces the output it produced before
