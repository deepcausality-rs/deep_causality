## Context

Stage 4 of `cfd-roadmap.md`. The Stage-3 wall substrate already clips dual volumes at
axis-aligned walls inside the Hodge star: `has_hodge_star.rs` applies a `boundary_clip`
factor `2^{-b}` per open-axis boundary incidence (a face dual is halved, an edge dual
quartered, a 3D-corner dual eighthed). Cut cells are the **fractional-aperture
generalization of that same integer clip** — an immersed surface replaces the integer
`2^{-b}` with a continuous wetted fraction and a clipped cell volume. This is why cut
cells are additive substrate, not a rewrite: they ride the existing volume → star →
operator dispatch.

The geometry is already representation-complete for this. `CubicalReggeGeometry<D, R, S>`
stores edge lengths as a four-level union `UnitEdge | Uniform | PerAxis | PerEdge { Vec<R> }`,
and `cell_volume` / `top_cell_volume` / the star diagonal already dispatch all four
(`mod.rs`, `volumes.rs`, `has_hodge_star.rs`). Cut cells add a *per-cell volume/aperture
override*, consumed by the same code paths.

The solver substrate is also in place: the constrained Leray projection, the no-slip
viscous rows (symmetric restriction `P_S Δ₁ P_S`), wall-aware CFL, and the moving-lid
lift all ship from Stage 3 (`wall-bounded-ns`, `no-slip-viscous`, `dec_ns_solver/no_slip.rs`).
Immersed no-slip is those same constraints applied on cut-face fragments instead of
axis-aligned wall edges.

The uncertain machinery is f64-native: `MaybeUncertain<f64>` (not generic over `R`)
carries `is_present: Uncertain<bool>` + `value: Uncertain<f64>`, with
`from_bernoulli_and_uncertain`, `sample() -> Option<f64>`, and the SPRT-gated
`lift_to_uncertain(threshold, confidence, epsilon, max_samples) -> Result<Uncertain<f64>, _>`
that returns `PresenceError` when presence evidence is insufficient. That gate is the
native dropout detector.

## Goals / Non-Goals

**Goals**
- A `CutCell<D>` geometry carrier and the cut-aware volume/aperture overrides that feed
  the existing star/operator dispatch, with axis-aligned cut reducing exactly to the
  Stage-3 integer wall clip (consistency gate — Poiseuille/Ghia must not move).
- Cube ↔ analytic-primitive intersection (cylinder/sphere/plane) and cube ↔ triangle
  (STL) producing apertures + cut-face fragments with outward normals.
- Small-cut-cell stabilization that restores a usable CFL bound near vanishing cuts.
- Immersed no-slip / slip wall BC on cut faces, wired through the existing constrained
  projection and no-slip stage.
- The first `MaybeUncertain` data zone: a sensor-fed inflow patch with native dropout
  handling composing with the BC-fallback `.intervene` pattern (§10.3).
- 3D cylinder validation (Re 100–3900) against Lehmkuhl et al. (2013) / Williamson.

**Non-Goals**
- High-Re wall functions on cut faces (Stage 5, with RANS).
- AMR / octree refinement (R3, Stage 5). Cut cells redistribute boundary geometry on a
  fixed lattice; they do not add DOF.
