# Tasks

## How each group is worked

Every group that produces API runs the same five phases. They are named in the task text as
**M / T / V / I / R** so a half-finished group says which phase it stopped in.

- **M — mock.** The full API lands first: every signature, every doc comment, every error variant,
  bodies `todo!()`. Nothing is decided later that a signature could have decided now.
- **T — tests.** The whole suite is written against the mock. It compiles and it fails.
- **V — verify the suite, before any implementation exists.** Three things are checked, and the
  check is recorded in the group's notes:
  - **No tautology.** A test that asserts a quantity is zero where the quantity vanishes for every
    input pins nothing. A test using only the identity, a diagonal, or one size pins nothing where
    distinct quantities coincide there.
  - **No circularity.** No expected value is read off a run of the code under test. No test
    computes both sides with the same routine.
  - **External reference.** Every numeric expectation comes from a publication or from
    `openspec/notes/homology/reference/reference.py`, which imports nothing from this workspace.
    Inputs cover a stated range rather than a chosen point.
- **I — implement.** Replace the `todo!()`s until the suite passes. The suite does not change in
  this phase.
- **R — root cause.** A failing test is diagnosed before it is touched. The diagnosis says which of
  the two is wrong, the API or the test, and why. AGENTS.md: a test is never edited to let a broken
  API pass.

Groups 1, 3, 6 and 10 have no new API. Group 4 moves code that already works, so its M and I phases
are the move itself; its **V phase is not degenerate** — those tests have never been audited, and
one of them already passes by coincidence (task 8.5).

## 1. Build the reference oracle

Everything in every later V phase resolves here, so it comes first.

- [x] 1.1 Write `openspec/notes/homology/reference/reference.py` in the style of
      `openspec/notes/linear/reference/reference.py`: exact arithmetic, no import of this
      workspace, emits constants to paste
- [x] 1.2 Emit boundary matrices and Betti numbers over ℚ and 𝔽₂ for the minimal triangulations of
      point, interval, `S¹`, `S²`, `T²`, `T³`, cylinder, Möbius band, `RP²` and the Klein bottle
- [x] 1.3 Cite the published values for each in the docstring — Hatcher, *Algebraic Topology*,
      §2.1 and Example 2.42 for the mod-2 homology of `RP^n` — so the script's own output is
      checked against a source rather than trusted
- [x] 1.4 Emit 𝔽₂ kernel and image bases for a range of shapes: square, wide, tall, zero,
      full-rank, and rank-deficient, at sizes crossing the 64-bit word boundary (63, 64, 65, 129)
- [x] 1.5 Record why `RP²` and the Klein bottle are in the set: every complex the workspace ships
      today is orientable and torsion-free, so `HomologyField::Rational` and `HomologyField::Gf2`
      agree at every grade of every fixture and the field parameter is never discriminated.
      `β₁(RP²; ℚ) = 0` and `β₁(RP²; 𝔽₂) = 1` is the smallest case that separates them

## 2. Repair the basis orientation in `deep_causality_linear`

Independently useful and independently releasable. Do this before the crate exists: it is a live
defect, and the extraction would otherwise carry it into a new crate.

- [x] 2.1 **M** Add `PackedGf2Vector::from_column` and `Gf2Chain::from_column` as signatures with
      `todo!()` bodies and full docs
- [x] 2.2 **T** Write the suite: for each shape from task 1.4, read every `kernel_basis_gf2` vector
      through `from_column` and assert `A · v = 0`; read every `image_basis_gf2` vector and assert
      it lies in the column space; assert the basis count equals the reference nullity and rank
- [x] 2.3 **T** Add the negative test that fixes the orientation: reading the same basis through
      `from_row` gives a vector of the wrong length whenever the matrix is not square
- [x] 2.4 **V** Confirm no expectation came from a run of `kernel_basis_gf2`; `A · v = 0` is a
      property and the counts come from task 1.4. Confirm the shape range includes non-square and
      word-boundary sizes, so a `rows`/`cols` swap cannot pass
- [x] 2.5 **I** Implement `from_column`, reading across the word stride rather than copying a
      contiguous word slice
- [x] 2.6 Correct the four docstrings that say rows: `packed_gf2_vector/mod.rs:107-108`,
      `gf2_chain/mod.rs:73`, `packed_gf2_vector_tests.rs:177`, `gf2_chain_tests.rs:100`
- [x] 2.7 Move `widen_to_dense_i64` from `topology/src/types/homology_field/mod.rs` into
      `linear/src/extensions/conversions.rs` as `csr_i8_to_dense_i64`, exported from `lib.rs`
- [x] 2.8 Bump `deep_causality_linear` to 0.1.2; `cargo test -p deep_causality_linear`, clippy, fmt

## 3. Create the crate skeleton

- [x] 3.1 `deep_causality_homology/Cargo.toml` at 0.1.0 with description, license, repository,
      `[lints] workspace = true`, and path deps on `deep_causality_linear`,
      `deep_causality_algebra`, `deep_causality_num`
