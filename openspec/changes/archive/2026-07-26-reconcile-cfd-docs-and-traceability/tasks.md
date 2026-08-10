## 1. Reconcile the DEC kernel and spectral-projector docs (item 17)

Concrete, high-value, and self-contained — the code is confirmed correct, so every edit here is prose
describing the operator the code already marches.

- [x] 1.1 Corrected the DEC NS rate kernel docstrings to the skew-symmetrized `conv' = ½[G_ω u − G*_ω u]`
      (with `G_ω : x ↦ i_x du♭`, `G*_ω` its M-adjoint), citing the fix-dec-convective-instability change:
      module doc + type doc + `eval_projected` doc + `eval_unprojected` doc + the `energy_budget` comment in
      `dec_ns_rate.rs`; the governing block, the placement paragraph, and the module-layout line in
      `src/solvers/dec/mod.rs`; and the `rate` docstring in `src/theories/incompressible_dec.rs`. **Found
      against the tree:** `eval_unprojected` also marches the skew form (`convective_skew_generic`), so the
      raw `−i_u(du♭)` is not a marched or exposed surface; the docstrings that named it were all wrong, not
      only the projected ones.
- [x] 1.2 Corrected both spectral-projector comments in `src/tensor_bridge/projection.rs`: the eigenvalue
      comment now names the consistent `−sin²(2πk/N)/dx²` the code applies (was the compact 5-point
      `−(2−2cos)/Δ²`, which also contradicted the correct comment three lines above), and the solve comment
      now reads `p̂ = rhs/λ = −rhs/(lamx+lamy)`, matching `inv = 1/λ` (was `rhs/(−λ)`, the wrong sign).
- [x] 1.3 Corrected `src/solvers/dec/mod.rs`: the placement paragraph now describes the in-stage projection
      (`P∘rhs` per `Rk4` stage, no splitting error), replacing the "Chorin placement / first order in time"
      prose. **Confirmed against the code first** (step.rs:65-69 runs `Rk4` over `eval_projected`; line 47
      states there is no unprojected marching path), so the doc was the misstatement, not the code.
- [x] 1.4 Verified each edit against the implementation, not the prior comment: read
      `fill_convective_skew_fused`, `convective_skew_generic`, `eval_unprojected`, and the `step.rs` RK4
      closure. The skew form and in-stage projection are what the code marches.

## 2. Qualify the convergence claim (item 21)

- [x] 2.1 Qualified the order claim as **2nd-order in space, 1st-order in time (explicit Euler at fixed
      `dt`)** at the printed `render()`/`summary()` in `print_utils.rs`, the TG `README.md`, and
      `verification/README.md`; regenerated `baseline.txt` (2 lines changed, gates still PASS).
