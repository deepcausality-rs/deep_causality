# CFD Roadmap: From Gap Closure to 3D Causal Fluid Dynamics

Status: sequencing note across the three CFD documents. 2026-06-10.

## Document map

| Note | Altitude | Role |
|---|---|---|
| `cfd-gap.md` | Mathematical core | **Ground truth.** Gaps G1–G6 and their closure inside the uniform math; the periodic DEC-native solver; the validation ladder. |
| `causal_cfd.md` | Platform vision | Cut-cell industrial solver, the four amplifiers, NASA Vision 2030 frame, Phases 1–4. Revised 2026-06-10 against cfd-gap. |
| `3DCausalFluidDynamics.md` | Analysis pipeline | `HodgeDecomposition` → `FluidSignature` → `RollingHistory` → SURD attribution. Revised 2026-06-10 against cfd-gap. |
| `variable-grid-geometry.md` | Mesh program | Variable/graded/adaptive meshes via the topology–geometry separation. R1 (graded metrics) lands with Stage 3; R2 (metric adaptation, causal indicator) with Stage 4; R3 (topological AMR) at Stage 5. |
| `references.md` | Bibliography | Exact citations for the whole deck: Teschner-group practitioner evidence, NASA Vision 2030, DEC foundations (Hirani, MHS, Elcott), metric adaptation (Loseille–Alauzet), validation reference data (Ghia, Driver–Seegmiller, Lehmkuhl, Re-1600 TGV), causal methodology (Martínez-Sánchez & Lozano-Durán). |
| `cfd-roadmap.md` | This file | Which note ships when, and why. |

## The three synergies that fix the ordering

1. **The solver computes the analysis pipeline's input for free.** `leray_project`
   evaluates the gradient half of the Hodge decomposition every time step; the
   `FluidSignature` pipeline consumes a `HodgeDecomposition<R>` on the same
   `Manifold`, same `R`. Therefore the analysis tap comes *immediately after* the
   solver, not after the industrial platform.
2. **The projector needs one gauge-fixed CG solve** (`P(ω) = ω − dΔ₀⁻¹δω`), so the
   periodic solver has no dependency on the G6 harmonic-deflation fix — only the
   full-decomposition tap does. The solver and G6 can proceed independently.
3. **One type-state carries both programs.** `SolenoidalField<R>` (divergence-free by
   construction; constructed only by `leray_project` / `from_hodge_projection`) is
   simultaneously the solver's march invariant and the pipeline's carrier, and it
   deletes corrective intervention 10.4 from `causal_cfd.md` by making the failure
   unrepresentable.

## Stages, first to last

### Stage 0 — Foundations: close the cfd-gap gaps
**Source:** `cfd-gap.md` G1–G4, G6. **Crates:** topology, num/haft (isos), physics (typed forms).

- G1 wedge + interior product (the only new mathematics; the critical path of the
  entire program; Leibniz/Cartan property tests as acceptance).
- G2 de Rham/♯ isos (law-tested via `iso::test_support`).
- G3 typed-form carriers incl. the unified `SolenoidalField<R>`.
- G4 sign/orientation conventions, pinned and tested.
- G6 harmonic-kernel deflation in `hodge_decompose` (small; gates only Stage 2's
  torus tap, scheduled here because it is bounded and shares the CG context).

**Why first:** every later stage consumes these APIs; nothing here depends on
anything later. **Exit:** operator law tests green at f32/f64/Float106.

