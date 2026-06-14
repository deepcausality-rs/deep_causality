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
- [ ] A5 ~~Cube ⋂ triangle (STL path)~~ **POSTPONED** (Open Question 4 resolved): no STL /
      file reading in this change; intersection is analytic-primitive-only (A4). Deferred to a
      later change with its own degenerate-triangle / near-tangent-cut handling.
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

- [ ] B1 Prototype both small-cell stabilizers on the cylinder slice: Berger–Helzel
      cell-merging and Colella–Graves–Modiano flux-redistribution (Open Question 1).
- [ ] B2 Select one on Strouhal/drag accuracy vs. complexity; record the decision in
      `design.md` D4; express stabilization as a named corrective intervention where it
      fits the `.intervene` pattern.
- [ ] B3 Small-cell stability test: a deliberately tiny cut volume marches without CFL
      blow-up under the chosen stabilizer; an unstabilized control aborts (proves the
      stabilizer is load-bearing, not decorative).
- [ ] B4 Immersed no-slip / slip on cut-face fragments: extend the symmetric restriction
      `P_S Δ₁ P_S` and the constrained Leray constraint set to cut-adjacent edges with
      the fragment normal as constraint direction; moving-surface reuse of the lid lift.
- [ ] B5 Solver wiring: `DecNsSolver` accepts a `CutCellRegistry` as immutable `Context`
      (static rigid-body geometry — D10), cut-aware CFL (global max includes cut-adjacent
      speeds); seeding projects + constrains so the march starts divergence-free and
      no-slip-consistent on the immersed boundary.
- [ ] B6 Equivalence: with an axis-aligned cut registry, the wall-bounded results match
      the Stage-3 no-slip solver to rounding (no behavioral drift).
- [ ] B7 Group gate: format, clippy, full physics + topology tests both feature configs;
      prepare Group B commit message and ask the user to commit.

## C. First MaybeUncertain data zone (uncertain-inflow-zone)

> **Prerequisite (LANDED 2026-06-14):** `generalize-uncertain-over-realfield` is archived;
> `MaybeUncertain<R>` over `R: RealField` is shipped (living specs `uncertain-realfield-generic`
> + `rand-realfield-sampling`), so the inflow patch composes with the solver's `R` without an
> `R → f64` cast. Group C consumes the shipped API directly. f64 behavior is preserved
> bit-for-bit, so the f64 cylinder validation is unaffected.

- [ ] C0 Consume the shipped `MaybeUncertain<R>` API at the solver's precision (prerequisite
      already landed — a build/use check, not a wait gate).
- [ ] C1 `UncertainInflowZone`: tag a set of inflow boundary cells fed by a
      `MaybeUncertain<R>` stream; per-step `lift_to_uncertain(threshold, confidence,
      epsilon, max_samples)` to a presence-confirmed `R` inflow value for assembly. The
      zone's last-good value lives in mutable `State` (D10).
- [ ] C2 Dropout path: `Err(PresenceError)` fires the BC-fallback corrective intervention
      (§10.3) — substitute last-good / configured-default inflow via `.intervene`, record
      the dropout in the `EffectLog` at the configured verbosity (D6: default records each
      dropout + fallback; a knob can throttle to onset/recovery transitions).
- [ ] C3 The solver core stays `R: RealField`; the uncertain types never enter the march
      (compile-level: no `MaybeUncertain` in the rate/step signatures).
- [ ] C4 Tests: dropout stream triggers the intervention and the logged fallback;
      no-dropout stream reproduces the deterministic-inflow control run to rounding;
      memory cost is confined to the tagged patch.
- [ ] C5 Group gate: format, clippy, full physics tests both feature configs; prepare
      Group C commit message and ask the user to commit.

## D. Validation — 3D cylinder Re 100–3900 (cut-cell-validation)

- [ ] D1 `examples/avionics_examples/dec_cylinder_wake`: cube ⋂ cylinder geometry,
      immersed no-slip, stabilized march; streams CSV per step (failed runs keep
      evidence, per the dec-ns-stability pattern).
- [ ] D2 Re 100: steady/periodic shedding onset; Strouhal number vs. Williamson lineage.
- [ ] D3 Re 300–3900: drag coefficient and Strouhal vs. Lehmkuhl et al. (2013); document
      the agreement and the resolution at which it holds.
- [ ] D4 Cheap CI regression rungs (no heavy march): geometric exactness, the
      axis-aligned-cut consistency gate, and the small-cell stability smoke test.
- [ ] D5 Group gate: format, clippy, full tests both feature configs; update the example
      README with the validation numbers; prepare Group D commit message and ask the
      user to commit. Stage 4 exit: cylinder agreement recorded.
