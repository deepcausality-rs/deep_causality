# Migration Guide — add-cubical-complexes

This document lists the call-site edits required across the monorepo when the change set lands. It exists so that the agent (or any contributor) can mechanically walk through every breaking site without re-running grep, and so that the verification step in Stage C task 3.20 has a single source of truth.

The migration is performed **after** all three stages have shipped and been signed off (Stage A trait refactor, Stage B Manifold genericization, Stage C cubical rename). It is the last step before the change set is considered closed.

Status legend used below:
- ⬜ — not yet migrated
- ✅ — migrated and verified (file builds + tests pass)

---

## 1. Migration rules (apply to every site)

| # | From | To | Stage |
|---|---|---|---|
| R1 | `use deep_causality_topology::CWComplex;` | `use deep_causality_topology::ChainComplex;` | A |
| R2 | `<K: CWComplex>` (any generic bound) | `<K: ChainComplex>` | A |
| R3 | `CWComplex::method(&x, ...)` (fully-qualified) | `ChainComplex::method(&x, ...)` | A |
| R4 | `let m: CsrMatrix<i8> = complex.boundary_matrix(k);` | `let m = complex.boundary_matrix(k); // &*m to read, .into_owned() to consume` | A |
| R5 | `Manifold<C, F>` (where `C` was the simplex coordinate type) | `SimplicialManifold<C, F>` | B |
| R6 | `ManifoldWitness<C>` | `SimplicialManifoldWitness<C>` (add alias) or `ManifoldWitness<SimplicialComplex<C>>` | B |
| R7 | `Lattice<D>`, `Lattice::new`, `Lattice::torus`, `Lattice::cubic_open`, `Lattice::square_open`, etc. | `CubicalComplex<D>`, `CubicalComplex::new`, ... | C |
| R8 | `LatticeCell<D>`, `LatticeCell::edge`, `LatticeCell::vertex` | `CubicalCell<D>`, `CubicalCell::edge`, ... | C |
| R9 | `use deep_causality_topology::types::lattice::*;` (direct module path) | `use deep_causality_topology::types::cubical_complex::*;` | C |

**NOT renamed (do not change):** `LatticeGaugeField`, `LatticeGaugeFieldWitness`, `LinkVariable`, `FlowParams`, `SmearingParams`, `FlowMethod`, and any other `*Lattice*` identifier inside `src/types/gauge/gauge_field_lattice/`. These are physics names ("lattice gauge theory") and are retained per design decision D4. If a file references *both* the topological `Lattice` (now `CubicalComplex`) and `LatticeGaugeField`, apply R7 to the former and leave the latter untouched.

---

## 2. Sites to migrate

### 2.1 `deep_causality_topology` own examples

- ⬜ `deep_causality_topology/examples/manifold_analysis.rs` — apply R5 to `Manifold<...>` type signature(s).
- ⬜ `deep_causality_topology/examples/differential_field.rs` — apply R5 to `Manifold<...>`.
- ⬜ `deep_causality_topology/examples/lattice_gauge_simulation.rs` — apply R7 to `Lattice::new(...)` (line ~46) and `Lattice` type spelling (line ~26). Leave `LatticeGaugeField`, `FlowMethod`, etc. untouched. Keep the example's filename and physics-facing identifiers ("Lattice Gauge Simulation", "Lattice gauge field") because the physics term is correct.

### 2.2 `examples/` (workspace examples)

