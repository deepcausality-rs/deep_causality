/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The per-term energy budget of the DEC Navier–Stokes rate (the
//! dec-ns-stability capability's diagnostic).
//!
//! For a state `u`, each field is the M-inner product `⟨u, term⟩_M`
//! (`Σ_e u_e (⋆term)_e`) of the state against one term of the marched
//! rate, **with the rate's signs** — so every entry reads directly as an
//! energy contribution: along the semi-discrete flow,
//! `dE/dt = ⟨u, P(rate)⟩_M`, and for a divergence-free state the
//! M-orthogonal projector drops out of the inner product, so
//! `projected ≈ convective + viscous + body_force` to solve tolerance.
//!
//! The diagnostic exists to *localize* energy injection: unforced
//! viscous flow must have `viscous ≤ 0` and `convective ≈ 0` (the
//! continuum convective term is energy-neutral); a term whose cumulative
//! contribution turns positive on a marched trajectory is the defect
//! (the 2026-06-12 TGV instability finding).

use deep_causality_algebra::RealField;

/// Per-term M-inner products of a state against the marched rate's
/// terms; see the module doc for the sign convention.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnergyBudget<R: RealField> {
    pub(crate) convective: R,
    pub(crate) viscous: R,
    pub(crate) body_force: R,
    pub(crate) projected: R,
}

impl<R: RealField> EnergyBudget<R> {
    /// `⟨u, −i_u(du♭)⟩_M` — the convective term's energy contribution
    /// (zero in the continuum; its discrete residue is the aliasing
    /// diagnostic).
    pub fn convective(&self) -> R {
        self.convective
    }

    /// `⟨u, −ν Δ_dR u♭⟩_M` — the viscous dissipation (must be ≤ 0).
    pub fn viscous(&self) -> R {
        self.viscous
    }

    /// `⟨u, g♭⟩_M` — the body-force power (zero when unforced).
    pub fn body_force(&self) -> R {
        self.body_force
    }

    /// `⟨u, P(rate)⟩_M` — the projected rate's energy contribution: the
    /// semi-discrete `dE/dt` the integrator marches.
    pub fn projected(&self) -> R {
        self.projected
    }

    /// The unprojected sum `convective + viscous + body_force`; matches
    /// [`Self::projected`] to solve tolerance for a divergence-free
    /// state (the projector is M-orthogonal and drops out).
    pub fn unprojected_sum(&self) -> R {
        self.convective + self.viscous + self.body_force
    }
}
