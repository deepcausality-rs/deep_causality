## Context

`REDUCED_MASS_AMU = 7.0` feeds `Park2tClosure`, which sets the Millikan–White vibrational relaxation
time `τ_vt`, which sets how far the vibrational-electron temperature `T_ve` catches up to the
translational `T_tr` over the residence time, which sets the Park rate-controlling temperature
`T_a = √(T_tr·T_ve)`, which drives ionization and therefore peak electron density, plasma frequency
and the blackout determination. It is the first link in the crate's headline scientific chain.

The value is wrong in a specific way. Millikan–White's `μ_sr` is the reduced mass of the colliding
pair, `s` the relaxing diatomic and `r` the partner. 7.00 amu is `14·14/28` — the N–N pair, two
nitrogen *atoms*. Atomic nitrogen has no vibrational mode, so this is not a relaxation pair at all;
the comment deriving it substituted nitrogen's atomic mass 14 for N₂'s molecular mass 28. No valid
candidate yields 7.00:

| Pair | μ (amu) | `A_sr` at θ_v = 3393 K | `τ·p` at 8044 K (atm·s) |
|---|---|---|---|
| N₂–N | 9.33 | 180.6 | 6.9e-7 |
| N₂–O | 10.18 | 188.7 | 7.5e-7 |
| **N₂–N₂** | **14.00** | **221.3** | **1.02e-6** |
| N₂–O₂ | 14.93 | 228.5 | 1.08e-6 |
| *N–N (committed)* | *7.00* | *156.5* | *5.42e-7* |

Two properties of the codebase let this survive. First, `μ_sr` was a bare literal — nothing related it
to a pair, so the arithmetic slip lived only in prose and no check could catch it. Second, the constant
was **defined twice**, independently, in `verification/qtt_ramc_stagline/config.rs` and
`examples/avionics_examples/src/shared/constants.rs`, so one slip reached both the verification harness
and all three plasma-blackout examples.

Phase 1 is relevant background: the harness's gates now declare an evidence class, and the RAM-C
±0.70-decade band is already labelled `tripwire` (pinned from its own measurement). That labelling is
what makes this change tractable — the band is already known not to be an external accuracy claim, so
re-deriving it is a correction, not a retraction.

## Goals / Non-Goals

**Goals:**

- `μ_sr` derived from a named, justified collision pair, defined once.
- The RAM-C chain and the three plasma-blackout examples re-baselined against the corrected closure.
- Acceptance bands re-derived from the corrected physics, with the resulting offset reported honestly.
- The pre-correction figure retained as recorded history so the change is auditable.

**Non-Goals:**

- **The Park two-temperature model, the finite-rate network, the Saha surrogate, the `T_e = T_ve`
  lumping, the single associative-ionization channel.** These are the harness's own documented open
  levers and are independent of `μ_sr`. Fixing a constant is not an invitation to reopen the model.
- **Restoring the previous headline agreement.** Explicitly out of scope, see D3.
- **The mixture-weighted relaxation time.** Out of scope *for this change* but explicitly **in scope
  for the fidelity goal** — see D2a. This change corrects the pair; the follow-up replaces the
  single-pair simplification. The boundary is drawn here so the constant fix stays reviewable, not
  because the simplification is acceptable.
- Any other Phase-2 item. Items 8–15 are separate changes.

## Decisions

### D1 — Derive `μ` from constituent masses; do not write the number

`μ_sr` becomes a value computed from two named masses rather than a literal with a prose derivation.

*Why:* the defect was undetectable precisely because the number and its justification lived in
different representations — one in code, one in a comment — with nothing tying them together. Deriving
the value makes the pair the source of truth and the number a consequence, so the next slip is a
compile-or-test failure rather than a comment nobody re-derives.

*Alternatives considered.* Correcting the literal to `14.0` with a fixed comment was rejected: it
repairs this instance and leaves the class of defect intact. A full species/mixture table was also
rejected — it is a model change, not a constant fix, and belongs to whatever addresses the single-pair
simplification properly.

### D2 — Choose the pair on the flight condition, and record the alternatives

The pair is a physical judgement to be made and justified, not defaulted to N₂–N₂ because that is what
the old comment said.