- [x] 3.2 Add the crate to the workspace `members` list
- [x] 3.3 `deep_causality_homology/BUILD.bazel` and `tests/BUILD.bazel` with a `rust_test_suite`
      per test directory
- [x] 3.4 Register in `.github/workflows/formalization.yml` — omission produces a false
      `MISSING Rust witness` on every new theorem id
- [x] 3.5 Register in `.github/workflows/rust_deps.yml` — omission is silent, cargo-machete simply
      never inspects the crate
- [x] 3.6 Add the crate index entry and tier block to `AGENTS.md`
- [x] 3.7 `cargo build -p deep_causality_homology` and `bazel build //deep_causality_homology/...`
      on an empty lib

## 4. Move the chain-complex layer

- [x] 4.1 **M** Land the full moved API as signatures with `todo!()` bodies: `ChainComplex` with
      `num_cells`, `max_dim`, `boundary_matrix`, `coboundary_matrix` and the provided
      `betti_number_over` and `betti_number`; `HomologyField` and `rank_of`; `Gf2Chain<W>`;
      `HomologyError`
- [x] 4.2 **M** Keep `boundary_matrix` and `coboundary_matrix` returning `Cow<'_, CsrMatrix<i8>>`;
      add no coefficient parameter
- [x] 4.3 **T** Move the existing tests and rewrite every expectation against task 1.2's values,
      dropping any that was a transcript of a previous run
- [x] 4.4 **V** Audit the moved suite. Record which tests asserted a quantity only where it
      vanishes, which used a single size, and which compared two results from the same routine
- [x] 4.5 **V** Add the Euler-characteristic cross-check for every fixture:
      `Σ(−1)ᵏ nₖ = Σ(−1)ᵏ βₖ`. The two sides come from cell counts and from ranks, so it is two
      computations agreeing rather than one rearranged
- [x] 4.6 **I** Fill the bodies, calling linear's `csr_i8_to_dense_i64` from task 2.7
- [x] 4.7 Add a probe asserting the crate's dependency set excludes `deep_causality_topology`

## 5. Add the topology seam

- [x] 5.1 Add `deep_causality_homology` as a path dependency of `deep_causality_topology`
- [x] 5.2 Define `pub trait CellularComplex: ChainComplex` in `topology/src/traits/` with `CellType`,
      `CellIter`, `Metric`, `cells(k)` and `uniform_lattice_layout()`
- [x] 5.3 Re-export `ChainComplex`, `HomologyField` and `Gf2Chain` from `topology/src/lib.rs`
- [x] 5.4 Split each of the three implementors — `SimplicialComplex`, `CellComplex<C>`,
      `LatticeComplex` — into a `ChainComplex` impl and a `CellularComplex` impl
- [x] 5.5 Build `deep_causality_cfd` and `deep_causality_physics` with **no** `Cargo.toml` edit.
      **Result: zero manifest edits, and two `use` edits, both in `deep_causality_cfd`.**
      `deep_causality_cfd/src/solvers/dec/spectral_diffusion.rs:34` imported `ChainComplex` and
      called `uniform_lattice_layout()`, a geometry method the split assigns to `CellularComplex`;
      and `tests/solvers/dec/cut_cell_wiring_tests.rs` called `cells()` at four sites. No
      re-export changes which trait owns a method, so both imports were retargeted to
      `CellularComplex`. All 18 `num_cells` sites, both `deep_causality_physics` files and the rest
      of the CFD tree resolve through the re-export untouched
- [x] 5.6 Bump `deep_causality_topology` to 0.7.4

## 6. Make `Gf2Chain`'s chain group one guard

The carrier is settled in design Decision 5: `(degree, len)` identifies `C_k`, and no complex handle
is added. What is missing is that the two halves of that condition are checked in two places and
reported by two error types.

- [x] 6.1 **T** Write the table test first: equal degree with unequal length, unequal degree with
      equal length, and both unequal — each refused, each with the same error variant
- [x] 6.2 **V** Confirm the table covers both halves independently. A test that varies degree and
      length together cannot tell which guard fired
- [x] 6.3 **I** Replace `same_degree` with `same_group`, checking degree and length together and
      naming `C_k` in the message
- [x] 6.4 **I** Route `add`, `intersect` and `inner` through it, so a length mismatch stops
      surfacing as a `PackedGf2Vector` error from a layer below
- [x] 6.5 Record in the type docstring why there is no complex handle — every operation it has
      belongs to `C_k`, and the complex enters with `∂`

## 7. State and test the differential law

- [x] 7.1 **M** State `∂ₖ ∘ ∂ₖ₊₁ = 0` in the `ChainComplex` docstring and in `Cell`'s, since
      `CellComplex<C>` derives every operator from `Cell::boundary()`
