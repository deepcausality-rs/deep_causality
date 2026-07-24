# Fix the RAM-C vibrational-relaxation collision pair

## Why

`REDUCED_MASS_AMU = 7.0` is not a valid Millikan–White reduced mass for any nitrogen vibrational
relaxation pair, and it is the constant the crate's headline scientific claim rests on.

Millikan–White gives the vibrational relaxation time as `τ_vt·p = exp[A(T^(−1/3) − B) − 18.42]` with
`A = 1.16e-3·μ_sr^(1/2)·θ_v^(4/3)` and `B = 0.015·μ_sr^(1/4)`, where `μ_sr` is the reduced mass of the
colliding pair — `s` the relaxing **diatomic**, `r` the collision partner. The committed value matches
none of the candidates:

| Pair | μ (amu) |
|---|---|
| N₂–N₂ | **14.00** — what the code's own comment claims |
| N₂–O₂ | 14.93 |
| N₂–O | 10.18 |
| N₂–N | 9.33 |
| N–N | **7.00** — the committed value |

7.00 amu is the N–N pair: two nitrogen *atoms*. Atomic nitrogen is monatomic and has no vibrational
mode, so N–N is not a vibrational-relaxation pair at all. This is not a defensible choice recorded
under the wrong label — it is a value with no physical referent, produced by an arithmetic slip in the
comment that derives it (`N₂–N₂ ≈ 14·14/28 = 7` uses nitrogen's *atomic* mass 14 where N₂'s molecular
mass 28 belongs).

**The error is load-bearing.** `μ` enters inside the exponential. At the harness's post-shock
`T = 8044 K` and `θ_v = 3393 K`, correcting `μ: 7 → 14` moves `τ·p` from `5.42e-7` to `1.02e-6 atm·s`
— `τ_vt` roughly **1.9× longer**. A longer relaxation time keeps the vibrational-electron temperature
`T_ve` colder for longer, so the Park rate-controlling temperature `T_a = √(T_tr·T_ve)` is cooler and
the predicted electron density falls. The pre-certification audit's module report measured the
corrected prediction at roughly **−1.27 decades** against the RAM-C II anchor.

That is the uncomfortable part and the reason this change is scoped alone: correcting the physics is
expected to **remove** the crate's headline agreement (`peak nₑ = 1.08e19`, "+0.0 decades vs the RAM-C
II anchor"), not preserve it. The audit is explicit that the gates must be re-derived from the
corrected physics rather than re-tuned to restore the previous number.

Audit blocker **B-1**, `openspec/notes/cfd_audit/AUDIT-REPORT.md` §4 and §9 Phase 2 item 7.

## What Changes

- **Correct the collision pair and its reduced mass**, at both definition sites, with the pair named
  and its `μ` derived in the doc comment rather than asserted.
- **BREAKING (result-level):** re-baseline the RAM-C chain. `qtt_ramc_stagline`'s reported peak
  electron density, ionization fraction, plasma frequency and blackout determination all move. The
  harness's acceptance bands are re-derived from the corrected physics; the previous agreement figure
  is retained in the README as recorded history, not restored.
- **BREAKING (result-level):** re-baseline the three plasma-blackout examples. The constant reaches
  the corridor, weather and retropulsion examples through
  `examples/avionics_examples/src/shared/world.rs`, so their committed `output.txt`, CSV artifacts and
  any gate bounds derived from electron density change with it.
- **De-duplicate the constant.** It is currently defined independently in two places
  (`verification/qtt_ramc_stagline/config.rs` and `examples/avionics_examples/src/shared/constants.rs`),
  which is how one arithmetic slip reached two subsystems. One definition, one derivation, one citation.
- **State the pair selection as a physical judgement**, not a default. At RAM-C post-shock conditions
  the air is partially dissociated, so N₂–N₂ is not automatically the dominant partner; the spec
  requires the chosen pair to be justified against the flight condition and the alternatives recorded.

Explicitly **not** in scope: the Park two-temperature model itself, the finite-rate ionization network,
the Saha surrogate, the `T_e = T_ve` lumping, and the single associative-ionization channel. Those are
the documented open levers in the harness's own disclaimer and are unaffected by `μ_sr`.

## Capabilities

### New Capabilities

- `vibrational-relaxation-closure`: the Millikan–White vibrational relaxation closure as a specified
  contract — the collision pair is named and physically justified, `μ_sr` is *derived* from that pair's
  constituent masses rather than asserted as a literal, the constant has exactly one definition, and
  the closure carries its citation. This capability exists because the defect was not a wrong number
  so much as an unchecked one: nothing in the codebase related `μ_sr` to a pair, so an arithmetic slip
  in a comment became the physics.

### Modified Capabilities

- `compressible-qtt-validation`: the RAM-C stagnation-line requirement's acceptance criteria are
  re-derived from the corrected relaxation closure. The requirement currently gates peak electron
  density against the RAM-C II reference "within a recorded tolerance"; that tolerance was earned
  under the incorrect `μ_sr` and must be restated, together with what the corrected prediction is and
  whether it still supports an order-of-magnitude claim.

## Impact

**Code**
- `deep_causality_cfd/verification/qtt_ramc_stagline/config.rs` — the constant, its derivation, its citation.
- `examples/avionics_examples/src/shared/constants.rs` — the duplicate definition.
- `examples/avionics_examples/src/shared/world.rs` (2 call sites) — consumes the constant.
- `deep_causality_cfd/src/solvers/qtt/compressible/fitting.rs` — `Park2tClosure::reduced_mass_amu`
  doc comment (`N₂–N₂ ≈ 7`), and `src/types/flow/blackout.rs`'s corresponding doc.
- `deep_causality_cfd/tests/solvers/qtt/compressible_fitting_tests.rs` — two test fixtures pin `7.0`.

**Evidence**
- `qtt_ramc_stagline` acceptance bands, its `baseline.txt`, and its README section.
- The three plasma-blackout examples' `output.txt` and CSV artifacts.
- `deep_causality_cfd/verification/README.md` and the crate README's RAM-C figures.

**Risk**
- The corrected prediction may fall outside any band that can honestly be called agreement with RAM-C
  II. If so, that is the finding, and the harness must report it as such — a band widened to
  re-admit the old headline would be exactly the back-fitting the audit exists to remove.
- No public API change; `deep_causality_cfd` is `publish = false`, so no downstream release impact.
- The DEC solvers, the compressible marchers and the navigation stack are untouched.
