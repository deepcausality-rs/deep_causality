<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

Staged: each stage ends with a self-verifying gate. Stage 4 (RAM-C stagnation line) is the buildable milestone
and a valid standalone deliverable. Stages 5–6 carry the named open-research nodes. Prerequisite:
`add-park2t-blackout-tier-a` (the reused physics layer).

## 0. 3-D QTT codec + operators (`tensor_bridge/`)

- [x] 0.1 `quantize_3d` / `dequantize_3d` (`tensor_bridge/codec.rs`) — dense `2^Lx×2^Ly×2^Lz` ⇄
  `CausalTensorTrain`, matching the 2-D bit-ordering. Round-trip + dimension-mismatch tests.
- [x] 0.2 `gradient_x` / `gradient_y` / `gradient_z` / `laplacian_3d` + a divergence helper
  (`tensor_bridge/operators.rs`), hand-built via `from_cores` shift operators + the stencil algebra (the 2-D
  pattern extended). Tests: each matches its analytic derivative to scheme order; bounded rank on smooth input.
- [x] 0.3 Register test modules in `mod.rs` + `tests/BUILD.bazel`; crate-root imports; 100% coverage.

## 0.5 `MetricProvider` seam — body-fit as data, not a code path (D8, `coordinate/`)

- [x] 0.5.1 `MetricProvider<R>` trait (static dispatch): `dims` / `sample` / `physical_gradient` / `jacobian`,
  so the marcher runs generically over any structured coordinate. `CartesianIdentity` (capture limit) and
  `BodyFittedCoordinate` (fitted limit) both impl it; tests gate identity-gradient-vs-analytic, the constant
  cell-volume Jacobian, the fitted chart reached through the trait, and a generic-over-`M` consumer.
- [x] 0.5.2 `BlendedMap` (the continuous `λ` dial, `coordinate/blended.rs`) — implements `MetricProvider` from a
  `BlendedMapConfig`. The two charts' **forward** Jacobians blend linearly (`J_λ = (1−λ)J_cart + λ J_fit`),
  inverted pointwise to the low-rank inverse metric the marcher consumes (sampled analytically — no TT
  reciprocal). `λ=1` reproduces `BodyFittedCoordinate` exactly, `λ=0` is the Cartesian-capture rectangle;
  validity (BM-A) holds by construction (compatible charts, derived fan chord). Gates: λ=1 metric match,
  free-stream-exact across λ, and the **rank dial** BM-B (capture bond → `O(10)` fitted), per
  `studies/qtt_blend_metric`.

## 1. Body-fitted / shock-aligned coordinate (`coordinate/`)

- [x] 1.1 A smooth analytic curvilinear map (sphere-cone baseline) aligning wall + bow shock to coordinate
  surfaces; the Jacobian/metric computed from geometry (no hardcoded components), carried as a low-rank TT.
- [x] 1.2 Chain-rule transform of the §0 operators to physical derivatives via the Jacobian. Test: physical
  gradient matches analytic to scheme order.
- [x] 1.3 **Rank-lever gate** (study-style example): a representative curved shell is `χ ~ O(10)` and
  resolution-independent in the fitted coordinate, vs `χ ~ √side` captured on Cartesian — matching
  `studies/qtt_rank_3d`. Free-stream preservation holds discretely.

## 2. Compressible conservative flux (`solvers/qtt/compressible/flux.rs`)

- [x] 2.1 Conservative state `(ρ, ρu, ρv, ρw, ρE, {ρY_s})` as tensor trains; conservative flux divergence
  `∇·F(U)` via the §0/§1 operators (telescoping/conservative).
- [x] 2.2 Approximate Riemann flux — Rusanov/LLF baseline (state-derived wave speed), HLLC option; reduces to a
  centred flux on smooth fields.
- [x] 2.3 EOS closure `p,T = EOS(ρ,e,{Y_s})` (ideal-gas baseline; Tier-A 2-T mixture option) via TT-cross,
  bounded rank on smooth fields.
- [x] 2.4 **Gate: Sod shock tube** — `verification/qtt_sod/` self-verifying example vs the exact Riemann
  solution (ρ/u/p, shock/contact/expansion speeds); conservation + free-stream preservation tests.

## 3. IMEX time integration + conservation/positivity (`solvers/qtt/compressible/imex.rs`)

