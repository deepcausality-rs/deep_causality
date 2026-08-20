## Context

`deep_causality_topology` already carries the objects a CSS code is made of: `ChainComplex` with
`boundary_matrix`, `coboundary_matrix` and `betti_number`, over `SimplicialComplex`, `CellComplex`
and `LatticeComplex`, plus a lattice gauge module with link variables and Wilson loops. Running the
crate confirms that `LatticeComplex::<2, f64>::square_torus(L)` reproduces the toric code family
(`β₁ = 2`, `n = 2L²`, weight-4 checks) with no code-specific code written, and that
`cubic_torus(3)` returns the Betti vector `[1, 3, 3, 1]`.

`deep_causality_quantum` ships the Haruna logical gates, which consume gauge-field cochains. The
operation that connects the two, the cup product, is absent.

Three constraints shape everything below.

**The ordering is already there.** The roadmap recorded a global vertex ordering as the unbuilt
keystone. `Simplex::new` sorts its vertices and the struct doc states the invariant;
`Simplex::subsimplex` is public and is exactly the Alexander–Whitney extractor;
`LatticeCell<D> { position, orientation }` orders corners deterministically. The work is to document
and abstract, not to construct.

**Half the formula is already written, in the wrong crate.**
`deep_causality_physics::kernels::mhd::ideal::wedge_product_1form_1form` implements Alexander–Whitney
at degree `(1,1)`, privately, with one caller.

**Seven crates and five example packages depend on topology.** Nothing here may change an existing
signature.

## Goals / Non-Goals

**Goals:**

- A degree-general cup product over both shipped complex families, and an `n`-fold form.
- A splitting abstraction that unifies the simplicial and cubical formulas without a common vertex
  type.
- Correctness pinned by the Leibniz rule against the crate's own coboundary operators.
- A demonstrated logical `CZ` on a torus and a logical `CCZ` on a 3-torus, both verified to depend
  only on homology class.
- Zero blast radius.

**Non-Goals:**

- Higher cup products `∪ᵢ`, Steenrod squares, higher Pontryagin powers, addressability via
  higher-form symmetries, and the Betti-vector gate catalogue. Costed in Decision 6.
- Emitting fault-tolerant constant-depth physical decompositions. A cup product yields a logical
  action, not a circuit.
- Z_N or qudit coefficients. See Decision 5.
- A `Cochain` type. See Decision 2.

## Decisions

### Decision 1: A separate splitting trait, not a method on `Cell`

`Cell` is public, has three in-crate implementors (`Simplex`, `LatticeCell<D>`, `HoneycombCell`) and
appears as a bound in 132 non-test locations. A required method on it would break every external
implementor.

*Alternatives considered.* A required method on `Cell`: rejected, breaking. A defaulted method on
`Cell`: non-breaking, but a default that cannot be written meaningfully for an arbitrary cell would
either panic or return nothing, and a trait method that silently returns nothing is the worst kind of
gap in a correctness-critical path. A free function matching on concrete types: rejected, closes the
trait to external complexes for no benefit.

**Chosen:** a new trait taking `Cell` as a supertrait. Opt-in, additive, and a complex family that
has no sensible splitting simply does not implement it.

### Decision 2: No `Cochain` type

The repository already has a cochain convention: a flat `Vec<R>` or `CausalTensor<R>` indexed by cell
index within a skeleton. It is pervasive in `deep_causality_physics` (velocity one-forms, pressure
zero-forms, the `random_cochain` fixtures) and in `deep_causality_cfd` downstream.

*Alternatives considered.* A `Cochain<R>` newtype carrying its degree and complex: better type safety
in isolation, but every physics and CFD call site would need conversion, which is a cost paid by
existing code to benefit new code. That is the wrong direction.

**Chosen:** operate on slices plus the complex, matching `wedge_product_1form_1form`'s existing
signature style, and validate length against `num_cells(degree)` at the boundary.

### Decision 3: Splitting, not vertex listing, is the shared abstraction

A simplex's vertices are `usize` indices; a lattice cell's are `[usize; D]` positions. There is no
natural common vertex type, and inventing one would force allocation and translation on both sides.

But both formulas need the *same* thing at the level above vertices: given a cell and a left degree
`p`, enumerate the pairs of cells the two cochain factors are evaluated on, with a sign.
Alexander–Whitney returns exactly one such pair with sign `+1` (Chen & Tata Eq. 5). The cubical
formula returns `C(k, p)` pairs with sign `(−1)^{|S_β|} · sgn(S_α, S_β)` (Chen & Tata Eq. 26).

The pair is named for its algebraic role rather than a geometric one. Alexander–Whitney's left cell
is the leading vertices of the simplex, while the cubical left cell sits at the high corner and the
right cell at the base position. "Front face" is true simplicially and false cubically.

**Chosen:** one trait method returning the splittings. The cup product then reads identically for
both families: for each `(p+q)`-cell, sum `sign · α(front) · β(back)` over its splittings.

*Why genericity is a requirement and not a convenience.* Haruna's construction applies to general CSS
codes, with no manifold, product structure or locality requirement, and that generality is the reason
to build on it rather than on the geometric constructions. qLDPC codes, which is where encoding rate
forces the field, are products of chain complexes and expander constructions carrying arbitrary
structure. A cup product specialised to `LatticeComplex` would reproduce the toric code and reach
nothing past it, and would forfeit the property that makes a search over candidate codes possible at
all: a computable native gate set for a code nobody has studied. The implementation is therefore
written against `ChainComplex` and the splitting trait, and the tori are test cases rather than the
target.

### Decision 4: Relocate Alexander–Whitney, keep the wedge in physics

The cup product belongs where the complex lives. The antisymmetrisation
`α ∧ β = α ∪ β − β ∪ α` is a differential-geometry concern and stays in physics.

