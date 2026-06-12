# Milestone structure

Three sub-groups, each ending green (full tests passing, clippy/fmt clean)
with a prepared commit message finalizing it, so progress is de-risked at
two intermediate landing points. Group A is the perf track; Group B is the
wall substrate (transforms, star, projection); Group C is the solver
wiring and validation.

## A. Perf track — stencils, fused rate, spectral diffusion

### A1. Compiled stencil operators (dec-stencil-operators)

- [x] A1.1 Implement `DecStencilTables<R>`: per-manifold compilation of flat index/coefficient tables for `d`/`δ` (grades 0–2), diagonal Hodge factors, interior-product transport, and sharp, folding metric weights into coefficients; explicit construction API
- [x] A1.2 Streaming apply kernels over the tables (caller buffers, no allocation, `parallel`-feature fan-out with the established threshold pattern)
- [x] A1.3 Equivalence tests: every compiled operator vs the generic composition on randomized fields, 2D/3D, periodic and mixed-periodicity, unit/uniform/per-axis metrics, f64 + Float106, ≤ 100·ε
- [x] A1.4 Implement `RateWorkspace<R>` and the fused unprojected-assembly path in `DecNsRate`; keep the generic compositional path intact as the reference
- [x] A1.5 Fused-vs-generic rate equivalence test + Taylor–Green convergence-table reproduction through the fused path

### A2. Spectral diffusion (spectral-diffusion)

- [x] A2.1 Implement per-edge-family spectral evaluation of `νΔ₁u♭` on fully periodic lattices (three shifted-sub-lattice rFFT round trips; reuse the per-axis eigenvalue machinery)
- [x] A2.2 Rounding-level equivalence test vs `laplacian(1)` on 2D/3D tori incl. anisotropic spacings; typed construction error on non-periodic manifolds
- [x] A2.3 `with_spectral_diffusion()` opt-in on the solver; run the 2D Taylor–Green table with it enabled and record whether observed orders match (gates any future default-on)

### A3. Gate and close Group A

- [x] A3.1 Extend `dec_solver_benchmark` with stencil and spectral-diffusion configurations; record speedup vs the 30 ms / 11.2 ms (32³) baseline; switch the solver default to stencils only if serial ≥ 2×
- [x] A3.2 Run every existing CI ladder rung through the stencil pipeline (strategy-agnostic ladder requirement, dec-ns-validation delta)
- [x] A3.3 Group gate: `make format`, `make fix`, full tests on touched crates in both feature configurations; update the example README performance numbers; prepare the Group A commit message and ask the user to commit

## B. Wall substrate — DCT, corrected star, Neumann projection

### B1. Cosine transforms (fft-dct)

- [x] B1.1 Implement naïve O(n²) DCT-I/II/III references (test-only)
- [x] B1.2 Implement `DctPlan<R>` types II and III on the power-of-two core via the even-symmetric embedding + twiddle post-pass, Bluestein-backed for awkward lengths; II↔III round-trip tests under the documented normalization
- [x] B1.3 Implement DCT-I via the length-2(N−1) embedding; self-inverse round-trip test
- [x] B1.4 Precision-generic accuracy tests (f32/f64/Float106) vs the naïve references; extend the FFT benchmark with DCT sizes; update the fft README

### B2. Boundary-corrected Hodge star (wall-hodge-star)

- [x] B2.1 Add the clip-exponent pass (`2^{-b}` per open-axis boundary incidence) to the cubical star across unit/uniform/per-axis tiers; per-edge tier corner products gain the same factors
- [x] B2.2 Tests: interior entries unchanged; face/edge/corner exponents on open 2D/3D lattices; periodic axes unclipped on mixed lattices; fully periodic bit-unchanged
- [x] B2.3 M-symmetry test for `Δ₀`/`Δ₁` assembled with the corrected star on walled lattices
- [x] B2.4 Audit and re-derive existing open-lattice tests whose expected values assumed the interior-only star (correct the expectations, never loosen tolerances)

### B3. Neumann projection (neumann-poisson)

- [x] B3.1 Add `cg_solve_preconditioned` (Jacobi) to `deep_causality_sparse` (additive API + tests; existing `cg_solve` untouched)
- [x] B3.2 Determine the DCT type that diagonalizes the implemented boundary-corrected `Δ₀` (residual probe test decides I vs II); implement the per-axis eigenvalue tables for wall axes
- [x] B3.3 Implement the direct Neumann solve: pure-DCT path for all-walls boxes; complex-carrier mixed path (DFT periodic axes × DCT wall axes) for mixtures; gauge-mode zeroing
- [x] B3.4 Extend the `solve_laplacian` dispatch: uniform Euclidean + every axis periodic-or-walled → direct solve; otherwise preconditioned CG (boundary-corrected diagonal) where available, plain CG else
- [x] B3.5 Tests: residual at rounding vs the implemented `Δ₀` (all-walls + mixed); agreement with preconditioned CG on multiple shapes incl. anisotropic; no wall-normal flux in `dφ`; preconditioned-vs-plain iteration-count benchmark case
- [x] B3.6 Leray dispatch tests across the three domain classes (periodic DFT / walled DFT-DCT / per-edge CG) incl. the no-flux boundary-trace scenario

### B4. Gate and close Group B

- [x] B4.1 Group gate: `make format`, `make fix`, full tests on touched crates in both feature configurations; prepare the Group B commit message and ask the user to commit

## C. Wall-bounded solver — no-slip, wiring, validation

### C1. No-slip viscous operator (no-slip-viscous)

- [ ] C1.1 Implement the no-slip constraint stage (zero wall-tangential edges) as a typed chain stage + seeding constraint
- [ ] C1.2 Implement mirror-consistent viscous boundary rows (reflection folded into stencil coefficients, no ghost storage); M-symmetry test on walled 2D/3D lattices
- [ ] C1.3 Couette pure-diffusion sanity test (linear profile at discretization order)
- [ ] C1.4 Wire the constrained rows into both the generic operator path and the compiled stencil tables (the tables compile from the corrected star + constrained rows)

### C2. Solver wiring and validation (wall-bounded-ns, dec-ns-validation)

- [ ] C2.1 `DecNsSolver` acceptance of mixed/all-walls manifolds: construction validation (corrected star required), no-slip stage in the march chain, wall-aware CFL, constrained seeding; periodic suite bit-unchanged regression gate
- [ ] C2.2 Poiseuille CI rung: body-force-driven channel to steady state; profile convergence ≥ 1.9 observed order; wall-consistency assertions
- [ ] C2.3 Coarse lid-driven-cavity CI rung (≤ 64²) vs Ghia centerline tables: pinned RMSE gate + refinement trend
- [ ] C2.4 Full-resolution cavity example program emitting centerline CSVs and the vortex-center table vs Ghia (reuses the Q-criterion kernels; harness shared with the challenge-entry tooling)

### C3. Closeout and close Group C

- [ ] C3.1 Re-1600 example: adopt the stencil (+ spectral diffusion, if gated in) configuration; record the 64³ step time and update the example README performance table
- [ ] C3.2 Update READMEs (topology operator docs, physics solver docs) for the new dispatch classes and the stencil pipeline
- [ ] C3.3 Group gate: `make format`, `make fix`, `make build`, `make test` workspace-wide; both feature configurations on touched crates; prepare the Group C commit message and ask the user to commit
