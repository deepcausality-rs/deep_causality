<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# `deep_causality_cfd/papers/`

Load-bearing constants, kernels, and validation references cite their source by author-year in the
docstring or harness where they are used; the source PDF lives here. This index maps each PDF to the
code that cites it, so an uncited PDF is visible rather than silently carried (AUDIT-REPORT Phase 4,
item 23).

## Present and cited

| PDF | Reference | Cited by |
|-----|-----------|----------|
| `kirkpatrick2003.pdf` | Kirkpatrick et al. (2003) | `src/solvers/dec/surface_force.rs` |
| `Droege2005.pdf` | Dröge & Verstappen (2005) | `verification/dec_cylinder_verification/` (St and C_d reference bands) |
| `mohamed2016.pdf` | Mohamed, Hirani & Samtaney (2016), "Discrete exterior calculus discretization of incompressible Navier–Stokes equations over surface simplicial meshes", J. Comput. Phys. 312:175–191 | `src/solvers/dec/mod.rs` (the DEC NS formulation this solver follows, on a periodic lattice complex rather than a surface simplicial mesh) |
| `mittal2005.pdf` | Mittal & Iaccarino (2005), "Immersed Boundary Methods", Annu. Rev. Fluid Mech. 37:239–261 | `src/solvers/qtt/immersed_2d.rs` (the immersed-boundary method class the Brinkman penalization belongs to) |

Both `mohamed2016.pdf` and `mittal2005.pdf` were carried here uncited until the Phase-4 sweep; each
was read, confirmed on topic, and cited at the module it supports.

## Cited in code, PDF not yet present

Add these PDFs to complete the convention. The bibliographic details below are what the citing code
states; confirm each against the published record when adding the file.

- **Angot, Bruneau & Fabrie (1999)**, "A penalization method to take into account obstacles in
  incompressible viscous flows", Numerische Mathematik 81(4):497–520. Cited by
  `src/solvers/qtt/immersed_2d.rs` and `verification/qtt_cylinder_verification/` for the Brinkman
  penalization method and its `η → 0` convergence rate (`O(η^{3/4})`).
- **Peddinti et al. (2024)**, "A quantum-inspired framework for computational fluid dynamics",
  Communications Physics 7, 135. Cited by `src/tensor_bridge/mod.rs` and the QTT verification
  READMEs as the MPS Navier–Stokes construction the bridge follows.
- **Kazeev & Khoromskij**, "Low-Rank Explicit QTT Representation of the Laplace Operator and Its
  Inverse". Cited by `src/tensor_bridge/mod.rs` for the QTT finite-difference operator construction.
  Author and title are as recorded in
  `openspec/notes/archive/cfd-plasma-blackout/gap-1/gap-one-cfd-tensor-bridge.md`; **venue and year are
  not recorded anywhere in this repository**, so they are deliberately omitted here rather than
  supplied from recall. Fill them in from the published record when adding the PDF.

Other references cited in prose without a PDF here (Park, RAM-C II, Millikan–White, Ghia 1982,
Taylor & Green 1937, Sod 1978, Gourianov et al. 2022) are named at their use sites with enough
detail to locate them; the crate README's `papers/` row points at this index rather than claiming
every cited source is present.
