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

The uncertain machinery is now precision-generic (it was f64-native when this change was
drafted; the `generalize-uncertain-over-realfield` prerequisite landed 2026-06-14).
`MaybeUncertain<R>` over `R: RealField` carries `is_present: Uncertain<bool>` +
`value: Uncertain<R>`, with `from_bernoulli_and_uncertain`, `sample() -> Option<R>`, and the
SPRT-gated `lift_to_uncertain(threshold, confidence, epsilon, max_samples) -> Result<Uncertain<R>, _>`
that returns `PresenceError` when presence evidence is insufficient. That gate is the native
dropout detector. Internally a closed `SampledValue { Float(f64), DoubleFloat(Float106),
Bool(bool) }` dispatcher keeps the sample cache, computation graph, and sampler non-generic
(Rust has no generic statics) behind the generic public surface; f64 behavior is preserved
bit-for-bit, so consuming this at the solver's `R = f64` is a no-op on existing numerics.

## Goals / Non-Goals

**Goals**
- A `CutCell<D>` geometry carrier and the cut-aware volume/aperture overrides that feed
  the existing star/operator dispatch, with axis-aligned cut reducing exactly to the
  Stage-3 integer wall clip (consistency gate — Poiseuille/Ghia must not move).
- Cube ↔ analytic-primitive intersection (cylinder/sphere/plane) producing apertures +
  cut-face fragments with outward normals. (Cube ↔ triangle / STL is postponed — see D3.)
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
- STL ingestion and any file reading (postponed to a later change — see D3); consequently no
  `deep_causality_io` crate and no cube ↔ triangle intersection here.
- Compressible cut-cell terms (Stage 5).
- R1 graded metrics (now landed) is *composed with*, not re-derived here (see D7); R2 causal
  adaptation is *forward-compatible with*, not built here. This change does not depend on
  graded grading being active, but composes with it when it is.

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

### D3: Intersection is analytic-primitive-only in this change; STL is postponed
The cylinder validation case needs only cube ↔ analytic-primitive (infinite cylinder, sphere,
plane) intersection, which is exact and cheap — that is the entire intersection scope of this
change. Cube ↔ triangle and STL ingestion (any file reading) are **postponed to a later change**:
no STL parser, no `deep_causality_io`, no on-disk geometry here. This keeps the change inside the
repo's no-new-IO posture and removes the STL degeneracy/robustness surface from the Stage-4 gate
entirely. (Resolves Open Question 4.)

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
The `generalize-uncertain-over-realfield` prerequisite **has landed** (archived 2026-06-14;
living spec `uncertain-realfield-generic`): `MaybeUncertain<R>` is precision-generic over
`R: RealField`, so the sensor patch is `R`-typed end to end and collapses to `R` for assembly
with no cast island. (Before it landed, `MaybeUncertain` was f64-only and the patch would have
forced an `R → f64` cast at the boundary.) The bit-for-bit f64-preservation guarantee from that
change means the cylinder validation (run at `R = f64`) sees no numerical change from the
generalization, and a no-dropout stream reproduces the deterministic-inflow control run
exactly. Per step the patch calls
`lift_to_uncertain(threshold, confidence, epsilon, max_samples)`:
- `Ok(uncertain)` → take a presence-confirmed value (mean / sample) as the Dirichlet
  inflow `R` for assembly;
- `Err(PresenceError)` → **dropout**: fire the BC-fallback corrective intervention
  (§10.3) — substitute the last-good inflow (or a configured default) via `.intervene`,
  and record the dropout in the `EffectLog`.

Dropout logging verbosity is **configurable** (resolves Open Question 3): per-step records are
cheap, but a verbosity knob lets a long run throttle the log (e.g. record every dropout, or
only dropout-onset/recovery transitions). The default records each dropout and its fallback.

The solver core and any downstream analysis tap stay `R: RealField`; the uncertain types
never enter the march. The 8–16× memory cost applies only to the tagged inflow patch
(`causal_cfd.md` §2.7), so the hybrid-storage concern is bounded to the boundary.

