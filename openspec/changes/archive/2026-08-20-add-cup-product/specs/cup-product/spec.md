## ADDED Requirements

### Requirement: Cochains are carried in the existing representation
The cup product MUST operate on the repository's established cochain representation, a flat slice of
scalars indexed by cell index within a skeleton, and MUST NOT introduce a dedicated `Cochain` type.
That representation is already pervasive: `deep_causality_physics` carries velocity one-forms,
pressure zero-forms and test fixtures as bare `Vec<R>` or `CausalTensor<R>` over cell indices. A new
wrapper type would fight the convention and force conversions across the physics and CFD stack for
no gain.

#### Scenario: A physics cochain is accepted directly
- **WHEN** a caller passes an existing edge one-form, held as a slice indexed by edge index, to the
  cup product
- **THEN** it is accepted without conversion or wrapping

#### Scenario: Length is validated against the complex
- **WHEN** a cochain slice's length does not equal the number of cells of its stated degree
- **THEN** the call returns a typed error naming the expected and actual lengths

### Requirement: The cup product is degree-general over splittable complexes
The crate MUST provide a cup product that takes a `p`-cochain and a `q`-cochain and returns a
`(p+q)`-cochain, generic over any complex whose cells implement the splitting trait. It MUST NOT be
fixed to a particular degree pair. The value on a `(p+q)`-cell is the signed sum over that cell's
splittings of the left cell's `α` value times the right cell's `β` value, each carrying the
splitting's sign.

#### Scenario: Simplicial case reproduces Alexander–Whitney
- **WHEN** the cup product is taken of two 1-cochains on a simplicial complex
- **THEN** the value on the 2-simplex `[v₀, v₁, v₂]` equals `α([v₀, v₁]) · β([v₁, v₂])`

#### Scenario: Cubical case reduces to the published two-dimensional formula
- **WHEN** the cup product is taken of two 1-cochains on a `LatticeComplex<2, R>`
- **THEN** the value on each face is `α(bottom x-edge)·β(right y-edge) − α(left y-edge)·β(top x-edge)`,
  which reduces mod 2 to `α(□₀₁)β(□₁₃) + α(□₀₂)β(□₂₃)` of Chen & Tata (arXiv:2106.05274) Fig. 1

#### Scenario: The cubical sign is the shuffle sign against this crate's coboundary
- **WHEN** a term is formed from a split `A = S_α ⊔ S_β`
- **THEN** its sign is `sgn(S_α ascending, then S_β ascending)`, with no additional degree-dependent
  factor. Chen & Tata Eq. (26) carries a `(−1)^{|S_β|}` prefactor stated relative to their own
  coboundary convention (their Eq. 22–23); against this crate's `coboundary_matrix` the
  Leibniz-compatible sign is the plain shuffle sign, which is why Leibniz against the shipped
  operators is the acceptance criterion rather than a transcribed formula

#### Scenario: Degree sum exceeding the complex is rejected
- **WHEN** `p + q` exceeds the complex's maximum cell dimension
- **THEN** the call returns a typed error rather than an empty result, since the caller has asked
  for a cochain in a degree the complex does not have

### Requirement: Genericity over complex families is executed, not asserted
The cup product MUST be implemented against `ChainComplex` and the splitting trait rather than
against any concrete complex, and MUST be exercised on at least one non-lattice complex. Haruna's
construction applies to general CSS codes, and that generality is why it is the construction being
built on; qLDPC codes carry arbitrary structure and no geometry. A test suite covering only lattice
complexes would exercise one implementor and leave the property untested.

#### Scenario: Both shipped families are covered by the same code path
- **WHEN** the cup product is taken over a `SimplicialComplex<R>` and over a `LatticeComplex<D, R>`
- **THEN** both resolve through the same generic implementation, with no complex-specific branch

#### Scenario: A hand-built simplicial complex is tested
- **WHEN** the law tests are run
- **THEN** at least one runs on a `SimplicialComplex` constructed by hand rather than on a torus, so
  genericity is executed rather than claimed

### Requirement: The cup product is associative and supports an n-fold form
The cup product MUST be associative, `(α ∪ β) ∪ γ = α ∪ (β ∪ γ)`, and the crate MUST expose an
`n`-fold form that folds the binary product over a slice of cochains. Associativity is what makes the
`n`-fold form free of new machinery, and the `n`-fold form is what yields `CCZ` from a triple product
and `C^{n−1}Z` from an `n`-fold product, which is the multi-controlled family
(`openspec/notes/quantum/dynamic-qcm.md` §3.2).

#### Scenario: Associativity holds on both families
- **WHEN** three cochains of degrees summing to at most the complex's maximum dimension are combined
  in both bracketings, on a simplicial complex and on a lattice complex
- **THEN** the two results agree exactly for integer-valued inputs, and to floating-point tolerance
  otherwise

#### Scenario: The n-fold form agrees with repeated binary application
- **WHEN** the `n`-fold product is taken over a slice of cochains
- **THEN** the result equals the left fold of the binary product over the same slice