- [x] 3.1 IMEX step (`AcousticImex1d`) — explicit convection, **implicit acoustic** on the **D10 split**
  (constant-coefficient core implicit, variable remainder lagged), so the solve is always against the
  well-conditioned core. The core is advanced by its **closed-form low-rank inverse**
  (`AcousticCoreInverse`, `tensor_bridge/acoustic_inverse.rs`): `A₀ = (s/ρ)(I−ρS₊)(I−ρS₋)` factors exactly
  through the cyclic shift, so `A₀⁻¹ = (ρ/s)R₋R₊` is applied in `O(l)` shift-applies by binary doubling — **no
  iterative solve, no AMEn-convergence gamble**, and **free-stream-exact** (the property an AMEn-per-step solve
  loses to its residual tolerance). **D10 gate 1** (`A₀A₀⁻¹ = I` to round-off, resolution-stable bond) and the
  isolated free-stream/run gates pass, realizing the `studies/qtt_acoustic_precond` closed-form-core ideal.
- [x] 3.2 Conservation-preserving rounding (`conservation_round`) — carry the conserved total + rank-1 uniform
  fixup; tests: coarse-round integral restored, and **zero secular mass drift** over a 200-step run.
- [x] 3.3 Positivity (`positivity_floor` limiter; entropy/log-variable evolution noted as the structural
  upgrade); test positivity through a steep front. **Gate: stability beyond the explicit acoustic CFL** —
  `imex_stable_beyond_explicit_cfl` (fully-explicit control diverges at acoustic diffusion number 1.0; IMEX
  stays bounded). Built on the **isolated 1-D acoustic operator** (task 3.1 "in isolation first"); the full
  system coupling lands in the Stage-5 marcher.

## 4. Shock fitting + the RAM-C stagnation line — the buildable milestone (`solvers/qtt/compressible/fitting.rs`)

- [x] 4.1 Fitted interface with exact Rankine–Hugoniot jump (`FittedNormalShock`); **1-D fitted normal
  shock** — post-shock `ρ/u/p/T` match the exact RH relations at the flight Mach (test); the smooth
  post-shock relaxation profile is `O(1)` rank (bond 2). No flux is marched *through* the front.
- [~] 4.2 Interface motion — for the 1-D **stagnation line** the fitted shock is a standing normal shock (no
  motion); the **dynamic** interface motion + per-step bulk coupling is the 2-D bow-shock concern, deferred to
  Stage 5.
- [x] 4.3 **RAM-C stagnation line**: the exact-RH `T₂` is the **transported energy** (recovery-temperature
  reconstruction retired), driving the **reused Tier-A** Saha/Park-2T ionization in the smooth post-shock zone
  with the **grounded nonequilibrium lag** (`τ_ion = 1/(k_f·n₂)`, the Park associative-ionization rate).
  **Gate: peak electron density / blackout onset vs RAM-C II** within ~2 decades (order-of-magnitude;
  measured `n_e ≈ 1.2e20`, +1.1 decades vs the `1e19` anchor) — `verification/qtt_ramc_stagline/` (exit 0).

## 5. 2-D body-fitted compressible reacting marcher (`solvers/qtt/compressible/marcher_2d.rs`)

- [~] 5.1 Assemble the 2-D compressible reacting marcher (`CompressibleMarcher2d`) over the `MetricProvider`
  seam, **implementing `Marcher`** (`advance` = one IMEX step on the tensor-train state `EulerStateTt2d`, gated
  by `marcher_trait_advance_matches_one_step_and_preserves_free_stream`). The time step is **IMEX** (design
  D10): explicit convective flux + **implicit acoustic dissipation** via the closed-form 2-D ADI inverse
  (`AcousticCoreInverse2d` = `(I−β∂ₓ²)⁻¹(I−β∂ᵧ²)⁻¹`), the 2-D system analogue of `AcousticImex1d` —
  free-stream-exact, bounded beyond the explicit acoustic-diffusion limit, no iterative solve. **Remaining:**
  the `CfdFlow`/`QttMarchRun` enum wiring (mirroring `QttImmersed2d`) for an end-to-end DSL march.
- [~] 5.2 Blunt-body bow shock — `verification/qtt_blunt_body_2d/` (exit 0). **Gate (static rank lever,
  passes):** the bow shock is a constant-physical-radius surface, so its χ is **bounded + resolution-stable**
  in the fitted coordinate (`BlendedMap` λ=1: χ 3→5, flat) vs the Cartesian capture (λ=0: χ 16→61, growing
  ~√side and overtaking). The same `CompressibleMarcher2d` runs both over the `MetricProvider` seam.
  **Reported, not gated (open):** marching a flux-through-front in the fitted coordinate still grows χ —
  re-pinning + an exact-RH interface (no flux marched across the front) is the open remainder (D9 /
  `qtt_repin_marcher`); the Tier-A LER reacting stages are unchanged from Stage 4.

