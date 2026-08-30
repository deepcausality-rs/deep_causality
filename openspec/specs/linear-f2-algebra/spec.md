# linear-f2-algebra Specification

## Purpose
TBD - created by archiving change add-linear-algebra-crate. Update Purpose after archive.
## Requirements
### Requirement: Mod-2 elimination returns rank, kernel basis and image basis
`deep_causality_linear` SHALL provide Gaussian elimination over 𝔽₂ returning the rank, a basis of the kernel and a basis of the image.

`openspec/notes/quantum/qcl-gaps.md` G-01 names these three outputs as the closure condition, and R1,
R4 and R6 depend on them: a chain complex over 𝔽₂ needs `∂₁∂₂ = 0` checked exactly, and 𝔽₂ homology
with representatives needs `ker ∂₁ / im ∂₂` as spanning sets rather than as dimensions.

#### Scenario: Rank of a known matrix
- **WHEN** an 𝔽₂ matrix of known rank is reduced
- **THEN** the reported rank equals it exactly

#### Scenario: Kernel vectors are annihilated
- **WHEN** the kernel basis of `M` is computed
- **THEN** `M · v = 0` over 𝔽₂ for every returned `v`
- **AND** the basis has exactly `cols − rank` elements

#### Scenario: Image vectors span the column space
- **WHEN** the image basis of `M` is computed
- **THEN** it has exactly `rank` elements
- **AND** every column of `M` is an 𝔽₂ sum of them

#### Scenario: The zero matrix
- **WHEN** an all-zero matrix is reduced
- **THEN** the rank is zero, the image basis is empty, and the kernel basis has `cols` elements

### Requirement: Mod-2 results carry no tolerance
The 𝔽₂ elimination SHALL be exact, and SHALL NOT accept, expose or apply a numerical tolerance.

Rank over ℝ is not rank over 𝔽₂. `qcl-gaps.md` G-02 records that the two agree for the toric code
and diverge for a complex with even-weight dependencies, where the reported `k` would be wrong with
no error raised. Exactness is what removes that failure mode.

#### Scenario: No tolerance parameter exists
- **WHEN** the 𝔽₂ elimination surface is enumerated
- **THEN** no function accepts a tolerance, an epsilon or a threshold

#### Scenario: A matrix where the two ranks differ
- **WHEN** an integer matrix whose ℝ-rank exceeds its 𝔽₂-rank is reduced over 𝔽₂
- **THEN** the smaller 𝔽₂ rank is reported

### Requirement: Elimination updates whole words rather than single bits
The 𝔽₂ row update SHALL combine rows one machine word at a time.

This is what makes packing worth its complexity. Measured ad hoc at `--release` on an M3 Max with 16
cores and 128 GB, packed `u64` against a `Field`-satisfying byte scalar runs 1.7× faster at n=128,
1.9× at n=512, 2.4× at n=1024 and 3.2× at n=2048, on one eighth the memory; the ratio grows with n as
cache pressure does. The crate carries no bench target, so those numbers are the rationale for
packing rather than a gate the suite can decide. What the suite decides is the property.

#### Scenario: The row update runs a word at a time
- **WHEN** one row is combined into another over 𝔽₂
- **THEN** the update reads and writes whole `W`-sized words, from the pivot column's word to the end of the row
- **AND** no path walks the row one bit at a time

#### Scenario: The packing is one bit per entry
- **WHEN** the packed representation is compared against one byte per entry
- **THEN** it stores one bit per entry, `W::BITS` entries to a word
- **AND** that is the eighth of the memory the rationale records

### Requirement: Homology ranks are computed over 𝔽₂ rather than by thresholded SVD
`deep_causality_topology` SHALL compute boundary-matrix ranks for complexes read as codes through the exact 𝔽₂ elimination, and SHALL NOT reach that answer by counting floating-point singular values above a tolerance.

The crate used to reach these ranks by densifying a `CsrMatrix<i8>` into floating point, calling
`svd()` and counting the singular values above a tolerance, which left every Betti number it reported
depending on that threshold. The rank is now `HomologyField::rank_of`
(`deep_causality_unified_math/deep_causality_homology/src/types/homology_field/mod.rs:55`), one helper over two exact fields:
fraction-free elimination over ℤ for `Rational`, packed mod-2 elimination for `Gf2`. Neither rounds.
`deep_causality_topology` re-exports it (`src/types/homology_field/mod.rs:25`), so the field is a
call-site choice.

#### Scenario: The duplicated helpers are replaced by one
- **WHEN** the topology crate is searched for rank helpers
- **THEN** one implementation remains
- **AND** it does not construct a floating-point tensor

#### Scenario: Existing complexes report unchanged Betti numbers
- **WHEN** the topology test suite runs against the exact rank
- **THEN** every complex currently under test reports the Betti numbers it reported before

#### Scenario: The choice of field is visible at the call site
- **WHEN** a caller computes a Betti number
- **THEN** whether the rank is taken over ℝ or over 𝔽₂ is determined by the call, not by a global default

### Requirement: The 𝔽₂ matrix is separable from the chain-complex objects built on it
The 𝔽₂ matrix and its elimination SHALL be usable by a crate that needs mod-2 rank without needing chain complexes.

G-01 assigns this to `deep_causality_topology` so that topology need not learn about codes. This
requirement weakens that to a separability property rather than a placement, because the placement
argument that would have justified moving it does not hold: `qcl-gaps.md` records G-07 and G-09
(quantum) as needing G-04 (homology representatives) and G-05 (the `Chain` type), both owned by
`deep_causality_topology`. Quantum therefore takes a topology dependency for the 𝔽₂ work whichever
crate owns the matrix, so moving the matrix removes no dependency edge.

#### Scenario: The gap register is updated
- **WHEN** this requirement is implemented
- **THEN** G-01 and G-02 in `openspec/notes/quantum/qcl-gaps.md` are marked closed
- **AND** the owner field records the implementing crate

