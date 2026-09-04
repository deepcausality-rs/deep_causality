## 1. Plume-shaped mask (tensor_bridge)

- [x] 1.1 Plume-region mask builder reusing the smoothed-mask codec (tanh volume-fraction
      skirt): geometry from `cordell_braun_plume_boundary` at a given C_T / momentum-flux
      ratio, quantized and rounded; report the resulting bond dimension
- [x] 1.2 Tests: mask is rank-bounded under the cap; dequantized volume fraction within
      rounding tolerance; two different C_T values produce measurably different masks

## 2. Compressible forcing region (solvers/qtt/compressible + carrier)

- [x] 2.1 Optional `ForcingRegion` (mask train, target conserved state, strength η) on the
      compressible march path; application per step via fused Hadamard + round (insertion
      point chosen against stability per design D1: rate-level inside IMEX or post-step
      relaxation, `enforce_inflow` precedent); `None` executes today's code path
- [x] 2.2 Config/builder plumbing so a harness world can carry a forcing region
      (`CompressibleMarchConfigBuilder` seam or marcher-level member per D1)
- [x] 2.3 Tests: interior converges toward the target to the penalization floor while the
      exterior evolves; forced state stays under the truncation cap; **bit-identity — the same
      world marched N steps with `None` forcing matches the pre-change marcher exactly**
- [x] 2.4 Register new test files in their `mod.rs` chain; verify Bazel suite globs pick them
      up (`bazel test //deep_causality_cfd/...`)

## 3. Drag contraction observable (solvers/qtt)

- [x] 3.1 Forebody-strip pressure-integration observable: pressure from the conserved
      components (ideal-gas closure) contracted against a strip mask (train `inner` + cell
      volume) → axial-force coefficient
- [x] 3.2 Preserved-drag fraction as the powered/unpowered same-configuration ratio
- [x] 3.3 Tests: contraction on a uniform-pressure field recovers the analytic strip integral;
      an unpowered run's own fraction is one within tolerance; observable series lands on the
      report

## 4. Verification: srp_drag_decrement

- [x] 4.1 `verification/srp_drag_decrement/` binary (family layout: `main.rs`, `config.rs`,
      `print_utils.rs`): M∞ = 2.0, γ = 1.4, central nozzle; C_T sweep 0 → ≈ 4 driving the
      imprint from the propulsion kernels; contraction per point; geometry caveats printed
      (2-D vs axisymmetric, skirt, blockage)
- [x] 4.2 Gates: per-point fraction within pinned band of `srp_preserved_drag_fraction`;
      collapse < 0.10 by `JARVINEN_ADAMS_TRANSITION_CT_M2`; total-axial-force dip below
      unpowered with minimum location within tolerance; nonzero exit on any FAIL
- [x] 4.3 First measured run: pin the bands as constants (provenance comments), commit the
      representative `output.txt` beside the binary
- [x] 4.4 Register the binary in the workspace (Cargo + Bazel) the way the sibling
      verifications are registered

## 5. Study: qtt_rank_plume (rank + fork economics)

- [x] 5.1 `studies/qtt_rank_plume/` binary (family layout): C_T sweep recording `max_bond` in
      Cartesian and blend-metric coordinates; both series printed; cap gate (at least one
      coordinate under cap)
- [x] 5.2 Fork-economics leg: march the imprinted layer to a mid-run pause; fork; roster —
      coast, two sign-flip straddlers, nominal, high — each publishing
      `"commanded_throttle"` into its branch world; continue all branches
- [x] 5.3 Measurements + gates: fork sharing (`shares_fluid_with`/`shares_field_with`) hard
      gate; per-branch step wall-clock ratio vs unforked trunk (recorded; band pinned at first
      run); post-fork `max_bond` under cap per branch (hard gate); branch flow observables
      spread across the roster
- [x] 5.4 Degraded-but-measured path: non-structural misses print flagged findings and still
      exit zero (the verdict makes the call); structural failures exit nonzero
- [x] 5.5 First measured run: pin the step-cost band, commit `output.txt`; register the binary
      (Cargo + Bazel) like the sibling studies

## 6. Verdict + cross-change sync

- [x] 6.1 Author `openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md` from the measured
      runs: three risk answers with numbers, pinned bands and their provenance, the
      green/amber/red call, and the explicit M3/M5 consequence table
- [x] 6.2 Update the roadmap's M1 row/section to point at this change and the verdict note
- [x] 6.3 Cross-check `plasma-retropulsion-cfd-contracts` remains consistent (shared
      `"commanded_throttle"` name; no file overlap; guard scope) — adjust its artifacts if
      implementation revealed drift

## 7. Corridor inheritance guard (prong A) + SDD verification

- [x] 7.1 `cargo run --release -p avionics_examples --example plasma_blackout_corridor` — exit
      0, witnesses equal the committed `output.txt` (the forcing seam touched the marcher path
      the corridor flies)
- [x] 7.2 `bazel build //deep_causality_cfd/...` and `bazel test //deep_causality_cfd/...`
      green
- [x] 7.3 `make format && make fix` — clippy clean without suppressions
- [x] 7.4 `make test` and `make check` (SDD pre-PR gate)
- [x] 7.5 Prepare the commit message(s) per task group and hand to the user (never commit);
      draft the PR summary referencing this change, the roadmap M1 go/no-go, and the verdict
