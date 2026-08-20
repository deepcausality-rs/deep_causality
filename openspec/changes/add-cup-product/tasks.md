## 1. Document the ordering invariants

Doc-only, zero risk, and it makes the invariant the rest of the change depends on explicit.

- [ ] 1.1 Add rustdoc to `Simplex::vertices()` in `deep_causality_topology/src/types/simplex/getters/mod.rs` stating that the returned slice is strictly increasing, that no repeats occur, and that this order is the branching structure the cup product depends on
- [ ] 1.2 Add rustdoc to `LatticeCell::vertices()` in `deep_causality_topology/src/types/lattice_complex/lattice_cell.rs` stating the corner enumeration order: active axes ascending, corners by binary counter over active axes, corner `0` at the base position
- [ ] 1.3 Add tests asserting both invariants: sortedness across every `Simplex` construction path, and the corner-order contract across cell dimensions in `LatticeComplex<2>` and `LatticeComplex<3>`
- [ ] 1.4 Register the new test files in the corresponding `tests/**/mod.rs` and in `deep_causality_topology/tests/BUILD.bazel`

## 2. The splitting trait

- [ ] 2.1 Add the splitting trait in `deep_causality_topology/src/traits/`, with `Cell` as supertrait and one method returning the left/right cell pairs with signs for a given left degree; name the pair for its algebraic role, not a geometric one, per the spec; document that a left degree exceeding the cell dimension yields no terms
- [ ] 2.2 Export the trait from `deep_causality_topology/src/lib.rs`
- [ ] 2.3 Implement it for `Simplex`, returning the single Alexander–Whitney term built from `subsimplex`, with sign `+1`
- [ ] 2.4 Implement it for `LatticeCell<D>`, returning `C(k, p)` terms, one per choice of front axes, each carrying the shuffle sign against the cell's ascending axis order
- [ ] 2.5 Add tests for both implementations: term counts, left and right cell identity, sign values against Chen & Tata Eq. (26), the published 2-D example from their Fig. 1 and Fig. 4, and the empty result beyond the cell dimension
- [ ] 2.6 Verify `Cell` is untouched: `impl Cell for Simplex`, `impl Cell for LatticeCell<D>` and `impl Cell for HoneycombCell` compile unmodified, and no `: Cell` bound anywhere in the workspace changes
- [ ] 2.7 Register test files in `mod.rs` and `BUILD.bazel`

## 3. The binary cup product

- [ ] 3.1 Add the cup product module in `deep_causality_topology/src/types/`, generic over `R: RealField` and over any complex whose cells implement the splitting trait, taking cochains as slices indexed by cell index
- [ ] 3.2 Add typed errors for cochain length mismatch against `num_cells(degree)` and for `p + q` exceeding `max_dim()`, following the crate's existing `TopologyError` conventions
- [ ] 3.3 Export the cup product from `lib.rs`
- [ ] 3.4 Add a unit test that the simplicial case reproduces `α([v₀,v₁]) · β([v₁,v₂])` on a single 2-simplex
- [ ] 3.5 Add a unit test that the cubical case reproduces Serre's formula on one face of a `LatticeComplex<2, f64>`, checking both the front-axis terms and their signs by hand
- [ ] 3.6 Add error-path tests for both rejection cases
- [ ] 3.7 Register test files in `mod.rs` and `BUILD.bazel`

## 4. The algebraic laws

This group is the acceptance gate. A cubical sign error is invisible without it.

- [ ] 4.1 Add the Leibniz property test `δ(α ∪ β) = δα ∪ β + (−1)^p · α ∪ δβ` on a simplicial complex, using the complex's own `coboundary_matrix`, across every degree pair the complex admits
- [ ] 4.2 Add the same Leibniz test on `LatticeComplex::<2, f64>::square_torus(L)` and `LatticeComplex::<3, f64>::cubic_torus(L)` for several `L`
- [ ] 4.3 Add the graded-commutativity test: on cocycles, `α ∪ β − (−1)^{pq} · β ∪ α` is a coboundary, verified by pairing to zero against cycles
- [ ] 4.4 Add a negative test confirming the two orderings are permitted to differ on arbitrary cochains, so the previous test is not passing trivially
- [ ] 4.5 If any sign convention proves inconsistent with the shipped boundary operators, fix the splitting implementation rather than the test, and record the resolved convention in the module documentation
- [ ] 4.6 Run at least one law test on a hand-built `SimplicialComplex` rather than on a torus, so genericity over `ChainComplex` is executed rather than asserted
- [ ] 4.7 Register test files in `mod.rs` and `BUILD.bazel`

## 5. The n-fold product and multi-controlled actions

- [ ] 5.1 Add an associativity test `(α ∪ β) ∪ γ = α ∪ (β ∪ γ)` on both complex families before building anything on it
- [ ] 5.2 Add the `n`-fold cup product as a left fold of the binary product over a slice of cochains
- [ ] 5.3 Add tests: agreement with repeated binary application, a single-element slice returning its input unchanged, and an empty slice returning a typed error
- [ ] 5.4 Add a triple-product test on `cubic_torus(L)` yielding a 3-cochain, and confirm the same request on a two-dimensional complex returns the degree-exceeds-dimension error
- [ ] 5.5 Add the `CCZ` homology-class-invariance test: build the logical action from three 1-cochains on `cubic_torus(L)`, vary each input by a coboundary, and assert the action is unchanged
- [ ] 5.6 Document in the module rustdoc that a logical action is computed and verified, and that emitting a constant-depth fault-tolerant physical decomposition is out of scope
- [ ] 5.7 Register test files in `mod.rs` and `BUILD.bazel`

## 6. Relocate the physics Alexander–Whitney implementation

Kept as its own group so a bisect isolates any numerical change.

- [ ] 6.1 Record the current MHD ideal-induction kernel outputs as a baseline before touching anything
- [ ] 6.2 Rewrite `wedge_product_1form_1form` in `deep_causality_physics/src/kernels/mhd/ideal.rs` to call the topology cup product, keeping the antisymmetrisation in physics
- [ ] 6.3 Run the existing `deep_causality_physics` ideal-induction kernel tests unmodified and confirm they pass with unchanged expected values
- [ ] 6.4 Confirm `deep_causality_physics` exposes the same public items as before

## 7. Verification and follow-through

- [ ] 7.1 `cargo test -p deep_causality_topology` and `cargo test -p deep_causality_physics` green
- [ ] 7.2 `bazel test //...` green across the workspace
- [ ] 7.3 `make format && make fix` clean, with any clippy findings fixed rather than suppressed
- [ ] 7.4 Confirm zero blast radius by building every dependent crate and example package unchanged: `deep_causality_cfd`, `deep_causality_algorithms`, `deep_causality_discovery`, and the five example packages
- [ ] 7.5 Update `openspec/notes/quantum/dynamic-qcm.md` §3.3 and the `SPEC-T1` entry to record that the branching structure was already present, and re-scope the remaining topology ladder against what this change delivers
- [ ] 7.6 Update `openspec/notes/quantum/example-geometric-qec.md` §6 to reflect what is now shipped, and note that a logical `CZ` and `CCZ` are available to the example
- [ ] 7.7 Prepare a commit message per task group, per the repository convention