## 6. 3-D forebody (`solvers/qtt/compressible/marcher_3d.rs`)

- [~] 6.1 3-D compressible Euler marcher (`CompressibleMarcher3d`, `marcher_3d.rs`) — conservative state
  `(ρ, ρu, ρv, ρw, ρE)` on a periodic Cartesian `2^Lx × 2^Ly × 2^Lz` lattice (the §0 3-D operators), with the
  §2 flux/EOS machinery in 3-D and the **IMEX** time step: explicit convective `∂ₓF+∂ᵧG+∂_zH` + implicit
  acoustic dissipation via the closed-form **3-D ADI inverse** (`AcousticCoreInverse3d` =
  `(I−β∂ₓ²)⁻¹(I−β∂ᵧ²)⁻¹(I−β∂_z²)⁻¹`). Implements `Marcher`; free-stream-exact, stable in 3-D. The 1-D inverse
  was also made **exact at all N** (the `1/(1−ρ^N)²` finite-sum correction), so 3-D free-stream holds to
  round-off at small `l`. **Remaining (= §1 in 3-D):** a 3-D body-fitted `MetricProvider` (the marcher is
  Cartesian-capture so far); the **wake** stays out of scope.
- [~] 6.2 **3-D forebody sheath** — `verification/qtt_reentry_3d/` (exit 0). **Gate (bounded forebody χ,
  passes):** the curved bow-shock sheath is a constant-radius surface, so its χ is **bounded + flat at scale**
  in the body-fitted (radial-axis) representation (χ 2→4, plateau) vs the Cartesian capture (χ 10→59,
  growing) — the 3-D rank lever on the crate's `quantize_3d` codec. **Reported, not gated:** the **wake**
  bond (a separated multi-lobe structure, χ≈41) — **out of scope** (turbulence; no single fitted coordinate
  aligns it), a datapoint for the standing `qtt_rank_3d` question (D9). The dynamic *marched* forebody rank
  (Cartesian `CompressibleMarcher3d`) is reported as the open remainder — a **3-D body-fitted
  `MetricProvider`** + re-pinning is what would bound the marched χ (not yet built).
- [x] 6.3 Cross-references with scope labels (`verification/README.md`): the QTT compressible gates are
  labelled by validation tier — **analytic** (`qtt_sod` vs exact Riemann), **flight-data order-of-magnitude**
  (`qtt_ramc_stagline` `n_e` vs RAM-C II; Apollo dwell = corridor-time anchor), and **structural / rank-lever**
  (`qtt_blunt_body_2d`, `qtt_reentry_3d` — gate rank, not accuracy). Dynamic marched-rank + wake reported, not
  asserted.

## 7. Finalize

- [x] 7.1 `cargo fmt -p deep_causality_cfd` applied; clippy `--all-targets` **clean** (fixed, not suppressed
  — `too_many_arguments` resolved via tuple/config params, `approx_constant`, `needless_range_loop`,
  `neg_cmp_op_on_partial_ord`); `cargo test -p deep_causality_cfd` **green (393 passed)**; every verification
  example exits 0 (`qtt_sod`, `qtt_ramc_stagline`, `qtt_blunt_body_2d`, `qtt_reentry_3d`). Only `cfd` was
  touched (no `deep_causality_tensor` changes).
- [x] 7.2 Constraints verified: static dispatch only (marchers generic over `MetricProvider`, no `dyn`); no
  `unsafe`/lib-macros; crate-root imports (the one intra-crate `tensor_bridge::operators` path is a documented
  `pub(crate)` lift helper); lib float literals only via `R::from_f64` / `R::one()/two/four` (no bare
  literals); dynamic-by-construction (metric/flux/EOS from state); new lib code covered (every public method +
  error path tested); tests registered in `mod.rs`, Bazel test targets are glob-covered, verification examples
  cargo-only (consistent with `qtt_sod`).
- [x] 7.3 `openspec validate add-cfd-compressible-qtt-marcher --strict` **passes**.
- [x] 7.4 Notes updated: `gap-2/tier-b-compressible-marcher.md` §1a now reads **Stages 0–6 built and gated**
  (with the named open remainders); `gap-analysis.md` §4 Gap-2 updated to **Stages 0–6 built**; the 3-D
  **forebody** §6.2 gate recorded; the 3-D **wake** rank kept as the standing **out-of-scope** `qtt_rank_3d`
  research question (not claimed resolved).
