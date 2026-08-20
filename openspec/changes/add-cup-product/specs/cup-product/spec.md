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

### Requirement: Multi-controlled logical actions are demonstrated on a higher-dimensional complex
The change MUST demonstrate a `CCZ` logical action built from a triple cup product on a
three-dimensional complex, and MUST verify it depends only on the homology classes of its inputs.
`CZ` on a surface is Clifford; the triple product is where the construction reaches non-Clifford
territory, and a change that stopped at `CZ` would leave the more valuable half untested.

#### Scenario: CCZ is homology-class invariant
- **WHEN** a `CCZ` logical action is built from three 1-cochains on `cubic_torus(L)` and each input
  is varied by adding a coboundary
- **THEN** the resulting logical action is unchanged to within floating-point tolerance

#### Scenario: No fault-tolerance claim accompanies it
- **WHEN** the `CCZ` demonstration is documented
- **THEN** it states that a logical action is computed and verified, and that emitting a
  constant-depth fault-tolerant physical decomposition is out of scope

### Requirement: The cup product satisfies the Leibniz rule against the shipped coboundary
The implementation MUST satisfy `δ(α ∪ β) = δα ∪ β + (−1)^p · α ∪ δβ` when checked against the
crate's existing `coboundary_matrix`, for both complex families and across a range of degrees. This
is Chen & Tata (arXiv:2106.05274) Proposition 3; over `ℤ₂` the sign vanishes and it reduces to their
Proposition 1. This
is the acceptance criterion that matters most, because it is the only place a sign error in the
cubical splittings can hide: wrong signs still produce a well-formed cochain and only surface later
as a logical gate that acts incorrectly.

#### Scenario: Leibniz holds on a simplicial complex
- **WHEN** random cochains of degrees `p` and `q` are drawn over a simplicial complex and both sides
  of the Leibniz identity are computed using the complex's own coboundary operators
- **THEN** they agree to within floating-point tolerance for every degree pair the complex admits

#### Scenario: Leibniz holds on a torus
- **WHEN** the same check is run on `LatticeComplex::<2, f64>::square_torus(L)` and on
  `LatticeComplex::<3, f64>::cubic_torus(L)` for several `L`
- **THEN** they agree to within floating-point tolerance

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

### Requirement: The relocated Alexander–Whitney implementation preserves MHD results
`deep_causality_physics::kernels::mhd::ideal::wedge_product_1form_1form` MUST become a caller of the
topology cup product and MUST produce numerically identical results. It already implements the
Alexander–Whitney formula at the fixed degree pair `(1, 1)`; the general implementation belongs in
topology, where the complex lives. Physics keeps the antisymmetrisation that turns a cup product into
a wedge product, which is its own concern.

#### Scenario: Existing MHD kernel tests pass unchanged
- **WHEN** the ideal-induction kernel tests in `deep_causality_physics` are run after the relocation
- **THEN** they pass without modification to the tests or to their expected values

#### Scenario: No physics public surface changes
- **WHEN** the relocation is applied
- **THEN** `deep_causality_physics` exposes the same public items as before, since the relocated
  function is private with a single caller

### Requirement: A logical gate built from the cup product acts on the homology class
The change MUST be demonstrated end to end on a toric code: a logical `CZ` built from the cup product
MUST give the same logical action for every representative cycle of a homology class. This is the
downstream obligation the whole change exists to serve, and it is the same invariance the
`geometric_qec` example checks.

#### Scenario: Representatives agree
- **WHEN** a logical operator's cycle `γ` on `LatticeComplex::<2, f64>::square_torus(L)` is varied by
  adding face boundaries, producing several representatives of one class, and a cup-product `CZ` is
  formed from each
- **THEN** the resulting logical actions agree to within floating-point tolerance

#### Scenario: Distinct classes differ
- **WHEN** the two independent homology classes of the torus are used
- **THEN** the resulting logical actions differ, confirming the check has discriminating power rather
  than passing trivially