- [x] 7.2 **T** Assert it in the conformance harness for every implementor at every grade, widening
      coefficients past `i8` before multiplying so the assertion does not run on wrapping arithmetic
      in release
- [x] 7.3 **T** Add `CellularComplex` to the conformance harness with a concrete `Cell` fixture; it is
      absent today
- [x] 7.4 **V** Verify the harness discriminates: a deliberately malformed complex — one incidence
      sign flipped — must fail it. A harness that passes on the malformed complex is measuring
      nothing
- [x] 7.5 **V** Confirm the assertion runs at grades where `∂` is non-zero, not only at the
      degenerate ends where the product is empty

## 8. Fix the degenerate-grade shape

- [x] 8.1 **T** Write the shape assertions first: `∂₀` has shape `(0, num_cells(0))` and `∂_{max+1}`
      has shape `(num_cells(max), 0)`, for all three implementors. They fail against
      `CsrMatrix::new()`
- [x] 8.2 **T** Assert every Betti number is unchanged at every grade of every fixture over both
      fields, against task 1.2's values
- [x] 8.3 **I** Change all three implementors
- [x] 8.4 Extend `assert_shape_invariant` from `1..=max_d` to cover both ends
- [x] 8.5 **R** Repair `lattice_complex_test.rs:183-193`, which asserts `cols(∂₂) == rows(∂₁)` —
      comparing `n₂` with `n₀` — and passes on a torus by coincidence. Composability of `∂₁ ∘ ∂₂` is
      `cols(∂₁) == rows(∂₂)`. Record the root cause before editing: the assertion is wrong, the
      implementation is right

## 9. Formalize the differential law

`gf2_betti_from_ranks` in `lean/DeepCausalityFormal/Linear/RankNullity.lean` takes
`range ∂ₖ₊₁ ≤ ker ∂ₖ` as an unproved hypothesis. Group 7 makes it an obligation in Rust; this group
discharges it in Lean. Do it after group 7, whose harness supplies the witnesses.

- [x] 9.1 Add `lean/DeepCausalityFormal/Homology/ChainCondition.lean` proving
      `homology.chain.dd_zero_implies_range_le_ker` — `∂ₖ ⬝ ∂ₖ₊₁ = 0 → range ∂ₖ₊₁.mulVecLin ≤
      ker ∂ₖ.mulVecLin` over `ZMod 2`, matching `RankNullity.lean`'s carrier so the two compose
- [x] 9.2 Prove `homology.chain.betti_from_dd_zero`: `gf2_betti_from_ranks` restated with
      `∂ₖ ⬝ ∂ₖ₊₁ = 0` in place of the inclusion. Zero `sorry`; `#print axioms` reports only
      `propext`, `Classical.choice`, `Quot.sound`
- [x] 9.3 Add `"Homology"` to `_NAMESPACES` in `lean/BUILD.bazel`, anchored on
      `DeepCausalityFormal/Homology/ChainCondition.lean`. Without it the file is never type-checked
- [x] 9.4 Add any new Mathlib import to `cache_roots` in `MODULE.bazel` — `Matrix.mulVecLin_mul` is
      in `Mathlib.LinearAlgebra.Matrix.ToLin`, absent from that list today. A missing root is never
      fetched and the build fails on it
- [x] 9.5 Write the two Rust witnesses in
      `deep_causality_homology/tests/formalization_lean/chain_condition_tests.rs`, one `#[test]` per
      id, computing `∂ₖ ⬝ ∂ₖ₊₁` and the Betti number by separate routines so the identity is two
      computations agreeing
- [x] 9.6 Add two rows to `lean/THEOREM_MAP.md` in the existing column format
- [x] 9.7 Write `deep_causality_homology/LEAN_HOMOLOGY.md` following `LEAN_LINEAR.md`: what is
      formalized and why it is load-bearing, the L1/L2/bridge split, model fidelity, and what is
      left unformalized with the reason
- [x] 9.8 `bazel test //lean:Homology` and `bazel test //lean:proofs`

## 10. Verify and record

- [x] 10.1 `bazel test //...` — every test passes, and the count is at least the pre-change count
- [x] 10.2 Run `cargo mutants` over `deep_causality_homology/src`. A survivor is a decision the
      verified suite does not pin; each one is either killed or entered in `.cargo/mutants.toml`
      with the measurement that settles it, confirmed by the `comm` check that file carries
- [x] 10.3 `cargo clippy --workspace --all-targets -- -D warnings`, fixing by rewriting rather than
      by `#[allow]`
- [x] 10.4 `cargo fmt --all -- --check`
- [x] 10.5 Generate the SBOM pair for the new crate
- [x] 10.6 Record in `openspec/notes/quantum/qcl-gaps.md` that G-04 and G-08 now have a home, and
      that the `deep_causality_quantum` dependency edge goes to homology rather than topology
- [ ] 10.7 Prepare the commit message; do not commit
