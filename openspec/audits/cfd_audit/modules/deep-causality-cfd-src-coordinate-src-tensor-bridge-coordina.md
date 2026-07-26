# deep_causality_cfd :: src/coordinate/* + src/tensor_bridge/* (coordinate transformations and the CFD↔tensor-network bridge)

**Production readiness: `needs-work`**

The tensor-network layer is in good shape: I verified the shift MPO ripple-carry construction, the (S₋−S₊)/(2Δx) central difference (the 2h is present), the (S₊+S₋−2I)/Δx² Laplacian, the row-major↔mode-block codec/lift correspondence, and the closed-form acoustic-core factorization ρ = (1+2s−√(1+4s))/(2s) with its exact (1−ρ^N)² doubling correction — all against independent reference forms, and all backed by tests that compare to dense stencils rather than to the code's own output. The coordinate layer is where it fails the bar. `BlendedMap::new` divides by `det J` at every lattice point with no fold or magnitude guard (blended.rs:168-199) while its own module doc states "the constructor rejects a fold" and that validity "holds by construction"; I constructed accepted inputs (r0=1, dr=1, θ0=0, Δθ=3π/2, λ=0.25) where det J changes sign, and (Δθ=2π, λ=0) where det J ≡ 3.7e-16, both returning Ok with an inverse metric of ~1e15. Second, `jacobian()` is never consumed by any solver (grep over the whole workspace returns only tests), the flux divergence is taken in chain-rule quasi-linear form (marcher_2d.rs:244-247), so the curvilinear discretization is not in strong-conservation form despite marcher_3d_fitted.rs:8 claiming "identical conservative physics" — and the two `free_stream_preserved` tests that would catch a metric error are structurally incapable of failing. Third, `jacobian()` returns two different quantities (cell volume vs det J, differing by Nx·Ny) across impls of one trait. The bridge alone would be near-ready; the coordinate module and its verification story need work before an avionics consumer should trust a body-fitted result.

- Files read: **30**
- Findings raised: **17** — surviving adversarial verification: **17** (refuted: 0)
- Surviving by severity: critical 1, major 4, minor 9, info 3
- Independently confirmed-correct items: **10**

## Verified correct against reference

These were positively confirmed, not merely un-flagged.