At the RAM-C II condition the harness models (M = 25, post-shock T₂ ≈ 8044 K) the air behind the shock
is partially dissociated, so the bath is not pure N₂. N₂–N₂ (`μ = 14.00`) is the conventional default
and the largest of the plausible values; N₂–N (`μ = 9.33`) is materially different and physically live
given dissociation. The difference is not academic: `τ·p` spans 6.9e-7 to 1.08e-6 across the candidate
pairs, roughly a factor of 1.6.

**Recommendation under the high-fidelity goal: N₂–N₂ (`μ = 14.00`).** Three reasons, in order of
weight:

1. **It cannot be outcome-driven.** N₂–N (`μ = 9.33`) shortens `τ_vt`, raises `nₑ`, and partially
   restores the old headline. Choosing it would be indistinguishable from back-fitting even if it
   happened to be right. N₂–N₂ is the largest plausible `μ`, hence the longest `τ`, hence the most
   conservative prediction — it cannot be accused of being chosen for its verdict.
2. It is the conventional baseline of the two-temperature literature, and it is what the code's own
   comment intended.
3. It makes the current model self-consistent before the model itself is revisited (D2a).

*Why not just take N₂–N₂ without argument:* it may well be right, but taking it because a wrong comment
named it would repeat the original failure — asserting a pair rather than choosing one. The reasoning
above is the choice; the comment is not.

### D2a — The single-pair simplification is the next fidelity limit, and it biases unsafely

Correcting `μ` makes the closure self-consistent. It does not make it faithful, and under a
high-fidelity goal the gap should be named rather than left as an unremarked default.

At the RAM-C post-shock condition the bath is not pure N₂. O₂ is fully dissociated by ~5000 K and N₂
is partially dissociated at 8044 K, giving roughly `x_N₂ ≈ 0.46`, `x_O ≈ 0.31`, `x_N ≈ 0.23`. The
faithful form is the mixture-averaged relaxation time `1/τ_mix = Σ_r x_r / τ_(s,r)`. The harness
**already carries the composition it needs** — its own output prints `x_N = 4.617e-1, x_O = 6.364e-1`.

**The direction of the simplification's error matters.** Lighter partners have smaller `μ`, hence
shorter `τ_vt`, hence a faster `T_ve` rise, a hotter `T_a`, and *more* ionization:

| Partner | μ (amu) | relative `τ·p` |
|---|---|---|
| N₂–N | 9.33 | shortest |
| N₂–O | 10.18 | ↓ |
| N₂–N₂ | 14.00 | longest |

So a mixture including N and O would legitimately recover part of the ≈1.27-decade gap — **not by
tuning, but because the bath genuinely contains lighter partners**. Pure N₂–N₂ is therefore the
*slowest* relaxation and the *lowest* `nₑ` of the physically plausible options.

That is the unsafe direction. Under-predicting `nₑ` means under-predicting blackout, which means being
**optimistic about comms availability** — the wrong way to be wrong for an avionics consumer. The
single-pair simplification should be replaced, not merely disclosed, and it should be sequenced
promptly after this change rather than deferred indefinitely.

### D3 — Re-derive the bands from the corrected physics; never re-tune to restore the headline

