## ADDED Requirements

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

This is what makes packing worth its complexity. Measured on an M3 Max at `--release`, packed `u64`
against a `Field`-satisfying byte scalar runs 1.7× faster at n=128, 1.9× at n=512, 2.4× at n=1024 and
3.2× at n=2048, on one eighth the memory; the ratio grows with n as cache pressure does.

#### Scenario: Packed elimination outruns the byte-scalar alternative
- **WHEN** a 1024×1024 𝔽₂ matrix is reduced through both the packed and a one-byte-per-entry representation
- **THEN** the packed reduction is at least twice as fast

#### Scenario: The advantage grows with size
- **WHEN** the comparison is repeated at 2048×2048
- **THEN** the ratio is no smaller than at 1024×1024

### Requirement: Homology ranks are computed over 𝔽₂ rather than by thresholded SVD
`deep_causality_topology` SHALL compute boundary-matrix ranks for complexes read as codes through the exact 𝔽₂ elimination, and SHALL NOT reach that answer by counting floating-point singular values above a tolerance.

`chain_complex_impl.rs:94` and `cell_complex/mod.rs:172` densify a `CsrMatrix<i8>` into `Vec<f64>`,
call `svd()`, and count singular values above `1e-5`. Betti numbers are derived from those ranks, so
every homology dimension the crate reports currently depends on that threshold.

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

### Requirement: The 𝔽₂ layer is owned by the linear crate
The 𝔽₂ matrix and its elimination SHALL live in `deep_causality_linear`, and SHALL be usable without depending on `deep_causality_topology`.

G-01 assigns this to `deep_causality_topology` so that topology need not learn about codes. The same
reasoning places it better in a linear-algebra crate, which knows about neither chain complexes nor
codes, and lets `deep_causality_quantum` use it without taking a dependency on topology.

#### Scenario: Quantum reaches 𝔽₂ without topology
- **WHEN** a crate needs mod-2 rank and does not need chain complexes
- **THEN** it depends on `deep_causality_linear` alone

#### Scenario: The gap register is updated
- **WHEN** this requirement is implemented
- **THEN** G-01 and G-02 in `openspec/notes/quantum/qcl-gaps.md` are marked closed
- **AND** the owner field recorded there is corrected to name the implementing crate