*Status: **complete 2026-06-11*** — implemented as
`openspec/changes/add-dec-solver-foundations`; exit criterion met (operator law
tests green at all three precisions), plus the §6-style DEC-vs-pointwise
cross-validation already wired in CI. G6 closed by empirical pinning rather
than deflation (see the change's design D6).

### Stage 1 — The periodic DEC-native solver
**Source:** `cfd-gap.md` §5.4, §7 (validation ladder). **Depends on:** Stage 0 (G1–G4).

Rate closure → `Rk4` arrow march → `bind: leray_project` → `bind: cfl_check` →
`iterate_until`. Validation ladder: 2D Taylor–Green (analytic) → 2D-in-3D TG
(analytic, all 3D paths) → inviscid energy/helicity conservation → 3D TG at Re 1600
vs. published DNS dissipation data.

**Why second:** it is the smallest artifact that proves the composition claim
end-to-end, and it is the data source for Stage 2. **Exit:** convergence tables in
CI; Re-1600 dissipation curve at 64³–128³. **Strategic output:** the CFD community
challenge entry and the seed of the CPC/AIAA software paper.

*Status: **complete 2026-06-11*** — implemented as
`openspec/changes/add-dec-periodic-ns-solver`. The solver lives in
`deep_causality_physics::theories::fluid_dynamics::dec`
(`DecNsSolver`/`DecNsRate`, the `SolenoidalField` type-state step, run
loops, seeding, diagnostics, two-convention pressure recovery); CI carries
the 2D Taylor–Green convergence table (observed spatial order ≥ 1.9 at
f64/Float106), the 2D-in-3D rung, the inviscid energy/helicity drift gates,
and the double shear layer with the Q-criterion tap; the Re-1600 case ships
as `examples/avionics_examples/dec_taylor_green_re1600` (precision a
parameter, causal-flow staged). One finding recorded in the change's design
D4: the validation ladder falsified the Chorin post-step projection (it
bleeds energy at first order in `dt`), so the projector moved **inside**
the `Rk4` stages — exactly the `∂u♭/∂t = P(…)` equation of `cfd-gap.md` §2
— at ~4 CG solves per step.

### Stage 2 — The causal-analysis tap
**Source:** `3DCausalFluidDynamics.md` B1b → B1c → B2 → B3 (B4 ships in Stage 0's G3;
B5 is an audit, not construction). **Depends on:** Stage 0 (G6 for torus data);
solver output from Stage 1 for in-house flows.

`FluidSignature` extraction → `RollingHistory` lift → SURD wiring → the B3 synthetic
ground-truth gate. With the solver tap, `simulate → decompose (free) → attribute`
closes the loop that `causal_cfd.md` §8 originally deferred as research-flagship
territory.

**Why third (and not later):** the marginal cost collapsed — the expensive input is a
Stage 1 byproduct, B5 already shipped, B4 merged into G3 — while the output is the
program's unique differentiator and a publication path squarely on the project's
causal-discovery thesis. **Parallelism:** B1b–B3 run on *open-lattice* data today;
they may proceed concurrently with Stage 1, converging on the torus tap when both
land. **Exit:** B3's synthetic ground-truth test passes (the load-bearing
methodological gate). **Strategic output:** the causal-attribution-of-simulated-flow
result; second branch of the deferred JHU validation note.

### Stage 3 — Walls
**Source:** `cfd-gap.md` G5; `causal_cfd.md` Phase 1 (revised). **Depends on:** Stage 1.

Boundary-corrected Hodge star duals, Neumann–Poisson projection path, no-slip
Laplacian rows. Validate analytic-first on laminar Poiseuille (periodic x, walls y),
then lid-driven cavity vs. Ghia et al. (1982). Ship the minimum corrective library
(10.1, 10.2, 10.10) as upgrades of the binds already in the Stage 1 chain.

**Why fourth:** G5 is the one place new substrate must be built (≈ G1+G2+G3 combined
in effort) and nothing in Stages 1–2 waits on it; deferring it front-loads the
challenge entry and the publication while losing nothing — it is purely additive.
**Exit:** Ghia-table agreement. **Strategic output:** `causal_cfd.md` Phase 1
complete; entry point for the grant frame.

*Status: **complete 2026-06-14*** — implemented as
`openspec/changes/archive/2026-06-12-add-walls-and-dec-stencils` (boundary-corrected
Hodge star with clipped dual volumes, DCT-accelerated Neumann–Poisson projection,
no-slip viscous rows, mixed-periodicity solver wiring), with the compiled-stencil and
spectral-diffusion perf track folded into the same change. Validated analytic-first on
laminar Poiseuille (exact parabolic steady state) and against Ghia et al. (1982) at
Re 1000 via `examples/avionics_examples/dec_lid_cavity_re1000` (coarse centerline RMSE
gated in CI; the refinement trend runs in the example, per the tests-fast /
examples-verify split). Re 10⁴ is deliberately held for the R1 graded-metric rung
(`variable-grid-geometry.md` §R1) — a uniform mesh cannot resolve the Re-10⁴ boundary
layer — now **in preparation** as `openspec/changes/add-graded-metrics` (per-axis
geometric/tanh stretching constructors on the existing four-level `PerEdge` metric;
exact-structure-at-any-grading headline; independent of Stage 2, composes with Stage-4
cut cells). A Stage 1 follow-up shipped alongside: `fix-dec-convective-instability`
(archived 2026-06-14, spec `dec-ns-stability`) skew-symmetrized the convective term in
the vector slot, removing the under-resolved-turbulence energy-growth instability and
unblocking the Re-1600 error-per-cost benchmark (`error-per-cost-vs-spectral.md`).

### Stage 4 — Cut cells + first probabilistic zone
**Source:** `causal_cfd.md` Phase 2 (§4.2, §4.10). **Depends on:** Stage 3.

`CutCell<D>`, cube–triangle intersection, small-cell stabilisation, wall BCs on cut
faces; first `MaybeUncertain` zone (sensor-fed inflow with dropout, composing with
intervention 10.3). Validate on the 3D cylinder (Re 100–3900).

**Why fifth:** cut cells presuppose the wall machinery of Stage 3; the industrial
moat starts here. **Exit:** cylinder validation. **Strategic output:** the §3.1
probabilistic-zone amplifier demonstrated; competitive with research cartesian solvers.

