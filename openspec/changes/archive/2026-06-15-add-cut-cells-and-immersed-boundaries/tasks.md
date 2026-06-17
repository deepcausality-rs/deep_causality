# Milestone structure

Four sub-groups, each ending green (full tests passing on touched crates in both
feature configurations, clippy/fmt clean) with a prepared commit message finalizing it,
so progress is de-risked at four landing points. Group A is cut-cell geometry; Group B
is stabilization + immersed wall BC + solver wiring; Group C is the first
`MaybeUncertain` data zone; Group D is the 3D-cylinder validation. No group opens with
an unclosed gate from the prior group.

Per AGENTS.md golden rules: agents never `git commit` and never delete files — each
group gate prepares a commit message and asks the user to commit. `make fix` / `make
build` / `make test` are run by the user on review, not by the agent.

## A. Cut-cell geometry (cut-cell-geometry)

- [x] A1 `CutCell<D>` carrier: clipped fluid volume, per-face aperture (wetted fraction
      in `[0,1]`), `Vec<CutFaceFragment>` (area, outward unit normal, source-geometry
      tag), and `Fluid | Cut | Solid` classification. Private fields + getters per the
      one-type-one-module convention. (`src/types/cut_cell/{carrier,cut_face_fragment,
      cell_class,source_geometry}.rs`.)
- [x] A2 `CutCellRegistry<D>`: sparse `cell_id -> CutCell<D>` map keyed in the lattice's
      `iter_cells(D)` ordering; absent cells are full fluid cells. Construction API
      (`new`/`from_map`/`insert`/`from_primitive`). (`src/types/cut_cell/registry.rs`.)
- [x] A3 Cut-aware volume/aperture accessor on the geometry path: `clipped_cell_volume`
      overrides registered top cells and falls through for the rest; `dual_fluid_fraction`
      generalizes the `boundary_clip` `2^{-b}` factor to a continuous wetted fraction; full
      cells keep the existing fast path untouched. (Folding into the shipped `hodge_star`
      build is B5 solver wiring — A3 delivers the override + the reduction proof.)
