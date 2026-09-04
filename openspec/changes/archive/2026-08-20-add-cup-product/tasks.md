## 1. Document the ordering invariants

Doc-only, zero risk, and it makes the invariant the rest of the change depends on explicit.

- [x] 1.1 Add rustdoc to `Simplex::vertices()` in `deep_causality_topology/src/types/simplex/getters/mod.rs` stating that the returned slice is strictly increasing, that no repeats occur, and that this order is the branching structure the cup product depends on
- [x] 1.2 Add rustdoc to `LatticeCell::vertices()` in `deep_causality_topology/src/types/lattice_complex/lattice_cell.rs` stating the corner enumeration order: active axes ascending, corners by binary counter over active axes, corner `0` at the base position
- [x] 1.3 Add tests asserting both invariants: sortedness across every `Simplex` construction path, and the corner-order contract across cell dimensions in `LatticeComplex<2>` and `LatticeComplex<3>`
- [x] 1.4 Register the new test files in the corresponding `tests/**/mod.rs` and in `deep_causality_topology/tests/BUILD.bazel`

## 2. The splitting trait

- [x] 2.1 Add the splitting trait in `deep_causality_topology/src/traits/`, with `Cell` as supertrait and one method returning the left/right cell pairs with signs for a given left degree; name the pair for its algebraic role, not a geometric one, per the spec; document that a left degree exceeding the cell dimension yields no terms
- [x] 2.2 Export the trait from `deep_causality_topology/src/lib.rs`
- [x] 2.3 Implement it for `Simplex`, returning the single Alexander–Whitney term built from `subsimplex`, with sign `+1`
- [x] 2.4 Implement it for `LatticeCell<D>`, returning `C(k, p)` terms, one per choice of front axes, each carrying the shuffle sign against the cell's ascending axis order
- [x] 2.5 Add tests for both implementations: term counts, left and right cell identity, sign values against Chen & Tata Eq. (26), the published 2-D example from their Fig. 1 and Fig. 4, and the empty result beyond the cell dimension
- [x] 2.6 Verify `Cell` is untouched: `impl Cell for Simplex`, `impl Cell for LatticeCell<D>` and `impl Cell for HoneycombCell` compile unmodified, and no `: Cell` bound anywhere in the workspace changes
- [x] 2.7 Register test files in `mod.rs` and `BUILD.bazel`

## 3. The binary cup product

- [x] 3.1 Add the cup product module in `deep_causality_topology/src/types/`, generic over `R: RealField` and over any complex whose cells implement the splitting trait, taking cochains as slices indexed by cell index
- [x] 3.2 Add typed errors for cochain length mismatch against `num_cells(degree)` and for `p + q` exceeding `max_dim()`, following the crate's existing `TopologyError` conventions
- [x] 3.3 Export the cup product from `lib.rs`
- [x] 3.4 Add a unit test that the simplicial case reproduces `α([v₀,v₁]) · β([v₁,v₂])` on a single 2-simplex
- [x] 3.5 Add a unit test that the cubical case reproduces Serre's formula on one face of a `LatticeComplex<2, f64>`, checking both the front-axis terms and their signs by hand
- [x] 3.6 Add error-path tests for both rejection cases
- [x] 3.7 Register test files in `mod.rs` and `BUILD.bazel`

## 4. The algebraic laws

This group is the acceptance gate. A cubical sign error is invisible without it.

- [x] 4.1 Add the Leibniz property test `δ(α ∪ β) = δα ∪ β + (−1)^p · α ∪ δβ` on a simplicial complex, using the complex's own `coboundary_matrix`, across every degree pair the complex admits
- [x] 4.2 Add the same Leibniz test on `LatticeComplex::<2, f64>::square_torus(L)` and `LatticeComplex::<3, f64>::cubic_torus(L)` for several `L`
- [x] 4.3 Add the graded-commutativity test: on cocycles, `α ∪ β − (−1)^{pq} · β ∪ α` is a coboundary, verified by pairing to zero against cycles
- [x] 4.4 Add a negative test confirming the two orderings are permitted to differ on arbitrary cochains, so the previous test is not passing trivially
- [x] 4.5 If any sign convention proves inconsistent with the shipped boundary operators, fix the splitting implementation rather than the test, and record the resolved convention in the module documentation
- [x] 4.6 Run at least one law test on a hand-built `SimplicialComplex` rather than on a torus, so genericity over `ChainComplex` is executed rather than asserted
- [x] 4.7 Register test files in `mod.rs` and `BUILD.bazel`

## 5. The n-fold product and multi-controlled actions

- [x] 5.1 Add an associativity test `(α ∪ β) ∪ γ = α ∪ (β ∪ γ)` on both complex families before building anything on it
- [x] 5.2 Add the `n`-fold cup product as a left fold of the binary product over a slice of cochains
- [x] 5.3 Add tests: agreement with repeated binary application, a single-element slice returning its input unchanged, and an empty slice returning a typed error
- [x] 5.4 Add a triple-product test on `cubic_torus(L)` yielding a 3-cochain, and confirm the same request on a two-dimensional complex returns the degree-exceeds-dimension error
- [x] 5.5 Add the triple-product verification on `cubic_torus(L)`: assert the three direction cochains are cocycles, that `∫ e₀ ∪ e₁ ∪ e₂ = L³`, and that the integral is unchanged when an input is shifted by a coboundary
- [x] 5.6 Document in the module rustdoc that this crate delivers the cup product only: gate construction lives in `deep_causality_quantum`, which does not depend on this crate, and emitting fault-tolerant circuits is out of scope entirely
- [x] 5.7 Register test files in `mod.rs` and `BUILD.bazel`

## 6. Verification and follow-through

- [x] 6.1 `cargo test -p deep_causality_topology` green, and `cargo test -p deep_causality_physics` green as an unchanged-neighbour check
- [x] 6.2 `bazel test //...` green across the workspace
- [x] 6.3 `make format && make fix` clean, with any clippy findings fixed rather than suppressed
- [x] 6.4 Confirm zero blast radius by building every dependent crate and example package unchanged: `deep_causality_cfd`, `deep_causality_algorithms`, `deep_causality_discovery`, and the five example packages
- [x] 6.5 Update `openspec/notes/quantum/dynamic-qcm.md` §3.3 and the `SPEC-T1` entry to record that the branching structure was already present, and re-scope the remaining topology ladder against what this change delivers
- [x] 6.6 Update `openspec/notes/quantum/example-geometric-qec.md` §6 to reflect what is now shipped, and note that a logical `CZ` and `CCZ` are available to the example
- [x] 6.7 Prepare a commit message per task group, per the repository convention
