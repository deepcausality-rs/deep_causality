## ADDED Requirements

### Requirement: A mod-2 chain binds its bit vector to the chain group it lives in
`Gf2Chain<W>` SHALL identify its chain group by the pair `(degree, len)`, SHALL refuse any binary
operation whose operands disagree on either component, and SHALL report both disagreements through
one error that names the chain group.

`C_k = 𝔽₂^{n_k}` is fixed by `n_k` alone, and every operation the type offers — the sum, the
intersection, the pairing, the support enumerations, the weight — is an operation of that group.
Two complexes with the same cell count in degree `k` have the same `C_k`, so no further identity is
available to check and none is needed. The complex enters with the boundary operator, and there the
compatibility check is made against the complex being applied.

Today the degree is checked in the chain and the length in the packed vector beneath it, so two
halves of one condition surface as two different error types.

#### Scenario: Chains of different degree have no operation
- **WHEN** a 1-chain and a 2-chain of equal length are added, intersected or paired
- **THEN** each call returns an error naming both degrees

#### Scenario: Chains of different length have no operation
- **WHEN** two chains of equal degree and unequal length are added, intersected or paired
- **THEN** each call returns the same error variant as the degree mismatch, raised by the chain
  rather than by the packed vector beneath it

#### Scenario: The complex is checked where the boundary is applied
- **WHEN** a boundary operator drawn from a complex is applied to a chain
- **THEN** the chain's length is checked against that complex's cell count in the chain's degree, at
  that call

### Requirement: The support is enumerable as elements, pairs and triples
`Gf2Chain` SHALL expose `supp(γ)` in ascending order, together with the unordered pairs and the
unordered triples of that support, each strictly ascending within a tuple.

Every logical gate in Haruna Table 1 is a product of physical gates ranging over `supp(γ)`, its
pairs for the two-qubit factors, and its triples for the `CCZ` factors.

#### Scenario: Support is ascending and independent of word boundaries
- **WHEN** a chain of length 200 has its support set at indices spanning several machine words
- **THEN** `supp(γ)` yields exactly those indices in ascending order

#### Scenario: Tuple counts follow the binomial coefficients
- **WHEN** a chain has support of weight `w`
- **THEN** the pair count is `C(w, 2)` and the triple count is `C(w, 3)`

#### Scenario: Enumeration costs the weight, not the length
- **WHEN** a chain of length 10000 has weight 3
- **THEN** `supp(γ)` visits set bits rather than scanning every index

### Requirement: The mod-2 pairing and intersection are distinct operations
`Gf2Chain` SHALL provide the pairing `⟨γ₁, γ₂⟩ = Σᵢ γ₁ⁱγ₂ⁱ` returning a field element, and the
intersection `γ₁ ∩ γ₂` returning a chain, and the two SHALL NOT be conflated with the 𝔽₂ sum.

A support present in both operands survives the intersection and cancels in the sum. The pairing is
the parity of the intersection's weight.

#### Scenario: Sum cancels a shared support and intersection keeps it
- **WHEN** two chains share exactly the supports `{5, 70}` and differ elsewhere
- **THEN** the sum's support excludes 5 and 70, and the intersection's support is exactly `{5, 70}`

#### Scenario: The pairing agrees with its entrywise definition
- **WHEN** the pairing is computed word-parallel and also by summing `γ₁ⁱ · γ₂ⁱ` one index at a time
- **THEN** the two agree for every operand pair

#### Scenario: The pairing is the parity of the intersection
- **WHEN** two chains are paired and separately intersected
- **THEN** the pairing equals the parity of the intersection's weight

### Requirement: Padding bits are never counted as data
The packed representation SHALL keep every bit beyond the chain's length zero, and the weight and
pairing SHALL count only bits below that length.

Both operations count whole machine words, so a padding bit that was ever set would be read as a
coefficient.

#### Scenario: A fully set chain of non-word-multiple length reports its length
- **WHEN** every coefficient of a 130-bit chain is set to one
- **THEN** the weight is 130 rather than 192

#### Scenario: Padding stays zero through every mutation path
- **WHEN** a chain is built from a support, from a basis vector, and by setting entries one at a
  time, then cleared and set again
- **THEN** the bits above the length are zero after every operation

### Requirement: A basis vector becomes a chain without transposition
The crate SHALL provide construction of a chain from a basis vector of a `PackedGf2` matrix in the
orientation that `kernel_basis_gf2` and `image_basis_gf2` actually produce, which is **columns**.

`kernel_basis_gf2` allocates `zeros(cols, free.len())` and `image_basis_gf2` allocates
`zeros(rows, pivots.len())`; both write basis vectors down columns. The existing `from_row` is a
contiguous word-slice copy and cannot read a column, and four committed docstrings described the
bases as rows before this change. A caller following them received a vector of the wrong length.

#### Scenario: A kernel basis vector round-trips into a chain
- **WHEN** `kernel_basis_gf2` is called on a matrix with a known kernel and each basis vector is read
  into a chain
- **THEN** each chain has length equal to the matrix's column count, and multiplying it by the matrix
  gives zero

#### Scenario: The row path returns a differently-shaped vector
- **WHEN** a kernel basis is read with the row-oriented constructor instead of the column one
- **THEN** the chain's length is the number of basis vectors rather than the dimension they live
  in, so a caller detects it from `len()`, and the column constructor is the one the docstrings
  name for a basis

#### Scenario: The documentation names the orientation the code produces
- **WHEN** the docstrings of the basis functions and the chain constructors are read
- **THEN** each states columns, matching the allocation in the implementation