*Risk.* The relocated implementation feeds the MHD ideal-induction kernel, so it must be numerically
identical. The guard is the existing kernel test suite, run unmodified.

**Chosen:** topology gains the general implementation; physics's private helper becomes a thin caller
plus the antisymmetrisation.

### Decision 5: Scalars stay on `RealField`

The natural coefficient ring for CSS codes is `Z₂`, and for qudits `Z_N`. Bounding the cup product on
a ring would serve both. That is not available here: integer types were deliberately dropped from the
`deep_causality_algebra` hierarchy, and re-adding integer algebra impls is a standing project ruling.

*Alternatives considered.* Re-add integer impls: ruled out by project decision. A local
cup-product-scalar trait in topology: possible, but it would duplicate part of the algebra tower in a
crate that should not own one, for a benefit this change does not need. Generic over a semiring from
a new dependency: adds an external dependency to a crate that has none.

**Chosen:** generic over `RealField`, matching every other numeric surface in topology. `Z₂`
semantics are applied by the caller by reducing mod 2 at the boundary, which is exactly what the
existing MHD code already does when it lifts `i8` boundary entries through `R::from_f64`. Z_N support
is a follow-up, and is out of scope here regardless.

### Decision 6: Which of the Hsin–Kobayashi–Zhu line to include, and why

The HKZ gate line splits into two halves that were initially conflated. `dynamic-qcm.md` §3.2 states
them separately, and they have very different cost profiles.

| Tier | What | Machinery needed | Cost | Gain |
|---|---|---|---|---|
| A | `CZ` from a binary cup product | the cup product | this change | Clifford gate on a surface code; unblocks `geometric_qec` |
| B | the `n`-fold cup product those gates are built from | **none beyond A**, because the cup product is associative | an `n`-ary fold, its tests, and a 3-torus verification | reaches the degree where `CCZ` and `C^{n−1}Z` live, the more valuable half |
| C | Steenrod squares from higher cup products `∪ᵢ` | Steenrod's cup-`i` recursion, both families, with signs | comparable to all of A | prerequisite for D and E; little standalone value |
| D | `R_k`, `C^m R_k` from higher Pontryagin powers | C, plus HKZ App. B formulas on projective-space codes | larger than C | QFT, phase estimation and Shor compilation |
| E | the Betti-vector gate catalogue | C and D, plus addressability via higher-form symmetries | largest | the co-design search instrument |

**Chosen: A and B in, C through E out.**

Tier B was excluded in the first draft of this change and that was an error of costing. `CCZ` is a
triple cup product and `C^{n−1}Z` an `n`-fold one; the cup product is associative, so both are folds
of the binary operation. The marginal cost is an `n`-ary wrapper plus tests, and
`LatticeComplex::<3, R>::cubic_torus` already supplies the three-dimensional complex a triple product
needs. Against that, the gain is the step from Clifford to non-Clifford, which is where the whole
construction earns its keep. Excluding it would have shipped the cheap half and left the valuable
half behind a follow-up for no saving.

Tier C inverts that ratio. The cup-`i` recursion is materially more intricate than the plain product,
its signs are harder to pin, and its standalone value is nil: Steenrod squares matter because they
lead to D and E, and stopping at C delivers machinery with no consumer. A change that reached C
without D and E would be the worst of both, so C waits until it is taken together with them.

Two constraints apply to B and are written into the spec. First, the verification stays at **cochain
level**: `deep_causality_quantum` does not depend on `deep_causality_topology`, so a `CCZ` gate cannot
be built here at all, and the gate demonstration belongs to the `geometric_qec` example, which can
depend on both. Second, even there the cup product yields a logical *action* and its invariance;
HKZ's constant-depth fault-tolerant decompositions are circuits, and this change emits none.

## Risks / Trade-offs

**Cubical sign errors are invisible without a law test** → the Leibniz rule against the crate's own
`coboundary_matrix` is a required acceptance criterion, not an optional check. A wrong shuffle sign
still yields a well-formed cochain and would only surface later as a logical gate that acts
incorrectly, which is the most expensive place to find it.

**The MHD relocation could change numerics** → the existing ideal-induction kernel tests run
unmodified as the regression guard, and the relocation lands as its own task so a bisect isolates it.

**Associativity is assumed by the `n`-fold form** → it is asserted as a test on both families rather
than taken on faith, since a splitting implementation with an inconsistent sign convention could be
associative in one family and not the other.

**`RealField`-only coefficients defer the natural `Z₂` setting** → accepted. Callers reduce mod 2 at
the boundary, following the pattern the MHD kernel already uses for `i8` boundary entries. Revisit
only if qudit codes come into scope.

**A new trait is one more concept in a crate that already has several** → mitigated by keeping it
small, giving it one method, and making it opt-in so nothing that does not need cup products has to
know it exists.

## Migration Plan

None required. Every element is additive: a new trait, new free functions, and doc-only edits to two
accessors. No existing signature changes, so no downstream crate needs modification.

Rollback is deletion of the new module plus reverting the physics helper to its inlined form.

## Open Questions

- Should the splitting method return an allocated `Vec` or an iterator? A `Vec` is simpler and the
  counts are small (`C(k, p)` for `k` at most the complex dimension); an iterator avoids allocation
  in the inner loop of a cup product over a large lattice. Decide by measuring on a `square_torus`
  large enough to matter, and default to the simpler form until it does.
- Should `HoneycombCell` implement the splitting trait? It is the third `Cell` implementor. Left
  unimplemented unless a consumer appears.
- Does the cubical shuffle sign convention need to agree with the sign convention already baked into
  `LatticeComplex`'s boundary operators, or only be internally consistent? The Leibniz test answers
  this empirically; the answer should be written into the module documentation once known.
