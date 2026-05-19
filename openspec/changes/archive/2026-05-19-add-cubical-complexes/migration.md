# Migration Guide — add-cubical-complexes

This document lists the call-site edits required across the monorepo when the change set lands. It exists so that the agent (or any contributor) can mechanically walk through every breaking site without re-running grep, and so that the verification step in Stage C has a single source of truth.

The migration is performed **after** all three stages have shipped and been signed off (Stage A trait refactor, Stage B Manifold genericization, Stage C cubical surfacing + neighborhood strategies + `CubicalReggeGeometry` rename). It is the last step before the change set is considered closed.

**Migration outcome:** all sites were migrated incrementally during the stage in which they broke (Stages B and C each migrated their own downstream callers before clearing their gate). This document's checkboxes are filled in retroactively as the **final audit walk** confirming nothing was missed.

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
| R6 | `ManifoldWitness<C>` | `SimplicialManifoldWitness<C>` (alias added in Stage B) | B |
| R7 | `Lattice<D>`, `Lattice::new`, `Lattice::torus`, `Lattice::cubic_open`, `Lattice::square_open`, etc. | `LatticeComplex<D>` (canonical) or `CubicalComplex<D>` (alias) | C |
| R8 | `LatticeCell<D>`, `LatticeCell::edge`, `LatticeCell::vertex` | `LatticeCell<D>` (canonical, kept) or `CubicalCell<D>` (alias) | C |
| R9 | `use deep_causality_topology::types::lattice::*;` (direct module path) | `use deep_causality_topology::types::lattice_complex::*;` | C |
| R10 | `CubicalMetric<D>` (pre-rename intermediate name) | `CubicalReggeGeometry<D>` | C |

**NOT renamed (do not change):** `LatticeGaugeField`, `LatticeGaugeFieldWitness`, `LinkVariable`, `FlowParams`, `SmearingParams`, `FlowMethod`, and any other `*Lattice*` identifier inside `src/types/gauge/gauge_field_lattice/`. These are physics names ("lattice gauge theory") and are retained per design decision D4. If a file references *both* the topological `Lattice` (now `LatticeComplex`) and `LatticeGaugeField`, apply R7 to the former and leave the latter untouched.

---

## 2. Sites to migrate

### 2.1 `deep_causality_topology` own examples

- ✅ `deep_causality_topology/examples/manifold_analysis.rs` — R5 applied during Stage B.
- ✅ `deep_causality_topology/examples/differential_field.rs` — R5 applied during Stage B.
- ✅ `deep_causality_topology/examples/lattice_gauge_simulation.rs` — R7 applied during Stage C (`Lattice::new` → `CubicalComplex::new`, `Lattice<2>` → `CubicalComplex<2>`). Physics terms (`LatticeGaugeField`, `FlowMethod`, file name) retained.
- ✅ `deep_causality_topology/examples/cubical_heat_diffusion.rs` — new in Stage C; uses `CubicalComplex<2>` alias intentionally to demonstrate the alias-path is fully functional.

### 2.2 `examples/` (workspace examples)

- ✅ `examples/medicine_examples/aneurysm_risk/main.rs` — R5 applied during Stage B (two `Manifold<f64, f64>` signatures + import).
- ✅ `examples/physics_examples/gauge_gr/main.rs` — R5 applied during Stage B (import updated).
- ✅ `examples/physics_examples/multi_physics_pipeline/model.rs` — R5 applied during Stage B (return type + import).
- ✅ `examples/physics_examples/gauge_lattice_u1_2d/main.rs` — R7 applied during Stage C (`Lattice` → `CubicalComplex` for the topological type; physics terms `LatticeGaugeField`, `U1`, "Lattice Gauge Theory" header retained).
- ✅ `examples/physics_examples/gravitational_wave/main.rs` — verified: uses `ReggeGeometry::new(...)` and `SimplicialComplexBuilder`, no `Manifold<C, F>` form; no edits required.
- ✅ `examples/mathematics_examples/triple_hkt_stress_field/main.rs` — R5 + R6 applied during Stage B.
- ✅ `examples/mathematics_examples/effect_diffusion_on_manifold/main.rs` — R5 + R6 applied during Stage B (5+ sites).
- ✅ `examples/mathematics_examples/capstone_spinor_minkowski/main.rs` — R5 + R6 applied during Stage B.
- ✅ `examples/mathematics_examples/tensor_x_topology_laplacian/main.rs` — R5 + R6 applied during Stage B.

