<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Staged: each stage ends with a self-verifying gate. Stage 4 (RAM-C stagnation line) is the buildable milestone
and a valid standalone deliverable. Stages 5–6 carry the named open-research nodes. Prerequisite:
`add-park2t-blackout-tier-a` (the reused physics layer).

## 0. 3-D QTT codec + operators (`tensor_bridge/`)

- [ ] 0.1 `quantize_3d` / `dequantize_3d` (`tensor_bridge/codec.rs`) — dense `2^Lx×2^Ly×2^Lz` ⇄
  `CausalTensorTrain`, matching the 2-D bit-ordering. Round-trip + dimension-mismatch tests.
- [ ] 0.2 `gradient_x` / `gradient_y` / `gradient_z` / `laplacian_3d` + a divergence helper
  (`tensor_bridge/operators.rs`), hand-built via `from_cores` shift operators + the stencil algebra (the 2-D
  pattern extended). Tests: each matches its analytic derivative to scheme order; bounded rank on smooth input.
- [ ] 0.3 Register test modules in `mod.rs` + `tests/BUILD.bazel`; crate-root imports; 100% coverage.

## 1. Body-fitted / shock-aligned coordinate (`coordinate/`)

- [ ] 1.1 A smooth analytic curvilinear map (sphere-cone baseline) aligning wall + bow shock to coordinate
  surfaces; the Jacobian/metric computed from geometry (no hardcoded components), carried as a low-rank TT.
- [ ] 1.2 Chain-rule transform of the §0 operators to physical derivatives via the Jacobian. Test: physical
  gradient matches analytic to scheme order.
- [ ] 1.3 **Rank-lever gate** (study-style example): a representative curved shell is `χ ~ O(10)` and
  resolution-independent in the fitted coordinate, vs `χ ~ √side` captured on Cartesian — matching
  `studies/qtt_rank_3d`. Free-stream preservation holds discretely.

## 2. Compressible conservative flux (`solvers/qtt/compressible/flux.rs`)

- [ ] 2.1 Conservative state `(ρ, ρu, ρv, ρw, ρE, {ρY_s})` as tensor trains; conservative flux divergence
  `∇·F(U)` via the §0/§1 operators (telescoping/conservative).
- [ ] 2.2 Approximate Riemann flux — Rusanov/LLF baseline (state-derived wave speed), HLLC option; reduces to a
  centred flux on smooth fields.
- [ ] 2.3 EOS closure `p,T = EOS(ρ,e,{Y_s})` (ideal-gas baseline; Tier-A 2-T mixture option) via TT-cross,
  bounded rank on smooth fields.
- [ ] 2.4 **Gate: Sod shock tube** — `verification/qtt_sod/` self-verifying example vs the exact Riemann
  solution (ρ/u/p, shock/contact/expansion speeds); conservation + free-stream preservation tests.

## 3. IMEX time integration + conservation/positivity (`solvers/qtt/compressible/imex.rs`)

- [ ] 3.1 IMEX step — explicit convection, **implicit acoustic** via `solve::linear` (AMEn). **Gate AMEn
  convergence** on the variable-coefficient acoustic operator in isolation first.
- [ ] 3.2 Conservation-preserving rounding — carry conserved totals + rank-1 projection fixup after `round`;
  test zero secular drift of `∫ρ,∫ρu,∫ρE` over a long periodic run.
- [ ] 3.3 Positivity — entropy/log-variable evolution (and/or limiter); test positivity through a strong
  rarefaction. **Gate: stability beyond the explicit acoustic CFL** (fully-explicit control diverges; IMEX
  stays bounded).

## 4. Shock fitting + the RAM-C stagnation line — the buildable milestone (`solvers/qtt/compressible/fitting.rs`)

- [ ] 4.1 Fitted interface with exact Rankine–Hugoniot jump; **1-D fitted normal shock** — post-shock state
  matches exact RH at the flight Mach; each side `O(1)` rank.
- [ ] 4.2 Interface motion from local jump/flow state (dynamic); couple to the bulk each step.
- [ ] 4.3 **RAM-C stagnation line**: drive the **reused Tier-A** `IonizationStage`/`BlackoutTrigger` (LER
  stack, unchanged) in the smooth post-shock relaxation zone behind the fitted shock; `T_tr`/`T_ve` are the
  transported energies (reconstruction retired). **Gate: peak electron density / blackout onset vs RAM-C II**
  within a recorded tolerance — `verification/qtt_ramc_stagline/`.

## 5. 2-D body-fitted compressible reacting marcher (`solvers/qtt/compressible/marcher_2d.rs`)

- [ ] 5.1 Assemble the 2-D compressible reacting marcher implementing `Marcher`; drop into `CfdFlow`.
- [ ] 5.2 Blunt-body bow shock in the fitted coordinate to quasi-steady standoff; carry the Tier-A LER stages
  unchanged. **Gate: bounded, resolution-stable χ** vs a Cartesian-captured control that reproduces
  `χ ~ √side` — `verification/qtt_blunt_body_2d/`.

## 6. 3-D forebody (`solvers/qtt/compressible/marcher_3d.rs`)

- [ ] 6.1 3-D compressible reacting marcher (the §0 3-D operators + §1 coordinate + §2–4 machinery).
- [ ] 6.2 March and validate the **3-D forebody sheath** in the body-fitted coordinate;
  **gate bounded forebody χ** — `verification/qtt_reentry_3d/`. The **wake is out of scope** (needs turbulence,
  a non-goal; downstream of the sheath): report any wake bond dimension as an out-of-scope datapoint for the
  standing `qtt_rank_3d` research question, never gated or asserted.
- [ ] 6.3 Cross-references with scope labels: Sod analytic, RAM-C II `n_e`, Apollo dwell.

## 7. Finalize

- [ ] 7.1 `make format && make fix` (cfd + any tensor additions); clippy `--all-targets` clean (fix, don't
  suppress); `cargo test -p deep_causality_cfd` (+ `-p deep_causality_tensor` if touched) green; run each
  verification example → exit 0.
- [ ] 7.2 Constraints: static dispatch, no `dyn`/`unsafe`/lib-macros; crate-root imports; lib float literals
  confined to `constants/` via `from_f64`; dynamic-by-construction (metric/curvature/thermo from state);
  100% coverage of new lib code; tests registered in `mod.rs` + Bazel for every crate touched.
- [ ] 7.3 `openspec validate add-cfd-compressible-qtt-marcher --strict` passes.
- [ ] 7.4 Update the notes: mark the Tier-B stages reached in
  [`gap-2/tier-b-compressible-marcher.md`](../../notes/plasma-blackout/gap-2/tier-b-compressible-marcher.md)
  and `gap-analysis.md` §4/§5; record the 3-D **forebody** result (the §6.2 gate). The 3-D **wake** rank
  remains the standing **out-of-scope** `qtt_rank_3d` research question — note it as such, do not claim it
  resolved.