- [x] A4 Cube ⋂ analytic primitive: exact closed-form clipped volume + apertures +
      fragment normals for **half-space (all D)**, **axis-aligned cylinder (D=3, the
      cylinder-validation path)**, and **disk (D=2)**. The **3D ball** closed form is
      deferred (off the validation path; unsupported combos return `TopologyError`).
      Cochain discipline (carried from R1's `graded-metrics` finding): clipped volumes and
      apertures are produced and consumed as *cell measures*, never pointwise samples.
      (Curved-surface *fragment* area uses high-resolution quadrature — not on the exactness
      scenario, which gates volume + apertures.)
- [x] A5 ~~Cube ⋂ triangle (STL path)~~ **POSTPONED / out of scope** (Open Question 4 resolved):
      no STL / file reading in this change; intersection is analytic-primitive-only (A4). Deferred to
      a later change with its own degenerate-triangle / near-tangent-cut handling. (Closed by
      deferral — not a blocker for this change.)
- [x] A6 Consistency gate: the cut-aware `dual_fluid_fraction` reproduces the Stage-3
      integer wall clip — pinned both against the closed-form `2^{-b}` and against the actual
      shipped `hodge_star` diagonal on open lattices (2D + 3D), and against a plane cut
      coincident with a cell boundary.
- [x] A7 Exactness tests: clipped volume and apertures of cube ⋂ {plane, disk, cylinder}
      against closed-form **measures** (not pointwise values — the R1 cochain lesson),
      2D/3D, f64 + Float106, ≤ tolerance; registry round-trip + sparsity. Graded-and-cut
      composition check: a cut cell's full volume equals `geom.cell_volume` over `PerEdge`
      graded edge lengths, confirming cut rides the verified second-order graded substrate.
- [x] A8 Group gate: `make format`, clippy clean (0 warnings), full topology tests both
      feature configs (1227 pass) + the 17 new cut-cell tests (cargo + bazel
      `//deep_causality_topology/tests:types/cut_cell`); commit message prepared. (User
      commits per the golden rule.)

## B. Stabilization + immersed wall BC + solver wiring (cut-cell-stabilization, immersed-wall-bc)

- [x] B1 Stabilizers assessed on the sliver case: **cell-merging** is realised as a
      volume-fraction floor on the cut star (`CutCellRegistry::with_cell_merging`, the
      Berger–Helzel family); **flux-redistribution** (Colella–Graves–Modiano) is assessed and
      rejected — it needs a per-cell conservative update the projected-rate RK4 formulation does
      not expose (design D4).
- [x] B2 Decision recorded in `design.md` D4, with the headline **finding**: the classic
      small-cell CFL instability does not arise in this DEC formulation at all — the consistent
      metric clip cancels in `δ = M⁻¹ ∂ M`, so cell-merging is selected but serves only
      masked-CG projection conditioning, not explicit stability. Exposed as a named geometric
      correction (`with_cell_merging`); `.intervene`/`EffectLog` wiring lands with that surface
      in Group C.
- [x] B3 Stability test (`cut_cell_wiring_tests::tiny_cut_cells_are_inherently_small_cell_stable`):
      four 0.1%-wetted free cut cells meeting at a vertex march **finite and non-amplifying at a
      normal `dt` with no stabilizer** (the inherent-stability finding); cell-merging preserves
      that and improves the projection's divergence residual. (The expected "unstabilized control
      aborts" did not occur — the formulation is inherently robust, which is the finding.)
- [x] B4 Immersed no-slip on the immersed body: `CutCellRegistry::solid_incident_edges`
      yields the staircase no-slip / no-penetration set (every edge incident to a `Solid`
      cell), and `NoSlipConstraint::new` unions it with the wall-tangential set — so the
      existing constrained Leray projector and the symmetric `P_S Δ₁ P_S` restriction cover
      the body with no new machinery (it reads the registry off the metric, like B5). Verified:
      the solid set is exactly the cube's boundary edges; an immersed solid block pins its
      edges to zero (no-slip + no-penetration) while the flow goes around it divergence-free.
      The wall-only / periodic paths stay bit-identical (sort+dedup is a no-op there).
      *Deferred refinements (documented):* slip (tangential-only via the fragment normal),
      moving immersed surfaces (prescribed values through the existing lift), and
      aperture-resolved no-slip on the sub-cell cut face (`Cut` cells currently carry flow,
      with their blockage already in the cut star).
- [x] B-foundation Cut-aware Hodge star: `build_star_diagonal(complex, k, clip)` extracted
      from `hodge_star_matrix` (behaviour-preserving); the per-cell dual clip is the integer
      `axis_boundary_clip` unless a registry is attached, then the continuous
      `CutCellRegistry::dual_fluid_fraction`. Empty registry ⇒ star byte-equal to Stage-3
      (proved across unit/uniform/per-axis/graded, open+periodic, 2D+3D).
- [x] B5 Solver wiring: the `CutCellRegistry` attaches to the **geometry** via
      `CubicalReggeGeometry::with_cut_cells` — the concrete immutable `Context` carrier the
      manifold borrows (D10) — so every Hodge-star read (compiled stencils, constrained Leray,
      codifferential) sees the body transparently with **no new solver plumbing**. Cut-aware
      CFL is satisfied by the existing global `max_speed` scan (covers cut-adjacent edges);
      seeding flows through the cut star automatically. Verified: an empty registry marches
      **bit-identically** to the plain geometry, and a solid-cell registry keeps a convergent
      divergence-free march (`cut_cell_wiring_tests.rs`).
- [x] B6 Equivalence — **done**. (1) The no-body case: an empty registry is bit-identical to
      Stage-3 at both the star level and the marched-solver level (Poiseuille unchanged). (2) The
      axis-aligned-solid-layer-reproduces-the-wall-solver case: an immersed solid cell layer pins
      a vertex-collocated no-slip wall and the fluid below develops the **exact** Poiseuille
      parabola for its reduced height — to rounding (< 1e-8), the same analytic profile the
      vertex-collocated wall solver is validated against
      (`cut_cell_wiring_tests::axis_aligned_solid_layer_reproduces_the_wall_poiseuille`). The
      immersed layer and a real wall yield the identical exact steady state.
- [x] B7 Group gate: format, clippy clean, full physics + topology tests both feature configs green
      (the shipped Group B work — B1–B6 + the cut-aware star — is committed). Commit message prepared
      per the golden rule; the user commits.

## C. First MaybeUncertain data zone (uncertain-inflow-zone)

> **Prerequisite (LANDED 2026-06-14):** `generalize-uncertain-over-realfield` is archived;
> `MaybeUncertain<R>` over `R: RealField` is shipped (living specs `uncertain-realfield-generic`
> + `rand-realfield-sampling`), so the inflow patch composes with the solver's `R` without an
> `R → f64` cast. Group C consumes the shipped API directly. f64 behavior is preserved
> bit-for-bit, so the f64 cylinder validation is unaffected.
>
> **Foundation generalization (done this group):** the `MaybeUncertain<T>` *collapse* and the
> `Uncertain<T>` *reduction* were still f64/Float106/bool-duplicated. They are now hoisted to
> single generic impls (`deep_causality_uncertain`): `lift_to_uncertain` over
> `T: ProbabilisticType`; `sample`/`sample_with_index`/`take_samples` over `T: ProbabilisticType`
> (via the already-generic `Sampler<T>` + `T::from_sampled_value`); `expected_value`/
> `standard_deviation` over `T: ProbabilisticType + RealField + FromPrimitive`. Behaviour is
> bit-identical at f64 (existing tests pass through the generic methods); the zone collapses
> `MaybeUncertain<R> → Uncertain<R> → R` with no cast island.

- [x] C0 Consume the shipped `MaybeUncertain<R>` API at the solver's precision. `deep_causality_uncertain`
      is a std-gated optional dependency of `deep_causality_physics` (`dep:` under the `std`
      feature); the zone is precision-generic over `R: DecNsScalar + ProbabilisticType`.
- [x] C1 `UncertainInflowZone` (`dec/uncertain_inflow/`): tags a prescribed inflow boundary (a
      moving wall — the Dirichlet boundary the solver already supports) fed by a
      `MaybeUncertain<R>` stream; per-step `lift_to_uncertain(threshold, confidence, epsilon,
      max_samples)` then `expected_value(collapse_samples)` to a presence-confirmed `R`. The
      zone's last-good value lives in mutable `State` (`InflowMarchState`, D10); the zone +
      stream are immutable `Context` (`InflowContext`, D10).
- [x] C2 Dropout path: `Err(PresenceError)` (or a non-finite mean) substitutes the last-good /
      configured-default inflow via `.intervene` (a logged `!!ValueAlternation!!`) and records
      the dropout in the `EffectLog` at the configured `DropoutVerbosity` (default `EachDropout`;
      `Transitions` throttles to onset/recovery).
- [x] C3 The solver core stays `R` and **unchanged** — the per-step value reconfigures the
      boundary through the existing `with_moving_wall` builder; `step(&self, field)` is untouched
      and stateless, the monad carries the state, and no `MaybeUncertain` enters the rate/step
      signatures (the collapse happens in the `inflow_march_step` bind stage, above `step`).
- [x] C4 Tests (`physics dec/uncertain_inflow_tests`, fast): no-dropout stream reproduces the
      deterministic moving-wall control to rounding and logs nothing; dropout stream completes via
      a logged `intervene`; `Transitions` verbosity logs only onset/recovery; memory cost is
      confined to the patch (the marched field stays the plain edge cochain).
- [x] C5 Group gate: format, clippy (0 warnings), full physics + uncertain tests both feature
      configs, the cut-cell cylinder example rewritten as the causal-monad march; commit message
      prepared. (User commits per the golden rule.)

## D. Validation — 3D cylinder Re 100–3900 (cut-cell-validation)

- [x] D1 `examples/avionics_examples/dec_cylinder_wake`: the cut-cell cylinder **harness** —
      `from_primitive` disk geometry (A4) + cut star (B5) + immersed no-slip (B4), marched in a
      periodic channel driven by a body force, streaming CSV per step (`step, t, KE, max_speed,
      div_residual, v_probe`) and a shedding-Strouhal estimate. Drives the flow with a body
      force because the solver has **no inflow/outflow BC yet** (that is Group C); includes a
      documented volume-fraction small-cell merge guard (placeholder for B1–B3).
- [x] D2/D3 Re-ladder vs Williamson / Lehmkuhl et al. (2013): all prerequisites now **shipped**
      (`add-boundary-zone-abstraction`: net-flux projection + `Inflow`/`Outflow`;
      `add-slip-boundaries-and-surface-forces`: `SlipWall` far-field + pressure surface force). The
      **isolated-cylinder harness is built** (`examples/avionics_examples/dec_cylinder_validation`):
      west `Inflow` / east `Outflow` / far-field `SlipWall` top-bottom / immersed cut cylinder, all
      composed via `with_zones`. **Verified:** it marches stably and is **interior-divergence-free
      to ≈ 1e-15** (the global residual is just the open-boundary inlet flux) — the composed
      primitive stack is correct (the gate for this change). **2D Re=100 validation DONE** (in the
      now-archived `add-aperture-resolved-noslip`): symmetry-breaking trigger + viscous (friction)
      traction (`viscous_surface_force`, Kirkpatrick one-sided wall-normal Δh) + a developed
      von-Kármán street — the aperture-resolved body sheds at **16 cells/D** (St ≈ 0.154, near
      Williamson 0.164) where the staircase stays steady. **DEFERRED to a follow-up validation change
      (compute-bound):** the 3D high-Re Re-ladder — the transition rung (Re ≈ 200–300) and Re ≈ 3900
      by DNS — vs Lehmkuhl et al. (2013). Not a blocker for this Stage-4 infrastructure change.
- [x] D4 Cheap CI regression rungs (no heavy march): geometric exactness — disk cut volumes
      sum to the exact `domain − π r²` (`cut_cell::consistency_tests`) + per-primitive f64 /
      Float106 exactness (`cut_cell::intersection_tests`); axis-aligned-cut consistency — empty
      cut star byte-equal to Stage-3 + empty-registry march bit-identical
      (`cut_cell::cut_star_tests`, physics `cut_cell_wiring_tests`). The small-cell stability
      smoke test lands with the B1–B3 stabilizer.
- [x] D5 Group gate: format, clippy, both feature configs green for the shipped rungs; example
      README written (now carries the aperture-resolved 16/D gate-result table). The 2D Re=100
      validation numbers are recorded; the deferred 3D high-Re Re-ladder is carved out to a follow-up
      validation change. Stage-4 cut-cell + immersed-boundary infrastructure is complete.
