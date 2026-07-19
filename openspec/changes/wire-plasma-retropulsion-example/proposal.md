## Why

The plasma-retropulsion roadmap
(`openspec/notes/cfd-plasma-retropulsion/plasma-retropulsion-roadmap.md`, milestone M5 /
design-note Stage 4) is the terminal milestone: the point at which the Plasma-Retropulsion Descent
stops being a set of library seams and becomes a flying example that simulates a full descent
correctly. M1 measured the coupling depth (verdict **AMBER**), M2 landed the contracts, M3 landed
the burn physics, and M4 lands the guidance and the terminal leg. Nothing yet flies them: no
example composes a propulsion stage, nothing publishes `"commanded_throttle"`, and no world seeds
the `"mass"`/`"propellant"` scalars the burn stages read.

This change assembles the example, runs the two counterfactual studies that are the reason the
example exists, and passes the full gate set. It also closes M2's carried-forward task 6.4 — the
weather-table loader glue that the canonical `weather-table-consumption` capability already
specifies and explicitly defers to "the M5 example, which owns the retropulsion example folder".

The survey behind this change found seven wiring hazards that are silent rather than loud, and
each becomes a normative requirement rather than a discovery during implementation: published
constants are re-written at the head of every step (so a published `"mass"` never depletes); the
throttle is read from three different seams by three different consumers; the marched imprint lags
ignition by one step; two library axes (`with_flight_axes`, `with_burn_sensing`) are opt-in and
inert until called; nothing in the crate produces `"q_inf"` or `"descent_rate"`; and the shared
`S_REF` constant is an acoustic wave speed, not the reference **area** the plume stage needs.

## What Changes

- A **retropulsion example** in the family layout —
  `examples/avionics_examples/cfd/plasma_blackout/retropulsion/` with the house file set
  (`main.rs`, `model.rs`, `constants.rs`, `utils_print.rs`, `README.md`, `output.txt`), a third
  `[[example]]` entry `plasma_blackout_retropulsion`, and a **powered-descent coupling stack**
  beside `corridor_coupling` in `src/shared/world.rs`. The burn stages are composed between the
  ④-writing lift stage and the first ④ consumer (the library's stated `add_aero_force` ordering
  contract), and they are carried from step 0 — ignition is a published-command event inside one
  world, never a stack swap, so the mid-burn fork has a plume-coupled state to fork.
- The **state-fork counterfactual centerpiece**: march into the burn, pause, `fork` the marched
  plume-coupled field copy-on-write, apply a small throttle roster one intervention per branch, and
  continue. The M1 verdict pivoted the *drag authority* to the A0 correlation but measured the
  state-fork **mechanics green** (per the committed artifact
  `studies/qtt_rank_plume/output.txt`: fork setup 42 ns with `Arc`-sharing witnessed, per-branch
  continuation 0.67–1.04× the trunk against a 2.0× band, post-fork bond flat at 16 under the 32 cap,
  branch flow observables spreading monotonically with throttle). This change therefore flies the
  **hybrid** the verdict
  named as measurement-consistent and left for M5 to propose: the A0 correlation is the in-flight
  drag authority, and the state fork carries the flow-realism and fork-economics witnesses.
- The **belief counterfactual**: the same measured cold day in two whole-world alternations from
  one baseline — one whose guidance interpolated `weather_table.csv` at the measured dT, one that
  assumed the standard-day row — both `!!ContextAlternation!!`-marked, with the table's
  `drift_mean + k·drift_sd` sizing the ignition margin and the propellant reserve. This implements
  the existing `weather-table-consumption` requirement that defers the CSV binding to M5.
- The **full numbered gate set** (0–9): integrity, corridor inheritance bit-identical, ignition
  corridor, regime cascade, the four counterfactual gates plus the audit trail, the table earning
  its place, touchdown, compression, bounded rebuilds, and the 600 s wall-clock budget — evaluated
  in full and rendered as one merged `Verdict`, with bands earned on the first measured run and
  regressed thereafter.

No breaking changes: this is net-new example code plus one new coupling assembler and a small set
of example-local stages in `src/shared/`. The corridor and weather examples are untouched and must
re-run bit-identically.

## Capabilities

### New Capabilities

- `retropulsion-example-wiring`: the example folder and Cargo entry, the powered-descent coupling
  stack and its ordering, the five-act leg structure, and the seeding / throttle-seam / opt-in-axis
  contracts that make the burn stages actually couple.
- `retropulsion-state-fork-study`: the mid-burn event fork, the throttle roster and its on-axis
  discipline, the three coupling witnesses (4a/4b/4c) with their stated authority under the A0
  depth, the fork-economics regression against M1's bands (4d), and the branch audit trail (4e).
- `retropulsion-belief-counterfactual`: the informed-vs-uninformed worlds on a measured day, the
  table row as a load-bearing flight input, provenance stamping of the row and any clamp, and the
  material-separation gate (5).
- `retropulsion-descent-gates`: the numbered gate set 0–9, the evaluate-all-then-decide contract,
  caller-side wall-clock gating, the const-threshold constraint that `GateFn` imposes, the
  earned-then-regressed band convention, and the shipped `output.txt` / `README.md`.

## Impact

- `examples/avionics_examples`: new `cfd/plasma_blackout/retropulsion/` example folder; one new
  `[[example]]` stanza in `Cargo.toml`; `src/shared/world.rs` gains a `powered_descent_coupling`
  assembler and a burn-seeded initial field; `src/shared/stages.rs` gains the example-local stages
  that publish `"q_inf"` and `"descent_rate"` and drive the throttle on both seams;
  `src/shared/constants.rs` gains a propulsion section. Example code only — no example-crate tests.
- `deep_causality_cfd`: consumed unchanged. M1's `ForcingRegion`/`PlumeImprint` seam, M2's throttle
  channel, `BurnEnvelope`, `KeyedTable` and atmosphere rows, M3's `RetroThrust`/`PlumeObstruction`/
  classifier axes, and M4's guidance and terminal-leg work are all composed, not modified.
- `deep_causality_physics`: consumed only (the propulsion kernel family); no changes.
- Documentation: a `## Where Things Live` README in the family form, a row in the crate README's
  examples table, cross-links swept between the three siblings, and the roadmap ledger closed out.
- Upstream: **M4 (`add-retropulsion-terminal-descent`) is a hard precondition** — Acts 2 and 4 and
  gates (2) and (6) consume `ThrottleGuidance`, live `CyberneticCorrect` burn-axis enforcement, and
  the terminal-leg re-seed. This change is derived ahead of M4 and MUST NOT be applied before it.