- [x] 2.2 Documented the temporal floor and the maximum usable ladder length on `DT`/`STEPS` and in both
      READMEs, with the signed-error table and the `max_level = 5` cap.
      **CORRECTED DURING THE D7 PASS — my first conclusion here was wrong.** I ran the ladder to N=64,
      measured `5.9e-6` at observed order `3.16`, and concluded "the audit's ~1e-5 temporal floor is not
      observed; still converging at ≥2nd order". That is false, and I shipped it into a source docstring
      and two READMEs. My N=128 confirmation run had timed out, and I wrote the conclusion without it.
      The audit's module report had already re-derived the truth, and I re-derived it independently in
      closed form: the signed error runs `+9.8e-4, +2.4e-4, +5.3e-5, +5.9e-6, −5.9e-6, −8.8e-6` for
      `N = 8…256`. The temporal error has the **opposite sign** to the spatial error, the two **cancel**
      near N=64–128, the `3.16` is a **cancellation artifact**, and the order collapses to **0.02** at
      N=128 — so the documented `max_level 7` **fails** the order gate. The floor is real (≈1e-5,
      asymptotically ≈2e-5); the committed 8²–32² ladder is legitimate precisely *because* it stops
      below it. All four sites rewritten to state this.
      **Lesson (the audit's own, landing on me): I treated an unfinished measurement as a completed one,
      and read a super-convergent order as good news rather than as the classic cancellation signature.**

## 3. Reconcile the doc-overclaim catalogue (item 16)

The unit of work is the ACTION-LIST row, not the number. Disprove each claim against the code before
rewriting it (D2): a false *and* unenforced "by construction" claim is a correctness finding, not a doc
edit — escalate it.

- [x] 3.1 Enumerated the catalogue mechanically (script over `ACTION-LIST.md`): **86 `doc-overclaim` +
      39 `doc-gap` = 125 rows**, grouped by file. Working list kept in the session scratchpad. Confirmed
      closed by Phases 1–2 and excluded: 7.1 (`blended.rs` fold / B-4), 8.1 (`boundary_zone` hook /
      item 15), 4.14 (`observe.rs` rename / item 11), 1.1 + 12.1 + 1.6 (Phase-1 gates and CI), 10.1
      (`REDUCED_MASS_AMU`), 6.18 + 12.4 (ESKF, item 9), 15.4 (RAM-C framing, item 19).
- [x] 3.2 Worked the rows by file, checking each claim against the code before rewriting. Closed this
      session (site → what was wrong):
      **DEC/spectral (Group 1):** 9.3 + 16.1 + 16.9 (`dec/mod.rs` Chorin split and symbols), 9.4
      (`dec_ns_rate` convective operator, 6 sites), 9.8 (`eval_projected` documented "infallible" but
      returns `Result`; `eval_unprojected` is the infallible one), 7.6 + 4.16 (`projection.rs` eigenvalue
      and sign comments).
      **Crate README:** 15.14 (χ²·L is *storage*, not runtime — item 10 measured runtime rising far
      faster at flat bond), 15.5 (shock-fitting scoped to the descent-schedule case), 11.3 (Knudsen
      classifies and logs; no slip/transitional/free-molecular closure exists — verified: `GoverningModel`
      has no consumer outside `regime.rs`), 6.3 (`RegimeSwitch`/`aero_gravity_ratio` are public API with
      **no call site** in `src/`; the engine does not switch integrators), 15.10 + 10.13 (`regime()` returns
      `Option<RegimeClass<R>>` with **7** fields, not `&RegimeClass` with 4), 15.15 (corridor block now
      byte-matches `output.txt`), 15.2 (Float106 claim caveated as pre-re-baseline), 15.8 (`papers/` row
      now points at the index).
      **Verification docs:** 15.1 (`CfdFlow::qtt_march` **does not exist**; corrected to `CfdFlow::march`
      at 6 sites across 3 READMEs), 4.19 (TG file-layout table assigned responsibilities to the wrong
      files), plus the TG gate-1/gate-2 text (order qualification; gate 2 now names the shipped path).
      **Other:** 7.9 (acoustic-inverse test pointer "Resolution 6, gate 1" → real test names), 7.15
      (`tensor_bridge` references given in full + serial-per-axis mode layout stated), 14.15 + 15.12
      (`Gates` "never exits or prints" — see 7.3: **resolved by retiring `Gates`**, so the README claim is
      now literally true; `src/` holds **0** `println!`).
      Also corrected two "flagship" usages to the plasma-blackout example (standing naming preference).
- [x] 3.3 **No B-4 repeat found so far.** Every claim checked this session was a doc that overstated or
      misnamed working code, not a false claim over broken code. Two findings worth recording:
      **(a)** 6.3's claim was false in the strong sense (no call site at all), fixed by restating the
      README, since wiring the switch is a feature, not a doc fix.
      **(b)** The audit's own item-21 estimate (~1e-5 temporal floor) was **not reproducible** — see 2.2.
- [x] 3.4 **The catalogue is fully accounted for: 125 of 125 rows.** Counted mechanically against
      `ACTION-LIST.md`, not from memory.
      **Closed by this change — 116.** Worked in three passes: 22 by hand (the crate README, the DEC
      kernel and spectral-projector docs, the `CfdFlow::march` naming); 40 by hand across the rest of
      `src/`; 52 by a 13-agent fan-out over `verification/`, `studies/`, `examples/`, `tests/` and the
      two sibling crates, each agent owning a disjoint file set; then 2 more (5.7, 5.16) that I had
      listed while planning and **left out of the agent assignments** — a bookkeeping miss on my part,
      caught by re-running the count rather than by review.
      **Confirmed already closed by Phases 1–2 — 9:** 1.1, 1.6, 4.14, 6.9, 6.18, 7.1, 8.1, 15.4, 15.9.
      **Remaining — 0.**
      *(Two earlier drafts of this line were wrong and are recorded rather than quietly replaced. The
      first claimed "24 of 125 closed / 101 open" here while 9.3 claimed "24 + 6 closed / 101 of 125
      open", which cannot both hold; the D7 pass caught it and re-enumerated to 22. A later statement of
      "54 remaining" was also an arithmetic slip: the true figure was 52.)*
      Most remaining rows are single-site docstring corrections in `src/solvers/qtt/compressible/`,
      `src/solvers/dec/boundary/`, `studies/`, and the example READMEs; a subset (13.9, 8.17, 6.14, 15.7,
      9.5) asks for **code** changes (constructor validation, `Result` returns, re-exports, visibility)
      that are outside this change's stated no-code-change scope and should be planned separately.

## 4. Close the doc-gaps (item 18)

- [x] 4.1 **Already satisfied before this change — nothing was added.** The crate README's "Selected
      Capabilities" section already documents all four: suspend/resume
      (`save_resume_state`/`load_resume_state`), `DuctMarchRun`, `IgnitionCorridor` (via
      `ThrottleGuidance`), and `AcousticCoreInverse` (2-D/3-D). Verified with `grep -E` against
      `README.md:260-270`.
      **FINDING (method, recorded per D2):** the pre-implementation reconnaissance reported these as
      absent from the README. That negative came from a **malformed grep** — unescaped `|` alternation,
      so the shell piped between processes instead of matching alternatives — and the empty result was
      read as evidence of absence. This is §5c's lesson landing on this change's own scouting: *a claim
      resting on "X does not exist" is only as good as the search behind it, and the search must be
      stated so it can be checked.* An earlier draft of this task recorded the section as added by this
      change; that was an overclaim and is corrected here.
- [x] 4.2 **All 39 doc-gap rows closed** (37 by this change, 2 already closed by Phases 1–2: 6.18 and
      15.9). The work ran from units and conventions in `src/` (the `BodyForceZone` acceleration line
      integral, the `relax_length` fraction, the `coupling.rs` stage-ordering and stale-read rule, the
      `marcher_3d_fitted` periodic-`ζ` boundary caveat, the projection's Nyquist kernel) out to the
      harness and study documentation (five missing per-example sections in `verification/README.md`
      with their gate predicates read from source, corrected file-layout tables, the `papers/` index).
      Counted against the catalogue, not asserted.

## 5. Give load-bearing constants provenance (item 23)

Follow the crate's paper convention (full author-year citation at the definition, PDF in `papers/`).

- [x] 5.1 Added provenance to `SMOOTH_CELLS = 2.0` at both sites: numerical mask-regularization width
      (`tanh` over `SMOOTH_CELLS·dx`), not a physical constant and no external source; the 6.1× `C_d`
      sensitivity (§5b) and the resolved-envelope caveat are stated.
- [x] 5.2 Added provenance to `qtt_park2t_blackout` `ETA = 0.016`: pinned by the explicit-stability
      ratio `dt/η = 0.25`, not a wall-error target; this harness gates the LER ionization criteria, not
      drag, so the layer resolution is not a gated lever here (cross-references the cylinder derivation).
- [x] 5.3 Justified the Mach-1.05 floor: a 5% buffer above sonic; a 1-D normal shock is not the model in
      the transonic sliver `M ∈ (1, 1.05]`, where the jump is within a few percent of the identity. A
      modeling guard, not a measured constant.
- [x] 5.4 Added the full Angot / Bruneau & Fabrie (1999) citation to the cylinder config docstring (was
      text-only in `print_utils.rs`). **BLOCKED (owner):** the PDF file itself cannot be fetched here;
      the `papers/` index records it as cited-but-missing for the owner to add.
- [x] 5.5 Created `deep_causality_cfd/papers/README.md` mapping each PDF to its citing code. Flagged the
      two orphans (`mittal2005.pdf`, `mohamed2016.pdf`, cited by author name nowhere) for the owner to
      confirm-and-cite or remove. **No PDF deleted** (golden rule).

## 6. Give a load-bearing test an independent reference (item 24)

Public-API behavioural only — no source-text scraping (§5c standing rule). Verified under `bazel test //...`.

- [x] 6.1 Added `rate_pair_convection_matches_the_analytic_reference_with_nonzero_velocity` to
      `incompressible_2d_tests.rs`: drives `rate_pair` with `u = sin(x)`, `v = sin(y)` (both nonzero) on
      64², comparing against the hand-derived `ru = −½sin(2x) − ν·sin(x)`, `rv = −½sin(2y) − ν·sin(y)`;
      passes at `max_err ≈ 8e-4 < 3e-3` (bound covers centered-FD truncation). `rate_pair` applies no
      projection, so convection survives (unlike the projection-annihilated solver tests).
- [x] 6.2 Demonstrated the bite: flipping the shipped convection sign (`conv.scale(neg)` →
      `conv.scale(R::one())`) fails the test at `max_err = 0.999` (≈ 1.0, as predicted), far outside the
      bound. Fault injection reverted.
- [x] 6.3 Routed the TG harness convection check through the shipped `rate_pair` (zero-viscosity
      instance → pure convection `−(u·∇)u`), deleting the `gradient_x`/`gradient_y` re-assembly and its
      now-unused imports. Reported convection error **unchanged** (`3.207e-3`, amp `0.497`), confirming
      the re-route is behavior-preserving; `baseline.txt` regenerated.

## 7. Resolve `Gates` (item 22)

Owner decision on adopt-vs-retire; the non-deleting corrections land now.

- [x] 7.1 **Superseded by the retirement (7.3).** The empty-set fix landed first (`finish()` returning
      `false` instead of a vacuous `true`), then became moot when the owner approved retiring the type;
      the file and its tests are now removed.
- [x] 7.2 Corrected the prose: the `gates.rs` module doc no longer claims `Gates` is "the block every
      self-verifying program prints" (it names `GateSeq`/`Verdict` as what the programs use); the
      `flight_envelope_placard` README's two `Gates` type references corrected to `GateSeq` (the type
      `model.rs:161` returns). The `verification/README.md:237` "Gates" is a verb, left as-is.
- [x] 7.3 **RETIRED — owner decision, 2026-07-26.** `Gates` was a parallel API no shipped program
      constructed (`Gates::new` appeared only in its own unit test); `GateSeq`/`Verdict` is the live,
      evidence-class-labelled contract. Removed with `git rm`: `src/types/flow/gates.rs` and
      `tests/types/flow/gates_tests.rs`, plus the `mod gates;` / `pub use gates::Gates;` wiring in
      `types/flow/mod.rs`, the `Gates` re-export in `lib.rs`, and the `pub mod gates_tests;`
      registration.
      **Consequence worth recording: a doc-overclaim was fixed by correcting the code rather than
      weakening the prose.** `Gates` held the only five `println!` in `src/`, which made the crate
      README's "the DSL never exits or prints" false (rows 14.15 / 15.12). After the retirement `src/`
      contains **0** `println!`, so the README claim is now literally true and needed no softening.

## 8. Re-verify the items Phases 1–2 already delivered (items 19, 20, 23-cylinder)

Confirm still-correct; captured in the spec so they cannot silently rot. No re-implementation.

- [x] 8.1 Item 19 confirmed: RAM-C is order-of-magnitude / ±0.70-decade in both `README.md:224` and
      `verification/README.md:127,97`. Unchanged.
- [x] 8.2 Item 20 confirmed: the lid-cavity row reports the 65² default (RMSE 0.0617) at
      `verification/README.md:90,213`. Unchanged.
- [x] 8.3 Item 23 (cylinder) confirmed: `ETA = 0.012` carries its wall-error-target provenance
      (`qtt_cylinder_verification/config.rs:36-40`). Unchanged.

## 9. Verify

- [x] 9.1 `cargo clippy -p deep_causality_cfd -p avionics_examples --all-targets --all-features -D warnings`
      clean (exit 0); `cargo fmt` applied; no new `#[allow]`. **`bazel test //...` → 1153/1153 pass**
      (the workspace check that matters, per §5c). `bazel test //deep_causality_cfd/...` 15/15 PASSED.
      `cargo test -p deep_causality_cfd --release` → **895 passed, 0 failed, 2 ignored** (897
      before, minus the 3 retired `Gates` tests, plus the new `rate_pair` convection test).
      `cargo doc -p deep_causality_cfd` introduces **no new** rustdoc warning; the 12 that remain are
      pre-existing and are catalogue row 12.6.
- [x] 9.2 **No example figure moved.** The `plasma_blackout_corridor` example reproduces its committed
      `output.txt` exactly (diff empty once cargo's stderr build lines and the wall-clock line are
      excluded). The TG harness convection error is unchanged at `3.207e-3` (amp `0.497`) after the
      `rate_pair` re-route, so the re-route is behaviour-preserving; the only `baseline.txt` movement is
      the two Group-2 wording lines. All TG gates PASS.
- [x] 9.3 Counts recorded and **enumerated by ref** in 3.4 and 4.2: **all 125 catalogued rows accounted
      for — 116 closed by this change, 9 already closed by Phases 1–2, 0 remaining.** The code arms those
      rows offered are recorded as deferred with their reasons. Listing the refs makes the count
      recomputable from `ACTION-LIST.md` instead of asserted from memory, which is how three earlier
      drafts of this line went wrong (see 3.4).
- [x] 9.4 **Adversarial pass over the finished diff (D7) — run, refute-by-default, over six dimensions:
      (1) factual accuracy of every new prose claim, (2) code correctness of the changes, (3) doc–code
      parity of the rewritten docstrings, (4) overclaims in this change's own artifacts, (5) regression
      risk from the `Gates` removal, (6) completeness and honesty of the task records. Four defects
      confirmed, all mine, all fixed:**

      **(a) MAJOR — a wrong conclusion shipped into a source docstring and two READMEs (item 21).**
      I concluded the temporal floor "is not observed" and that the ladder was "still converging at ≥2nd
      order" at N=64. Both false. The N=128 confirmation run had **timed out** and I wrote the conclusion
      anyway. Re-derived in closed form: the temporal error is **opposite in sign** to the spatial error,
      they **cancel** near N=64–128 (`+5.9e-6 → −5.9e-6 → −8.8e-6`), the `3.16` is a **cancellation
      artifact**, and the order collapses to **0.02** at N=128, so the documented `max_level 7`
      **fails** the order gate. All four sites rewritten; the maximum usable ladder length (`max_level = 5`)
      is now stated, which is what item 21 actually asked for. See 2.2.
      **(b) MAJOR — an unverifiable citation added by a change about removing unverifiable claims.**
      The Kazeev–Khoromskij reference: this repository records only author and title, but I wrote year,
      venue, volume and pages into a source docstring from recall. Reduced to what the repo confirms, with
      the gap stated explicitly in `papers/README.md`. (Peddinti was checked and *is* repo-verified.)
      **(c) MODERATE — row 16.9 recorded as closed when it was not.** The governing equation still mixed
      an unsubscripted `Δ` with the `Δ_dR` defined beneath it; I had fixed only the convective half.
      Now written as `− ν Δ_dR u♭`, the operator the code applies.
      **(d) MODERATE — the count in 3.4/9.3 was internally inconsistent** ("24 of 125 / 101 open" vs
      "24 + 6 closed / 101 open", which cannot both hold) and was written from memory in the very task
      meant to make completeness checkable. Recounted and **enumerated by ref**: 22 closed, 9 already
      closed, 94 remaining at the time of that pass. (The tail was closed later in the same change; the
      final figure is 116 closed, 9 already closed, 0 remaining.)

      Also checked and **refuted** (no change needed): the `src/` print-macro claim is stronger than
      stated (0 `println!`/`eprintln!`/`panic!`/`process::exit`); all four convective call sites do march
      the skew form; no dangling `Gates` reference survives; ν = 0 is an explicitly supported
      configuration, so the harness re-route is legitimate; the convection test's measured `8.31e-4`
      matches the `≈8e-4` claimed. `MIN_ORDER` intra-doc link downgraded to plain text (private item).

      **Residue, stated plainly: 4 defects found and fixed, 94 catalogue rows deferred, and 2 code-arm
      items recorded but not taken** (the TG two-sided order gate / `dt ∝ dx²` refinement, and the
      `max_level` cap). **Not "clean" — the lesson holds a fifth time.**
- [x] 9.5 `openspec validate reconcile-cfd-docs-and-traceability --strict`: valid.