- A `deep_causality_io` crate. STL reading is a thin example-level utility.
- Compressible cut-cell terms (Stage 5).
- R1 graded metrics and R2 causal adaptation — *forward-compatible with*, not built
  here (see D7). R1 is independently shippable (see the proposal's R1 note); this change
  does not depend on it but must not foreclose it.

## Decisions

### D1: `CutCell<D>` is a per-cell geometric overlay, not a new complex
The lattice and its connectivity are unchanged; a cut cell is a record keyed by lattice
`cell_id` carrying: clipped fluid volume, per-face aperture (wetted fraction in `[0,1]`),
the list of cut-face fragments (each: area, outward unit normal, source-geometry tag),
and a `Fluid | Cut | Solid` classification. A `CutCellRegistry<D>` maps the (sparse) set
of intersected cells to their `CutCell<D>`. The registry is consulted by the cut-aware
volume/aperture accessor; cells absent from it are full fluid cells and hit the existing
uniform fast path unchanged. **This keeps the cut data O(boundary cells), not O(volume).**

### D2: Cut geometry lives in `deep_causality_topology`
Per the crate-responsibility rule (`3DCausalFluidDynamics.md` §6: geometry is topology),
`CutCell<D>`, `CutCellRegistry<D>`, the intersection routines, and the cut-aware
volume/aperture overrides sit beside `CubicalReggeGeometry`. The star already reads
per-cell volumes; cut support is a volume/aperture override on that read, generalizing
`boundary_clip` from `2^{-b}` to a continuous fraction. No physics-crate dependency is
added by the geometry layer.

### D3: Intersection is analytic-primitive-first, STL-second
The cylinder validation case needs only cube ↔ infinite-cylinder (analytic) intersection,
which is exact and cheap — build that first and validate against it. Cube ↔ triangle
(for STL) lands second as the general path; STL parsing is a small example-level reader,
not a new crate (defers `causal_cfd.md` §4.9 I/O and open question 3, STL-first).

### D4: Small-cell stabilization — both prototyped, one selected on evidence
This is the load-bearing design decision (`causal_cfd.md` §7 Q5) and an **Open Question**
below. Cell-merging (Berger–Helzel: merge a small cut cell into a full neighbor, solve on
the union) and flux-redistribution (Colella–Graves–Modiano: redistribute the conservative
update of a small cell over neighbors) are both prototyped on the cylinder case; the one
with the better Strouhal/drag accuracy at acceptable complexity is committed, and the
decision is recorded in this design before the validation gate closes. Stabilization is
expressed as a named corrective intervention where it fits the `.intervene` pattern
(`causal_cfd.md` §3.3 item 7).

### D5: Immersed wall BC reuses the Stage-3 no-slip machinery
No-slip on a cut face is the symmetric restriction `P_S Δ₁ P_S` (already in
`dec_ns_solver/no_slip.rs`) applied to the edges adjacent to cut-face fragments, with the
constraint direction set by the fragment normal rather than the axis. Slip is the
tangential-only variant. The constrained Leray projection is unchanged — it already
accepts a constraint set; cut faces extend that set. Moving immersed surfaces reuse the
moving-lid lift (a prescribed boundary velocity on the constrained edges).

### D6: The uncertain inflow zone is a precision-generic boundary patch that collapses to `R`
Selective typing only. A `UncertainInflowZone` tags a set of inflow boundary cells whose
values arrive as `MaybeUncertain<R>` (sensor stream), where `R` is the solver's precision.
This depends on the `generalize-uncertain-over-realfield` prerequisite — without it,
`MaybeUncertain` is f64-only and the patch would force an `R → f64` cast island at the
boundary; with it, the sensor patch is `R`-typed end to end and collapses to `R` for
assembly with no cast. Per step the patch calls
`lift_to_uncertain(threshold, confidence, epsilon, max_samples)`:
- `Ok(uncertain)` → take a presence-confirmed value (mean / sample) as the Dirichlet
  inflow `R` for assembly;
- `Err(PresenceError)` → **dropout**: fire the BC-fallback corrective intervention
  (§10.3) — substitute the last-good inflow (or a configured default) via `.intervene`,
  and record the dropout in the `EffectLog`.

The solver core and any downstream analysis tap stay `R: RealField`; the uncertain types
never enter the march. The 8–16× memory cost applies only to the tagged inflow patch
(`causal_cfd.md` §2.7), so the hybrid-storage concern is bounded to the boundary.

### D7: Forward-compatible with R1 graded metrics; foreclosed by nothing
The cut-cell volume/aperture override composes with `PerAxis`/`PerEdge` grading because
both feed the same `cell_volume` dispatch — a graded *and* cut cell is a clipped volume
computed from graded edge lengths. The constructors keep the **axis-aligned wall-normal
case first-class** (`variable-grid-geometry.md` §4) so that wall-normal grading near an
immersed surface is expressible. R1 (graded constructors + operator tests) is
independently shippable and is *not* a prerequisite of this change; if R1 lands first,
the Re-range of the cylinder case can be pushed without changing cut-cell code.

### D8: Validation is analytic-first and example-hosted
Cheap CI regressions: geometric exactness (cut volume + aperture of a cube ⋂ analytic
primitive against closed form), the axis-aligned-cut == Stage-3-wall-clip consistency
gate, and a small-cell stability smoke test (a deliberately tiny cut marches without CFL
blow-up). The heavy Re 100–3900 cylinder ladder (Strouhal, C_d, wake transition) lives in
`examples/avionics_examples/dec_cylinder_wake`, per the tests-fast / examples-verify split
established in Stage 3.

### D9: No breaking change to Stage 1–3 behavior
Cut support activates only when a `CutCellRegistry` is supplied. Fully periodic and
axis-aligned-wall solves take the existing paths untouched. The axis-aligned-cut
consistency gate (D8) pins that the generalized clip reproduces the integer wall clip, so
the Poiseuille and Ghia results are invariant.

## Risks / Trade-offs

- **Small cells are the canonical cut-cell hazard.** Mitigated by D4 (two algorithms
  prototyped, evidence-selected) and the D8 stability smoke test; the risk is accuracy
  near the cut, never conservation (combinatorial `d` is cut-independent —
  `variable-grid-geometry.md` §2).
- **Intersection robustness on STL** (degenerate triangles, near-tangent cuts) is real;
  D3's analytic-first path sidesteps it for the validation gate, and STL hardening is
  scoped to its own group with explicit degeneracy tests.
- **Uncertain-zone memory blow-up** is bounded to the inflow patch by D6.
- **Scope.** This is the largest single CFD change (≈ G1+G2+G3). The tasks are grouped so
  geometry (A) lands and gates before stabilization/BC (B), the uncertain zone (C), and
  validation (D), giving four de-risking landing points.

## Open Questions

1. **Stabilization algorithm (D4):** Berger–Helzel merge vs. Colella–Graves–Modiano flux
   redistribution. Decision deferred to the Group B prototype on the cylinder; both built,
   one committed, recorded here. (`causal_cfd.md` §7 Q5.)
2. **Solver `State` shape:** does the cut-cell registry live in the solver `State` channel
   alongside the time-step controller history, or as immutable `Context`? Cut geometry is
   static for a rigid body (→ Context) but the uncertain-zone last-good value is dynamic
   (→ State). Likely split. (`causal_cfd.md` §7 Q9.)
3. **`EffectLog` granularity for dropout events:** per-step dropout records are cheap;
   confirm a configurable verbosity is not over-engineering for the single inflow patch.
   (`causal_cfd.md` §7 Q13.)
4. **STL reader:** hand-rolled minimal ASCII/binary STL in the example vs. a vetted crate
   — weigh against the repo's `unsafe_code = "forbid"` and dependency posture.
