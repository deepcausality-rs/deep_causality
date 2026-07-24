## Context

The navigation stack is a 17-state error-state Kalman filter (`NavFilter`) driven by
`ReentryNavEngine`, used by the plasma-blackout corridor and weather examples to estimate drift through
a GNSS-denied interval and the error remaining after reacquisition. Those are the numbers the examples
gate on.

The audit confirmed the parts that are right, which bounds this change usefully:

- the 17-state composition and `nav_transition_matrix` match the standard ESKF form: `δp ← δv`, the
  `−[f]×` block coupling attitude error into velocity error, and the `−I·dt` accel-bias→velocity and
  gyro-bias→attitude couplings. (The Tier-A model omits the transport/`−[ω]×` self-coupling on the
  attitude block, consistent with its stated `C ≈ I`, no-Earth-rotation assumption — verified 2026-07-24;
  an earlier draft of this note mentioned a `−[ω]×` block that the code does not carry.)
- `update_scalar`'s covariance update is a correct **Joseph form** with re-symmetrisation, citing
  Groves (2013) §3.4.3 — PSD-preserving where the simple form is not.

Three things around that core are wrong, and they are independent:

| | Site | Defect |
|---|---|---|
| 9a | `eskf.rs::predict` | `Q` added with no `dt`; covariance grows per step, not per second |
| 9b | `eskf.rs::update_scalar` | divides by unvalidated `s = h·P·hᵀ + r`; returns `()`, so cannot reject |
| 9c | `reentry_nav.rs::correct_position` | zeroes the attitude error it never injected |

9c is deeper than a missing line. `ReentryNavEngine` has fields `gm`, `position`, `velocity`, `filter`,
`tau_offset`, `elapsed` — **no nominal attitude** — and `attitude_error` is read nowhere in the crate's
`src` or examples (only one test reads it, asserting it is zero). So the attitude error state is
estimated, credited in the covariance through cross-covariance with the position update, and then
discarded. There is currently nowhere to put it.

A subtlety that bounds what 9c *means for the shipped examples*: they are point-mass 3-DOF with no true
angular rate, and the corridor's `IMU_GYRO_BIAS = [0,0,0]`, so the *predict* path leaves the nominal
attitude exactly at identity. The defect is therefore **mostly** about the **covariance** — the position
fix shrinks the attitude block through cross-covariance, leaving the filter overconfident about an
attitude it never corrected. Option (a) fixes this by giving the reduction a real correction to attribute
to (a nominal attitude that is injected into). But note the correction is **small, not zero**: the same
`−[f]×` cross-covariance the fix rides on folds a small nonzero `δψ`, and injecting it nudges the nominal
off identity, so `C(q) ≠ I` on later steps. Measured, this shifts the corridor's leg4 error
`0.2802 → 0.2804 m` and leg2 variance `2.6711e1 → 2.6710e1 m²` (≲0.07%, deterministic, all gates hold) —
the reduction is now not merely *justified* but *applied*, and the applied part is visible. This is the
correction working, and it confirms the proposal's "figures move" via (a) rather than the `Q` re-tune.

That the attitude block still *couples* is not a mistake: the transition matrix's `−[f]×` term is what
lets an attitude error grow a velocity error, and removing it would change the position/velocity error
propagation the examples depend on. The coupling is real and worth having. It is the correction half of
the loop that is missing.

## Goals / Non-Goals

**Goals:**

- Process noise applied under a stated, cited discretisation; covariance growth tracks elapsed time.
- The measurement update cannot be driven to `NaN` by inputs reachable from the public API.
- A covariance is validated as one at construction and restoration.
- The error-state lifecycle is closed: nothing is zeroed that was not injected.
- The examples' navigation figures re-derived from the corrected filter.

**Non-Goals:**

- **The structure of `nav_transition_matrix`, the 17-state composition, and the Joseph update.**
  Confirmed correct by the audit; their code is not altered. **Documented exception (option a):** the
  specific force fed to the `−[f]×` coupling changes from the Tier-A `C ≈ I` value to `C(q)·f_body`,
  where `C(q)` is the nominal attitude's DCM. This changes the error-dynamics *behaviour*, not the
  matrix's structure — `nav_transition_matrix` and `propagate` already take the specific force as an
  argument, so the rotation happens in the caller (`ReentryNavEngine::predict`), not inside those
  functions. Recorded here so the change is a deliberate departure from `C ≈ I`, not an accidental one.
- **The Encke/Cowell integrator switch, the IMU noise model, the relativistic clock.** Separate
  concerns; the clock and the integrator were confirmed or are untouched by this change.
- **Full attitude *observability*.** Option (a) carries and corrects a nominal attitude so the ESKF
  loop is closed and the reset is legitimate, but it does not add an attitude *measurement*. In the
  point-mass examples the nominal stays ≈ identity; making the filter *estimate attitude well* under a
  real rotational profile (6-DOF, an attitude sensor) is out of scope and would be its own work.

## Decisions

