## ADDED Requirements

### Requirement: A dense vector type exists alongside the matrices
`deep_causality_linear` SHALL provide a dense vector type carrying its length in its own type, usable without enabling a feature.

The rank census found **60 rank-1 `CausalTensor` constructions** across the consumer crates — more
than the 46 rank-2 ones. Every one of them is a vector expressed as a tensor that happens to have one
dimension, so every access pays a runtime rank check and every signature admits a matrix. A vector
type is not an ornament on the matrix work; it is the larger half of what the census found.

#### Scenario: The vector is reachable from the crate root
- **WHEN** the crate is imported with default features
- **THEN** the dense vector type is in the public surface

#### Scenario: A matrix is not a vector
- **WHEN** a function expecting a vector is offered a matrix
- **THEN** it fails to compile

#### Scenario: Length is carried by the type
- **WHEN** a consumer holds a vector
- **THEN** it needs no separate field recording the length

### Requirement: Vectors support the operations that make them useful
The vector SHALL provide element access, scaling by a scalar, addition and subtraction, the dot product, the outer product into a dense matrix, and conversion to and from a slice.

These are the operations the 60 census sites perform today by reaching into a rank-1 tensor's data.
The outer product is included because it produces a matrix, which is the one place the two types must
know about each other.

#### Scenario: Dot product agrees with the manual sum
- **WHEN** the dot product of two vectors is taken
- **THEN** it equals the sum of the elementwise products

#### Scenario: Outer product has the expected shape
- **WHEN** the outer product of an `m`-vector and an `n`-vector is taken
- **THEN** the result is an `m × n` dense matrix

#### Scenario: Length mismatch is rejected
- **WHEN** two vectors of different lengths are added, subtracted or dotted
- **THEN** the call fails with a typed error rather than truncating or panicking

#### Scenario: Round-trip through a slice
- **WHEN** a vector is built from a slice and read back
- **THEN** the values and the length are unchanged

### Requirement: The Hermitian inner product is distinct from the dot product
The crate SHALL provide a conjugating inner product bounded on `ConjugateScalar`, separate from the plain dot product.

Over ℂ the plain dot product is not an inner product — `⟨v, v⟩` is not real and not non-negative, so
it cannot induce a norm. `deep_causality_quantum` works in `Complex<R>` throughout, so a single
"dot" that silently does the wrong thing there would be a defect waiting on its first complex caller.

#### Scenario: Over the reals the two agree
- **WHEN** both products are taken on real vectors
- **THEN** they return the same value

#### Scenario: Over the complexes the inner product is conjugate-linear
- **WHEN** the Hermitian inner product of a complex vector with itself is taken
- **THEN** the result is real and non-negative
- **AND** it equals the square of the vector's 2-norm

### Requirement: Matrix–vector products exist for every matrix representation
Each of the three matrix representations SHALL multiply a vector, and a sparse matrix SHALL do so without densifying.

Matrix–vector is the operation sparse storage exists for. `deep_causality_physics` already relies on
it: `kernels/mhd/ideal.rs` and `kernels/mhd/grmhd.rs` hand-roll `apply_csr_real` and `apply_csr_i8`
against Hodge-star and coboundary operators precisely because a shared one was not available.

#### Scenario: Sparse matrix–vector does not densify
- **WHEN** a CSR matrix multiplies a vector
- **THEN** the work is proportional to the number of stored entries, not to rows × columns

#### Scenario: The three representations agree
- **WHEN** the same mathematical matrix is stored sparse, dense and bit-packed and each multiplies the same vector
- **THEN** all three produce the same result

#### Scenario: Dimension mismatch is rejected
- **WHEN** a matrix multiplies a vector whose length is not the matrix's column count
- **THEN** the call fails with a typed error

### Requirement: Norms are defined once, for vectors and matrices
The crate SHALL provide the 1-, 2- and ∞-norms for vectors and the 1-, ∞- and Frobenius norms for matrices, bounded on `NormedScalar`, each defined in exactly one place.

The workspace answers this question twice already — `CausalTensor::norm_l2` and `norm_sq`
(`tensor/src/types/causal_tensor/api/mod.rs:109,122`) and `frobenius_norm` in
`quantum/src/types/qgates/operator_linalg.rs:64`. `NormedScalar` is the right bound because
`modulus_squared` lands in an ordered `Real`, which makes the complex case work without a separate
surface.

#### Scenario: Norms are correct on a known vector
- **WHEN** the norms of `[3, -4]` are taken
- **THEN** the 1-norm is 7, the 2-norm is 5, and the ∞-norm is 4

#### Scenario: The complex 2-norm uses the modulus
- **WHEN** the 2-norm of a complex vector is taken
- **THEN** it equals the square root of the sum of `modulus_squared` over the entries

#### Scenario: The Frobenius norm agrees with the flattened 2-norm
- **WHEN** a matrix's Frobenius norm is taken
- **THEN** it equals the 2-norm of its entries read as one vector

#### Scenario: The zero vector
- **WHEN** the norms of a zero vector are taken
- **THEN** each is zero and none is `NaN`

### Requirement: Sparse vectors are out of scope
The crate SHALL NOT provide a sparse vector type in this change.

No measured call site needs one: the 60 census sites are dense, sparse matrix–vector produces a dense
result, and the conjugate-gradient solvers already work matrix-free against `&[R]`. Adding one
without a consumer would be the accretion this change exists to undo.

#### Scenario: The vector surface is dense only
- **WHEN** the crate's vector types are enumerated
- **THEN** exactly one is present, and it is dense
