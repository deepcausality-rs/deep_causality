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
overridable method keeps both correct without weakening the bound. What an override may not do is
skip the search — see *Elimination selects a pivot by search, never by position* below, which sets
the floor every rule has to clear.

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

### Requirement: Elimination selects a pivot by search, never by position
Every elimination path SHALL choose its pivot by searching the column at or below the current row, and SHALL NOT assume the diagonal entry is usable.

This is load-bearing, not a quality nicety. Both Laplace determinants in `deep_causality_topology`
are fed Cayley-Menger matrices, which have `m[0][0] = 0` by construction
(`manifold/geometry/mod.rs:41` writes `one` only into indices `1..matrix_dim`). The existing
`gaussian_determinant` at `lazy_hodge_star.rs:97` takes `mat[i][i]` as the pivot and returns zero
when it is small, so consolidating onto it unpivoted returns **zero for every simplex volume**.
Measured on a regular unit tetrahedron: Laplace and pivoted elimination both give `det = 4.0` and
`vol = 0.117851130198` (exactly √2⁄12); unpivoted gives `det = 0.0` and a NaN volume. That helper is
correct today only because its own caller feeds it a Gram matrix with a positive diagonal.

#### Scenario: A matrix with a zero leading entry
- **WHEN** a determinant is evaluated on a matrix whose `(0,0)` entry is zero and which is non-singular
- **THEN** a non-zero determinant is returned

#### Scenario: Cayley-Menger volumes are preserved
- **WHEN** the 5×5 Cayley-Menger determinant of a regular unit tetrahedron is evaluated
- **THEN** the result matches the Laplace expansion it replaces
- **AND** the derived volume equals √2⁄12

#### Scenario: A genuinely singular matrix still reports zero
- **WHEN** a determinant is evaluated on a singular matrix
- **THEN** zero is returned

### Requirement: Determinant keeps closed forms for small matrices
The determinant SHALL use direct closed-form expressions for matrices of order three or below, and pivoted elimination above that.

At order three or below a closed form is faster than elimination and introduces no pivoting
round-off. `manifold/geometry/mod.rs:145` already special-cases orders zero, one and two, and
`deep_causality_physics` carries five fixed-size closed forms of its own for the same reason.

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

`matmul` alone has thirteen call sites inside the physics Kalman filter and eighteen across that
crate. A trait boundary in this position monomorphises, and the prototype measured a comparable seam at 0.92–0.95× of a
non-generic loop; the benchmarks confirm it rather than assume it.

#### Scenario: Benchmarks are compared across the move
- **WHEN** the tensor benchmarks run before and after the relocation on the same machine
- **THEN** no benchmark regresses beyond its measurement noise
- **AND** both figures are recorded with the machine they were taken on
