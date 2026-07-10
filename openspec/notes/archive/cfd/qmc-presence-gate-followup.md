# Follow-up: QMC-compatible presence gate for the uncertain inflow

Date: 2026-06-16. Status: **deferred / design note.** Related:
[qmc-sampler-handover.md](qmc-sampler-handover.md), `deep_causality_uncertain` QMC sampler,
`deep_causality_physics` `UncertainBoundarySource` / `UncertainInflowZone`.

## Context

The sensor-fed inflow resolves each step in two stages
(`UncertainBoundarySource::resolve`):

1. **Presence gate** — `MaybeUncertain::lift_to_uncertain(...)` → `Uncertain<bool>::to_bool(...)`,
   a **Sequential Probability Ratio Test (SPRT)**.
2. **Collapse** — `present.expected_value(n)`, the mean of the present `Uncertain<R>`.

The collapse is a batch estimator over a statically-structured tree, so it now has a QMC variant
(`expected_value_qmc(n, seed)`) and can be wired to the variance-reduced path. **The presence gate
has no QMC variant**, and the inflow therefore stays *MC for presence, (optionally) QMC for collapse*.

## Why SPRT has no QMC variant (the real constraint)

SPRT accumulates a **log-likelihood ratio** over a *sequential* stream of draws and stops as soon as
the ratio crosses an accept/reject boundary. Its correctness and its sample-count guarantees both
rest on the draws being **i.i.d.** (independent increments to the LLR). Quasi-Monte-Carlo points are
the opposite by construction: a Sobol sequence is **deliberately correlated / low-discrepancy**, and
its variance-reduction guarantee is a property of the *whole batch of N points*, not of a prefix.
Feeding a Sobol prefix into an SPRT breaks the i.i.d. assumption the LLR boundary is calibrated on,
so the early-stopping decision is no longer valid. This is a genuine statistical incompatibility, not
an implementation gap — `to_bool` / `estimate_probability_exceeds` / `implicit_conditional`
deliberately have **no** `_qmc` form.

## Options for a QMC-compatible presence test

1. **Fixed-N QMC presence estimate (recommended for full-QMC UQ).** Replace the SPRT gate with a
   fixed-budget probability estimate: `is_present.estimate_probability_qmc(n, seed) >= threshold`.
   This *is* a QMC batch estimator (already implemented), so the whole step — gate + collapse — runs
   on low-discrepancy points and is reproducible from a single per-step seed. Cost: loses SPRT's
   adaptive early stopping (always draws N), and needs a chosen N + threshold instead of a confidence
   target. For the cylinder's `threshold 0.5, confidence 0.95, epsilon 0.05` gate, a fixed N on the
   order of `1/epsilon^2` with QMC's faster convergence is comparable in accuracy at lower N.

2. **Hybrid (current design): SPRT gate (MC) + QMC collapse.** Keep the proven SPRT presence test on
   the MC sampler (seeded via `seed_sampler` for reproducibility) and use QMC only where it is valid —
   the mean collapse. Simplest, preserves SPRT's efficiency; the gate keeps its small MC variance.
   This is what the inflow does today after wiring the collapse.

3. **QMC-randomized SPRT (research).** A scrambled/randomized-QMC SPRT that restores approximate
   independence per LLR increment via a fresh digital shift per draw. Plausible but unproven here;
   would need its own validity analysis. Not worth it unless the gate is shown to dominate the UQ
   error budget.

## Recommendation

Ship **Option 2** (hybrid) as the default — it is correct and already in place. Add **Option 1**
(`estimate_probability_qmc`-based gate) as an explicit opt-in on `UncertainBoundarySource` /
`UncertainInflowZone` *if and when* a fully-QMC inflow UQ pass is wanted, with its own follow-up
change. Do **not** attempt a QMC SPRT without a validity proof.

## Pointers

- SPRT entry: `deep_causality_uncertain` `Uncertain::<bool>::to_bool`,
  `Uncertain::<f64>::estimate_probability_exceeds`, `MaybeUncertain::lift_to_uncertain`.
- QMC presence primitive already available: `Uncertain::<bool>::estimate_probability_qmc(n, seed)`.
- Collapse wiring: `UncertainBoundarySource::resolve` (`expected_value` → `expected_value_qmc`).
