## Context

Roadmap M3 (design-note Stage 3, physics half): make the burn physically couple, on the seams M1
and M2 landed and archived (2026-07-19).

- **M1** (`plasma-retropulsion-de-risk`): the `ForcingRegion` masked-relaxation seam on the
  compressible march path (`solvers/qtt/compressible/forcing.rs`), `plume_mask_2d`, and the measured
  **AMBER** verdict (`derisk-verdict.md`): the A0 Jarvinen–Adams correlation is the drag authority;
  neither a pinned imprint nor a momentum-carrying jet reproduces the drag collapse at this harness
  fidelity (harness capacity, not model class). A field-contracted decrement is *not* a cited
  authority in flight.
- **M2** (`plasma-retropulsion-cfd-contracts`): the throttle command channel
  (`throttle_action` + published `"commanded_throttle"`), the `"mass"`/`"propellant"`/`"ignited"`
  scalars, the additive `CoupledField::add_aero_force` idiom (compose after `BankSteeredLift`, before
  the force consumers), the `BurnEnvelope` gate axes, and the inert `PropulsionStub` — the exact seam
  these production stages fill. The stub bundles thrust + A0 decrement in one contract-validation
  stage; production splits them.
- The compressible carrier publishes `"flight_mach"`, `"flight_altitude"`, `"flight_speed"`,
  and the post-shock projections `"speed"`/`"T_tr"`/`"n_tot"` each step
  (`compressible_march_run.rs:251-326`). `RegimeClass`/`RegimeClassify` (`corridor.rs:69-191`)
  already classify rarefaction (Knudsen) and comms-denial (plasma frequency), logging only on a
  `key()` change.
- Existing propulsion kernels (Stage 1, archived): `propellant_mass_flow_kernel`,
  `momentum_flux_ratio_kernel`, `srp_thrust_coefficient_kernel`, `srp_preserved_drag_fraction_kernel`,
  `cordell_braun_plume_boundary_kernel`, `srp_total_axial_force_coefficient_kernel`.

Constraint carried from M2: the corridor example stays **bit-identical**, and any burn-phase stage
is strictly inert when no burn is commanded, so the stack can carry it from the start.

## Goals / Non-Goals

**Goals.**

- Productionize the thrust half (`RetroThrust`) and the plume half (`PlumeObstruction`) of the M2
  stub behind the same contract, so a coupled burn leg feels thrust in the force channel and the
  IMU and carries the A0 drag decrement.
- Detect and log the burn-phase flight regimes (Mach crossings, burn↔coast, touchdown) on the
  existing classifier.
- Prove the coupling with a burn-leg harness: the regime cascade in provenance, the in-flight
  decrement inside M1's band, integrity across the leg.

**Non-Goals.**

- No `ThrottleGuidance`, no live `CyberneticCorrect` burn-axis enforcement wired into a flying stack,
  no cutoff/terminal-leg re-seed (M4).
- No retropulsion example folder, no counterfactual study, no full §10 gate set (M5).
- No new physics kernel (Stage 1 shipped them; M3 consumes them).
- No revival of a field-contracted drag decrement as the flight authority — the AMBER verdict stands
  (a `ForcingRegion` imprint is state realism only).

## Decisions

**D1 — Split the stub into two production stages.** `PropulsionStub` (M2) bundles thrust +
propellant depletion + the A0 decrement in one contract stub. Production splits it: `RetroThrust`
owns thrust + depletion + `ignited`; `PlumeObstruction` owns the A0 decrement (and the optional
state-realism imprint). Rationale: the two couple different channels (thrust is a body force felt by
the IMU; the decrement is an aerodynamic drag adjustment) and M5 composes them independently; a world
that wants only the force-channel drag closure runs `PlumeObstruction` without any marched imprint.
Both keep the stub's zero-throttle inertness, so swapping stub → {RetroThrust, PlumeObstruction}
changes no consumer.

**D2 — `RetroThrust` reads the flight velocity direction and the carried mass.** The retro-thrust
acceleration is `−(T/m)·v̂`, `T = commanded_throttle · thrust_ref`, `m` the carried `"mass"`, `v̂`
the flight-velocity unit vector (the corridor's motion axis; the carrier publishes `"flight_speed"`
and the marched momentum sets the sign). Composed with `add_aero_force` (never `set_aero_force`),
after the lift stage. Mass-aware normalization means the acceleration grows as propellant burns off,
which the trajectory and IMU both see through the summed force. Depletion `ṁ = T/(Isp·g₀)` via
`propellant_mass_flow_kernel`; `propellant` and `mass` fall by `ṁ·Δt`; `ignited` is set.

