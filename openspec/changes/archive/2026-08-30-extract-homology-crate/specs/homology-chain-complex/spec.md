## ADDED Requirements

### Requirement: The chain-complex trait names no geometry
The `ChainComplex` trait SHALL expose only the graded chain groups and their boundary and coboundary
operators, and SHALL NOT name cells, cell iterators, metrics, or lattice layout. Its required items
are `num_cells(k)`, `max_dim()`, `boundary_matrix(k)` and `coboundary_matrix(k)`.

A quantum error-correcting code is a chain complex that is not a space: `H_X` and `H_Z` are
parity-check matrices with no cells and no metric. A trait that named cells could not describe one.

#### Scenario: A complex with no geometry implements the trait
- **WHEN** a type is built from a pair of `CsrMatrix<i8>` parity-check matrices with no cell type,
  no metric and no cell iterator
- **THEN** it implements `ChainComplex` with no unimplementable associated type

#### Scenario: The crate compiles without a topology dependency
- **WHEN** `deep_causality_homology` is built
- **THEN** its dependency set contains `deep_causality_linear`, `deep_causality_algebra` and
  `deep_causality_num`, and does not contain `deep_causality_topology`

### Requirement: Boundary operators carry incidence numbers as `i8`
`boundary_matrix` and `coboundary_matrix` SHALL return `Cow<'_, CsrMatrix<i8>>`, and the trait SHALL
NOT take a coefficient-ring parameter.

The entries are incidence numbers, which for a cell complex lie in `{-1, 0, 1}` by construction.
That is an invariant of the boundary operator rather than a storage choice. The coefficient field is
a property of the computation, and it is carried by `HomologyField` at the call site.

#### Scenario: Every entry of a boundary matrix is an incidence number
- **WHEN** `boundary_matrix(k)` is called on any implementor at any grade
- **THEN** every stored entry is `-1`, `0` or `1`

#### Scenario: Borrowing is preserved for cached operators
- **WHEN** an implementor caches its boundary matrices
- **THEN** `boundary_matrix(k)` returns `Cow::Borrowed` and performs no allocation

### Requirement: The differential composes to zero
The trait SHALL state `∂ₖ ∘ ∂ₖ₊₁ = 0` as its defining law, and the conformance harness SHALL assert
it for every implementor at every grade where both operators are non-degenerate.

Without this the trait admits a family of matrices that is not a chain complex, and every homology
computed from it is meaningless rather than merely wrong.

#### Scenario: The law holds on every shipped implementor
- **WHEN** the conformance harness runs against `SimplicialComplex`, `CellComplex` and
  `LatticeComplex`
- **THEN** the product `∂ₖ · ∂ₖ₊₁` is the zero matrix at every grade `k` in `1..max_dim`

#### Scenario: The product is formed on widened coefficients
- **WHEN** the harness multiplies two boundary matrices
- **THEN** the entries are widened beyond `i8` before multiplication, so the assertion does not run
  on wrapping arithmetic in a release build

#### Scenario: A violating complex is rejected by the harness
- **WHEN** a deliberately malformed complex whose `∂₁ ∘ ∂₂` is non-zero is passed to the harness
- **THEN** the harness fails and names the grade

### Requirement: Degenerate grades carry the shape their dimension implies
`boundary_matrix(k)` SHALL return a matrix of shape `(num_cells(k-1), num_cells(k))` at every grade,
including the ends of the range, rather than the empty `(0, 0)` matrix.

`∂₀` therefore has shape `(0, num_cells(0))` and `∂_{max+1}` has shape `(num_cells(max), 0)`.
Rank–nullity survives the `(0, 0)` shape because `n_k.saturating_sub(rank_k)` recovers the answer; a
kernel basis does not, and homology with representatives will need one.

#### Scenario: The zeroth boundary has full column count
- **WHEN** `boundary_matrix(0)` is called on a complex with `n` vertices
- **THEN** the result has shape `(0, n)`

#### Scenario: Above the top grade the operator has full row count
- **WHEN** `boundary_matrix(max_dim() + 1)` is called
- **THEN** the result has shape `(num_cells(max_dim()), 0)`

#### Scenario: Betti numbers are unchanged by the shape fix
- **WHEN** `betti_number_over(k, field)` is computed at every grade before and after this change
- **THEN** the two agree for every shipped complex over both fields

### Requirement: Homology is computed over a chosen coefficient field
`HomologyField` SHALL offer `Rational` and `Gf2`, and `ChainComplex::betti_number_over(k, field)`
SHALL compute `dim ker ∂ₖ − rank ∂ₖ₊₁` over the named field with no tolerance anywhere on the path.

Rank over ℚ and rank over 𝔽₂ are different numbers for a complex with 2-torsion, and a caller that
cannot say which it wants gets one of them by accident.

#### Scenario: The two fields disagree on a complex with torsion
- **WHEN** the chain complex `ℤ --·2--> ℤ` is evaluated at grades 0 and 1
- **THEN** `Rational` yields `0` and `0`, and `Gf2` yields `1` and `1`

#### Scenario: Neither path uses a floating-point threshold
- **WHEN** either arm of `HomologyField::rank_of` runs
- **THEN** no floating-point value and no tolerance constant is read

#### Scenario: Overflow is reported rather than wrapped
- **WHEN** the `Rational` arm's fraction-free elimination exceeds the integer range
- **THEN** it returns an error naming the overflow rather than a wrapped rank