### 2.3 Topology crate internal tests

- ✅ `deep_causality_topology/tests/types/gauge/gauge_field_lattice/verification_tests.rs` — R8 applied during Stage C (`LatticeCell::edge(...)` → `CubicalCell::edge(...)`, surrounding gauge code untouched).

### 2.4 Topology crate own examples that are *not* impacted

Re-verified by workspace build:

- ✅ `deep_causality_topology/examples/basic_graph.rs` — builds clean.
- ✅ `deep_causality_topology/examples/complex_operators.rs` — builds clean.
- ✅ `deep_causality_topology/examples/chain_algebra.rs` — builds clean.
- ✅ `deep_causality_topology/examples/hkt_graph_convolution.rs` — builds clean.

### 2.5 Cross-crate consumers

- ✅ `deep_causality_effects/` — 3 source/test files migrated during Stage B (R5 + import-alias inject).
- ✅ `deep_causality_physics/` — 32 source/test files migrated during Stage B (R5 + import-alias inject).

---

## 3. Verification protocol

Run in order:

1. ✅ **`cargo build --workspace --all-targets`** — succeeded (clean build across 30+ crates in ~9.5 s).
2. ✅ **`cargo test --workspace --all-targets`** — **8,917 tests pass, 0 failed, 2 ignored** workspace-wide.
3. ✅ **`make format && make fix`** — clean (no new clippy warnings, no format diffs).
4. ✅ **Example runtime check** — `cargo run -p deep_causality_topology --example cubical_heat_diffusion` runs to completion and produces the expected symmetric ASCII heatmap diffusion pattern. Stage A/B/C tests cover the rest of the example surface (618 → 633 unit + 30 new `CubicalReggeGeometry` tests + integration + doctest, all green per Stage C gate evidence).
5. ✅ **Stale-identifier audit grep** (`grep -RIn 'CWComplex\|\bLattice\b\|\bLatticeCell\b' examples deep_causality_topology/examples`):
   - Code matches: zero (clean).
   - Documentation matches: 5 hits in `examples/physics_examples/gauge_lattice_u1_2d/main.rs` (doc comments, print strings: "2D U(1) Lattice Gauge Theory Verification", "Lattice: {L}×{L}", etc.) and `examples/README.md` ("Gauge Lattice U(1) 2D"). All are **physics-domain "lattice gauge theory" text**, intentionally preserved per design D4.
6. ✅ **Manifold<C, F> heuristic audit** (`grep '\bManifold<X, Y>' examples deep_causality_topology/examples`):
   - Code matches: zero.
   - Documentation matches: 2 hits in `examples/mathematics_examples/triple_hkt_stress_field/README.md` showing hypothetical future function signatures inside a diff-style code block; not live code. Acceptable.
7. ✅ **`proposal.md` § Impact completeness** — re-checked against the breaking-changes inventory; every category is reflected in the migration table above. No late-discovered sites.

---

## 4. Sign-off

- ✅ Migration-completion summary surfaced to the user (this document is the artifact).
- ⬜ Awaiting explicit user sign-off on the migration audit walk.
- ⬜ Commit message prepared for the user to commit (see below).
- ⬜ After the migration commit lands, the change set is closed and ready to archive via `openspec` archival tooling.

### Proposed commit message

```text
chore(topology): complete migration audit walk for #487 (Stage 5)

Mark all migration.md entries ✅ after the Stage 5 audit walk. Every
call site was already migrated incrementally during its owning stage
(B or C); this commit reflects the final-audit verification:

- cargo build --workspace --all-targets: clean
- cargo test --workspace --all-targets: 8917 passed, 0 failed, 2 ignored
- make format && make fix: clean
- example runtime: cubical_heat_diffusion produces expected diffusion
- stale-identifier audit: zero code hits (only physics-domain lattice
  gauge theory documentation text retained per D4)

Closes the add-cubical-complexes change set; ready for openspec archive.

Co-Authored-By: Claude Opus 4.7 <noreply@anthropic.com>
```