The corrected prediction is expected to land near **−1.27 decades** against the RAM-C II anchor (the
audit module report's measurement), replacing the current "+0.0 decades". The acceptance band is
re-derived from the corrected closure and its own uncertainty; it is not widened to re-admit the old
result.

*Why:* this is the single decision the whole change turns on. A band stretched until the previous
headline passes would be back-fitting of exactly the kind the audit was commissioned to find — and it
would be worse than the original defect, because it would be done knowingly. If the corrected
prediction cannot support an order-of-magnitude claim, the honest output is the measured offset.

*Consequence to accept up front:* the crate may lose a headline result. The README currently reads
"peak nₑ = 1.08e19 m⁻³, +0.0 decades vs the RAM-C II anchor". That sentence is likely to change.

### D4 — Single definition, in the crate, not the examples

The surviving definition lives with the closure that consumes it — `deep_causality_cfd` — and the
examples' shared constants module refers to it.

*Why that direction:* the examples depend on the crate, not the reverse, so the crate is the only
place both consumers can see. It also puts the constant next to the `Park2tClosure` documentation that
describes it, where a reviewer checking the closure will encounter it.

### D5 — Sequence measurement before gate edits

Order: correct the constant → **measure** what the chain now produces → then re-derive bands and update
documentation from that measurement.

*Why:* deriving a band before knowing the corrected value invites choosing a band that produces a
comfortable verdict. Measuring first makes the band a consequence of the physics rather than of the
desired outcome. The same discipline as Phase 1's "wire CI first, against the suite as it stands".

### D6 — Retain the pre-correction figure as history

The `+0.0 decades` result and the `μ = 7.0` closure that produced it are recorded — in the harness
documentation and this change's notes — as superseded, with the reason.

*Why:* the crate already does this well (`qtt_ramc_stagline` keeps the single-temperature surrogate's
"12×-over-prediction history" as recorded history). Deleting the old number would make the change
unauditable and would obscure that the previous agreement was an artifact.

## Risks / Trade-offs

- **The crate loses a headline claim.** → Accepted, and the point. A claim resting on a
  physically meaningless constant was not a claim. Better found here than by a reader reproducing it.
- **The corrected prediction may be a large under-prediction.** → Report it. `−1.27 decades` is still
  informative — it bounds what an uncalibrated single-channel network achieves — and the harness's own
  disclaimer already lists the model levers that would close it. Do not close the gap by re-tuning `μ`.
- **Choosing N₂–N instead of N₂–N₂ shortens `τ_vt` and partially restores the old result.** → This is
  the tempting failure mode: picking the pair that produces the nicer number. D2 requires the choice
  to be justified on the flight condition *before* its effect on the verdict is known, and D5 sequences
  measurement so the justification cannot be written backwards from the answer.
- **Example re-baselining churns three committed outputs and their CSVs.** → Expected. The examples are
  deterministic and their outputs reproduce, so the diff is reviewable; each committed artifact is
  regenerated from a clean run.
- **Test fixtures pin `7.0` in two places.** → They must move with the constant. A fixture left at 7.0
  would keep a superseded value alive in the test suite.

## Migration Plan

No runtime migration: no public API change, `publish = false`, no downstream consumers.

1. **De-duplicate first**, keeping the current value. Isolates the refactor from the physics change, so
   the subsequent diff shows only the number moving.
2. **Correct the pair and derivation.** One commit, no gate or documentation edits.
3. **Measure.** Run the RAM-C harness and the three examples; record the new figures before touching a
   band.
4. **Re-derive bands and update documentation** from step 3's measurements.
5. **Regenerate baselines and example artifacts** from clean runs.

Rollback is per-step; steps 1 and 2 are independently revertible.

## Deferral check (2026-07-22)

D2a defers the mixture-weighted relaxation time and states "the composition it needs is already
computed". **Tested against the tree: true.** `air_n2_mole_fraction` and `air_o2_mole_fraction` are
shipped kernels in `deep_causality_physics/src/constants/hypersonic.rs`, and
`qtt_ramc_stagline/main.rs:197-198` already calls both to form the N and O concentrations.

So the follow-up is a weighting over pairs whose composition is in hand, not a new physics module. The
deferral stands — it keeps the constant fix reviewable, which is a real reason — but it is small, and
the proposal should be raised as soon as this change lands rather than carried as a standing
disclaimer.

## Open Questions

- **Which collision pair?** ✅ **Resolved: N₂–N₂ (`μ = 14.00`)**, per D2's recommendation under the
  high-fidelity goal. Recorded before the corrected prediction is measured (D5), so the justification
  cannot be retrofitted to the answer.
- **Does the corrected prediction still support any RAM-C claim?** Unknown until step 3. If it lands at
  ≈ −1.27 decades, the honest framing is "an uncalibrated single-channel network under-predicts the
  flight anchor by ~1.3 decades", which is a result, not an agreement. Note D2a: part of that gap is
  expected to be recovered legitimately by the mixture treatment, so the single-pair figure should be
  read as a lower bound on `nₑ`, not as the model's final word.
- **When does the mixture-weighted `τ` land?** D2a establishes that it should, and that the single-pair
  simplification errs toward under-predicting blackout — the unsafe direction. It stays out of *this*
  change to keep the constant fix reviewable, but it should be proposed as soon as this one lands
  rather than left as a standing disclaimer. The composition it needs is already computed.
