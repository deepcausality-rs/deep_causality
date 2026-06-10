# CFD Roadmap: From Gap Closure to 3D Causal Fluid Dynamics

Status: sequencing note across the three CFD documents. 2026-06-10.

## Document map

| Note | Altitude | Role |
|---|---|---|
| `cfd-gap.md` | Mathematical core | **Ground truth.** Gaps G1–G6 and their closure inside the uniform math; the periodic DEC-native solver; the validation ladder. |
| `causal_cfd.md` | Platform vision | Cut-cell industrial solver, the four amplifiers, NASA Vision 2030 frame, Phases 1–4. Revised 2026-06-10 against cfd-gap. |
| `3DCausalFluidDynamics.md` | Analysis pipeline | `HodgeDecomposition` → `FluidSignature` → `RollingHistory` → SURD attribution. Revised 2026-06-10 against cfd-gap. |
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

### Stage 4 — Cut cells + first probabilistic zone
**Source:** `causal_cfd.md` Phase 2 (§4.2, §4.10). **Depends on:** Stage 3.

`CutCell<D>`, cube–triangle intersection, small-cell stabilisation, wall BCs on cut
faces; first `MaybeUncertain` zone (sensor-fed inflow with dropout, composing with
intervention 10.3). Validate on the 3D cylinder (Re 100–3900).

**Why fifth:** cut cells presuppose the wall machinery of Stage 3; the industrial
moat starts here. **Exit:** cylinder validation. **Strategic output:** the §3.1
probabilistic-zone amplifier demonstrated; competitive with research cartesian solvers.

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

## Strategic checkpoints

| After stage | Available artifact |
|---|---|
| 1 | CFD community challenge entry; software-paper seed (composition + verification claim, divergence-free-by-type, precision ladder) |
| 2 | Causal attribution of simulated flow — the flagship loop no other CFD code can run; publication on the causal-discovery thesis |
| 3 | `causal_cfd.md` Phase 1 complete: the lid-driven-cavity milestone that opens the grant frame (Innovate UK / ATI posture per `causal_cfd.md` §0.2) |
| 4 | Industrial-relevant geometry handling + the `MaybeUncertain` amplifier demo |
| 5 | Vision-2030-aligned platform story at full scope |