#### Scenario: A triple product needs a three-dimensional complex
- **WHEN** a triple product of 1-cochains is requested on `LatticeComplex::<3, f64>::cubic_torus(L)`
- **THEN** it yields a 3-cochain, and the same request on a two-dimensional complex returns the
  degree-exceeds-dimension error

#### Scenario: The n-fold form of an empty or single slice
- **WHEN** the `n`-fold product is given a single cochain
- **THEN** it returns that cochain unchanged, and an empty slice returns a typed error rather than a
  silent unit

### Requirement: The triple product is verified at cochain level on a three-dimensional complex
The `n`-fold product MUST be verified by a triple product on a three-dimensional complex, checking
that it is nonzero on the generators and that its cohomology class is unchanged when any input is
shifted by a coboundary. A binary product on a surface is Clifford territory; the triple product is
where the construction reaches the non-Clifford gates, so a change that verified only the binary case
would leave the more valuable half untested.

This requirement is stated at **cochain level and no higher**. Building a `CCZ` logical action is a
`deep_causality_quantum` concern, and that crate does not depend on `deep_causality_topology`, so the
gate demonstration belongs to the `geometric_qec` example, which can depend on both. This change
delivers the operation such a gate is built from, and claims nothing about the gate.

#### Scenario: The triple product of the three generators is nonzero
- **WHEN** `∫ e₀ ∪ e₁ ∪ e₂` is summed over the 3-cells of `cubic_torus(L)`, where `e_d` is the
  1-cochain equal to 1 on every edge in direction `d`
- **THEN** it equals `L³`, the pairing of the three fundamental cycles

#### Scenario: The class is invariant under a coboundary shift
- **WHEN** any one input is replaced by `α + δf` for an arbitrary 0-cochain `f`, the others being
  cocycles
- **THEN** the summed triple product is unchanged, which follows from Leibniz and is the cochain-level
  form of the homology-class invariance a logical gate needs

#### Scenario: The inputs are verified to be cocycles first
- **WHEN** the triple-product test runs
- **THEN** it asserts `δe₀ = δe₁ = δe₂ = 0` before pairing, so the result is a statement about
  cohomology classes

### Requirement: The cohomological pairing on a torus matches its intersection number
The cup product MUST reproduce the known `H¹ × H¹ → H²` pairing on a 2-torus. Taking `α_x` as the
1-cochain that is 1 on every x-directed edge and 0 elsewhere, and `α_y` likewise for y, both are
cocycles representing the two generators of `H¹(T²)` scaled by `L`, and their pairing is the
intersection number of the two fundamental cycles. This is ground truth independent of any sign
convention and is the check that the operation is topologically correct rather than merely
Leibniz-consistent.

#### Scenario: Distinct generators pair to the intersection number
- **WHEN** `∫ α_x ∪ α_y` is summed over all faces of `square_torus(L)`
- **THEN** it equals `+L²`, and `∫ α_y ∪ α_x` equals `−L²`

#### Scenario: A generator pairs to zero with itself
- **WHEN** `∫ α_x ∪ α_x` is summed over all faces
- **THEN** it is zero, since a cycle has no self-intersection on the torus

#### Scenario: The inputs are verified to be cocycles first
- **WHEN** the pairing test runs
- **THEN** it first asserts `δα_x = 0` and `δα_y = 0`, so the pairing is a statement about cohomology
  classes rather than about arbitrary cochains

### Requirement: The cup product is graded-commutative up to a coboundary
The implementation MUST satisfy `α ∪ β − (−1)^{pq} · β ∪ α = δ(something)` on cocycles, that is, the
two orderings MUST agree on cohomology even though they differ as cochains. This is the property that
makes a logical gate depend on the homology class rather than on the representative cycle, which is
the correctness criterion Haruna proves for the gauge-field gates.

#### Scenario: Cocycles commute on cohomology
- **WHEN** `α` and `β` are cocycles and both orderings of the cup product are computed
- **THEN** their difference, after the appropriate sign, is a coboundary, verified by checking that
  it pairs to zero against every cycle

#### Scenario: Cochains do not commute in general
- **WHEN** `α` and `β` are arbitrary cochains rather than cocycles
- **THEN** the two orderings are permitted to differ, and no test asserts they are equal

### Requirement: The binary product's class is independent of the representative
The binary cup product MUST give the same cohomology class when an input cocycle is replaced by
another representative of the same class, that is, when it is shifted by a coboundary. This is the
two-dimensional counterpart of the triple-product requirement above and follows from Leibniz; it is
stated separately because it is the property a logical gate built on this operation would rely on.

Like that requirement this is stated at **cochain level**. A gate demonstration needs
`deep_causality_quantum`, which does not depend on this crate.

#### Scenario: Representatives of one class agree
- **WHEN** `∫ α ∪ β` is summed over the faces of `square_torus(L)` and `α` is replaced by `α + δf`
  for an arbitrary 0-cochain `f`, with `β` a cocycle
- **THEN** the sum is unchanged

#### Scenario: Distinct classes are distinguished
- **WHEN** the pairing is taken of two distinct generators, and of a generator with itself
- **THEN** the first is nonzero and the second is zero, so the operation separates classes rather
  than collapsing them
