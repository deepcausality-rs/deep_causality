/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Gauge Theories: Particle Physics + Gravity Implementations
//!
//! This module implements particle physics and General Relativity gauge theories
//! built on top of the `GaugeField<G, A, F>` infrastructure provided by `deep_causality_topology`.
//!
//! ## 1. Implementation Scope
//!
//! | Theory                 | Gauge Group          | Module Path        | Status    |
//! |------------------------|----------------------|--------------------|-----------|
//! | **Electromagnetism**   | U(1)                 | `theories::electromagnetism` | Completed |
//! | **Weak Force**         | SU(2)                | `theories::weak`   | Completed |
//! | **Electroweak**        | SU(2) × U(1)         | `theories::ew`     | Completed |
//! | **General Relativity** | SO(3,1) / Lorentz    | `theories::gr`     | Completed |
//!
//! ## 2. Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        ARCHITECTURE                                     │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  deep_causality_physics                                                 │
//! │  ┌────────────────────────────────────────────────────────────────────┐ │
//! │  │                      theories/                                     │ │
//! │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐       │ │
//! │  │  │gauge_em │ │  weak   │ │   ew    │ │   qcd   │ │   gr    │       │ │
//! │  │  │  (U1)   │ │ (SU2)   │ │(SU2×U1) │ │ (SU3)   │ │(Lorentz)│       │ │
//! │  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘       │ │
//! │  │       │           │           │           │           │            │ │
//! │  │       └───────────┴───────────┼───────────┴───────────┘            │ │
//! │  │                               │                                    │ │
//! │  │                    uses GaugeField<G, A, F>                        │ │
//! │  └───────────────────────────────┼────────────────────────────────────┘ │
//! │                                  │                                      │
//! │  ────────────────────────────────┼───────────────────────────────────── │
//! │                                  ▼                                      │
//! │  deep_causality_topology                                                │
//! │  ┌────────────────────────────────────────────────────────────────────┐ │
//! │  │  GaugeField<G, A, F>  │  CurvatureTensor  │  HKT Witnesses         │ │
//! │  │  GaugeGroup trait     │  Adjunction d⊣∂   │  Promonad, RiemannMap  │ │
//! │  └────────────────────────────────────────────────────────────────────┘ │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 3. Convention Summary
//!
//! > **IMPORTANT**: GR and Particle Physics use opposite sign conventions.
//!
//! | Theory       | Convention | Signature | g_{μν}           | Metric Type       |
//! |--------------|------------|-----------|------------------|-------------------|
//! | Gauge EM, EW | West Coast | (+---)    | diag(1,-1,-1,-1) | `WestCoastMetric` |
//! | GR           | East Coast | (-+++)    | diag(-1,1,1,1)   | `EastCoastMetric` |
//!
//! ## 4. Type Aliases & Mapping
//!
//! The `alias` module defines the mapping between high-level theory names and generic gauge fields:
//!
//! * **Electromagnetism**: `GaugeField<U1, f64, f64, f64>`
//! * **WeakField**: `GaugeField<SU2, f64, f64>`
//! * **ElectroweakField**: `GaugeField<Electroweak, f64, f64>`
//! * **GR**: `GaugeField<Lorentz, f64, f64>`
//!
//! ## 5. Theory Specifications
//!
//! ### EM (Electromagnetism)
//! * **Module:** `theories::electromagnetism`
//! * **Gauge Group:** U(1)
//! * **Solves:** Maxwell's equations, Lorentz force, Energy/Lagrangian densities.
//! * **Key Integrations:** Uses `WestCoastMetric` (+---), maps $F_{0i} \to E_i$ (Electric) and $\epsilon_{ijk}F^{jk} \to B_i$ (Magnetic).
//! * **Note:** Computes classical observables; true QED would require quantum propagators.
//!
//! ### Electroweak
//! * **Module:** `theories::electroweak`
//! * **Gauge Group:** SU(2) × U(1)
//! * **Solves:** Symmetry breaking (Higgs), Weinberg angle mixing ($\theta_W$), Mass generation ($W^\pm, Z^0$).
//! * **Key Integrations:** manages product structure of two disjoint gauge bundles.
//!
//! ### General Relativity
//! * **Module:** `theories::general_relativity`
//! * **Gauge Group:** SO(3,1) / Lorentz
//! * **Solves:** Einstein field equations, Geodesic equation, Tidal forces (Geodesic deviation).
//! * **Key Integrations:** Uses `EastCoastMetric` (-+++), implements full geometric contraction for invariants like Kretschmann scalar.
//!
//! ### Weak Force
//! * **Module:** `theories::weak_force`
//! * **Gauge Group:** SU(2)
//! * **Solves:** Non-abelian field strength $W_{\mu\nu}^a$, Chirality ($P_L$), Weak currents.
//! * **Key Integrations:** Handles non-abelian commutator term $[A, A]$ in curvature.
//!
pub mod alias;
pub mod electromagnetism;
pub mod electroweak;
pub mod general_relativity;
pub mod weak_force;

pub use alias::*;
pub use electromagnetism::*;
pub use electroweak::*;
pub use general_relativity::*;
pub use weak_force::*;
