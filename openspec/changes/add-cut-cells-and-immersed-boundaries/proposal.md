## Why

CFD Stage 4 per `cfd-roadmap.md` and `causal_cfd.md` Phase 2 (§4.2, §4.10): the
**cut-cell** substrate that lets the uniform cubical lattice carry real geometry, plus
the first selectively-typed `MaybeUncertain` data zone. This is the start of the
industrial moat — "what differentiates a research toy from something that can mesh a
turbine blade" (`causal_cfd.md` §4.2) — and it is the hardest single component of the
program (sized roughly G1+G2+G3 combined, per the same section).

It is taken up now, ahead of Stage 2 (the causal-analysis tap), on the cross-impact
finding recorded in `cfd-roadmap.md` §"Cross-impact: Stage 2 and Stage 4 are
independent": Stage 4 is purely additive substrate, Stage 2 is a read-only downstream
consumer, the dependency runs substrate → analysis and never the reverse, and the
topology/geometry separation (`variable-grid-geometry.md` §2) routes cut-cell geometry
into any future analysis tap transparently through `⋆`. Nothing in Stage 2 alters
Stage 4; Stage 4 unblocks the §3.1 probabilistic-zone amplifier and the 3D-cylinder
validation that Stage 5 (compressible/RANS/CRM) presupposes.

The substrate it builds on already exists from Stage 3: the boundary-corrected Hodge
star already clips dual volumes at axis-aligned walls (`wall-hodge-star`); the
constrained Leray projection and no-slip viscous rows already wire walls into the
march (`wall-bounded-ns`, `no-slip-viscous`); the geometry is per-edge-length over a
fixed lattice (`CubicalReggeGeometry`), so a cut cell is a *clipped-volume / aperture*
state on an unchanged lattice — not connectivity surgery. Cut cells generalize the
axis-aligned wall clip Stage 3 already does to arbitrary immersed surfaces.

## What Changes

- **`CutCell<D>` geometry carrier (NEW, `cut-cell-geometry`).** A partial cube tagging
  the lattice cells an immersed surface intersects: clipped cell volume, per-face
  aperture (wetted-area fraction), cut-face fragments with outward normals and source-
  geometry tags, and a fluid/solid/cut classification. The cut data feeds the Hodge
  star (primal/dual volumes) and the operator flux weights, so the existing DEC
  operator stack consumes it without an API break — the cut clip is the same mechanism
  as the Stage-3 wall clip, extended off axis.
- **Surface intersection (NEW, `cut-cell-geometry`).** Cube ↔ triangle (for STL inputs,
  STL-first per `causal_cfd.md` open question 3) and cube ↔ analytic primitive (cylinder,
  sphere, plane) intersection producing the apertures and fragments above. STL reading
  is example-level / a thin utility, not a new crate (defers `deep_causality_io`).
- **Small-cut-cell stabilization (NEW, `cut-cell-stabilization`).** Arbitrarily small
  cut volumes violate the CFL bound catastrophically; this is the canonical reason
  cut-cell solvers are hard (`causal_cfd.md` §4.2). One of cell-merging (Berger–Helzel)
  or flux-redistribution (Colella–Graves–Modiano) — the selection is the load-bearing
  design decision of this change (Open Question 1; `causal_cfd.md` §7 Q5) and both are
  prototyped on the cylinder before committing.
- **Immersed wall BC on cut faces (NEW, `immersed-wall-bc`).** No-slip Dirichlet and
  slip on the cut-face fragments, wiring into the existing constrained Leray projection
  and no-slip viscous machinery from Stage 3. High-Re wall functions are out of scope
  (deferred to Stage 5 with RANS).
- **First `MaybeUncertain` zone (NEW, `uncertain-inflow-zone`).** A sensor-fed inflow
  boundary consuming a `MaybeUncertain<R>` stream with native dropout handling
  (`causal_cfd.md` §4.10, §3.1 item 1), composing with the BC-fallback corrective
  intervention (§10.3) on the existing `.intervene`/`EffectLog` chain. Selective typing
  only: the uncertain zone collapses to `R` before assembly, so the solver core and any
  downstream analysis tap stay `R: RealField`. **Depends on the
  `generalize-uncertain-over-realfield` prerequisite change**, which makes
  `MaybeUncertain<R>` precision-generic so the inflow patch composes with the solver's
  `R` without an `R → f64` cast island; Group C of this change is sequenced after that
  prerequisite lands.
- **Validation (NEW, `cut-cell-validation`).** Flow around a 3D cylinder at Re 100–3900
  against Lehmkuhl et al. (2013) and the Williamson lineage — Strouhal number, drag
  coefficient, and the laminar→wake-transition behavior — as the stage exit gate. Heavy
  runs live in an example (`examples/avionics_examples/dec_cylinder_wake`), per the
  tests-fast / examples-verify split; CI carries cheap geometric-exactness and
  small-cell-stability regressions.

## Impact

- **Affected specs (new capabilities):** `cut-cell-geometry`, `cut-cell-stabilization`,
  `immersed-wall-bc`, `uncertain-inflow-zone`, `cut-cell-validation`.
- **Affected code:** `deep_causality_topology` (the `CutCell<D>` carrier, intersection,
  cut-aware star/operator weights — geometry is topology's responsibility);
  `deep_causality_physics` (immersed wall BC stage, uncertain inflow zone + dropout
  intervention, cylinder validation harness); `deep_causality_uncertain` consumed, not
  modified. New example crate `dec_cylinder_wake`.
- **No breaking changes to Stage 1–3 behavior.** Fully periodic and axis-aligned-wall
  solves are unchanged; cut cells activate only when an immersed surface is supplied.
  The cut clip reduces to the Stage-3 wall clip on axis-aligned faces (a consistency
  gate), so the existing Poiseuille/Ghia results must not move.
- **Forward-compatibility, not dependency:** the constructors keep the axis-aligned
  wall-normal case first-class so R1 graded metrics (`variable-grid-geometry.md` §R1,
  §4) compose at walls; the R2 causal-adaptation indicator and Stage 2 are *not*
  prerequisites and are not wired here.
