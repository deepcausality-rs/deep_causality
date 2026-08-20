## ADDED Requirements

### Requirement: Simplex vertex ordering is a documented contract
`Simplex::vertices()` MUST document that the returned slice is strictly increasing, and that
invariant MUST be enforced at every construction path. Chen & Tata (arXiv:2106.05274) §II state the
dependency directly: these constructions "require a branching structure on the triangulation in order
to determine local vertex orderings, whereas the boundary operators did not." The invariant already holds, since
`Simplex::new` sorts, and the struct doc already states it; what is missing is the statement on the
accessor that downstream code reads. A cup product that silently depends on an undocumented ordering
is a defect waiting for a refactor, and the existing MHD wedge kernel already depends on it.

#### Scenario: Accessor documents the ordering
- **WHEN** a reader consults the rustdoc for `Simplex::vertices()`
- **THEN** it states that the vertices are returned in strictly increasing index order, and that the
  order is the branching structure the cup product depends on

#### Scenario: Construction preserves the ordering
- **WHEN** a `Simplex` is built from vertices supplied in any order, including by `Simplex::new` and
  by internal face construction
- **THEN** `vertices()` returns them strictly increasing, with no repeats

### Requirement: Lattice cell corner ordering is a documented contract
`LatticeCell::vertices()` MUST document the order in which it enumerates a cell's `2^k` corners.
The enumeration is deterministic today (active axes ascending, corners by binary counter over the
active axes), and Serre's cubical cup product depends on exactly that order, so it MUST become a
stated guarantee rather than an implementation accident.

#### Scenario: Corner order is stated
- **WHEN** a reader consults the rustdoc for `LatticeCell::vertices()`
- **THEN** it states that active axes are taken in ascending index order and corners are enumerated
  by the binary counter over those axes, with corner `0` being the cell's base position

#### Scenario: Corner order is stable across cells and dimensions
- **WHEN** `vertices()` is called on cells of any dimension in any `LatticeComplex<D, R>`
- **THEN** corner `0` is the base position, and corner `i` offsets the base by `+1` along the `j`-th
  active axis exactly when bit `j` of `i` is set

### Requirement: Cell splitting is exposed as a trait separate from `Cell`
The crate MUST introduce a new trait for cell splitting rather than adding a required
method to `Cell`. `Cell` is public, has three in-crate implementors, and appears as a bound in over
one hundred non-test locations; a required method on it would break any external implementor. A
separate trait keeps the change additive and lets a complex family opt in.

Splitting rather than vertex listing is the shared abstraction, because a simplex's vertices are
`usize` indices while a lattice cell's are `[usize; D]` positions, so no common vertex type is
workable while a common splitting is.

#### Scenario: `Cell` is unchanged
- **WHEN** the change is applied
- **THEN** the `Cell` trait's required methods are exactly as before, and every existing
  `impl Cell for ...` and every `: Cell` bound compiles unmodified

#### Scenario: Splitting is opt-in
- **WHEN** a type implements `Cell` but not the splitting trait
- **THEN** it still compiles and is usable everywhere `Cell` is required, and is simply not eligible
  for the cup product

### Requirement: A splitting enumerates the paired cells with signs
The splitting trait MUST, given a cell and a left degree `p`, return every pair of cells the two
cochain factors are evaluated on, each with the sign the ordering induces. A splitting whose left
degree exceeds the cell's dimension MUST return no terms rather than an error, so a cup product of
mismatched degrees contributes zero rather than failing.

The pair MUST be named for its algebraic role (left factor, right factor) rather than for a geometric
one (front face, back face), because the geometry differs between families and a name true in one and
false in the other would mislead every implementor.

**Convention.** The left cell takes the leading directions from the cell's base position and the
right cell begins where the left one ends. This is the direct analogue of Alexander–Whitney, whose
left cell is the leading vertices `(0 → p)`, and it reproduces Chen & Tata Fig. 1 under `ℤ₂`
reduction. Their Fig. 4 and Definition 1 use the mirror convention, placing the offset on the left
factor instead. Both satisfy the Leibniz rule against this crate's coboundary operators and both give
identical cohomology pairings, so the choice is a convention; it is fixed here for parallelism with
the simplicial case and MUST be documented as a convention rather than as the only possibility.

#### Scenario: Simplex yields a single term
- **WHEN** the splitting is requested for a `(p+q)`-simplex at left degree `p`
- **THEN** exactly one term is returned, whose left cell is the first `p+1` vertices `(0 → p)` and
  whose right cell is the last `q+1` vertices `(p → p+q)`, sharing the vertex at position `p`, with
  sign `+1`, per Chen & Tata Eq. (5)

#### Scenario: Lattice cell yields one term per direction split
- **WHEN** the splitting is requested for a `k`-cell with active axis set `A`, base position `p`, at
  left degree `n`
- **THEN** `C(k, n)` terms are returned, one per subset `S_α ⊆ A` with `|S_α| = n`, where
  `S_β = A \ S_α`, the left cell is `{ position: p, directions: S_α }`, the right cell is
  `{ position: p + Σ_{j ∈ S_α} e_j, directions: S_β }` with positions wrapped on periodic axes, and
  the sign is `sgn(S_α ascending, then S_β ascending)`

#### Scenario: The two-dimensional case reduces to the published example
- **WHEN** the splitting is requested for a 2-cell with active axes `{x, y}` at left degree 1
- **THEN** the two terms are `+ (bottom x-edge, right y-edge)` and `− (left y-edge, top x-edge)`,
  which reduce mod 2 to `α₁(□₀₁)β₁(□₁₃) + α₁(□₀₂)β₁(□₂₃)` of Chen & Tata (arXiv:2106.05274) Fig. 1

#### Scenario: Degree beyond the cell is empty
- **WHEN** the splitting is requested at a left degree greater than the cell's dimension
- **THEN** no terms are returned and no error is raised

### Requirement: Splitting is implemented for both shipped complex families
The splitting trait MUST be implemented for `Simplex` and for `LatticeCell<D>`, because those carry
the simplicial and cubical complexes the crate ships and the cubical case is the one CSS and qLDPC
codes are built on. `HoneycombCell` MAY be left unimplemented.

#### Scenario: Simplicial complexes support the cup product
- **WHEN** a cup product is taken over a `SimplicialComplex<R>`
- **THEN** it resolves through the `Simplex` splitting implementation

#### Scenario: Cubical complexes support the cup product
- **WHEN** a cup product is taken over a `LatticeComplex<D, R>`, including a `square_torus`
- **THEN** it resolves through the `LatticeCell<D>` splitting implementation
