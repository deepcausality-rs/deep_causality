## 1. Settle the physical decision (before any code)

Design D2 and D5: the pair must be chosen and justified before the corrected prediction is measured,
so the justification cannot be written backwards from the verdict.

- [ ] 1.1 Record the chosen pair — **N₂–N₂, μ = 14.00** (design D2, settled under the high-fidelity goal) — with its justification: it is the conventional two-temperature baseline, it is what the code's comment intended, and it is the largest plausible μ and therefore the most conservative prediction, so it cannot be accused of being chosen for its verdict
- [ ] 1.2 Record the alternatives with their reduced masses and the resulting `τ·p` spread (6.9e-7 to 1.08e-6 atm·s at 8044 K), so the sensitivity is visible: N₂–N₂ 14.00, N₂–O₂ 14.93, N₂–O 10.18, N₂–N 9.33
- [ ] 1.3 Record that the single pair stands in for a chemically mixed bath (`x_N₂ ≈ 0.46`, `x_O ≈ 0.31`, `x_N ≈ 0.23` at this condition) and that it biases `τ_vt` **long** — lighter partners relax faster, so pure N₂–N₂ under-predicts `nₑ`, which under-predicts blackout and is optimistic about comms availability (design D2a)
- [ ] 1.4 Note the mixture-weighted follow-up as a named next step, not a standing disclaimer: `1/τ_mix = Σ_r x_r/τ_(s,r)`, using the composition the harness already computes

## 2. De-duplicate the constant at its current value

Design D1/D4 and the migration plan step 1: land the refactor separately so the physics diff shows only
the number moving.

- [ ] 2.1 Establish one definition in `deep_causality_cfd`, sited with the `Park2tClosure` documentation that describes it
- [ ] 2.2 Point `examples/avionics_examples/src/shared/constants.rs` at the crate definition instead of restating the literal
- [ ] 2.3 Confirm `examples/avionics_examples/src/shared/world.rs` (2 call sites) and `verification/qtt_ramc_stagline/main.rs` resolve to the single definition
- [ ] 2.4 Verify nothing changed numerically: RAM-C harness output and the three examples reproduce their committed artifacts exactly

## 3. Correct the pair and derive the reduced mass

- [ ] 3.1 Replace the literal with a derivation from the chosen pair's constituent masses, with both masses stated in amu and the `μ = m_s·m_r/(m_s + m_r)` relation visible at the definition
- [ ] 3.2 Correct the four doc comments that assert the old value or its wrong derivation: `fitting.rs` (`Park2tClosure::reduced_mass_amu`), `src/types/flow/blackout.rs`, and both former constant sites
- [ ] 3.3 Update the two test fixtures pinning `7.0` in `tests/solvers/qtt/compressible_fitting_tests.rs`
- [ ] 3.4 Add a test pinning `μ` against the named pair's constituent masses, so editing one without the other fails
- [ ] 3.5 Add a test rejecting a monatomic relaxing species (no vibrational mode), per the spec's second scenario
- [ ] 3.6 Record the Millikan–White citation at the implementation with `A_sr`, `B_sr`, the `−18.42` constant and the unit convention (μ amu, p atm, τ s, θ_v K)

## 4. Measure before touching any gate

Design D5. Do not edit a band until this group is complete and its numbers are written down.

- [ ] 4.1 Run `qtt_ramc_stagline` and record: peak `n_e`, ionization fraction α, `T_ve` after relaxation, `T_a`, plasma frequency, blackout determination, and the decade offset vs the RAM-C II anchor
- [ ] 4.2 Run the three plasma-blackout examples (corridor, weather, retropulsion) and record every electron-density-derived figure that moved, including blackout onset/exit/dwell
- [ ] 4.3 Record the measured `τ_vt` change against the pre-correction value, confirming the direction predicted in the proposal (~1.9× longer at μ 7→14)
- [ ] 4.4 Note which harness gates now fail, before deciding what to do about any of them

## 5. Re-derive the acceptance bands

Design D3. A band is re-derived from the corrected closure and its uncertainty — never widened to
re-admit the previous headline.

- [ ] 5.1 Re-derive the `qtt_ramc_stagline` peak-`n_e` band from the corrected physics, stating what it encodes and its evidence class
- [ ] 5.2 Re-derive or retire the ±0.70-decade network band; it was pinned under the superseded `μ_sr`
- [ ] 5.3 If the corrected prediction cannot support an order-of-magnitude claim against the RAM-C II anchor, make the harness report the measured offset as its result rather than presenting a re-tuned band as agreement
- [ ] 5.4 Re-check the blackout-onset gate: it is a deterministic consequence of the `n_e` gate, so confirm it still carries information after the bands move
- [ ] 5.5 Update any plasma-blackout example gate whose bound derives from electron density

## 6. Re-baseline the evidence

- [ ] 6.1 Regenerate `verification/qtt_ramc_stagline/baseline.txt` from a clean run (stdout+stderr, per the baseline convention)
- [ ] 6.2 Regenerate the three plasma-blackout examples' `output.txt` and CSV artifacts from clean runs
- [ ] 6.3 Update the `qtt_ramc_stagline` section and summary row in `deep_causality_cfd/verification/README.md`
- [ ] 6.4 Update the crate README's RAM-C figures, including the "Everything Self-Verifies" paragraph
- [ ] 6.5 Retain the pre-correction figure and the `μ = 7.0` closure as recorded superseded history with the reason (design D6), following the harness's existing precedent for the single-temperature surrogate

## 7. Verify

- [ ] 7.1 `μ` equals `m_s·m_r/(m_s + m_r)` for the named pair, and the test added in 3.4 fails if either is edited alone
- [ ] 7.2 The constant has exactly one definition; grep confirms no site restates the literal
- [ ] 7.3 `cargo test -p deep_causality_cfd --release` — no regression against the 828-pass baseline
- [ ] 7.4 `make format && make fix` clean, no new `#[allow]`
- [ ] 7.5 Every acceptance band in the RAM-C chain states the closure it was derived under and its evidence class
- [ ] 7.6 No band was widened to restore the previous agreement — each change to a bound is traceable to a measurement from group 4
- [ ] 7.7 The three examples reproduce their regenerated artifacts on a second run (determinism preserved)
- [ ] 7.8 Confirm the diff touches no file outside the RAM-C chemistry chain and its evidence — the Park model, the finite-rate network and the Saha surrogate are Non-Goals
