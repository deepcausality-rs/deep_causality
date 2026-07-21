## 1. Baseline before touching anything

- [ ] 1.1 Record the DEC momentum regression baseline (test count, and the lid-cavity / Poiseuille /
      immersed-block marched fields), so "the momentum path is untouched" is a measurement rather than
      an intention
- [ ] 1.2 Confirm no shipped case marches a scalar on the DEC path, making this change additive

## 2. The scalar rate (design D1)

- [ ] 2.1 Add a scalar rate evaluating `−i_u(dT) − κ·Δ_dR T` on 0-cochains, reusing the manifold's
      `interior_product`, `exterior_derivative_of` and `laplacian_of` rather than a parallel
      discretisation
- [ ] 2.2 State the diffusive sign against the Stage-0 pin (`Δ_dR = −∇²`) in the docstring, as the
      viscous term does, so the two are checkable against each other
- [ ] 2.3 Validate inputs at construction: finite non-negative `κ`, scalar length equal to the vertex
      count — matching the DEC family's envelope-validation convention (§7 of the audit)
- [ ] 2.4 Test: a constant field is stationary under any velocity and any `κ` (both terms vanish
      identically — the cheapest test that catches a grade or sign mix-up)
- [ ] 2.5 Test: pure diffusion decays `cos(kx)` as `exp(−κk²t)` to the scheme's order
- [ ] 2.6 Test: pure advection by a uniform divergence-free velocity translates the scalar and
      conserves its integral to round-off
- [ ] 2.7 Test: `κ = 0, u = 0` leaves the field bit-identical

## 3. The wall Dirichlet condition (design D2)

- [ ] 3.1 Derive the pinned vertex set from the same cut-cell registry the momentum no-slip uses, so
      the thermal and mechanical boundaries describe one body
- [ ] 3.2 Pin `T = T_w` on that set each step
- [ ] 3.3 Test: the constrained degrees of freedom hold `T_w` for every step
- [ ] 3.4 Test: a body hotter than the fluid raises the near-body temperature, with the far field
      lagging — the property that makes a gradient exist to measure

## 4. `wall_heat_flux` (design D3, D5)

- [ ] 4.1 Implement `q = −k ∮_S ∇T·n dA` over the registry's fragments, reusing
      `viscous_surface_force`'s fragment iteration, `Δh` construction and multilinear sampling so the
      two diagnostics agree about where the wall is
- [ ] 4.2 Reconstruct `∂T/∂n` one-sided to the true surface distance: `(T_sample − T_w)/Δh`, citing
      Kirkpatrick et al. (2003) as the viscous force does
- [ ] 4.3 State the sign convention at the API: with `n` the body's outward normal, positive `q` is
      heat leaving the wall into the fluid
- [ ] 4.4 Document the relation between `k` and `κ` (design open question), so a caller supplying both
      gets a physically consistent answer
- [ ] 4.5 Cross-reference `penalization_heat_integral` in both directions, each saying what the other
      is and why they are not interchangeable
- [ ] 4.6 Test: an isothermal field at `T_w` gives zero flux to round-off
- [ ] 4.7 Test: reversing the sign of `T_w − T_fluid` reverses the flux and preserves magnitude
- [ ] 4.8 Test: refusal paths — a registry with no fragments, a non-axis-aligned geometry (as
      `viscous_surface_force` refuses), and a non-finite `k`

## 5. Verification against a closed-form reference (design D4)

- [ ] 5.1 Choose a conduction configuration with an exact solution and state the closed form and its
      source in the harness
- [ ] 5.2 Gate the computed flux against it, with the bound **derived from resolution** and labelled
      `[reference]` per the Phase-1 evidence-class convention — not pinned to the first run
- [ ] 5.3 Show the gate bites: perturb the sign, the area weighting and `Δh` in turn, confirm each
      fails, then revert
- [ ] 5.4 Add a resolution ladder and record the observed order; if it does not converge, that is the
      finding and it is recorded rather than tuned away (audit §5b's precedent)
- [ ] 5.5 Register the harness in `.github/workflows/cfd_verification.yml` on the appropriate cadence,
      so it cannot escape both lists
- [ ] 5.6 Commit a baseline carrying a verdict, per the Phase-1 convention

## 6. Verify

- [ ] 6.1 The scalar obeys the analytic decay and advection references (spec: scalar advection–diffusion)
- [ ] 6.2 The wall holds `T_w` (spec: Dirichlet wall temperature)
- [ ] 6.3 The flux is a fragment-area-weighted surface integral with the stated sign convention
      (spec: Fourier-law wall heat flux)
- [ ] 6.4 An isothermal field gives zero flux; reversing `ΔT` reverses the sign
- [ ] 6.5 The flux matches the analytic conduction reference within its resolution-justified bound
- [ ] 6.6 **The DEC momentum path is bit-identical to the 1.1 baseline** — the scalar is passive, so any
      movement means something outside scope was touched
- [ ] 6.7 `cargo test -p deep_causality_cfd --release` — no regression against the 838-pass baseline
- [ ] 6.8 `bazel test //deep_causality_cfd/...` green — the workspace runner, and the check that caught
      the `include_str!` defect that `cargo test` hid
- [ ] 6.9 `make format && make fix` clean, no new `#[allow]`
- [ ] 6.10 No test asserts on source text (standing rule); every new test goes through the public API