| Item | Location | Reference checked against |
|---|---|---|
| BodyFittedCoordinate 2-D inverse-Jacobian (polar) metric components | `deep_causality_cfd/src/coordinate/mod.rs:176-195` | For x=r cosθ, y=r sinθ, r=r0+ηΔr, θ=θ0+ξΔθ: J=[[-r sinθ Δθ, cosθ Δr],[r cosθ Δθ, sinθ Δr]], det J = -rΔθΔr, and J⁻¹ = (1/det)[[d,-b],[-c,a]] gives ξ_x=-sinθ/(rΔθ), ξ_y=cosθ/(rΔθ), η_x=cosθ/Δr, η_y=sin |
| BodyFittedCoordinate3d spherical-shell inverse metric and volume Jacobian | `deep_causality_cfd/src/coordinate/body_fitted_3d.rs:118-146` | Spherical gradients: ∇r = r̂ = (sinθcosφ, sinθsinφ, cosθ); ∇θ = θ̂/r with θ̂ = (cosθcosφ, cosθsinφ, −sinθ); ∇φ = φ̂/(r sinθ) with φ̂ = (−sinφ, cosφ, 0). Scaled by 1/Δ for computational coords. \|∂(x,y |
| shift_plus is the cyclic +1 permutation as a bond-2 MPO | `deep_causality_cfd/src/tensor_bridge/operators.rs:49-78` | Binary ripple-carry increment mod 2^L with MSB-first mode ordering: per bit out = in XOR carry_in, carry_out = in AND carry_in; LSB injects carry_in=1; MSB drops the overflow carry. (Kazeev–Khoromskij |
| Central-difference gradient MPO carries the correct 1/(2Δx) | `deep_causality_cfd/src/tensor_bridge/operators.rs:103-108` | ∂ₓu ≈ (u[k+1] − u[k−1])/(2Δx), the standard 2nd-order periodic central difference. |
| Laplacian MPO has the correct 1/Δx² power | `deep_causality_cfd/src/tensor_bridge/operators.rs:123-130` | ∂²ₓu ≈ (u[k+1] − 2u[k] + u[k−1])/Δx², the standard 2nd-order periodic three-point stencil. |
| QTT mode layout matches the operator lifts (2-D and 3-D); no bit interleaving is claimed or used | `deep_causality_cfd/src/tensor_bridge/codec.rs:61-84, 111-142 with operators.rs:169-195, 242-287` | For a row-major [Nx,Ny] buffer with flat index i·Ny+j, reshaping to [2;Lx+Ly] puts the Lx MSB-first bits of i in the leading modes and the Ly bits of j in the trailing modes. An axis operator must the |
| Spectral Leray projection eigenvalue, null set, and FFT normalization | `deep_causality_cfd/src/tensor_bridge/projection.rs:158-179` | For the grad-of-grad operator D² with D = (S₋−S₊)/(2h), the Fourier eigenvalue is (i sinθ/h)² = −sin²(2πk/N)/h², θ=2πk/N. Its kernel is exactly k ∈ {0, N/2} per axis. p̂ = rhŝ/λ. |
| AcousticCoreInverse closed-form factorization, contracting root, and finite-sum correction | `deep_causality_cfd/src/tensor_bridge/acoustic_inverse.rs:118-143` | A₀ = (1+2s)I − s(S₊+S₋). With S₊S₋ = I, (s/ρ)(I−ρS₊)(I−ρS₋) = (s(1+ρ²)/ρ)I − s(S₊+S₋), so ρ solves sρ² − (1+2s)ρ + s = 0, i.e. ρ = (1+2s ± √(1+4s))/(2s); the contracting root takes the minus sign. Bin |
| The A₀/A₁ split used to obtain rho(A₀⁻¹A₁) = 0.59 is a genuine, exact split of the acoustic operator | `deep_causality_cfd/studies/qtt_acoustic_precond/main.rs:186-197` | A = I − Δt²c(x)²∂². Split A = A₀ + A₁ with A₀ = I − Δt²c̄²∂² and A₁ = −Δt²(c²(x)−c̄²)∂². Then A₀+A₁ = I − Δt²c²(x)∂² = A identically. |
| BlendedMap forward Jacobian is the linear blend the docs state, and its inverse is the correct cofactor form | `deep_causality_cfd/src/coordinate/blended.rs:147-199` | T_λ = (1−λ)T_cart + λT_fit ⇒ J_λ = (1−λ)J_cart + λJ_fit, with J_cart = [[0, Δr],[span_y, 0]] and J_fit = [[−r sinθ Δθ, cosθ Δr],[r cosθ Δθ, sinθ Δr]]; det = ad − bc; J⁻¹ = (1/det)[[d,−b],[−c,a]]. |

## Findings

### 7.1 [CRITICAL] BlendedMap constructor performs no fold or singularity check despite documenting one; folded and near-singular metrics are silently produced

- **Verification verdict:** CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/coordinate/blended.rs:17`
- **Auditor confidence:** confirmed

**Claim.** The module doc states the constructor rejects a fold and that det J one-signedness holds 'by construction'. No such check exists anywhere in `BlendedMap::new`, and admissible inputs produce a sign-changing (folded) det J that the constructor accepts.

**Code evidence.**

```
blended.rs:17-18 — "so `det J_λ` keeps one sign across\n//! the sweep ... and the constructor rejects a fold."
blended.rs:163-166 — "// Validity (gate BM-A) holds **by construction**: the Cartesian-capture rectangle and the polar fan\n// share orientation ..., so `det J_λ` keeps one\n// sign and stays bounded away from zero across `λ ∈ [0,1]`"
The entire guard set in the constructor is blended.rs:119-128:
  if r0 <= R::zero() || dr <= R::zero() || dtheta <= R::zero() { ... }
  if lambda < R::zero() || lambda > one { ... }
There is no reference to `det_at` in any validation path; `det_at` is first used at line 171 as a divisor.
```

**Reference form.** A coordinate map is admissible only where det J ≠ 0 and holds one sign over the patch (the inverse-function theorem / standard grid-validity condition, e.g. Thompson, Warsi & Mastin, Numerical Grid Generation, §2: a fold is det J passing through zero). The published claim being relied upon is the qtt_blend_metric study, whose own README states the opposite of 'by construction': 'A blend over a full domain with strong curvature needs the guard checked, not assumed.' (studies/qtt_blend_metric/README.md:44-45).

**Impact.** I reproduced blended.rs's `forward`/`det_at` exactly and swept it. With r0=1.0, dr=1.0, theta0=0.0, dtheta=3*pi/2, lambda=0.25 — all of which pass every constructor guard — det J takes BOTH signs over the 256x256 lattice (min|det| = 3.1e-05). That is a folded chart. With dtheta=2*pi, lambda in {0.25, 0.40} det J is likewise mixed-sign. The constructor returns Ok in every case, and the resulting `dxi_dx`/`deta_dy` trains encode a metric with a sign discontinuity. An engineer running a body-fitted marcher over such a map gets silently wrong physics with no error, no warning, and a source comment telling them the case cannot occur.

**Recommended fix.** In `BlendedMap::new`, evaluate `det_at` over the lattice before building the metric trains (the sampling loop already exists) and return `PhysicsError::PhysicalInvariantBroken` if the sign is not constant or if min|det| falls below a documented, justified floor relative to the geometric scale (e.g. min|det| < eps * dr * span_y). Then either delete the 'by construction' / 'constructor rejects a fold' sentences at lines 17-18 and 163-166 or restate them as 'enforced by the constructor', matching the code.

**Adversarial check.** Every quoted line is verbatim correct. blended.rs:16-18 states "det J_λ keeps one sign across the sweep ... and the constructor rejects a fold"; blended.rs:163-166 states validity "holds **by construction**". The complete guard set in BlendedMap::new is lines 119-128 (r0/dr/dtheta > 0, lambda in [0,1]). `det_at` is defined at 158-161 and first *used* at 171 as a divisor — it appears in no validation path, and there is no early return between 128 and the sampling at 168. The auditor's reference form (inverse-function theorem / Thompson-Warsi-Mastin fold condition: det J one-signed and non-vanishing) is correct as stated. I re-implemented `forward`/`det_at` exactly and swept a 256x256 lattice: r0=1.0, dr=1.0, theta0=0.0, dtheta=3*pi/2, lambda=0.25 gives 7631 positive and 57905 negative samples with min|det| = 3.106e-05 — a genuine sign change, constructor returns Ok. dtheta=2*pi at lambda=0.25 (25856/39680) and lambda=0.40 (17664/47872) are likewise mixed-sign. The study's own README (studies/qtt_blend_metric/README.md, Caveats) does say "Two charts over one patch is also the easy case for non-folding. A blend over a full domain with strong curvature needs the guard checked, not assumed" — the opposite of the source comment's "by construction". Doc-vs-code contradiction plus a reachable silent wrong-geometry path.

> Evidence re-read: deep_causality_cfd/src/coordinate/blended.rs:16-18 ("...so `det J_λ` keeps one sign across the sweep ... and the constructor rejects a fold."), :106-128 (entire guard set: only r0/dr/dtheta and lambda range), :158-161 (det_at defined), :163-166 ("Validity (gate BM-A) holds **by construction**"), :168-199 (det_at used only as divisor); studies/qtt_blend_metric/README.md Caveats paragraph; numeric sweep reproduced independently

---

### 7.2 [MAJOR] Blended inverse metric divides by det J with no guard; det J is identically ~0 for a full-annulus blend, producing an unbounded metric with an Ok return

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/coordinate/blended.rs:171`
- **Auditor confidence:** confirmed

**Claim.** All four inverse-metric components and the volume factor are computed as cofactor/det_at(xi,eta) with no check that det_at is nonzero or bounded away from zero, so degenerate geometries yield inf/NaN or 1e15-magnitude metric entries and the constructor still succeeds.

**Code evidence.**

```
blended.rs:168-174:
  let dxi_dx = quantize_2d(
      &sample_grid(lx, ly, |xi, eta| {
          let (_, _, _, d) = forward(xi, eta);
          d / det_at(xi, eta)
      })?,
 and identically at lines 178 (`neg * b / det_at`), 186 (`neg * c / det_at`), 193 (`a / det_at`).
The degenerating quantity is set at blended.rs:143:
  let span_y = two * (r0 + half * dr) * (half * dtheta).sin();
```

**Reference form.** The inverse Jacobian J⁻¹ = adj(J)/det J is defined only where det J ≠ 0; a numerical implementation must either guard det J or document the admissible parameter range that keeps it bounded away from zero. Here det J_λ = −Δr·{λ²r sin²θ Δθ + [(1−λ)+λcosθ]·[(1−λ)span_y + λ r cosθ Δθ]}, which is not sign-definite for general Δθ.

**Impact.** For dtheta = 2*pi — explicitly the recommended value in the sibling chart's documentation (coordinate/mod.rs:139, 'Use `dtheta = 2π` for a full annulus') and accepted by BlendedMap's only guard `dtheta > 0` — span_y = 2*(r0+dr/2)*sin(pi) evaluates to 3.67e-16, so at lambda = 0 det J is 3.67e-16 everywhere and the inverse-metric entries are ~1e15. At dtheta = pi, theta0 = 0, lambda = 0.5 the minimum |det J| over a 256x256 lattice is 1.16e-04, a ~1e4 amplification of every metric component near the singular line. None of these produce an error; they produce a `BlendedMap` that a marcher will happily consume. Downstream, `hadamard_rounded` against a 1e15 metric field destroys the tensor-train truncation budget as well.

**Recommended fix.** Compute min|det_at| during construction and reject (PhysicsError::NumericalInstability) when it is below a documented floor tied to the geometric scale. Additionally guard the degenerate `span_y ~ 0` case explicitly: the Cartesian-capture partner is undefined when sin(dtheta/2) = 0, so dtheta >= 2*pi should be rejected outright with a message naming the reason.

**Adversarial check.** blended.rs:168-199 computes all four inverse-metric components as cofactor/det_at(xi,eta) and the volume factor as det_at().abs(), with no non-zero / bounded-away check anywhere. span_y is set at blended.rs:143 exactly as quoted. I re-derived det J_λ from lines 147-161: with a = λ(−r sinθ Δθ), b = (1−λ)Δr + λ cosθ Δr, c = (1−λ)span_y + λ r cosθ Δθ, d = λ sinθ Δr, det = ad − bc = −Δr·{λ² r sin²θ Δθ + [(1−λ)+λcosθ]·[(1−λ)span_y + λ r cosθ Δθ]} — the auditor's reference form is correct. At dtheta = 2π, span_y = 2(r0+Δr/2)·sin(π) = 3.674e-16, and at lambda = 0 the scan gives min|det| = 3.674e-16 uniformly (metric entries ~1e15), constructor Ok. At dtheta = π, theta0 = 0, lambda = 0.5 the 256x256 min|det| is 1.156e-04, matching the auditor's 1.16e-04. coordinate/mod.rs:139 does say "Use `dtheta = 2π` for a full annulus" for the sibling chart, and BlendedMap's only angular guard is dtheta > 0. Root cause is the same missing det-J validity guard as Finding 1, so I downgrade to major to avoid counting one defect as two certification blockers; the technical content stands.

> Evidence re-read: deep_causality_cfd/src/coordinate/blended.rs:143 (span_y), :147-161 (forward/det_at), :168-199 (four `/ det_at(xi, eta)` divisions + `det_at().abs()` jacobian), :119-128 (guards); src/coordinate/mod.rs:139 ("Use `dtheta = 2π` for a full annulus"); numeric scans reproduced (2pi lam=0: min|det|=3.67e-16; pi lam=0.5: min|det|=1.16e-04)

---

### 7.3 [MAJOR] Curvilinear flux divergence is taken in non-conservative chain-rule form with no Jacobian weighting, while the marcher doc claims 'identical conservative physics'

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:244`
- **Auditor confidence:** confirmed

**Claim.** The body-fitted marchers use the quasi-linear chain-rule divergence with no Jacobian weighting, so they are not discretely conservative on a curvilinear grid and captured shock speeds are not guaranteed (Lax–Wendroff). This is a real physics limitation, but it is not a doc contradiction: marcher_2d.rs:14 states the chain-rule form explicitly and marcher_3d_fitted.rs:18 names the GCL as an unimplemented Stage-2 refinement. 'Identical conservative physics' at marcher_3d_fitted.rs:8 refers, in its own sentence, to the conservative state variables / flux / acoustic step shared with the Cartesian marcher. The defensible finding is a missing explicit caveat that the fitted path is not discretely conservative.

**Code evidence.**

```
marcher_2d.rs:242-248:
  let fq = self.encode(fk)?;
  let gq = self.encode(gk)?;
  let (dfx, _) = self.metric.physical_gradient(&fq)?;
  let (_, dgy) = self.metric.physical_gradient(&gq)?;
  let div = dfx.add(&dgy)?;
  let predictor = uk.add(&div.scale(neg * self.dt))?.round(&self.trunc)?;
marcher_3d_fitted.rs:210-212 is the identical pattern in 3-D.
marcher_3d_fitted.rs:8 — "//! Identical conservative physics to the Cartesian [`CompressibleMarcher3d`]"
A workspace-wide grep for `.jacobian()` returns only test files (tests/coordinate/body_fitted_tests.rs:125, metric_provider_tests.rs:63, body_fitted_3d_tests.rs:71, blended_tests.rs:140,146, cartesian_3d_tests.rs:97) — no solver reads it.
```

**Reference form.** Strong-conservation-law form in generalized curvilinear coordinates (Vinokur 1974; Pulliam & Steger, AIAA J. 18(2), 1980; Thompson, Warsi & Mastin, Numerical Grid Generation §5): ∂(U/J)/∂t + ∂/∂ξ[(ξ_x F + ξ_y G)/J] + ∂/∂η[(η_x F + η_y G)/J] = 0. The chain-rule (weak/quasi-linear) form used here is equivalent only in the continuous limit; discretely it does not telescope, so mass/momentum/energy are not conserved and, by the Lax–Wendroff theorem, a captured discontinuity converges to the wrong propagation speed.

**Impact.** For a body-fitted bow-shock application, the discrete scheme conserves nothing globally and captured shock speeds are not guaranteed correct. Over `CartesianIdentity` the chain-rule form degenerates to a flux difference and IS conservative, so the Cartesian control test will pass and hide the defect on the fitted path — exactly the configuration the rank-lever studies compare. The marcher_3d_fitted header partially discloses this at line 18 ('the geometric conservation law for exact free-stream preservation are the named Stage-2 refinements'), but line 8's 'identical conservative physics' overrides that impression for a reader.

**Recommended fix.** Restate marcher_3d_fitted.rs:8 and marcher_2d.rs:8-14 to say explicitly that the curvilinear form is the non-conservative chain-rule form, that discrete conservation holds only for the Cartesian identity chart, and that captured-shock speeds in a fitted chart are not guaranteed. Longer term, implement the strong-conservation form (which requires the currently-unused `jacobian()` and a discretely-satisfied GCL) and add a conservation-of-total-mass regression over the fitted chart.

**Adversarial check.** The physics half is CONFIRMED exactly: marcher_2d.rs:240-249 is verbatim as quoted (encode F and G, take physical_gradient of each, add, `predictor = uk + div·(−dt)`), marcher_3d_fitted.rs is the same pattern, and a workspace grep for `.jacobian()` returns only the five test sites listed — no solver reads the volume factor. So the scheme is the quasi-linear chain-rule form, not the strong-conservation form ∂(U/J)/∂t + ∂_ξ[(ξ_x F + ξ_y G)/J] + ∂_η[...] = 0 (Vinokur 1974; Pulliam & Steger 1980), and it does not telescope discretely. The doc-overclaim half does not survive reading the header in context. marcher_3d_fitted.rs:8-15 reads "Identical conservative physics to the Cartesian CompressibleMarcher3d — the five-train IMEX state U = (ρ,ρu,ρv,ρw,ρE), the ideal-gas flux, and the closed-form implicit acoustic step — but the explicit convective flux divergence ... is taken through a MetricProvider3d, i.e. by the chain-rule **physical** gradient ... rather than raw Cartesian operators." The sentence enumerates its own referents (conservative *variables*, flux, acoustic step) and explicitly names the chain-rule form; line 18 then names "the geometric conservation law for exact free-stream preservation" as a Stage-2 refinement. marcher_2d.rs:14 likewise discloses the chain-rule form up front. The gap is a missing discrete-conservation caveat, not a false claim.

> Evidence re-read: deep_causality_cfd/src/solvers/qtt/compressible/marcher_2d.rs:240-249 (step_component, verbatim) and :14 ("assembled from the metric's chain-rule `physical_gradient`"); marcher_3d_fitted.rs:6-20 (full header incl. line 8 and line 18); grep `jacobian()` across the crate → tests/coordinate/{metric_provider_tests.rs:63, body_fitted_tests.rs:125, body_fitted_3d_tests.rs:71, cartesian_3d_tests.rs:97, blended_tests.rs:140,146} only

---

### 7.4 [MINOR] The two free-stream-preservation tests cannot fail for any metric and their comments assert a discrete metric identity that is never checked

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/coordinate/body_fitted_tests.rs:73`
- **Auditor confidence:** confirmed

**Claim.** free_stream_preserved / free_stream_preserved_across_the_blend assert a property that is automatic in the chain-rule formulation and carries essentially no information about the metric values; their comment 'the metric identity holds discretely' mislabels what is tested, and no test in the repo checks the discrete GCL. They are not strictly unfailable (they would catch a NaN or >~1e8-magnitude metric), and no public doc cites them as GCL evidence, so this is a test-labelling/coverage gap, not a certification blocker.

**Code evidence.**

```
tests/coordinate/body_fitted_tests.rs:73-84:
  // A uniform field has exactly zero physical gradient (the metric identity holds discretely).
  let u = coord.sample(|_xi, _eta| 3.7).unwrap();
  let (dudx, dudy) = coord.physical_gradient(&u).unwrap();
  ... assert!(v.abs() < 1e-9, "free-stream gradient nonzero: {v}");
tests/coordinate/blended_tests.rs:84-96 is the same test with the same comment.
The mechanism is coordinate/mod.rs:257-268:
  let du_dxi = self.g_xi.apply(u, &self.trunc)?;   // == 0 exactly for constant u
  let du_deta = self.g_eta.apply(u, &self.trunc)?; // == 0 exactly
  let du_dx = self.dxi_dx.hadamard_rounded(&du_dxi, ...)?.add(&self.deta_dx.hadamard_rounded(&du_deta, ...)?)?
```

**Reference form.** The geometric conservation law / discrete metric identity is a statement about derivatives OF the metric: ∂_ξ(J ξ_x) + ∂_η(J η_x) = 0 and ∂_ξ(J ξ_y) + ∂_η(J η_y) = 0 (Thomas & Lombard, AIAA J. 17(10), 1979). Free-stream preservation is a nontrivial consequence of the GCL only in the strong-conservation (divergence) form; in chain-rule form it is automatic and carries no information.

**Impact.** The repo presents free-stream preservation as evidence the metric is discretely consistent (the same phrasing appears in the marcher docs). It is not evidence of anything: I traced that (S₋−S₊)·const = 0 exactly in the MPO, so the product with any metric train is exactly zero regardless of the metric's values. A reviewer or certifier reading these tests would incorrectly conclude the GCL was verified. No test in the repo checks the actual metric identity.

**Recommended fix.** Change the comments to state that free-stream preservation is trivial in chain-rule form and carries no GCL information. Add a real GCL test that forms ∂_ξ(J ξ_x) + ∂_η(J η_x) with the actual MPOs and asserts it is small, and a test that perturbs one metric train and confirms a physical-gradient test detects it (a mutation check on the existing suite).

**Adversarial check.** All citations are verbatim: body_fitted_tests.rs:73-84 and blended_tests.rs:83-96 carry the comment "the metric identity holds discretely" and assert |gradient| < 1e-9 on a constant field; coordinate/mod.rs:257-268 is the quoted mechanism. I confirmed the operator: operators.rs:95-109 builds gradient = (S₋ − S₊)/(2Δ) from the cyclic shifts, whose row sums are 1 − 1 = 0, so it annihilates constants to round-off (~1e-17), and the subsequent hadamard with any finite metric train stays far under 1e-9. The assertion therefore carries no information about the metric values, and the auditor's reference form is right: the discrete GCL is ∂_ξ(J ξ_x) + ∂_η(J η_x) = 0 (Thomas & Lombard 1979), a statement about derivatives of the metric, and free-stream preservation is automatic in chain-rule form. Grep confirms no test checks that identity. Two corrections to the claim: the tests are not literally unfailable — the products are ~1e-17·|metric|, so a metric of magnitude >~1e8 (exactly what the unguarded det J of Finding 2 can produce) would break the 1e-9 bound, and a NaN metric fails outright; so they retain value as a finiteness/wiring smoke test. Severity is over-stated at major: no shipped number is wrong and no user-facing doc claims GCL verification.

> Evidence re-read: deep_causality_cfd/tests/coordinate/body_fitted_tests.rs:72-84 (verbatim); tests/coordinate/blended_tests.rs:83-96 (verbatim, same comment); src/coordinate/mod.rs:253-270 (physical_gradient); src/tensor_bridge/operators.rs:88-109 (gradient = (S₋−S₊)/(2Δx), row sums zero); grep across tests/ for any ∂_ξ(J ξ_x) metric-identity check → none

---

### 7.5 [MAJOR] MetricProvider::jacobian() returns two different physical quantities across impls — cell volume vs. patch determinant, differing by a factor Nx*Ny

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/coordinate/cartesian.rs:53`
- **Auditor confidence:** confirmed

**Claim.** The trait documents jacobian() as 'The Jacobian determinant |J| (the conservative volume factor)'. CartesianIdentity returns dx*dy (the per-cell volume). BodyFittedCoordinate and BlendedMap return |det ∂(x,y)/∂(ξ,η)| over the unit (ξ,η) patch. For the identity chart these differ by Nx*Ny.

**Code evidence.**

```
cartesian.rs:53-54:
  let cell = dx * dy;
  let jacobian = quantize_2d(&sample_grid(lx, ly, |_xi, _eta| cell)?, &trunc)?;
with the chart declared at cartesian.rs:8 as "x = ξ·Δx·Nx, y = η·Δy·Ny", so det ∂(x,y)/∂(ξ,η) = (Nx·dx)(Ny·dy), not dx·dy.
coordinate/mod.rs:196-199:
  let jacobian = quantize_2d(&sample_grid(lx, ly, |_xi, eta| radius_at(eta) * dtheta * dr)?, ...)
blended.rs:196-199: `det_at(xi, eta).abs()`.
traits/metric_provider.rs:52: "/// The Jacobian determinant `|J|` (the conservative volume factor) as a low-rank tensor train."
The divergence is locked in by tests/coordinate/metric_provider_tests.rs:58-71 (`cartesian_identity_jacobian_is_cell_volume`, asserting jacobian == dx*dy).
```

**Reference form.** For a chart T: (ξ,η) → (x,y) on the unit computational square, the coordinate Jacobian is det J = ∂(x,y)/∂(ξ,η) and the cell volume is det J · Δξ · Δη = det J/(Nx·Ny). These are different quantities; a trait method must return one of them consistently across impls.

**Impact.** A generic consumer written against `M: MetricProvider<R>` that uses jacobian() as a conservative volume weight — which is exactly what the trait doc invites, and exactly what the strong-conservation form would require — gets results that differ by Nx*Ny depending on which chart is plugged in. It also makes the two charts non-comparable: BlendedMap at λ=0 returns Δr·span_y (≈2.12 in the study geometry) where CartesianIdentity would return a cell volume ~1e-5. Impact is latent today only because no solver reads jacobian() (grep confirms tests only), so the inconsistency will surface the moment the GCL/conservative form is implemented.

**Recommended fix.** Pick one definition, state it unambiguously in the trait doc (I recommend det J on the unit computational patch, matching the curvilinear literature and the fitted impls), fix CartesianIdentity to return (Nx*dx)*(Ny*dy), and update `cartesian_identity_jacobian_is_cell_volume` accordingly. If the cell volume is wanted instead, divide the fitted charts by Nx*Ny and rename the method to `cell_volume`.

**Adversarial check.** Every citation checks out verbatim. cartesian.rs:8 declares the chart "x = ξ·Δx·Nx, y = η·Δy·Ny", for which det ∂(x,y)/∂(ξ,η) = (Nx·dx)(Ny·dy); cartesian.rs:53-54 stores the constant `cell = dx*dy` instead — smaller by Nx·Ny. coordinate/mod.rs:196-199 stores r·Δθ·Δr, which is exactly det ∂(x,y)/∂(ξ,η) for x = r cosθ, θ = θ0+ξΔθ, r = r0+ηΔr (I re-derived it: |det| = r·Δθ·Δr). blended.rs:196-199 stores det_at().abs(), also the patch determinant. traits/metric_provider.rs:52 documents one quantity for all impls, "The Jacobian determinant |J| (the conservative volume factor)". metric_provider_tests.rs:57-71 locks the Cartesian behaviour in (`cartesian_identity_jacobian_is_cell_volume`). The auditor's reference form is correct: cell volume = det J·Δξ·Δη = det J/(Nx·Ny). Nothing elsewhere reconciles the two — no consumer normalizes, because grep shows no solver calls jacobian() at all. A trait method whose return means different things per impl is a genuine contract defect and will produce an Nx·Ny error the moment the strong-conservation form is added.

> Evidence re-read: deep_causality_cfd/src/coordinate/cartesian.rs:6-11 (chart declaration) and :51-54 (`let cell = dx * dy`); src/coordinate/mod.rs:170-172 and :196-199 (|J| = r·Δθ·Δr); src/coordinate/blended.rs:196-199 (det_at().abs()); src/traits/metric_provider.rs:52; tests/coordinate/metric_provider_tests.rs:57-71

---

### 7.6 [MINOR] Contradictory Poisson eigenvalue in adjacent comments: line 157 states the compact 5-point eigenvalue, the code implements the grad-of-grad eigenvalue

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/tensor_bridge/projection.rs:157`
- **Auditor confidence:** confirmed

**Claim.** projection.rs states the pressure-Poisson symbol three inconsistent ways in eleven lines: the correct grad-of-grad eigenvalue at 153-155, the compact 5-point eigenvalue at 157, and a sign-flipped reciprocal at 169. The code (159-160, 170) is correct and the exact-projection property holds. The stale line-157 and line-169 comments should be deleted or corrected; the line 153-155 comment already states the right form, which limits the risk of a maintainer 'fixing' the code to match.

**Code evidence.**

```
projection.rs:153-155 (correct):
  // The projection applies grad-of-grad (centered difference squared), eigenvalue -sin^2(2pik/N)/dx^2
  // (the *consistent* operator, not the compact 5-point Laplacian) so div(project(u)) = 0 exactly.
projection.rs:157 (contradicts the above and the code):
  // λ_k = −(2 − 2cos(2πk/N))/Δ²; the periodic Laplacian eigenvalue (separable in 2-D).
projection.rs:159-160 (the code):
  let sx = (tau * from_usize::<R>(kx) / nxf).sin();
  let lamx = sx * sx / dx2;
projection.rs:169 (sign-inconsistent with line 170):
  // ∇²p = rhs with λ = −(lamx+lamy): p̂ = rhŝ / (−λ).
  let inv = R::zero() - R::one() / (lamx + lamy);
```

**Reference form.** For D = (S₋−S₊)/(2h), the Fourier symbol is i sin(2πk/N)/h, so D² has eigenvalue −sin²(2πk/N)/h². The compact three-point Laplacian (S₊+S₋−2I)/h² has the different eigenvalue −(2−2cos(2πk/N))/h² = −4sin²(πk/N)/h². Only the former makes div∘project vanish identically, since div and grad are both built from D. Given λ = −(lamx+lamy), the correct pressure is p̂ = rhŝ/λ = −rhŝ/(lamx+lamy), which is what line 170 computes; the line-169 comment says p̂ = rhŝ/(−λ), the opposite sign.

**Impact.** The code is correct — I verified both the eigenvalue and the sign, and confirmed the exact-projection property holds including on the four null modes. But the defining formula of the pressure-Poisson operator is stated three different ways in eleven lines, one of which is a different operator and one of which has the wrong sign. A maintainer reading line 157 and 'fixing' the code to match would silently break the exactness of the Leray projection (div would no longer vanish, by a factor sin²θ vs 4sin²(θ/2) per axis). This is a certification-relevant traceability defect even though the current binary is right.

**Recommended fix.** Delete the line-157 comment (it describes the compact Laplacian, which is not used here) and correct line 169 to `p̂ = rhŝ / λ`. Keep the lines 153-155 explanation, which is the accurate one, and cross-reference it from the `solve_poisson` public docstring so the operator identity is stated once.

**Adversarial check.** All three quoted comments and the code are verbatim at the cited lines (153-155, 157, 159-160, 169-170). I re-derived the operator independently: project() removes gx.apply(p) and gy.apply(p) from fields whose divergence is taken with the same gx/gy, so the operator inverted is D_x² + D_y² with D = (S₋−S₊)/(2h) (operators.rs:88-109), symbol i·sin(2πk/N)/h, hence eigenvalue −sin²(2πk/N)/h² per axis. The code's `lamx = sin²(2πkx/Nx)/dx²` and `inv = −1/(lamx+lamy)` are exactly right: p̂ = rhŝ/λ with λ = −(lamx+lamy) gives p̂ = −rhŝ/(lamx+lamy). So line 157's "(2 − 2cos(2πk/N))/Δ²" is a different (compact 5-point) operator and is stale/contradictory, and line 169's "p̂ = rhŝ / (−λ)" has the sign backwards relative to the code on the next line. The auditor's reference forms are all correct and the code is correct. Severity is over-stated: line 153-155 immediately above explicitly and correctly disowns the compact 5-point form ("the *consistent* operator, not the compact 5-point Laplacian"), so a maintainer has the correction in the same comment block; this is a stale-comment traceability defect with zero effect on the binary.

> Evidence re-read: deep_causality_cfd/src/tensor_bridge/projection.rs:109-124 (project), :153-155, :157, :158-172 (loop, lamx/lamy, is_null, inv); src/tensor_bridge/operators.rs:88-109 (gradient = (S₋−S₊)/(2Δx))

---

### 7.7 [MAJOR] Periodic MPO stencils wrap at non-periodic boundaries, giving an O(1) wrong gradient at the wall and outer boundary of every fitted chart

- **Verification verdict:** CONFIRMED
- **Axis:** physics-math
- **Location:** `deep_causality_cfd/src/coordinate/mod.rs:167`
- **Auditor confidence:** confirmed

**Claim.** The chain-rule gradients use the cyclic gradient MPO on every axis, including the radial (η) axis of the 2-D polar chart and the radial (ζ) / polar (η) axes of the 3-D shell, which are not periodic. The gradient at the first and last index of those axes is computed from data on the opposite side of the domain and is O(1) wrong. The marchers apply this every step with no guard.

**Code evidence.**

```
coordinate/mod.rs:167-168:
  let g_xi = gradient_x::<R>(lx, ly, dxi, &trunc)?;
  let g_eta = gradient_y::<R>(lx, ly, deta, &trunc)?;
both built from operators.rs:95-109, whose shifts are cyclic by construction (operators.rs:44-45, "The MSB mode drops the overflow carry (cyclic, mod `2^L`)").
body_fitted_3d.rs:109-111 does the same for all three axes.
The tests acknowledge and route around it rather than bound it:
  tests/coordinate/body_fitted_tests.rs:37-39 — "// Interior radial rows (the periodic operator wraps at the η boundary — a Stage-2 refinement)." then `for j in 1..ny - 1`
  tests/coordinate/blended_tests.rs:63-64, 72-73 — same skip.
```

**Reference form.** A non-periodic boundary requires a one-sided (or ghost-cell) stencil: e.g. the 2nd-order one-sided form (−3f₀ + 4f₁ − f₂)/(2h) at the low edge. Using a wrapped centered difference there computes (f₁ − f_{N−1})/(2h), which references a physically distant point and is not consistent with any derivative of the field.

**Impact.** In the intended application, the radial axis boundaries are exactly the body wall (η = 0, r = r0) and the outer/shock boundary (η → 1). Those are the two locations an avionics consumer cares most about — surface heating, stagnation pressure, shock standoff — and the physical gradient there is not merely low-order, it is wrong. The 2-D and 3-D marchers (marcher_2d.rs:244-245, marcher_3d_fitted.rs:210-212) consume `physical_gradient` unmodified over the full lattice every step, so the error is injected each step and advected inward. The limitation IS disclosed in the coordinate module docs (mod.rs:23-25, body_fitted_3d.rs:25-26), but neither marcher's doc repeats it, no runtime guard exists, and no test quantifies the boundary error magnitude.

**Recommended fix.** Either (a) implement non-periodic one-sided edge stencils for the radial/polar axes as an alternate operator build, or (b) until then, repeat the caveat prominently in the marcher docs, add a test that measures and records the boundary-row error magnitude (so it is a known number rather than an unmeasured one), and expose a validity mask so consumers can exclude the polluted rows from reported observables.

**Adversarial check.** Verified end to end. coordinate/mod.rs:167-168 builds g_xi/g_eta from gradient_x/gradient_y; those come from operators.rs:95-109, which is (shift_minus − shift_plus)/(2Δ), and shift_plus is documented and implemented as the cyclic ripple-carry increment that "drops the overflow carry (cyclic, mod 2^L)" (operators.rs:39-78). So the η (radial) axis of the polar chart is differenced cyclically even though it is bounded by the wall at r = r0 and the outer boundary at r = r0+Δr. body_fitted_3d.rs:109-111 does the same for all three axes, and its header (:24-26) confirms ζ radial / η polar are non-periodic (θ restricted to (0,π) to avoid the poles). blended.rs:139-140 is the same construction. The auditor's reference form is right — a bounded edge needs a one-sided stencil such as (−3f₀+4f₁−f₂)/(2h); the wrapped centered difference computes (f₁ − f_{N−1})/(2h) across the whole radial extent, an O(1) error, not a low-order one. The tests do route around it rather than bound it: body_fitted_tests.rs:37-39 and :62 and blended_tests.rs:63-64,72-73 all skip j = 0 and j = ny−1 with exactly the quoted comment. The marchers consume physical_gradient over the full lattice each step (marcher_2d.rs:244-245) with no guard. The limitation is disclosed in the coordinate module headers (mod.rs:23-25, body_fitted_3d.rs:24-26) and in BodyFittedCoordinate::physical_gradient's own docstring (mod.rs:248-249) — but not in BlendedMap::physical_gradient's docstring, not in either marcher's docs, and no test bounds the boundary error.

> Evidence re-read: deep_causality_cfd/src/coordinate/mod.rs:167-168, :23-25, :247-249; src/tensor_bridge/operators.rs:39-78 (cyclic shift_plus), :88-109 (gradient); src/coordinate/body_fitted_3d.rs:24-26, :109-111; src/coordinate/blended.rs:139-140, :256-259; tests/coordinate/body_fitted_tests.rs:37-39,62; tests/coordinate/blended_tests.rs:63-64,72-73; src/solvers/qtt/compressible/marcher_2d.rs:244-245

---

### 7.8 [MINOR] MetricProvider module doc states the lambda blend is not implemented and that the impls cannot support it; BlendedMap is implemented, is a MetricProvider, and is publicly exported

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/traits/metric_provider.rs:15`
- **Auditor confidence:** confirmed

**Claim.** traits/metric_provider.rs:15-17 is stale: it calls the λ blend a follow-on while BlendedMap ships, implements MetricProvider, and is exported from the crate root. The technical statement in that paragraph is still true, however — MetricProvider exposes no forward Jacobian, so a *provider-composing* blend remains unbuilt; BlendedMap hardcodes two analytic charts instead. The fix is to reword the paragraph to point at BlendedMap and scope the remaining gap to generic composition.

**Code evidence.**

```
traits/metric_provider.rs:15-17:
  //! The continuous body-fit blend parameter `λ` (a `BlendedMap` over two providers) is a follow-on: a
  //! correct blended metric needs the *forward* Jacobians of both charts, which the present impls do not
  //! expose. The blend itself is already validated numerically (`studies/qtt_blend_metric`).
Contradicted by blended.rs:280-305 (`impl<R> MetricProvider<R> for BlendedMap<R>`), by blended.rs:147-157 which computes the forward Jacobians analytically in-line (so the stated blocker does not apply), and by src/lib.rs:52-54 which exports `BlendedMap, BlendedMapConfig` from the crate root.
```

**Reference form.** Docs-vs-code parity: module documentation on a public trait must not deny the existence of a shipped public implementation of that trait.

**Impact.** An integrator reading the trait documentation — the natural entry point for 'what coordinates can I plug in?' — will conclude the lambda dial does not exist and is architecturally blocked, and will not find or use `BlendedMap`. Conversely, a reviewer will not know that a second, less-tested MetricProvider impl (the one carrying the unguarded det J division above) is on the public surface.

**Recommended fix.** Replace metric_provider.rs:15-17 with a pointer to `BlendedMap` describing that it computes both charts' forward Jacobians analytically and exposes the blended inverse metric through this trait, and state the currently-unenforced validity precondition (one-signed det J) that a caller must respect.

**Adversarial check.** The quotes are exact. traits/metric_provider.rs:15-17 says the λ blend "is a follow-on", blended.rs:280-305 implements MetricProvider<R> for BlendedMap<R>, and src/lib.rs:52-55 re-exports BlendedMap and BlendedMapConfig from the crate root. So the doc is stale and will mislead an integrator looking for the λ dial. But the stated *blocker* is factually accurate rather than contradicted: the doc describes "a BlendedMap over two providers", and the MetricProvider trait surface (dims / sample / physical_gradient / jacobian, lines 32-53) exposes no forward Jacobian, so a blend composed generically over two arbitrary providers is still impossible. BlendedMap does not compose providers — it hardcodes the two analytic charts and computes both forward Jacobians in-line (blended.rs:147-157), i.e. it sidesteps the blocker rather than disproving it. The finding's framing that "the stated blocker does not apply" is therefore wrong on the architecture; the real defect is a stale sentence that denies a shipped, exported impl.

> Evidence re-read: deep_causality_cfd/src/traits/metric_provider.rs:15-17 and :27-53 (full trait surface — no forward-Jacobian accessor); src/coordinate/blended.rs:147-157 (in-line analytic forward Jacobians of the two hardcoded charts) and :280-305 (impl MetricProvider); src/lib.rs:52-55 (pub use BlendedMap, BlendedMapConfig)

---

### 7.9 [INFO] acoustic_inverse module header describes a dropped-tail approximation that the implementation explicitly does not make, and points the reader at a gate that is not in the study it names

- **Verification verdict:** PARTIALLY CONFIRMED
- **Axis:** doc-overclaim
- **Location:** `deep_causality_cfd/src/tensor_bridge/acoustic_inverse.rs:28`
- **Auditor confidence:** confirmed

**Claim.** The acoustic_inverse module header (lines 28-29) justifies the finite doubling product by the smallness of the ρ^{2^l} tail, whereas the implementation (129-135) folds the exact 1/(1−ρ^N)² factor and is exact at all N. That is an imprecise header, mitigated by the fact that the same sentence states the exact identity. The claim that line 37 misattributes verification to the qtt_acoustic_precond study is refuted: line 37 names no study, and 'gate 1' resolves to tests/tensor_bridge/acoustic_inverse_tests.rs:68, which is exactly the A₀·A₀⁻¹ = I round-off gate described.

**Code evidence.**

```
acoustic_inverse.rs:28-29:
  //! so each resolvent costs `l` shift-applies. The cyclic tail dropped by the finite sum is `ρ^{2^l}`
  //! (`< 10⁻³⁰` for the working `l`), i.e. exact to roundoff
acoustic_inverse.rs:129-132 (the implementation, contradicting it):
  // Folding `1/(1−ρ^N)²` into the prefactor makes `A₀⁻¹` exact (and free-stream-exact) at **all** N,
  // not just in the large-N limit where `ρ^N → 0`.
acoustic_inverse.rs:37:
  //! construction is verified end-to-end by the `A₀·A₀⁻¹ = I` round-off gate (Resolution 6, gate 1).
studies/qtt_acoustic_precond/main.rs:23 imports only `{laplacian, quantize}`; its AC-A gate uses `solve::linear` (main.rs:169) — the AMEn solve the module doc says it replaces — and AC-B/AC-C use a dense Gauss–Jordan inverse (main.rs:198). The real A0*A0^-1 gate lives in tests/tensor_bridge/acoustic_inverse_tests.rs:66-80.
```

**Reference form.** Sum_{k<N} rho^k S^k = (1 - rho^N)(I - rho S)^-1 exactly, since S^N = I on a cyclic grid. The implementation uses this identity exactly (no dropped tail); the header describes a truncation argument.

**Impact.** The implementation is stronger than its own documentation claims, so no wrong number results. But an auditor checking the header's traceability chain follows it to the wrong artifact: the named study does not exercise AcousticCoreInverse at all, so a reader verifying 'gate 1' will find an AMEn-based measurement and either conclude the closed-form path is unverified or mistakenly credit the AMEn result to it. The actual gate (tests/tensor_bridge/acoustic_inverse_tests.rs:66-80) is sound and rebuilds A0 independently from shift MPOs; I confirmed its residual check is a genuine, non-circular verification.

**Recommended fix.** Rewrite acoustic_inverse.rs:28-29 to state the finite sum is exact via the (1-rho^N) identity, not approximate. Change the line-37 pointer from 'Resolution 6, gate 1' to the actual test path tests/tensor_bridge/acoustic_inverse_tests.rs. Separately, correct studies/qtt_acoustic_precond/main.rs:95 which attributes its conclusion to 'the closed-form-core preconditioner' though the study measures an AMEn solve and a dense inverse.

**Adversarial check.** First half stands, with a qualification. acoustic_inverse.rs:28-29 does say "The cyclic tail dropped by the finite sum is ρ^{2^l} (< 10⁻³⁰ for the working l), i.e. exact to roundoff", while the implementation (129-135) folds 1/(1−ρ^N)² into pre_scale and its comment says this makes the inverse exact "at **all** N, not just in the large-N limit where ρ^N → 0". So the header's smallness argument understates what the code does — nothing is actually dropped. The qualification: the same header sentence goes on to state the exact identity "(I − ρ·S₊)·Σ_{k<N} ρ^k S₊^k = (1 − ρ^N)·I", which is the correct reference form (S^N = I cyclically) and is precisely what the code compensates for, so the header is imprecise rather than contradictory. Second half is REFUTED as written: line 37 reads "verified end-to-end by the `A₀·A₀⁻¹ = I` round-off gate (Resolution 6, gate 1)" and names no study. Grep shows the only 'Gate 1' in the repo is tests/tensor_bridge/acoustic_inverse_tests.rs:68 ("Gate 1: A₀ A₀⁻¹ = I to round-off, at bounded bond, flat across resolution (L=8 vs L=10)") — the pointer resolves correctly and to a genuine test. The auditor supplied the study attribution themselves. I confirm the auditor's read of that test: inverse_residual rebuilds A₀ from shift MPOs and checks the residual, non-circular.

> Evidence re-read: deep_causality_cfd/src/tensor_bridge/acoustic_inverse.rs:28-29, :34-37, :121-135 (rho, rho_n, gain, pre_scale); tests/tensor_bridge/acoustic_inverse_tests.rs:66-80; grep for 'Resolution 6|gate 1|Gate 1|qtt_acoustic_precond' across src/ tests/ studies/ — acoustic_inverse.rs never names the study

---

### 7.10 [MINOR] Silent geometry-altering fallback: R::from_f64(0.5).unwrap_or_else(R::one) substitutes 1.0 for 0.5 in the blend's chart geometry

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/src/coordinate/blended.rs:118`
- **Auditor confidence:** confirmed

**Claim.** If the scalar type cannot represent 0.5, `half` silently becomes 1.0, changing span_y from 2*(r0+dr/2)*sin(dtheta/2) to 2*(r0+dr)*sin(dtheta) and shifting the Cartesian rectangle's y-origin — a wrong chart with no error.

**Code evidence.**

```
blended.rs:118:  let half = R::from_f64(0.5).unwrap_or_else(R::one);
blended.rs:143:  let span_y = two * (r0 + half * dr) * (half * dtheta).sin();
blended.rs:229 (same fallback in `position`):  let half = R::from_f64(0.5).unwrap_or_else(R::one);
blended.rs:234:  let yc = (R::zero() - half) * self.span_y + xi * self.span_y;
The same pattern appears in 13 other physics-path sites (marcher_2d.rs:88,122,277; marcher_3d.rs:86,123; marcher_3d_fitted.rs:76,135; euler_1d.rs:37,163; types/flow/coupling.rs:559; types/flow/branch.rs:155; and blackout.rs:356 substitutes 1.0 for 1.0e30).
```

**Reference form.** A failed numeric conversion in a physics path is an error condition, not a case with a sensible default. The correct handling is to propagate PhysicsError (the surrounding functions already return Result<_, PhysicsError>) or, at minimum, to use a constructed value such as R::one()/(R::one()+R::one()) which cannot fail.

**Impact.** For f32/f64 the conversion never fails, so today's numbers are correct. The defect is that the failure mode is silently-wrong geometry rather than an error, in a crate that is generic over the scalar type by design (CfdScalar) and is being audited for a certification consumer. blackout.rs:356 substituting 1.0 for 1.0e30 is the same pattern with a 30-order-of-magnitude swing. Both constructors here already return Result and could propagate.

**Recommended fix.** Replace `R::from_f64(0.5).unwrap_or_else(R::one)` with `R::one() / (R::one() + R::one())` where a Result is unavailable, or with `.ok_or_else(|| PhysicsError::NumericalInstability(...))?` in the constructors (blended.rs:118 is inside a Result-returning fn and can propagate directly). Apply the same treatment to the other 13 sites, especially blackout.rs:356.

**Adversarial check.** blended.rs:118 and :229 are verbatim; span_y at :143 and yc at :234 consume `half`, so a failed conversion would silently give span_y = 2(r0+Δr)·sin(Δθ) and shift the rectangle's y-origin — a different chart, no error, in functions that already return Result<_, PhysicsError>. A crate-wide grep confirms the pattern at 17 sites including all those the auditor lists and blackout.rs:356 (`R::from_f64(1.0e30).unwrap_or_else(R::one)`), which is the same shape with a 30-decade swing. One citation is off by a directory: the branch.rs site is src/types/flow/corridor/branch.rs:155, not src/types/flow/branch.rs:155. The auditor's own scoping is accurate: CfdScalar is RealField + FromPrimitive (src/traits/cfd_scalar.rs:16-22), so for f32/f64 the conversion never fails and no number is wrong today; the defect is that the failure mode is silently-wrong physics rather than an error, and the correct handling (propagate, or construct R::one()/(R::one()+R::one()), which cannot fail) is available. Note the crate already uses the correct pattern elsewhere — coupled_march.rs:44 uses .expect("scalar represents 1e30") rather than a substituted value.

> Evidence re-read: deep_causality_cfd/src/coordinate/blended.rs:118, :143, :229, :234; src/types/flow/blackout.rs:356; src/traits/cfd_scalar.rs:16-22 (CfdScalar: RealField + FromPrimitive); grep 'unwrap_or_else(R::one)' across src/ → 17 sites (branch.rs is under types/flow/corridor/)

---

### 7.11 [MINOR] Leray projector's public docs do not disclose that Nyquist/checkerboard velocity components are invisible to the projection

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/tensor_bridge/projection.rs:104`
- **Auditor confidence:** confirmed

**Claim.** `project` is documented as returning a discretely divergence-free field. The centered-difference divergence has a four-dimensional kernel (kx in {0, Nx/2} times ky in {0, Ny/2}); a checkerboard velocity component in that kernel is invisible to `divergence` and passes through `project` completely unchanged. This is recorded only in an internal comment, not in any public docstring or the module header.

**Code evidence.**

```
projection.rs:104-105 (public doc):
  /// Leray projection: returns `(u, v)` with `∇p` removed, so the result is discretely
  /// divergence-free. `u ← u* − ∂ₓp`, `v ← v* − ∂ᵧp`.
projection.rs:154-155 (internal only):
  // is singular at k in {0, N/2} per axis (constant + collocated checkerboard/Nyquist), all zeroed.
projection.rs:165:  let is_null = (kx == 0 || kx == half_x) && (ky == 0 || ky == half_y);
The module header (projection.rs:6-13) mentions only the k=0 constant mode: "with the constant (`k=0`) mode zeroed". No test covers a Nyquist-content input; tests/tensor_bridge/projection_tests.rs:50 deliberately restricts to "frequencies well below Nyquist".
```

**Reference form.** Odd-even (checkerboard) decoupling of centered differences on a collocated grid: D = (S₋−S₊)/(2h) has symbol i sin(2πk/N)/h, which vanishes at k = N/2 as well as k = 0. Standard treatments (Ferziger & Peric, Computational Methods for Fluid Dynamics, §7.3) require a staggered grid or Rhie–Chow interpolation to suppress the resulting spurious mode.

**Impact.** The statement 'discretely divergence-free' is true with respect to this specific discrete divergence but does not mean physically divergence-free: a spurious checkerboard pressure/velocity mode is neither detected nor removed and can grow unbounded across a time integration. A user reading only the public docstring would not know to filter it. The projector's own construction is correct — I verified that div̂ vanishes on the null set so zeroing p̂ there loses nothing — the gap is purely in disclosure and test coverage.

**Recommended fix.** Add to the `project` and `solve_poisson` docstrings that the divergence operator's kernel includes the per-axis Nyquist mode, so a collocated checkerboard component is invariant under the projection, and name the mitigation (spectral filtering or a staggered/Rhie–Chow variant). Add a test that injects a checkerboard field and records the (currently unchanged) result, so the behavior is pinned rather than unknown.

**Adversarial check.** Both texts read as quoted. The public docstring (projection.rs:104-105) promises the result "is discretely divergence-free" with no qualification; the module header (:6-13) mentions only "the constant (`k=0`) mode zeroed"; the Nyquist half of the kernel is recorded only in the internal comment at :154-155 and enacted at :165 (`is_null = (kx == 0 || kx == half_x) && (ky == 0 || ky == half_y)`). I re-derived the kernel: D = (S₋−S₊)/(2h) has symbol i·sin(2πk/N)/h, which vanishes at k = 0 and k = N/2, so the null set is exactly the four modes the code zeroes, and a checkerboard velocity component in that set passes through project() untouched — the auditor's reference (odd-even decoupling on a collocated grid; Ferziger & Perić §7.3, staggering or Rhie–Chow required) is correct. Test coverage confirms the gap: tests/tensor_bridge/projection_tests.rs restricts its input to "frequencies well below Nyquist" and no test injects Nyquist content. Disclosure-only defect, correctly scoped as minor by the auditor.

> Evidence re-read: deep_causality_cfd/src/tensor_bridge/projection.rs:6-13 (module header), :104-105 (public doc), :153-155, :165; src/tensor_bridge/operators.rs:88-109 (D symbol); tests/tensor_bridge/projection_tests.rs (projection_removes_divergence, "frequencies well below Nyquist")

---

### 7.12 [MINOR] qtt_blend_metric's BM-A gate never tests the fold marker its own scan produces

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/studies/qtt_blend_metric/main.rs:73`
- **Auditor confidence:** confirmed

**Claim.** `jacobian_scan` returns sign = 2 as an explicit 'mixed-sign marker (a fold)', but the gate only compares the marker across lambda values and never tests for the value 2. A fold present at every lambda is reported as sign-consistent and passes BM-A.

**Code evidence.**

```
main.rs:150-155 (the marker):
  let s = if det >= 0.0 { 1 } else { -1 };
  if sign == 0 { sign = s; } else if s != sign { sign = 2; // mixed-sign marker (a fold) }
main.rs:58-62 (the only consumer):
  if first_sign == 0 { first_sign = sign; } else if sign != first_sign { sign_consistent = false; }
main.rs:73 (the gate):
  if !sign_consistent || min_det_overall <= 1e-6 {
main.rs:65 also prints a fold as '+':
  let signs = if sign >= 0 { "+" } else { "-" };
```

**Reference form.** A validity gate for 'det J holds one sign' must assert that no intra-patch sign change occurred, i.e. `sign != 2` for every lambda. Comparing the marker across lambdas tests a different, weaker property.

**Impact.** If the geometry were changed such that every lambda in the sweep folds, jacobian_scan returns 2 for all five and sign_consistent stays true; the fold-marker prints as '+', and the study exits 0 announcing 'the blend is valid'. The min|detJ| <= 1e-6 backstop is also not reliable: on a 256x256 sample a continuous sign change can be straddled without any sampled point landing near zero (in my dtheta=3*pi/2, lambda=0.25 reproduction min|det| was 3.1e-05, thirty times above the threshold, while the sign was genuinely mixed). The current published run is not affected — all five lambdas are genuinely single-signed and I reproduced the numbers exactly — but the gate is weaker than it reads.

**Recommended fix.** Return an explicit `folded: bool` from `jacobian_scan` rather than overloading the sign integer, and make BM-A fail on `folded` for any lambda. Raise the min|detJ| floor to a scale-relative quantity (e.g. 1e-3 * dr * span_y) rather than the absolute 1e-6, and print 'FOLD' rather than '+' for the mixed case.

**Adversarial check.** All four cited lines are verbatim. jacobian_scan (main.rs:130-159) sets sign = 2 at :154 with the comment "mixed-sign marker (a fold)"; the only consumer (:56-62) compares sign against first_sign; the gate (:73) tests `!sign_consistent || min_det_overall <= 1e-6` and never tests sign == 2; the print at :65 maps 2 to "+" via `if sign >= 0`. I traced the failure case concretely: if every lambda in the sweep folds, each jacobian_scan returns 2, first_sign becomes 2, no comparison differs, sign_consistent stays true, all rows print "+", and the study exits 0 printing "the blend is valid". So the gate is genuinely weaker than it reads — the auditor's reference form (assert sign != 2 per lambda) is the right check. The min|detJ| backstop is also not a reliable substitute: my 256x256 reproduction of the dtheta=3*pi/2, lambda=0.25 case returned min|det| = 3.106e-05, thirty-fold above the 1e-6 threshold, while the sign was genuinely mixed (7631 positive vs 57905 negative samples). The published run is unaffected — the shipped geometry is DTHETA = PI/2 and lambda 0 gives sign −1 throughout, so a partial fold would still be caught by the cross-lambda comparison.

> Evidence re-read: deep_causality_cfd/studies/qtt_blend_metric/main.rs:55-67 (sweep + sign_consistent + print), :72-77 (gate BM-A), :130-159 (jacobian_scan, sign = 2 at :154); independent 256x256 det-sign reproduction

---

### 7.13 [MINOR] The rho unit test asserts against a re-typed copy of the implementation's own formula

- **Verification verdict:** CONFIRMED
- **Axis:** tautology-circular
- **Location:** `deep_causality_cfd/tests/tensor_bridge/acoustic_inverse_tests.rs:101`
- **Auditor confidence:** confirmed

**Claim.** `rho_matches_the_analytic_contracting_root` computes its 'expected' value with the identical expression the implementation uses, so it cannot detect an error in that expression — only a transcription error between two copies of it. The independently-checkable number is present only in a comment, not in the assertion.

**Code evidence.**

```
tests/tensor_bridge/acoustic_inverse_tests.rs:99-102:
  // ρ = (1 + 2s − √(1+4s)) / (2s); for s = 8 that is (17 − √33)/16 ≈ 0.7034, and 0 < ρ < 1.
  let inv = AcousticCoreInverse::new_1d(6, S, tr()).unwrap();
  let expected = (1.0 + 2.0 * S - (1.0 + 4.0 * S).sqrt()) / (2.0 * S);
  assert!((inv.rho() - expected).abs() < 1e-12, "rho = {}", inv.rho());
versus the implementation, src/tensor_bridge/acoustic_inverse.rs:122:
  let rho = (one + two * s - (one + four * s).sqrt()) / (two * s);
```

**Reference form.** A verification test must compare against a value derived independently of the code path under test — here, either the literal 0.703458..., or the defining property s*rho^2 - (1+2s)*rho + s = 0, or the factorization identity s*(1-rho)^2 = rho.

**Impact.** Bounded: the substantive verification is `closed_form_inverse_solves_to_roundoff_and_is_resolution_stable` (same file, lines 66-80), which rebuilds A0 independently from shift MPOs and checks the residual — that test is genuine and would catch a wrong rho. So the wrong root is not actually shippable. The finding is that this specific test contributes no verification value while reading as if it does.

**Recommended fix.** Replace the assertion with the residual of the defining quadratic: assert!((S*r*r - (1.0+2.0*S)*r + S).abs() < 1e-14) and additionally assert the factorization identity (S*(1.0-r)*(1.0-r) - r).abs() < 1e-14. Both are independent of how rho is computed.

**Adversarial check.** tests/tensor_bridge/acoustic_inverse_tests.rs:98-105 computes `expected = (1.0 + 2.0*S - (1.0+4.0*S).sqrt()) / (2.0*S)` and asserts inv.rho() matches it to 1e-12; src/tensor_bridge/acoustic_inverse.rs:122 is `let rho = (one + two*s - (one + four*s).sqrt()) / (two*s)` — the same expression term for term. The independently checkable value ((17−√33)/16 ≈ 0.7034) appears only in the comment at :99, never in an assertion. The test does add one non-tautological check the auditor did not credit: `assert!(inv.rho() > 0.0 && inv.rho() < 1.0)` at :104, which would catch selection of the expanding root — so it is not entirely without value. The auditor's reference form is right (assert the literal, or the defining quadratic s·ρ² − (1+2s)·ρ + s = 0, or the identity s(1−ρ)² = ρ; the latter two I verified are the correct characterizations from acoustic_inverse.rs:17-19,34). The bounding argument is also correct: closed_form_inverse_solves_to_roundoff_and_is_resolution_stable (:66-80) rebuilds A₀ from shift MPOs and checks the residual, so a wrong rho cannot ship.

> Evidence re-read: deep_causality_cfd/tests/tensor_bridge/acoustic_inverse_tests.rs:98-105 (verbatim, incl. the 0<rho<1 assertion at :104) and :66-80; src/tensor_bridge/acoustic_inverse.rs:121-122 and :15-19 (the quadratic)

---

### 7.14 [MINOR] BlendedMap accepts any theta0 but its Cartesian partner rectangle is only geometrically compatible with a fan centred on the +x axis

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/coordinate/blended.rs:143`
- **Auditor confidence:** confirmed

**Claim.** The Cartesian-capture chart is hard-coded as x = r0 + eta*dr, y = -span_y/2 + xi*span_y — a rectangle centred on the +x axis. The fan spans theta in [theta0, theta0+dtheta]. The two charts are 'compatibly oriented' only when theta0 = -dtheta/2. The constructor accepts any theta0 and the requirement is documented nowhere.

**Code evidence.**

```
blended.rs:143 (the partner geometry, independent of theta0):
  let span_y = two * (r0 + half * dr) * (half * dtheta).sin();
blended.rs:233-234 (Cartesian chart, independent of theta0):
  let xc = self.r0 + eta * self.dr;
  let yc = (R::zero() - half) * self.span_y + xi * self.span_y;
blended.rs:100-101 (doc, which asserts compatibility without stating the precondition):
  /// radial extent `dr` and transverse width `2·(r0+½dr)·sin(½dtheta)` (the fan chord at mid radius), so
  /// the two charts are compatibly oriented.
The test suite uses both conventions: tests/coordinate/blended_tests.rs:23 uses theta0 = -dth/2.0 (correct), while blended_tests.rs:131 and 152 construct with theta0 = 0.0.
```

**Reference form.** For a position blend T_lambda = (1-lambda)T_cart + lambda*T_fit to be a meaningful interpolation, the two charts must map the same computational patch onto overlapping physical regions with the same orientation. Rotating the fan by theta0 without rotating the rectangle breaks that correspondence.

**Impact.** With theta0 = 0 and dtheta = pi/2 the fan occupies the first quadrant while the rectangle straddles the x-axis; the blend at intermediate lambda is a chart with no geometric meaning even though det J happens to stay single-signed there (I confirmed min|det| = 0.91 at lambda = 0.6 for that case, so no error surfaces). Combined with the missing fold guard, a caller can silently obtain either a meaningless-but-valid chart or a genuinely folded one, with the same Ok return.

**Recommended fix.** Either derive the Cartesian partner from theta0 (rotate the rectangle to the fan's bisector, i.e. build it in the frame of theta_mid = theta0 + dtheta/2), or document and enforce the precondition theta0 = -dtheta/2 in `BlendedMap::new`. Update blended_tests.rs:131 and :152 to the supported convention.

**Adversarial check.** Verified in full. span_y (blended.rs:143) and the Cartesian chart (:233-234, xc = r0 + η·Δr, yc = −½·span_y + ξ·span_y) contain no theta0 term, so the rectangle is fixed on the +x axis while the fan spans [theta0, theta0+dtheta] (:148, :231). Compatibility therefore requires theta0 = −dtheta/2, and no doc states it: blended.rs:99-101 asserts "the two charts are compatibly oriented" as a conclusion, and the # Errors block (:103-105) lists only the sign and range guards. The test suite does split as claimed — blended_tests.rs:23 builds every cfg() with theta0 = −dth/2.0 and its comment at :35 acknowledges the requirement ("BlendedMap centers the fan at θ = 0 ...; the body-fitted partner must match"), while :131 and :152 construct directly with theta0 = 0.0. I confirmed the numeric claim: at theta0 = 0, dtheta = pi/2, lambda = 0.6 the 256x256 min|det| is 0.9103 with a uniform sign, so no error surfaces from a geometrically meaningless blend. The auditor's reference form (a position blend requires the two charts to map the patch onto overlapping regions with the same orientation) is correct.

> Evidence re-read: deep_causality_cfd/src/coordinate/blended.rs:99-105 (doc + Errors), :143 (span_y, no theta0), :147-156 (forward, theta0 only in the fan), :227-237 (position: xc/yc theta0-free); tests/coordinate/blended_tests.rs:21-24, :35-36, :127-133, :149-153; numeric scan theta0=0, dtheta=pi/2, lambda=0.6 → min|det| = 0.9103, single sign

---

### 7.15 [MINOR] Cited constructions (Peddinti, Kazeev–Khoromskij) have no resolvable reference and no paper in the crate's papers/ directory

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/tensor_bridge/mod.rs:12`
- **Auditor confidence:** confirmed

**Claim.** The bridge documents its operator construction as 'following the Peddinti (MPS Navier–Stokes) and Kazeev–Khoromskij (QTT operators) constructions' with no title, year, venue, or DOI, and neither paper is present in deep_causality_cfd/papers/, so a reviewer cannot check the implementation against what the cited work actually states.

**Code evidence.**

```
tensor_bridge/mod.rs:11-13:
  //! field codec ([`quantize`] / [`dequantize`]) and periodic finite-difference MPO assembly
  //! ([`shift_plus`] / [`shift_minus`] / [`gradient`] / [`laplacian`]), following the Peddinti
  //! (MPS Navier–Stokes) and Kazeev–Khoromskij (QTT operators) constructions.
`ls deep_causality_cfd/papers/` returns: Droege2005.pdf, kirkpatrick2003.pdf, mittal2005.pdf, mohamed2016.pdf — none of the cited works.
```

**Reference form.** The repository's own stated convention is a full reference in the kernel docstring plus the PDF in the crate's papers/ folder. Audit axis A requires that when code cites a paper, the code be checkable against what that citation states the formula is.

**Impact.** The stencils themselves I verified independently against textbook finite-difference forms and they are correct, so no wrong math follows. The gap is traceability: for a certification consumer, 'following the Kazeev–Khoromskij construction' is an unverifiable assertion as written. It also matters here because the crate uses a *serial* (block, non-interleaved) mode layout per axis, whereas the multidimensional QTT literature commonly uses an interleaved-bit layout with materially different rank behaviour — a reader cannot tell from the citation whether the deviation is deliberate.

**Recommended fix.** Add full references (authors, title, venue, year, DOI) in the module docstring and drop the PDFs into deep_causality_cfd/papers/, per the crate's own convention. Add one sentence stating that the mode layout here is serial-per-axis rather than interleaved, and why.

**Adversarial check.** tensor_bridge/mod.rs:6-13 reads exactly as quoted: "...following the Peddinti (MPS Navier–Stokes) and Kazeev–Khoromskij (QTT operators) constructions" — no title, year, venue, or DOI. `ls deep_causality_cfd/papers/` returns exactly Droege2005.pdf, kirkpatrick2003.pdf, mittal2005.pdf, mohamed2016.pdf; neither cited work is present, and grep finds no fuller reference for either name elsewhere in the crate. This is measured against the repository's own stated convention (full reference in the kernel docstring plus the PDF in the crate's papers/ folder), so the citation is genuinely untraceable as written. The substantive point about layout is fair too: operators.rs builds a serial per-axis block layout (gradient_x/gradient_y over MSB-first blocks, with lift_leading/lift_trailing composing axes), which a reader cannot check against the cited construction without the reference. No wrong math follows — I verified the stencils independently in Findings 6 and 7.

> Evidence re-read: deep_causality_cfd/src/tensor_bridge/mod.rs:6-13; ls deep_causality_cfd/papers/ → Droege2005.pdf, kirkpatrick2003.pdf, mittal2005.pdf, mohamed2016.pdf; grep for 'Peddinti|Kazeev|Khoromskij' across the crate → the single mod.rs mention

---

### 7.16 [INFO] The 'smooth interior contracts' result rests on a single unswept stiffness constant and a single profile, with gate bounds set just around the observed values

- **Verification verdict:** CONFIRMED
- **Axis:** magic-number
- **Location:** `deep_causality_cfd/studies/qtt_acoustic_precond/main.rs:31`
- **Auditor confidence:** confirmed

**Claim.** rho(A0^-1 A1) is a function of the stiffness s, yet the study measures it at exactly one value (STIFF = 8.0), one grid (2^7), and one smooth profile, then states the contraction result generally. The AC-B/AC-C gate thresholds sit just below the observed numbers.

**Code evidence.**

```
main.rs:31:  const STIFF: f64 = 8.0;   (doc: "Implicit-acoustic stiffness `s = Δt²·c̄²/Δx²` (> 1 ⇒ the acoustic CFL the IMEX step removes)" — no justification for the value 8)
main.rs:55:  let l = 7usize;   (single grid for AC-B/AC-C)
main.rs:123:  let c = 1.0 + 0.3 * (2.0 * PI * x).sin();   (single smooth profile; the 0.3 amplitude directly sets rho)
Gates vs. measured (output.txt:5-6 gives rho_smooth = 0.590, rho_jump = 0.872, ratio 1.478):
  main.rs:62:  if rho_smooth >= 0.8
  main.rs:70:  if rho_jump < 0.82 || rho_jump < 1.35 * rho_smooth
```

**Reference form.** For a claim of the form 'on a smooth sound-speed field the preconditioned operator contracts', the supporting evidence must establish the dependence on the parameters that control it — here the stiffness s and the coefficient contrast max(c^2)/min(c^2). A single point does not establish a bound.

**Impact.** The computation itself is sound and non-circular: A0 and A1 are an exact split (I verified this algebraically), the dense inverse and power iteration avoid any solver tolerance, and the periodic Laplacian matrix is the correct stencil — so 0.590 is a real number. The limitation is scope. 0.8 / 0.82 / 1.35 are regression locks chosen after seeing 0.590 / 0.872 / 1.478, which is legitimate as regression detection but is not a derived bound, and the README/reading text presents the result as a general property of smooth interiors. An engineer sizing the implicit step at a different Delta t (hence different s) has no basis to expect contraction.

**Recommended fix.** Sweep s over at least a decade (e.g. 0.5, 2, 8, 32, 128) and the smooth-profile amplitude over a range, tabulate rho, and state the observed dependence. Label the 0.8 / 0.82 / 1.35 thresholds explicitly as regression locks tied to the recorded output rather than as derived bounds, and justify STIFF = 8.0 by tying it to a representative Delta t and Delta x from an actual marcher configuration.

**Adversarial check.** Every citation is verbatim. main.rs:31 is `const STIFF: f64 = 8.0;` with the doc comment defining s but giving no basis for 8 (the header only requires s > 1); main.rs:55 fixes l = 7 for AC-B/AC-C; c2_profile (main.rs:116-132) has exactly one smooth family, c = 1 + 0.3·sin(2πx), and one jump family. The gates at :62 (rho_smooth >= 0.8) and :70 (rho_jump < 0.82 || rho_jump < 1.35*rho_smooth) sit just outside the recorded output.txt values 0.590 / 0.872 (ratio 1.478). No sweep over s or over the contrast max(c²)/min(c²) exists anywhere in the study. The auditor's reference standard is right — a contraction claim must establish dependence on the parameters that control ρ — and their concession is right too: the computation is non-circular (spectral_radius at :178-210 builds A₀ and A₁ densely from the same exact split, inverts densely, and power-iterates, with no solver tolerance in the loop), so 0.590 is a real number for that one configuration. Correctly filed as info: a scope/regression-lock limitation, not an error.

> Evidence re-read: deep_causality_cfd/studies/qtt_acoustic_precond/main.rs:30-35 (STIFF, MAX_RANK, tolerances), :55-72 (l = 7, gates AC-B/AC-C), :116-132 (c2_profile), :176-210 (spectral_radius, dense A0/A1 + dense inverse + power iteration); studies/qtt_acoustic_precond/output.txt (rho_smooth = 0.590, rho_jump = 0.872)

---

### 7.17 [INFO] No metric tensor g = J^T J, inverse metric, or Christoffel symbols exist in the coordinate module (scope determination)

- **Verification verdict:** CONFIRMED
- **Axis:** doc-gap
- **Location:** `deep_causality_cfd/src/traits/metric_provider.rs:24`
- **Auditor confidence:** confirmed

**Claim.** Recorded as a definitive scope answer: the module named 'MetricProvider' supplies only the inverse Jacobian components (the contravariant basis) and |det J|. There is no covariant metric tensor g = J^T J, no g^-1, and no Christoffel symbols anywhere in src/coordinate/.

**Code evidence.**

```
The complete state of each chart is the four/nine inverse-Jacobian trains plus one Jacobian train — coordinate/mod.rs:64-70 (dxi_dx, deta_dx, dxi_dy, deta_dy, jacobian), blended.rs:85-91 (same), body_fitted_3d.rs:56-65 (eight inverse components plus jacobian). The trait surface is exactly dims / sample / physical_gradient / jacobian (traits/metric_provider.rs:32-53). Grep for 'christoffel' and for any g_ij assembly across src/coordinate/ returns nothing.
```

**Reference form.** For chain-rule first derivatives in a curvilinear chart, only the contravariant basis dxi^i/dx^j is required; g = J^T J and the Christoffel symbols are needed for second-order operators (Laplace–Beltrami), covariant derivatives of vector/tensor fields, and the strong-conservation transformation.

**Impact.** None today — the docs never claim g or Christoffel symbols, and the chain-rule first derivative is the only operation the marchers request, for which the stored data is sufficient and correct. Recorded because the audit brief asked for the g = J^T J verification: it does not exist to verify. It also bounds what can be built on this seam: adding a curvilinear viscous term (Laplace–Beltrami) or the strong-conservation form will require data the trait does not currently expose.

**Recommended fix.** No change required for correctness. Consider a one-line note in the MetricProvider docs stating that the seam carries the contravariant basis and volume factor only, and that second-order/covariant operators would require extending it with g and the Christoffel symbols — so a consumer does not assume they are available.

**Adversarial check.** Verified by exhaustive read and grep. The 2-D charts store exactly four inverse-Jacobian trains plus one jacobian train (coordinate/mod.rs:64-70; blended.rs:85-91); body_fitted_3d.rs stores the nine inverse components plus the jacobian; the MetricProvider surface is exactly dims / sample / physical_gradient / jacobian (traits/metric_provider.rs:32-53). A case-insensitive grep across src/ for 'christoffel', 'metric_tensor', 'g_ij', and 'covariant' returns nothing. The auditor's reference framing is correct: the contravariant basis ∂ξ^i/∂x^j alone suffices for chain-rule first derivatives, which is the only operation the marchers request (marcher_2d.rs:244-245), while g = JᵀJ and the Christoffel symbols are needed for Laplace–Beltrami, covariant derivatives of vector/tensor fields, and the strong-conservation transformation — the last of which ties this directly to Findings 3 and 5. No doc claims g or Christoffel symbols, so this is a scope record, correctly filed as info.

> Evidence re-read: deep_causality_cfd/src/coordinate/mod.rs:61-72; src/coordinate/blended.rs:82-93; src/coordinate/body_fitted_3d.rs (nine inverse components + jacobian); src/traits/metric_provider.rs:27-54; grep -rni 'christoffel|metric_tensor|g_ij|covariant' src/ → no matches

---