### D7: R1 graded metrics have landed with verified second order; compose with cut cells
The cut-cell volume/aperture override composes with `PerAxis`/`PerEdge` grading because
both feed the same `cell_volume` dispatch — a graded *and* cut cell is a clipped volume
computed from graded edge lengths. The constructors keep the **axis-aligned wall-normal
case first-class** (`variable-grid-geometry.md` §4) so that wall-normal grading near an
immersed surface is expressible. R1 (graded constructors + operator tests) **landed
2026-06-14** (living spec `graded-metrics`), and its MMS study established that **both** march
operators — convective `i_X ω` and viscous `Δ₀ = δd` — retain second order under smooth
grading, in both the max- and L2-norms, to a 3:1 spacing ratio (only the error constant grows
with grading). So wall-normal clustering near the immersed cylinder is **accuracy-preserving**,
not merely expressible: a graded-and-cut cell rides a verified second-order substrate, and the
cylinder Re-range can be pushed by grading without touching cut-cell code.

**Cochain-consistency discipline (carried from R1, load-bearing for A4/A7).** R1's study first
mis-reported a convective "order-loss" on graded meshes; the cause was a *measurement* bug, not
an operator defect — DEC operators act on **cochains = integrals over cells**, and the MMS had
compared an edge-integral output to a *pointwise* reference while scaling one cochain (`X♭`) by
its edge measure but not the other (`ω`). Invisible on a uniform mesh (`ℓ = 1`), `O(ℓ)`-wrong on
a graded one. A cut aperture/volume is the same kind of object: a *fractional cell measure*. The
exactness tests in Group A therefore compare clipped volumes and apertures against closed-form
**measures** (not pointwise field values), and any quantity fed to the star is the measure the
star expects — the integer `boundary_clip` generalized to a continuous fraction, with no
pointwise/integral mismatch. This is why the A6 axis-aligned consistency gate (`continuous
fraction → 2^{-b}`) is exact: both sides are measures.

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

### D10: Cut registry is `Context` (immutable), uncertain last-good is `State` (mutable)
Applying the standing rule — **immutable data belongs in `Context`, mutable data in `State`**
— resolves the placement (Open Question 2):
- The `CutCellRegistry` is static for a rigid immersed body: built once from the geometry,
  read every step, never mutated by the march → **`Context`**. (A future moving/deforming body
  would promote it to `State`, but that is out of scope here — moving *surfaces* in D5 prescribe
  a boundary velocity on a fixed cut geometry, so the registry stays immutable.)
- The uncertain inflow zone's last-good value is updated each step on a present sample and read
  back on a dropout → **`State`**, alongside the existing time-step-controller history.

This split keeps the static geometry out of the per-step mutation path and confines mutable
state to the genuinely dynamic boundary value.

## Risks / Trade-offs

- **Small cells are the canonical cut-cell hazard.** Mitigated by D4 (two algorithms
  prototyped, evidence-selected) and the D8 stability smoke test; the risk is accuracy
  near the cut, never conservation (combinatorial `d` is cut-independent —
  `variable-grid-geometry.md` §2).
- **Intersection robustness on STL** (degenerate triangles, near-tangent cuts) is real but
  **out of scope** — STL is postponed (D3), so this change carries only the exact analytic
  cuts and none of the STL degeneracy surface.
- **Uncertain-zone memory blow-up** is bounded to the inflow patch by D6.
- **Scope.** This is the largest single CFD change (≈ G1+G2+G3). The tasks are grouped so
  geometry (A) lands and gates before stabilization/BC (B), the uncertain zone (C), and
  validation (D), giving four de-risking landing points.

## Open Questions

1. **Stabilization algorithm (D4):** Berger–Helzel merge vs. Colella–Graves–Modiano flux
   redistribution. **Still open — decision deferred to the Group B prototype on the cylinder**;
   both built, one committed, recorded in D4 before the validation gate closes.
   (`causal_cfd.md` §7 Q5.)

**Resolved:**

2. **Solver `State` shape (Q9) — RESOLVED.** Rule: immutable belongs in `Context`, mutable in
   `State`. The cut-cell registry is static for a rigid immersed body → **`Context`**. The
   uncertain inflow zone's last-good value (and the time-step controller history) is mutated each
   step → **`State`**. See D10.
3. **`EffectLog` granularity (Q13) — RESOLVED.** Dropout verbosity is **configurable**; per-step
   records are cheap, and a verbosity knob is the right call (not over-engineering) so a long run
   can throttle the log. See D6.
4. **STL reader (Q4) — RESOLVED: postponed.** No file reading in this change. Intersection is
   **analytic-primitive-only** (cube ⋂ cylinder/sphere/plane), which is all the cylinder
   validation needs. Cube ⋂ triangle and any STL ingestion are deferred to a later change. See
   D3 and Non-Goals.