- ⬜ `examples/medicine_examples/aneurysm_risk/main.rs` — R5 on two `Manifold<f64, f64>` signatures (lines ~114 and ~132) and one import (line ~18).
- ⬜ `examples/physics_examples/gauge_gr/main.rs` — R5 if any `Manifold<C, F>` form is used; verify the import line (~21).
- ⬜ `examples/physics_examples/multi_physics_pipeline/model.rs` — R5 on the `Manifold<f64, f64>` return type at line ~10 and any other site; update the import at line ~8.
- ⬜ `examples/physics_examples/gauge_lattice_u1_2d/main.rs` — R7 on `Lattice`, `Lattice::new`, `Lattice<2>` (multiple sites near lines 25, 92, 153). Leave `LatticeGaugeField`, `U1`, etc. untouched. The doc comment header references "Lattice Gauge Theory" — leave intact.
- ⬜ `examples/physics_examples/gravitational_wave/main.rs` — verify `ReggeGeometry::new(...)` still compiles (Stage B should leave the simplicial `ReggeGeometry` API stable). If `Manifold` is used elsewhere in the file, apply R5.
- ⬜ `examples/mathematics_examples/triple_hkt_stress_field/main.rs` — R5 on `Manifold<f64, FloatType>` (line ~220) and R6 on `ManifoldWitness` (line ~33).
- ⬜ `examples/mathematics_examples/effect_diffusion_on_manifold/main.rs` — R5 on every `Manifold<f64, FloatType>` (5+ sites around lines 48, 80, 106, 144) and R6 on `ManifoldWitness` (line ~22).
- ⬜ `examples/mathematics_examples/capstone_spinor_minkowski/main.rs` — R5 on three `Manifold<f64, FloatType>` sites (lines ~113, ~143, ~178) and R6 on `ManifoldWitness` (line ~36).
- ⬜ `examples/mathematics_examples/tensor_x_topology_laplacian/main.rs` — R5 on `Manifold<f64, FloatType>` (line ~83) and R6 on `ManifoldWitness` (line ~23).

### 2.3 Topology crate internal tests (mostly handled in their owning stage; listed here for completeness)

These should be migrated as part of their respective stages, not deferred to this migration pass. If anything was missed by the per-stage tasks, this file is the catch-all.

- ⬜ `deep_causality_topology/tests/types/gauge/gauge_field_lattice/verification_tests.rs` — verify the `LatticeCell::edge(...)` call at line ~494 is updated to `CubicalCell::edge(...)` (R8) at Stage C. The surrounding gauge code does not rename.

### 2.4 Topology crate own examples that are *not* impacted but worth re-verifying

The following examples were grepped and do not require edits, but should still be re-built to catch any transitive surface change:

- `deep_causality_topology/examples/basic_graph.rs` (uses `Graph`, `GraphTopology`)
- `deep_causality_topology/examples/complex_operators.rs` (uses `PointCloud`)
- `deep_causality_topology/examples/chain_algebra.rs` (uses `Chain`, `Simplex`, `SimplicialComplex`, `Skeleton`)
- `deep_causality_topology/examples/hkt_graph_convolution.rs` (uses `Graph`, `GraphWitness`)

### 2.5 Cross-crate consumers worth re-verifying (build-only check)

These crates import `deep_causality_topology` but use the types that do NOT rename (`SimplicialComplex`, `Simplex`, `Skeleton`, `Graph`, `PointCloud`, `BaseTopology`). They should compile without edits — but the migration MUST verify them:

- `deep_causality_effects/` — 3 source/test files.
- `deep_causality_physics/` — ~25 source/test files.

If any file in these crates fails to build, treat it as a regression and add a migration entry above.

---

## 3. Verification protocol

Run in order, fixing each site as it surfaces:

1. ⬜ `cargo build --workspace --all-targets` — must succeed.
2. ⬜ `cargo test --workspace --all-targets` — must succeed.
3. ⬜ `make format && make fix` — clean.
4. ⬜ For every example in section 2.1 and 2.2, run `cargo run --example <name>` (or the appropriate `cargo run -p <crate> --example <name>`) at least once to confirm runtime behavior. UI/visual examples need a manual eyeball check.
5. ⬜ `grep -RIn 'CWComplex\|\bLattice\b\|\bLatticeCell\b\|Manifold<[^=]*[a-zA-Z][^,>]*,' examples deep_causality_topology/examples` returns zero matches except for the permitted physics-named `LatticeGaugeField` family. The `Manifold<` pattern is heuristic — manually inspect any remaining hits to confirm they are `SimplicialManifold` or the new generic `Manifold<K, F>` form.
6. ⬜ Confirm the breaking-change list in `proposal.md` § Impact is complete; if a site was missed, append it to section 2 of this file before closing the change.

---

## 4. Sign-off

When every checkbox above is ✅:

- ⬜ Present a migration-completion summary to the user (files touched, build/test evidence, any sites discovered late and added to section 2).
- ⬜ Wait for explicit user sign-off.
- ⬜ Prepare a commit message for the user to commit. The agent NEVER commits per AGENTS.md §"Golden Rules" rule 1.
- ⬜ After the migration commit lands, the change set is closed and ready to archive via `openspec` archival tooling.