**D3 — `PlumeObstruction` carries the A0 correlation as the drag authority; the imprint is state
realism.** From `commanded_throttle`: form the thrust, `C_T = srp_thrust_coefficient(T, q∞, S_ref)`
(with `momentum_flux_ratio_kernel` sizing the plume input), read
`srp_preserved_drag_fraction_kernel(C_T)`, and scale the forebody axial drag on the force channel by
the preserved fraction (the M2 `PropulsionStub` A0 idiom). Optionally imprint the analytic plume on
the marched layer via `ForcingRegion` (mask from `cordell_braun_plume_boundary_kernel` at the world's
`C_T`) — **state realism only**, gated behind an explicit opt-in, and it MUST NOT alter the
force-channel drag closure. Rationale: the AMBER verdict measured that a field imprint does not carry
a J–A-faithful decrement at this fidelity, so the cited correlation is the authority; the imprint
exists for flow-realism witnesses (M5's state-fork), not for the drag number.

**D4 — Classifier axes are additive fields on `RegimeClass`, folded into `key()`.** Add a Mach
regime (supersonic ≥ 1.2 / transonic (0.8, 1.2) / subsonic ≤ 0.8 from `"flight_mach"`), a thrust
state (coast / burn from the `"ignited"` flag), and a touchdown flag (from `"flight_altitude"` vs a
configured floor). Extend `key()` to `(model, gnss_denied, mach_regime, thrust_state, touchdown)` so
a band crossing, a burn↔coast transition, or a touchdown is a logged regime change, while the
continuous values stay out of the key (no spurious re-logs). The corridor's existing behavior is
unchanged when `"flight_mach"`/`"ignited"`/`"flight_altitude"` are absent (the new axes default to a
neutral value that does not change `key()` from today's `(model, gnss_denied)` pair). Alternative — a
second classifier stage — rejected: one classifier owning one regime keeps the change-detection
single, mirroring the M2 decision to keep one gate owning one envelope.

**D5 — Verification is a burn-leg harness test, not an example.** M3 has no example (M5 owns that),
so the exit gates run at the harness level, extending the M2 inheritance-guard pattern: march a
corridor-class coupled world through a commanded burn, and assert (a) the ordered regime cascade
appears in provenance (aero → thrust-dominated, the Mach crossings under thrust, burn↔coast),
(b) the in-flight contracted decrement lies inside M1's measured band at the swept `C_T`, and
(c) integrity — no step captured an error, the field stayed finite. Rationale: the full Acts-0/1
burn-leg gate belongs to M5; M3 proves the three physics couplings it adds.

**D6 — Zero-throttle inertness is the inherited contract.** `RetroThrust` and `PlumeObstruction`
each reproduce `PropulsionStub`'s inertness at `throttle ≤ 0` (no force write, no scalar mutation, no
log entry), so the corridor-inheritance guard extends to them unchanged and the burn stack is
carry-from-start safe.

## Risks / Trade-offs

- [`RetroThrust` mass-aware normalization divides by a depleting mass] → the carried `"mass"` is a
  contract invariant (positive, from M2); the stage errors on a non-positive mass under active
  throttle (the stub precedent), so a mis-seeded world fails loudly rather than dividing by zero.
- [`PlumeObstruction` imprint could be mistaken for the drag authority] → the imprint is behind an
  explicit opt-in and a spec MUST that forbids it from altering the force-channel closure; the
  in-flight decrement is contracted from the correlation, cross-checked against M1's band.
- [Classifier `key()` extension re-logs the corridor] → the new axes default neutral when their
  scalars are absent, so `key()` reduces to today's pair for a corridor world; a harness asserts the
  corridor's rarefaction/comms cascade is unchanged.
- [`C_T` sizing needs `q∞` and `S_ref`] → both are world constants the burn world carries (the M2
  envelope sensing already configures them); `PlumeObstruction` reads them from the stage config, not
  from guessed defaults.

## Migration Plan

Additive throughout; no API removed or re-signed. Land order: `RetroThrust` (thrust + depletion) →
`PlumeObstruction` (A0 decrement + optional imprint) → classifier axes → burn-leg harness. Each lands
with its tests green under `bazel test //deep_causality_cfd/...` before the next. Rollback is
dropping the change; M4/M5 do not yet consume these stages.

## Open Questions

- None blocking. The A0-vs-A depth was resolved by the M1 AMBER verdict (A0 authority; imprint is
  state realism), so `PlumeObstruction`'s shape is fixed. Whether M5's state-fork uses the optional
  imprint for flow-realism witnesses is an M5 design question, not an M3 one.
