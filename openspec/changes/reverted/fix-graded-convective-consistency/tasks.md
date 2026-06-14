# Milestone structure

Four groups. Group A is the **gating spike** — it decides the default mechanism before any
production code lands; Groups B–D do not open until A confirms a mechanism recovers order.
Group B is the default (surgical correction); Group C is the fallback (Galerkin star); Group
D is the validation gate. Each group ends green (full tests on touched crates in both
feature configs, clippy/fmt clean) with a prepared commit message.

Per AGENTS.md golden rules: agents never `git commit` and never delete files — each group
gate prepares a commit message and asks the user to commit. `make` targets are run by the
user on review.

> **Already done (the measurement that scoped this change):** the `dec_graded_mms` example
> measures both operators in both norms. Verdict on record: the **viscous** operator is 2nd
> order on graded in both norms (excluded); the **convective** operator loses order in both
> norms (the target).

## A. Gating spike — confirm a mechanism recovers convective order

- [ ] A1 Prototype the surgical off-centering correction (D2) on the 2D convective MMS;
      derive the leading `∇ℓ` term from the truncation analysis of `±⋆(⋆ω ∧ X♭)`.
- [ ] A2 Run the prototype through `dec_graded_mms`: confirm the convective **L2 order
      returns to ≈ 2** across the amplitude sweep, and that it is **identically zero on
      uniform spacing**.
- [ ] A3 If A2 fails (correction not cleanly derivable/stable), prototype the Galerkin/Whitney
      Q1 star (D3) instead and confirm L2-order recovery on the same sweep.
- [ ] A4 Record the decision (default = D2 surgical, or D3 Galerkin) in `design.md` D2/Open
      Question 2, with the measured order table. Gate: no production code until a mechanism
      is confirmed.

## B. Default — surgical convective consistency correction (graded-convective-order)

- [ ] B1 Implement the confirmed off-centering correction in the `deep_causality_physics`
      convective rate assembly (`i_u(du)`), composed onto the existing vector-slot
      skew-symmetrized term; correction zero on uniform spacing.
- [ ] B2 Order test: the `dec_graded_mms` convective L2-order column returns to ≈ 2 across
      amplitudes (the headline acceptance), wired as the example's reported result.
- [ ] B3 Uniform non-regression: the Taylor–Green convergence table, Couette/Poiseuille
      exactness, and the energy budget are bit-unchanged (the correction vanishes on uniform).
- [ ] B4 Composition: the energy-neutrality and long-horizon stability gates of
      `fix-dec-convective-instability` stay green with the correction applied.
- [ ] B5 Group gate: format, clippy, full physics + topology tests both feature configs;
      prepare the Group B commit message and ask the user to commit.

## C. Fallback — Galerkin / Whitney (Q1) Hodge star (graded-convective-order)

- [ ] C1 Implement the opt-in cubical Whitney (Q1) Galerkin Hodge star in
      `deep_causality_topology` alongside the diagonal `has_hodge_star`; the diagonal star
      stays the default.
- [ ] C2 Equivalence on uniform: the Galerkin star reproduces the diagonal star's results to
      rounding on uniform meshes; second-order on strongly graded meshes where the surgical
      correction's smoothness assumption breaks.
- [ ] C3 Confirm the Q1 assembly composes with the Stage-3 boundary-corrected star
      (mixed-periodicity / walled lattices); a perf note on the banded Leray/Poisson solve.
- [ ] C4 Group gate: format, clippy, full topology tests both feature configs; prepare the
      Group C commit message and ask the user to commit.

## D. Validation gate

- [ ] D1 The `dec_graded_mms` example reports the recovered convective L2 order (≈ 2 with the
      default mechanism) alongside the unchanged viscous column; README updated.
- [ ] D2 Structure unchanged: `leray_projection_stays_divergence_free_under_strong_grading`
      and the divergence-free gates stay green at every grading.
- [ ] D3 Group gate: full validation ladder + examples green in both feature configs; prepare
      the final commit message. Change exit: second-order convective accuracy on smooth
      graded meshes, with the Galerkin fallback available for rough (cut-cell / AMR) meshes.
