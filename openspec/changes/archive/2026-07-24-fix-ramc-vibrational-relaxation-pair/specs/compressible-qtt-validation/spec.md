## MODIFIED Requirements

### Requirement: RAM-C stagnation-line verification (the buildable milestone)

A self-verifying **RAM-C stagnation-line** example SHALL march a 1-D fitted normal shock at the RAM-C flight
condition, apply the exact Rankine–Hugoniot post-shock state, run the **reused Tier-A reacting/ionization LER
stack** in the post-shock relaxation zone, and gate the peak **electron density** against the RAM-C II
reference within a recorded tolerance.

The tolerance SHALL be re-derived from the corrected vibrational-relaxation closure and SHALL NOT be
widened to re-admit a prediction the previous, incorrect `μ_sr` produced. The bands in force before
this change were earned under `μ_sr = 7.0`, a value with no valid collision pair; a band restored to
keep the former headline agreement would assert an accuracy the physics does not support.

Because `μ` sits inside the Millikan–White exponential, the correction moves the prediction
materially — at the harness's post-shock `T = 8044 K`, `μ: 7 → 14` lengthens `τ_vt` by roughly 1.9×,
keeping `T_ve` colder, cooling the Park controller `T_a = √(T_tr·T_ve)`, and lowering peak `nₑ`. If
the corrected prediction no longer supports an order-of-magnitude claim against the RAM-C II anchor,
the harness SHALL report that outcome rather than presenting a re-tuned band as agreement, and the
gate's evidence class SHALL reflect what the bound actually encodes.

#### Scenario: RAM-C peak electron density reproduced
- **WHEN** the stagnation-line example is run at the RAM-C flight condition
- **THEN** the peak electron density / blackout onset matches the RAM-C II reference within the recorded
  tolerance, with the fitted shock at `O(1)` rank

#### Scenario: The acceptance band is traceable to the corrected physics
- **WHEN** the recorded tolerance is inspected
- **THEN** it states the closure it was derived under and its evidence class, and it is not a
  reinstatement of a band earned under the superseded `μ_sr`

#### Scenario: A prediction outside the anchor band is reported, not absorbed
- **WHEN** the corrected prediction falls outside a band that could honestly be called agreement with
  the RAM-C II anchor
- **THEN** the harness reports the measured offset as its result, and the documentation states the
  offset rather than describing the comparison as validation
