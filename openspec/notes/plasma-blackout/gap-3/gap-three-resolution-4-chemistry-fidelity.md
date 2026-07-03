<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Resolution 3 — ARIZ re-evaluation of the finite-rate ionization network (lever 3)

> **STATUS (2026-07-02): analysis complete; proposal amended, not yet implemented.** This note
> is the full ARIZ/TRIZ re-evaluation of the Option A design proposed in
> `openspec/changes/archive/2026-07-03-add-finite-rate-ionization-network/` (scoped from
> [`../finite-rate-cfd-chemistry.md`](../finite-rate-cfd-chemistry.md)). Verdict: the
> architecture is **confirmed** at the right knee of the fidelity curve, and the analysis caught
> one substantive design flaw (the partial-equilibrium atom pool) plus one spec-level trap
> (detailed balance under two-temperature rating) before implementation. Three amendments were
> applied to the change artifacts and the change re-validates strict.

## 0. Frame

- **Key problem, no solution words:** the corridor's electron density must respond to the local
  thermodynamic state in *both directions*, building up and decaying, so that blackout onset,
  peak, and exit are consequences of the flow rather than consequences of a fitted target.
- **System:** the per-cell chemistry closure in the corridor's `PhysicsStage` slot, reading the
  evolved `T_tr`, `T_ve`, and `n_tot` from the compressible carrier.
- **Main function:** to *predict* electron density from local state.
- **Constraints treated as fixed:** per-cell quasi-steady evaluation inside the stage slot,
  rates from the Park (1990) tables, LER integration. These survive questioning; they are the
  architecture and they are cheap. The proposal, however, carried a **hidden fixed assumption
  that deserved the lever treatment: the neutral atom pool at partial equilibrium.** That is
  where the analysis bites (A7).

## A. Reformulate

### A1 Components and functions

- Useful: the LER kernel (unconditionally stable closed-form relaxation; the crate's whole
  chemistry contract), the Arrhenius kernel (generic rate form, reusable for dissociation), the
  evolved per-cell carrier state, the RAM-C II flight anchor, the stagnation-line verification
  harness, the counterfactual and parallel table machinery.
- Harmful or insufficient: the Saha equilibrium target (fitted, so every off-anchor number is
  extrapolation), the missing loss channel (no exit mechanism; the wake must be frozen by the
  sheath-renewal workaround), and, in the *proposed* design, the partial-equilibrium atom pool
  (insufficient; argued in A7).
- Supersystem: the corridor examples and the weather-dispersion table, whose gates inherit
  whatever the closure earns.

### A2 Technical contradictions

- **TC-1:** adding rate physics (improves predictiveness: exit, wake, off-anchor validity)
  worsens the printed anchor agreement (calibrated 1.43x into the honest 2x to 3x production
  band) and adds kernels.
- **TC-2:** keeping the calibrated closure keeps the pretty anchor number but forfeits exit
  prediction, the wake, and every off-anchor claim, including the entire weather table.

### A3 Intensify

Push TC-1 to infinity: full species-transported chemistry (Option B), an 11-species network
with charged diatomics. Best fidelity; kills the minutes budget and the change's scope, and
every added channel dilutes the validation story. Push to zero: pure Saha, the original 12x
over-prediction lever 1 already fixed. The extremes expose the knee, and the knee is
**structural, not incremental**: dissociative recombination is the one channel that changes the
model's *topology* by adding a sink. With it, an exit mechanism exists at all; without it, no
amount of rate refinement produces one. Electron impact and the atom pool refine magnitudes.
Option A sits exactly at the knee. TC-1 in its three-channel form best preserves the main
function.

### A4 Conflicting pair

Product: the carried electron population per cell. Tool: the equilibrium target it relaxes
toward. Conflicting states: the target must be *authoritative* (so the relaxation is stable and
closed-form) and must *not be believed* (because the flow never reaches it over one residence
time; believing it is the original 12x sin).

### A5 Resources

All in inventory, nothing new required: the LER kernel absorbs any `(target, tau)` pair; the
Arrhenius kernel already has the exact form dissociation rates need; the equilibrium-constant
curve fit is the single genuinely new kernel form, and the reverse rate is derived from it
rather than added as data; the Park coefficients the fits need overlap the coefficients the
pool needs. The stagnation-line harness and the anchor provide the uncalibrated measurement.

### A6 Operating zone and time

The conflict lives per cell, per coupled step, over one residence time `t_res`. The zone
separation that matters is **by condition, not by space**: wherever a process's rate clock
beats the residence clock, equilibrium closure is correct and free; wherever it does not, the
closure must lag. This observation is the whole resolution (B).

### A7 Ideal Final Result and the physical contradiction

**IFR:** *the chemistry stage, using only the existing LER and Arrhenius kernels plus cited
Park coefficients, predicts electron density in both directions with no fitted target and no
new integrator.* Option A satisfies this IFR with inventoried resources. Confirmed.

**The physical contradiction the proposal missed.** The chemistry-fidelity ladder's founding
insight (lever 1) is: the real flow sits roughly 400x below Saha because ionization is
rate-limited over one residence time. Apply that same test to every sub-closure of Option A:

- The electron population: lagged. Correct, that is the whole design.
- The recombination channel: rated. Correct.
- **The atom pool: proposed at partial equilibrium. Fails the test.** Dissociation is the
  *slowest* relaxation process behind a shock; that is why thermochemical nonequilibrium exists
  as a discipline. Over `t_res = 2e-5 s` at the 61 km condition, N2 and O2 dissociation is
  starved, and the atom fractions sit far below their `T_a` equilibrium. A partial-equilibrium
  pool therefore over-predicts `[N][O]`, and channel 1's production with it, potentially by a
  large factor, reintroducing through the back door exactly the equilibrium optimism lever 1
  removed for the electrons. The original D3 ("start with partial equilibrium, promote to rated
  dissociation only if the stagnation line demands it") had the burden of proof backwards: the
  miss would be built in, discovered at task 3.1, and fixed by the promotion anyway.
- The fix costs nothing structural: a lagged pool is the same LER pattern with
  `tau_pool = 1/(k_d[M])` from the Park dissociation rates, one more `(target, tau)` pair. The
  "promotion" collapses into the base design.

**A second, spec-level catch: two-temperature detailed balance.** The design rates heavy
channels at the controller `T_a` and electron channels at `T_e = T_ve` (both correct, both
recorded insights), and separately claims detailed balance by construction via
`k_b = k_f / K_eq`. The identity only closes when every temperature coincides. At a genuine
two-temperature state the network's fixed point *deliberately* differs from any
single-temperature equilibrium. As originally written, the detailed-balance test could have
been implemented as forcing single-temperature behavior at two-temperature states: wrong
physics passing a wrong test. The scenario must pin the identity in the thermal-equilibrium
limit (`T_tr = T_ve`) and explicitly permit the two-temperature departure.

## B. Solve

Reformulation already cracked it; the matrix was not needed. The resolution is **separation by
condition** (the separation-principle family): equilibrium closure is valid exactly where the
process's rate clock beats the residence clock, and must lag everywhere else. The LER lag *is*
that separation principle in executable form. The proposal already had the mechanism and
applied it to the electrons; it exempted one sub-closure (the atom pool) without justification.
Applying the mechanism uniformly removes the contradiction rather than compromising it: the
target stays authoritative (stable closed-form relaxation) while never being believed faster
than its own kinetics allow.

Alternatives weighed and rejected on the way:

- **Recombination bolted onto the existing calibrated Saha target** (minimal diff): fixes the
  exit topology but keeps the fitted target, so off-anchor numbers stay extrapolation. Named as
  the fallback if the uncalibrated measurement stalls the change, not as the solution.
- **Option B now** (species transport on tensor trains): rejected per the preparation note's
  own sequencing; the decode-react-encode round trip needs its own timing study first.
- **Offline-fitted lookup tables from a richer network**: adds data files and loses the
  prediction-from-cited-rates story that is the point of the change.

## C. Verify and harvest

- **Physical contradiction removed, not compromised:** the target is authoritative and
  disbelieved at the correct rate, uniformly across electrons and atoms.
- **Resources:** only inventoried ones; the IFR holds; no new integrator, no stiff solver, no
  new substances.
- **Implementable:** the fixed point of the three-channel network is a closed-form quadratic in
  `n_e` (production constant plus linear via electron impact, loss quadratic through
  quasi-neutrality); the LER contract carries the rest.
- **New harmful effects to watch (the next problems):** the electron-impact avalanche term near
  peak heating (bounded by the threshold at `T_e = T_ve` and the quadratic loss; unit tests pin
  the frozen limit and the capped approach), and the sheath-renewal question re-opened by a
  real loss channel (settled by the re-run A/B; both outcomes acceptable, the record
  mandatory).

### Amendments applied to `add-finite-rate-ionization-network` (re-validated strict)

1. **D3 inverted:** the atom pool ships lagged with the dissociation-rate clock from day one;
   partial equilibrium is the limit it relaxes toward, not the closure. Task 1.4 updated.
2. **Detailed balance re-scoped:** the identity is pinned in the thermal-equilibrium limit, and
   a new spec scenario requires two-temperature states to depart from single-temperature
   equilibrium by design.
3. **D7 made diagnostic:** the stagnation line is measured channel by channel (channel 1 plus
   the lagged pool first, then electron impact), so a band miss is attributable; task 3.1's
   stop-and-surface point names the attribution and the Zeldovich decision.

### Harvest: the reusable method

**When replacing a calibrated closure with rate physics, every remaining equilibrium assumption
inherits the original sin. Audit each sub-closure with the same rate-versus-residence test that
killed the main one.** The inverse also holds and is free fidelity: wherever the rate clock
beats the residence clock by orders of magnitude, equilibrium closure is exact enough and costs
nothing. Scaling behavior: as `t_res` grows (slower flows, larger standoff), the lagged
closures converge to their equilibrium limits and the audit passes trivially; as `t_res`
shrinks (sharper compression), every equilibrium assumption fails in sequence, slowest process
first, and the audit ranks them by `tau * / t_res`. This method transfers directly to Option B,
where the same question returns as "which source terms may be evaluated in partial equilibrium
per cell."

## Related

- [`../finite-rate-cfd-chemistry.md`](../finite-rate-cfd-chemistry.md): the Option A scope this
  analysis re-evaluated, and the Option B transition commentary the harvest feeds.
- [`chemistry-fidelity-gap.md`](chemistry-fidelity-gap.md): the lever ladder; this note is the
  lever-3 design review.
- [`gap-three-resolution-2-tve-controlled-ionization.md`](gap-three-resolution-2-tve-controlled-ionization.md):
  lever 1 shipped and lever 2 closed; the `T_e = T_ve` insight the network's electron channels
  reuse.
- `openspec/changes/archive/2026-07-03-add-finite-rate-ionization-network/`: the amended proposal, design, specs,
  and tasks this analysis modified.
