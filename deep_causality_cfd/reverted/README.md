<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Reverted binaries

Superseded verification and study binaries. Nothing here is deleted: each folder keeps its
source and its committed first-run output as the provenance of whatever verdict it produced,
moved with `git mv` (history preserved) and detached from `Cargo.toml` (no `[[example]]`
entry, so nothing builds or runs from this tree). Each entry names what supersedes it and why
it was reverted.

| Reverted | Superseded by | Why |
|---|---|---|
| `srp_drag_decrement/` | `studies/srp_momentum_jet/` | The SRP de-risk imprint-fidelity measurement (roadmap M1 risk 1, 2026-07-17). The original harness pinned the **entire Cordell plume envelope** to a uniform ambient-pressure state: a model class that cannot express the Jarvinen–Adams collapse mechanism in principle (interior gauge pressure zero by construction, momentum non-conservative inside the pin, exit-pressure ratio held at 1.0 when the correlation's own transition variable is ≈ 7), and whose measured "monotone shielding" curve was substantially the strip reading the pin itself (the envelope overlapped 20–72% of the strip height across the sweep). Two further methodology limits: a terminal-snapshot read on a still-drifting field (+2.4% between steps 500 and 2000), and node-sampled masks making the nominal 2×8 strip effectively 3×9. The superseding study measures the stronger model class (momentum-carrying jet, formed plume) on the same harness with tail-averaged reads, three strip bands, a momentum audit, and interface/probe/floor witnesses — and finds the harness itself (dissipation floor ν = ½·s_ref·Δx, domain blockage) cannot host the collapse under either coupling model, which is the corrected basis of the amber verdict. Authority: `openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md` (addendum 2026-07-17). The first-run `output.txt` in the reverted folder remains the pin provenance of the original amber call. |