*Status: **in preparation 2026-06-14*** — taken up now, ahead of Stage 2, because it
is purely additive substrate and **independent of Stage 2** (see the cross-impact note
below): the analysis tap is a read-only consumer of `HodgeDecomposition<R>` / `Manifold`
output, so the dependency runs substrate → analysis and never the reverse, and the
topology/geometry separation routes cut-cell geometry into Stage 2 transparently through
`⋆` if and when both land. The only place Stage 2 feeds the Stage-4 neighborhood is the
causal mesh-adaptation indicator, which is the deferred R2 *companion* rung
(`variable-grid-geometry.md` §R2), not part of the cut-cell core. The rung that does
interact with Stage 4 is R1 graded metrics (axis-aligned wall-normal grading, kept
first-class by the cut-cell constructors) — a separate axis from Stage 2. Proposal
scaffold: `openspec/changes/add-cut-cells-and-immersed-boundaries`.

### Stage 5 — Industrial scale-out
**Source:** `causal_cfd.md` Phases 3–4; `3DCausalFluidDynamicsValidation.md` (to be
opened). **Depends on:** Stage 4 (Phase 3); Stage 2 (validation note).

Compressible wiring + shock capturing + k-ω SST + NASA CRM (Phase 3); conjugate heat
transfer, AMR, FSI (Phase 4, provisional); the JHU reproduction with the in-house
solver branch added. Preconditioning for the CG (first performance item). GPU per the
Candle path and the MLX lessons; cluster as its own change set.

**Why last:** everything here is scale, closure models, and market reach — none of it
alters the core that Stages 0–3 fix, and all of it benefits from the validation
credibility those stages accumulate.

## Dependency picture

```text
Stage 0 (G1–G4) ──► Stage 1 (periodic solver) ──► Stage 3 (walls) ──► Stage 4 (cut cells) ──► Stage 5
Stage 0 (G6) ──────► Stage 2 (analysis tap) ◄──── Stage 1 (data source)
                     Stage 2 ∥ Stage 1 on open-lattice data
```

The critical path is G1 → Stage 1 → Stage 3 → Stage 4. Stage 2 hangs off the side of
Stage 1 at low marginal cost and produces the program's most differentiated output;
schedule it eagerly, not lazily.

### Cross-impact: Stage 2 and Stage 4 are independent (assessed 2026-06-14)

With Stages 0/1/3 complete, the two remaining near-term stages — 2 (analysis tap) and
4 (cut cells) — are mutually independent, so they can land in either order. The
assessment, recorded here because it gates the decision to take Stage 4 first:

- **Stage 2 is a read-only consumer of solver/geometry output.** Its blocks (B1b–B5)
  ingest a `HodgeDecomposition<R>` and a `Manifold<K, R>` and emit
  `FluidSignature → RollingHistory → SURD`. They author no operator, metric, lattice,
  or BC. Stage 4 modifies exactly those substrate objects. The dependency therefore
  runs substrate → analysis and never the reverse.
- **The topology/geometry separation makes cut-cell geometry transparent to Stage 2.**
  All geometry enters through `⋆` (`variable-grid-geometry.md` §2); cut cells are
  clipped volumes plus aperture weights inside `⋆`. Stage 2 reads geometry only through
  the star, so it picks up cut-cell-corrected values for free, in whichever order the
  two stages ship — no interface conformance is imposed either way.
- **No shared mutable type.** Stage 2's only solver-shared carrier is
  `SolenoidalField<R>` (B4), already shipped. Cut cells add operators, not field
  carriers, so there is no three-way type conflict.
- **The one coupling is deferred and out of Stage 4's core.** `variable-grid-geometry.md`
  §R2's *causal* mesh-adaptation indicator consumes Stage 2 attribution — but R2 is the
  medium-term Stage-4 *companion* rung, not the cut-cell deliverable. Taking Stage 4
  first simply leaves that indicator unwired until both R1 and Stage 2 exist; zero
  rework to the cut-cell core.

**Decision:** take Stage 4 now (additive industrial-moat substrate, no Stage 2
prerequisite); keep Stage 2 scheduled eagerly as the differentiated output, to follow.

## Strategic checkpoints

| After stage | Available artifact |
|---|---|
| 1 | CFD community challenge entry; software-paper seed (composition + verification claim, divergence-free-by-type, precision ladder) |
| 2 | Causal attribution of simulated flow — the flagship loop no other CFD code can run; publication on the causal-discovery thesis |
| 3 | `causal_cfd.md` Phase 1 complete: the lid-driven-cavity milestone that opens the grant frame (Innovate UK / ATI posture per `causal_cfd.md` §0.2) |
| 4 | Industrial-relevant geometry handling + the `MaybeUncertain` amplifier demo |
| 5 | Vision-2030-aligned platform story at full scope |