### D1 — Treat the caller's input as a spectral density; scale by `dt`

`Q_d = Q_c·dt`, with the API documenting that the supplied diagonal is a continuous-time spectral
density.

*Why:* it is the standard first-order discretisation, it is what "process noise" means for the
random-walk and white-noise terms this filter carries (IMU bias random walk, clock noise), and it is
the minimum that makes the tuning step-size independent. Van Loan would be more exact for the
cross-terms the transition matrix induces within a step, but the added machinery buys accuracy the
filter's other approximations do not warrant.

*Alternative considered.* Documenting the current behaviour as "already-discretised, valid at the
configured `dt`" was rejected: it is technically honest but leaves a filter whose tuning silently
breaks when anyone changes the step size, which is a trap rather than a contract.

### D2 — Give `update_scalar` a rejection channel

`update_scalar` returns a `Result` (or equivalent) so it can refuse, and refuses without mutating.

*Why:* the guard is only meaningful if the caller learns about it. A silent no-op would leave the
caller believing a fix was folded — which for a navigation filter through a blackout is precisely the
wrong failure mode, because the operator's confidence in the estimate would be unearned.

*Atomicity matters:* the update must leave state and covariance untouched on rejection, not
half-applied. The three sequential per-axis updates inside `correct_position` make this concrete — a
rejection on the second axis must not leave the first axis's fold applied and the caller unaware.

### D3 — Validate the covariance at its entry points, not at every use

`NavFilter::new` and `restore` check symmetry, non-negativity of the diagonal and finiteness.

*Why:* it is the same argument as the QTT constructor validation in the sibling change — the entry
point is the chokepoint, and validating there makes the degenerate-update path unreachable rather than
merely guarded. The guard in D2 remains as defence in depth, since `P` can in principle lose PSD-ness
through accumulated round-off even from a valid start.

### D4 — State the attitude invariant; leave the resolution to the owner

The spec requires *"an error state is reset only if it was injected"*. It does not mandate which of the
three admissible resolutions is taken.

The options, with their costs:

**(a) Carry a nominal attitude and inject `δψ` into it.** Completes the ESKF loop properly and makes
the 17-state filter deliver what its state vector promises. Cost: the engine needs an attitude
representation (quaternion or DCM), the IMU propagation needs to use it, and `correct_position` needs
to apply a rotation correction. This is a feature, not a fix, and it is the largest option.

**(b) Retain the attitude error rather than zeroing it.** `reset_navigation` stops clearing the
attitude block, so the estimate persists and continues to couple into velocity error through the
transition matrix. Small and honest. Cost: the error state is then not a true "error about the current
nominal" for that block, which is a departure from textbook ESKF bookkeeping and must be documented as
one.

**(c) Do not claim the covariance reduction.** Apply a covariance reset transform consistent with the
fact that no attitude correction was made. Cost: requires care to remain PSD, and arguably throws away
real information — the cross-covariance genuinely does say something about attitude.

**Recommendation under the high-fidelity goal: (b) now, (a) as the destination — sequenced.**

Options (b) and (c) make the filter *honest*; only (a) makes it *right*. Under a fidelity goal that
distinction is the whole point, so (a) is where this ends. Three reasons it is the correct destination:

1. **The machinery is already there and currently wasted.** The filter carries gyro-bias states and the
   `−[f]×` coupling that lets an attitude error grow a velocity error. That propagation is doing real
   work; without the correction half, the state vector promises an estimate the engine cannot use.
2. **The current failure direction is the dangerous one.** The filter is *overconfident* — it claims
   covariance reductions for corrections never applied, so it under-reports uncertainty through a
   blackout. For a GNSS-denied navigation estimate that is the wrong way to be wrong.
3. `ReentryNavEngine` is the crate's re-entry navigation engine. An engine that cannot correct attitude
   is not a high-fidelity navigation model, whatever its state count says.

~~**But (a) is feature-sized**~~ — **this was checked against the tree on 2026-07-22 and is false.**

`Quaternion`, `Quaternion32`, `Quaternion64` ship in `deep_causality_num_complex`, which
`deep_causality_cfd` **already depends on**. The type carries exactly the ESKF operations (a) needs:

| (a) needs | Already shipped |
|---|---|
| a nominal attitude representation | `Quaternion<F>` |
| gyro integration into it | quaternion multiply + `normalize()` |
| injecting the error state `δψ` | `from_axis_angle(axis, angle)` |
| the DCM the `−[f]×` coupling wants | `to_rotation_matrix()` |
| initialisation / interpolation | `from_euler_angles`, `slerp` |

So (a) is a field on `ReentryNavEngine` (which today holds `gm`, `position`, `velocity`, `filter`,
`tau_offset`, `elapsed` and no attitude) plus the call sites to integrate the gyro into the nominal and
apply a rotation correction in `correct_position`.

**Correction, checked again against the tree on 2026-07-24.** The 2026-07-22 revision was right that the
`Quaternion` *representation* ships, but it under-counted (a) a second time — the same failure mode this
change's B-1 sibling recorded. Two things the representation does not supply:

