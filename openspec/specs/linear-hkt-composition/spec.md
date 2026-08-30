# linear-hkt-composition Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: Every element-generic container carries an HKT witness
`deep_causality_linear` SHALL provide a `deep_causality_haft` witness for each container that is generic in its element type — the dense matrix, the vector and the sparse matrix — and each witness SHALL implement the same trait set the existing witnesses do.

Uniform composition across the mathematical crates is the reason `deep_causality_haft` exists, and
the containers this crate ships are the ones a caller would compose with `CausalTensor` and
`CausalTensorTrain`. `CsrMatrixWitness`
(`deep_causality_unified_math/deep_causality_linear/src/extensions/hkt/csr_matrix_witness.rs:27`) implements `HKT`, `Functor`,
`Foldable`, `Pure`, `Applicative` and `CoMonad`; a new container that stopped short of that would be
composable in some pipelines and not others, which is worse than being uniformly absent.

`Monad` and `Adjunction` are not in that set, and a new witness does not owe them. The sparse `bind`
flattens to `1 × count`, so `bind(m, pure)` turns a 2×2 into a 1×4 and monad right identity fails;
`Adjunction`'s `counit` is written in terms of that `bind` and inherits the defect. The cause is
structural, since `pure` must pick a shape for one value and a shaped container has no canonical one.
The reasoning sits at the impl site and in `openspec/notes/unified_math/HKT-LAW-FINDINGS.md`.

**The bit-packed 𝔽₂ matrix is excluded, and the exclusion is structural.** `HKT` projects `Type<T>`
to a container of `T`. `PackedGf2<W>` is generic in its *word* type and not in its element type,
which is fixed to `Gf2` by the storage: one bit per entry has no room for anything else. There is no
`PackedGf2<T>` for `Type<T>` to name, and `fmap` with a function returning `f64` would have nowhere
to put the result.

This follows from the packing decision, which `linear-matrix-representations` takes on measured
grounds — 3.2× faster on one eighth the memory at n=2048. A caller who wants to map over an 𝔽₂
matrix unpacks to `DenseMatrix<Gf2>`, which has a witness, and the conversion is explicit.

#### Scenario: Each element-generic container has a witness
- **WHEN** the crate's public surface is enumerated
- **THEN** the dense matrix, the vector and the sparse matrix each name a witness type

#### Scenario: The packed exclusion is stated where a reader will look
- **WHEN** the bit-packed 𝔽₂ matrix is inspected for a witness
- **THEN** the crate documents that it has none, and that the reason is a fixed element type rather than an omission
- **AND** it names the conversion to `DenseMatrix<Gf2>` as the route to the HKT surface

#### Scenario: The witness surface matches the existing one
- **WHEN** a new witness is compared against `CsrMatrixWitness`
- **THEN** it implements the same trait set, or documents at that impl site which trait it cannot support and why

#### Scenario: The moved witness keeps what it retained
- **WHEN** `CsrMatrixWitness` is compared against its behaviour before the move
- **THEN** the trait impls it retains produce identical results
- **AND** `Monad` and `Adjunction` are absent, with the law they broke recorded at the impl site

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
`fmap` over any of the crate's witnessed containers SHALL preserve the container's shape, and SHALL NOT change a matrix's dimensions or a vector's length.

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

