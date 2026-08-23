## ADDED Requirements

### Requirement: Elimination is written once against a row-operation trait
The crate SHALL express RREF, rank, kernel basis, image basis, determinant and linear solve as generic algorithms over a row-operation trait, with no algorithm naming a concrete representation, a concrete scalar or a word width.

The inner loop of every elimination is a row update. Putting that behind a trait method lets a
representation that can update a whole row at once do so, while the driver never sees individual
elements in the hot path.

#### Scenario: One algorithm serves several representations
- **WHEN** the same matrix is reduced through the dense and the bit-packed implementations
- **THEN** both report the same rank and the same pivot columns

#### Scenario: The seam does not degrade to per-element access
- **WHEN** the generic elimination runs over the bit-packed representation
- **THEN** its runtime is within 10% of a hand-written non-generic elimination over the same packed words

### Requirement: The pivot rule is chosen by the representation
The row-operation trait SHALL let an implementation supply its own pivot selection, defaulting to the first non-zero entry at or below the current row.

An exact field takes the first non-zero. Floating point takes the largest magnitude, and a `Field`
bound supplies neither an order nor an epsilon with which to express that. Making the rule an
overridable method keeps both correct without weakening the bound.

#### Scenario: A floating-point implementation pivots on magnitude
- **WHEN** a dense f64 matrix whose first candidate pivot is near zero is reduced
- **THEN** a larger-magnitude pivot below it is selected
- **AND** the reported rank matches the exact rank of the matrix

#### Scenario: An exact implementation pivots on the first non-zero
- **WHEN** an 𝔽₂ matrix is reduced
- **THEN** the first non-zero entry at or below the current row is the pivot
- **AND** no tolerance participates in the decision

### Requirement: The row-operation trait is implemented only by dense-layout representations
The crate SHALL implement the row-operation trait for the dense and bit-packed representations, and SHALL NOT implement it for the compressed-sparse-row representation.

Adding a multiple of one CSR row to another changes that row's non-zero pattern, which means
reallocating every row after it. Sparse elimination needs a fill-reducing ordering and a symbolic
factorisation — a different algorithm, not a different implementation of this one.

#### Scenario: A sparse matrix reaches elimination by conversion
- **WHEN** a caller eliminates on a sparse matrix
- **THEN** the conversion to a dense-layout representation is explicit at the call site

#### Scenario: The read side still covers sparse
- **WHEN** a sparse matrix's shape and entries are read through the read trait
- **THEN** it works without conversion

### Requirement: Determinant keeps closed forms for small matrices
The determinant SHALL use direct closed-form expressions for matrices of order three or below, and elimination above that.

Regge geometry evaluates Cayley-Menger determinants on small simplices, where the closed forms are
both faster and free of the pivoting round-off that elimination introduces. `manifold/geometry/mod.rs`
already special-cases orders zero, one and two for the same reason.

#### Scenario: A 3×3 determinant is unchanged
- **WHEN** a 3×3 determinant is evaluated through the new implementation
- **THEN** the result is bit-identical to the closed-form expression it replaced

#### Scenario: A larger determinant uses elimination
- **WHEN** a 6×6 determinant is evaluated
- **THEN** the cost is cubic in the order rather than factorial

### Requirement: The decompositions move without changing the tensor surface
The bodies of `svd`, `svd_decomp`, `svd_truncated`, `qr`, `eigen` and `inverse` SHALL move into `deep_causality_linear`, and `CausalTensor` SHALL keep the corresponding methods, delegating to them.

Those methods are inherent on `CausalTensor` and members of the `Tensor` trait. Eight in-workspace
crates and seven example crates call them. Moving the bodies gives the algorithms a home a dense
matrix can use directly; keeping the methods means no caller changes.

#### Scenario: Existing callers compile unchanged
- **WHEN** a crate calling `CausalTensor::svd` is rebuilt after the move
- **THEN** it compiles with no edit
- **AND** the returned factors are unchanged

#### Scenario: The error type is preserved
- **WHEN** a delegated method fails
- **THEN** it returns the same error type and variant it returned before the move

#### Scenario: A dense matrix uses the same implementation
- **WHEN** a decomposition is applied to the crate's dense matrix
- **THEN** it runs the same code as the tensor path, with no rank-2 tensor constructed

### Requirement: Delegation does not cost measurable throughput
The delegated methods SHALL show no throughput regression against the benchmarks recorded before the move.

`matmul` alone has fifteen call sites inside the physics Kalman filter. A trait boundary in this
position monomorphises, and the prototype measured a comparable seam at 0.92–0.95× of a
non-generic loop; the benchmarks confirm it rather than assume it.

#### Scenario: Benchmarks are compared across the move
- **WHEN** the tensor benchmarks run before and after the relocation on the same machine
- **THEN** no benchmark regresses beyond its measurement noise
- **AND** both figures are recorded with the machine they were taken on