1. **There is no angular-rate input to integrate.** `ReentryNavEngine::predict(dt, aero_accel, Q)` and
   `NavFilter::predict(dt, specific_force, Q)` carry no `ω̂`. Integrating a nominal attitude forces a
   **`predict` signature change** and a new argument at every call site (the corridor, weather and
   `ins_gnss_blackout` examples). So (a) is "a field plus call sites" *plus a new sensing argument
   threaded through the engine and its callers".
2. **The examples do not rotate.** Point-mass 3-DOF, and the corridor's `IMU_GYRO_BIAS = [0,0,0]`, so
   the nominal attitude stays ≈ identity and `δψ ≈ 0`. In these examples (a) is a covariance-legitimacy
   fix, not a visible attitude estimate (see Context).

**(a) is still the decision (owner, 2026-07-24), and (b) is dropped.** (b) is not merely costly — it
does not satisfy this change's own spec: retaining the attitude error while the position fix keeps
shrinking the attitude covariance fails the "attitude confidence is not claimed without a correction"
and "repeated fixes do not accumulate unjustified confidence" scenarios. Of the honest options only (a)
(apply the correction) and (c) (reset the covariance so no reduction is retained) satisfy the spec; the
owner chose (a) so the ESKF loop is genuinely closed. The cost above is accepted and documented, not
waved away — that is the lesson of the two prior changes, applied to this one *before* implementation
rather than after.

(a) also fixes the original defect properly rather than working around it: `reset_navigation` may then
zero the attitude block **because the correction was actually injected**, which is the invariant the
spec states.

(b)'s cost is real and must be documented: retaining the attitude error means that block is no longer
an error *about the current nominal* in the textbook ESKF sense. That is a deliberate, recorded
departure with a stated successor — not a permanent resting place.

### D5 — Re-tune after measuring, never to restore the old numbers

Order: fix the filter → measure what the examples now report → re-derive their gates from that.

*Why:* the same discipline Phase 1 established and change 1 repeats. With `Q` scaled by `dt` the
existing `q_diag` values change meaning, so the drift and reacquisition figures will move. A gate
adjusted until the previous number reappears would be back-fitting; a gate re-derived from the
corrected filter is a measurement.

## Risks / Trade-offs

- **Every tuned `q_diag` changes meaning.** → Expected and unavoidable: the current values are
  per-step, the new ones per-second. Re-tuning is part of the change, and the examples' navigation
  gates must be re-derived rather than restored.
- **The corrected filter may report worse drift than the current one.** → If so, that is the honest
  number. The current figures were produced by a filter whose covariance growth was a function of step
  count; there is no reason to expect them to have been right.
- **Option (a) for the attitude gap is a feature-sized piece of work.** → Which is why D4 states the
  invariant rather than the implementation. If the owner picks (a), it likely deserves its own change
  rather than riding along inside this one.
- **Adding a rejection path to `update_scalar` touches its callers.** → Only `correct_position` calls
  it in-crate; the surface is small. The atomicity requirement (D2) is the part to get right.
- **Validating the covariance may reject snapshots that currently restore.** → A snapshot carrying a
  non-PSD covariance was already broken; failing loudly at restore is better than continuing from it.

## Migration Plan

No runtime migration; `publish = false`, no downstream consumers.

1. **Covariance validation** (D3) — smallest, and makes the degenerate path unreachable, so it lands
   before the guard that defends against it.
2. **The update guard and its rejection channel** (D2) — touches one caller.
3. **The process-noise discretisation** (D1) — the change that moves numbers, landed alone so its
   effect on the examples is attributable.
4. **Measure** the examples' navigation figures.
5. **Re-derive** their gates from step 4.
6. **The attitude resolution** (D4) — last, because it needs the owner's decision and because options
   (a) and (b) have very different sizes.

Steps 1–3 are independently revertible. Step 6 may be deferred to its own change if (a) is chosen.

## Open Questions

- **Which attitude resolution?** ✅ **Resolved: (a), in this change.** Revised 2026-07-22 after
  testing the deferral against the tree. The earlier answer was "(b) now, (a) as a follow-up, because
  (a) is feature-sized" — and (a) is not feature-sized: `Quaternion` with `from_axis_angle`,
  `to_rotation_matrix`, `normalize` and `slerp` already ships in `deep_causality_num_complex`, an
  existing `deep_causality_cfd` dependency. (b) existed only to avoid (a)'s cost, and that cost was
  overstated. Doing (a) also removes (b)'s documented departure from textbook ESKF bookkeeping.
- **Do the examples' navigation gates survive re-derivation?** Unknown until step 4. The corridor gates
  on dead-reckoning drift and terminal reacquisition error; both are downstream of `Q`.
- **Should `Q` be Van Loan rather than `Q_c·dt`?** D1 chose the simpler form deliberately. If the
  corrected filter shows sensitivity to within-step coupling, that judgement is worth revisiting — but
  with evidence, not pre-emptively.
