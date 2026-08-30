# linear-matrix-representations Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: Three representations are available side by side
`deep_causality_linear` SHALL provide a compressed-sparse-row matrix, a dense row-major matrix, and a bit-packed 𝔽₂ matrix, each usable without enabling a feature.

Choosing a representation is the central decision in linear algebra. A caller with a boundary
operator wants CSR, a caller with a covariance matrix wants dense, and a caller doing mod-2
elimination wants one bit per entry. Splitting those across crates makes the choice an architectural
commitment rather than a local one.

#### Scenario: All three are reachable from the crate root
- **WHEN** the crate is imported with default features
- **THEN** the sparse, dense and bit-packed matrix types are all in the public surface

#### Scenario: Sparse behaviour is preserved
- **WHEN** code written against `deep_causality_sparse::CsrMatrix` is recompiled against the new path
- **THEN** it compiles unchanged
- **AND** its results are identical

### Requirement: Conversion between representations is explicit and total or fallible by construction
The crate SHALL provide conversions among its three representations, failing only where a target representation cannot hold the source's values.

Densifying a sparse matrix always succeeds and costs memory. Sparsifying a dense one always
succeeds. Packing into 𝔽₂ succeeds only if every entry is 0 or 1, so that conversion is fallible and
names the offending entry.

#### Scenario: Sparse to dense round-trips
- **WHEN** a sparse matrix is densified and then sparsified
- **THEN** the result equals the original, including its shape

#### Scenario: Packing rejects a non-binary entry
- **WHEN** a matrix containing an entry outside {0, 1} is packed into 𝔽₂
- **THEN** the conversion fails
- **AND** the error names the offending position

#### Scenario: Packing accepts the boundary-operator alphabet
- **WHEN** a matrix with entries in {-1, 0, 1} is reduced mod 2 and packed
- **THEN** the conversion succeeds
- **AND** -1 and 1 both map to the 𝔽₂ one

### Requirement: The bit-packed representation is generic over its word type
The 𝔽₂ matrix SHALL be generic over a word type bounded on `NaturalNumber`, and SHALL NOT fix the word width.

`deep_causality_num/src/integer/natural.rs` supplies the bit primitives the elimination needs. A
type parameter lets a caller pick the width that suits the target, and lets the crate be tested at a
narrow width where an edge case at a word boundary appears in a small matrix.

#### Scenario: Two widths agree
- **WHEN** the same matrix is packed at two different word widths and reduced
- **THEN** both report the same rank and the same pivot columns

#### Scenario: A column count that is not a multiple of the word width
- **WHEN** a matrix whose column count is not a multiple of the word width is reduced
- **THEN** the trailing padding bits do not affect the rank
- **AND** the result matches the same matrix reduced unpacked

### Requirement: Storage cost is proportional to the representation, not to the scalar
The bit-packed 𝔽₂ matrix SHALL store one bit per entry.

The alternative the algebra tower alone permits — a `Gf2` scalar satisfying `Field` — stores one byte
per entry. At n=2048 that is 4,096 KiB against 512 KiB, and the measured elimination is 3.2× slower.

#### Scenario: Allocation matches the bit count
- **WHEN** an `r`×`c` 𝔽₂ matrix is allocated over a `w`-bit word
- **THEN** it holds `r * ceil(c / w)` words

### Requirement: The dense type makes rank and squareness type-level
The dense matrix SHALL carry its two dimensions in its own type, and a square-matrix operation SHALL reject a non-square input without a separate dimension field being maintained by the caller.

The census found 46 rank-2 constructions across the consumer crates, and the crates holding them
maintain the invariant by hand: `DensityMatrix` stores `dim: usize` beside its tensor
(`quantum/src/types/density_matrix.rs:29-32`) because `CausalTensor` cannot express squareness, and
topology's `AdjacencyMatrix`, `IncidenceMatrix` and `LaplacianMatrix` are bare aliases of it. Physics,
quantum and topology together call 56 two-dimensional operations and zero N-d ones.

#### Scenario: A non-square input to a square-only operation
- **WHEN** a determinant, inverse or eigendecomposition is asked for on a non-square dense matrix
- **THEN** it fails without the caller having supplied a dimension

#### Scenario: The dimension is not duplicated
- **WHEN** a consumer holds a dense matrix
- **THEN** it needs no separate field recording the matrix's order

#### Scenario: Rank is not a runtime property
- **WHEN** a dense matrix is passed to a two-dimensional operation
- **THEN** no runtime check on the number of dimensions is required

### Requirement: Every representation reports its shape and its entries
The crate SHALL define a read trait exposing row count, column count and element access by value, and SHALL implement it for all three representations.

Access is by value rather than by reference: a bit-packed representation has no element to lend a
reference to. The scalars in this workspace are `Clone`, so nothing is lost.

#### Scenario: A sparse matrix answers for a structural zero
- **WHEN** an entry outside the stored pattern is read
- **THEN** the zero of the scalar type is returned

#### Scenario: Out-of-bounds access is rejected
- **WHEN** an index outside the matrix shape is read
- **THEN** the call fails rather than returning a value

