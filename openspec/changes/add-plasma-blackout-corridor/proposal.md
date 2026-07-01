## Why

Gap 2 is closed (Tier-A + Tier-B Stages 0–6 built and gated). What remains between here and a buildable
**plasma-blackout-corridor flagship** ([`plasma-blackout-corridor.md`](../../notes/plasma-blackout/plasma-blackout-corridor.md))
is: the Gap-2 CFD remainders (3-D body-fitted metric, marched-rank re-pin, `CfdFlow` wiring), the Gap-3
trajectory/timing axis, the chemistry-fidelity polish (lever 3), and the Gap-4 composition glue. The scattered
notes sequence this work **feature-first**, which builds several pieces twice — a mock aero kick and a mock
blackout trigger in an early trajectory phase, then a rewrite when the real Tier-B interface lands; a
Cartesian-capture 3-D marcher, then a body-fitted rewrite; Tier-A recovery-temperature reconstructions that
later stages read directly.

This change **reorders the remaining work to build each real component once**, on a **contract-first**
principle: a mock *behind a stable interface* is a one-line swap; a mock the design is *shaped around* is a
rewrite. So we define the seams and promote the proven primitives to libraries **first**, then fill them
bottom-up. The single linchpin is the **blackout coupling interface** (gap item ④) — the per-step
force / heat / electron-density / blackout-flag contract the marcher exposes and both the trajectory kick and
the regime classifier consume. Build that first, and every remaining mock becomes a swappable stub, never a
coupled placeholder.

The work is organized into six dependency-ordered **stages** (0–5). Two Gap-3 primitives are already shipped
(the planar two-body matrix-exponential core, the forward relativistic clock kernel) and three feasibility
studies passed (FS-1/2/3); this change promotes the rest and builds on them.

## What Changes

- **Stage 0 — Foundations & contracts.** Define the **blackout coupling interface** (④) as a real typed seam
  (per-step aero force, heat flux, `n_e`, blackout flag) plus the regime-classifier input and provenance
  schema. Promote the two proven FS primitives to `deep_causality_physics`: the **3-D KS conformal propagator**
  (FS-1, generalizing the shipped planar core) with a between-step perturbation hook, and the **Sp(2,R)/KS
  constraint-projection** kernel (FS-2/B2). Consume the already-shipped forward clock kernel (B3).
- **Stage 1 — CFD real-fidelity.** Complete the compressible-marcher's named remainders (3-D body-fitted
  `MetricProvider`; dynamic marched-rank re-pin) and wire the marcher to **emit the Stage-0 interface** through
  `CfdFlow` — real force / heat / `n_e` / blackout flag, replacing the Stage-0 stub. Optional: chemistry lever 3
  (finite-rate ionization network) to firm peak `n_e` from ~1.1× toward the production band.
- **Stage 2 — Trajectory/nav engine (built once).** The KS-propagate + 17-state tightly-coupled ESKF +
  constraint-projected correct + two-clock carry engine, consuming the Stage-0 interface — no mock/real split,
  because the aero force arrives through the contract. Includes the Encke↔Cowell `select_integrator` regime
  switch (B4), which lights up automatically once Stage 1 delivers real ε through the interface.
- **Stage 3 — Composition.** The regime classifier (Knudsen + `n_e` + GNSS state), the counterfactual
  bank-angle branches (`continue_with`), the **cybernetic bounded-correction gate**
  (`deep_causality_haft::CyberneticLoop` — a deterministic, real-time-capable feedback step whose correction is
  clamped to a verified safety envelope; replaces the corridor §4 [6] Effect Ethos gate for the latency-bound
  inner loop), and the provenance log — filling the Stage-0 seams.
- **Stage 4 — CFD Flow DSL (re)design.** A dedicated design + implementation step. The capability surface has
  grown so much (the ④ coupling interface, the KS trajectory engine, the cybernetic correction gate, the
  counterfactual branches) that the current `CfdFlow` design may no longer hold. Redesign the Flow DSL around
  **elegance, concise expressiveness, and very low overhead** over the underlying causal mechanism, so the
  flagship's **central control loop is expressible in ~10–30 lines of Rust**. A **preliminary design** ships in
  this spec now (the flagship's requirements make it clear enough to draft); minor revision at Stage 4 is
  expected and fine.
- **Stage 5 — Flagship example.** The plasma-blackout-corridor demonstrator itself, built **with the Stage-4
  Flow DSL** — the corridor §4 chain [1]–[7] as one auditable `CausalFlow`, with a self-verifying coupled gate.

No breaking changes; purely additive. `hypersonic_2t` and `ins_gnss_blackout` are untouched.

## Capabilities

### New Capabilities

- `blackout-coupling-interface` (Stage 0): the ④ contract — the per-step physics→navigation seam
  (force / heat / `n_e` / blackout flag), the classifier-input contract, and the provenance schema.
- `ks-conformal-propagator` (Stage 0): the 3-D KS matrix-exponential core + between-step perturbation hook (B1).
- `sp2r-constraint-projection` (Stage 0): projection onto the KS/Sp(2,R) constraint surface (B2 correct step).
- `blackout-marcher-coupling` (Stage 1): the compressible marcher implementing the Stage-0 interface, on the
  completed 3-D body-fitted coordinate + re-pinned marched rank.
- `trajectory-nav-engine` (Stage 2): the KS + ESKF + projection + two-clock + regime-switch engine, built once
  against the Stage-0 interface.
- `blackout-composition` (Stage 3): the regime classifier, counterfactual bank-angle branches, the cybernetic
  bounded-correction gate (`CyberneticLoop`, correction inside a verified safety envelope), and provenance log.
- `blackout-flow-dsl` (Stage 4): the redesigned `CfdFlow` — a thin, static-dispatch, near-zero-overhead
  combinator surface that expresses the whole flagship control loop in ~10–30 lines, with a preliminary design.
- `plasma-blackout-flagship` (Stage 5): the self-verifying flagship example, built with the Stage-4 Flow DSL,
  wiring the corridor §4 chain.

### Modified Capabilities

<!-- None. The shipped two_body / forward_clock kernels and the existing compressible-marcher / body-fitted
     coordinate capabilities are consumed and completed, not re-specified. This change is purely additive. -->

## Impact

- **Crates:** `deep_causality_physics` (KS propagator + constraint-projection kernels + tests + papers);
  `deep_causality_cfd` (the ④ interface type, the marcher adapter, the completed 3-D body-fitted remainders,
  `CfdFlow` wiring); `examples/avionics_examples` (the flagship example + gate). Bazel registration updated.
- **Conventions:** precision-generic, static dispatch (no `dyn`), no `unsafe`, no lib-code macros,
  `[lints] workspace = true`, one-type-one-module, 100% coverage of new library code (examples verified by
  running their gates). Kernels cited in docstring + PDF in `papers/`.
- **Ordering guarantee:** each stage builds only on completed lower stages; the only mocks are Stage-0 stubs
  behind the final interface, swapped (not rewritten) when Stage 1 lands.
- **Out of scope:** the Bars `(4,2)` conformal packaging (optional per FS-1); geopotential harmonics beyond J2;
  IERS 2PN clock terms; a full 6-DOF entry (the representative RAM-C trajectory is used); GPU/parallel
  acceleration (gated behind the tensor-network acceleration survey).
