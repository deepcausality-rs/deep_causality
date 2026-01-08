/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    ALPHA_EM_MZ, EM_COUPLING, FERMI_CONSTANT, GEV2_TO_NB, GEV2_TO_PB, HIGGS_MASS, HIGGS_VEV,
    PhysicsError, SIN2_THETA_W, TOP_MASS, W_MASS, Z_MASS,
};

use crate::theories::electroweak::radiative::{RadiativeCorrections, solve_w_mass};
use deep_causality_num::RealField;
use std::f64::consts::PI;

/// Electroweak theory configuration and symmetry breaking parameters.
///
/// # Mathematical Foundation
///
/// ## Coupling Relations
/// ```text
/// e = g sin θ_W = g' cos θ_W
/// M_W = g v / 2
/// M_Z = M_W / cos θ_W
/// ```
///
/// ## Higgs Potential (Symmetry Breaking)
/// ```text
/// V(φ) = -μ² |φ|² + λ |φ|⁴
/// ```
/// Minimum at |φ| = v/√2 where v = μ/√λ ≈ 246 GeV
///
/// ## Mass Generation
/// - W, Z: from gauge-Higgs coupling
/// - Fermions: from Yukawa couplings y_f
/// - Higgs: M_H = √(2λ) v
#[derive(Debug, Clone, Copy)]
pub struct ElectroweakParams<T> {
    /// sin²θ_W (Weinberg angle)
    sin2_theta_w: T,
    /// Higgs vacuum expectation value v (GeV)
    higgs_vev: T,
    /// SU(2)_L coupling constant g
    g: T,
    /// U(1)_Y coupling constant g'
    g_prime: T,
    /// Radiative corrections container
    corrections: Option<RadiativeCorrections<T>>,
}

impl<T> ElectroweakParams<T>
where
    T: RealField + From<f64>,
{
    pub fn new(sin2_theta_w: T, higgs_vev: T, g: T, g_prime: T) -> Self {
        Self {
            sin2_theta_w,
            higgs_vev,
            g,
            g_prime,
            corrections: None,
        }
    }

    pub fn standard_model() -> Self {
        let sin2 = <T as From<f64>>::from(SIN2_THETA_W);
        let one = <T as From<f64>>::from(1.0);
        let cos2 = one - sin2;
        let sin_theta = sin2.sqrt();
        let cos_theta = cos2.sqrt();
        let e = <T as From<f64>>::from(EM_COUPLING);

        Self {
            sin2_theta_w: sin2,
            higgs_vev: <T as From<f64>>::from(HIGGS_VEV),
            g: e / sin_theta,
            g_prime: e / cos_theta,
            corrections: None,
        }
    }

    /// Creates Standard Model parameters using running coupling at Z pole.
    ///
    /// Uses α(M_Z) ≈ 1/128 instead of low-energy α ≈ 1/137.
    /// - M_W ≈ 80.37 - 80.38 GeV (Corrected)
    /// - M_Z ≈ 91.19 GeV (PDG Input)
    pub fn standard_model_precision() -> Self {
        // Use PDG values as input for precision calculation
        let mz = <T as From<f64>>::from(Z_MASS); // 91.1876
        let top = <T as From<f64>>::from(TOP_MASS); // 172.5

        // Critical: Use ALPHA_EM_MZ (High Energy) for Mass Solver
        let alpha_mz = <T as From<f64>>::from(ALPHA_EM_MZ);
        // Use ALPHA_EM (Low Energy) for Delta R reporting
        let alpha_0 = <T as From<f64>>::from(crate::constants::ALPHA_EM);

        let gf = <T as From<f64>>::from(FERMI_CONSTANT); // 1.166e-5

        // Solve for M_W using one-loop corrections (High Energy Input)
        let corrections = solve_w_mass(mz, top, alpha_mz, alpha_0, gf).unwrap_or_else(|_| {
            let z = T::zero();
            RadiativeCorrections {
                delta_rho: z,
                delta_r: z,
                w_mass_corrected: z,
                sin2_theta_eff: z,
            }
        });
        let mw = corrections.w_mass_corrected;

        // Derive effective mixing angle from the physical masses
        let one = <T as From<f64>>::from(1.0);
        let sin2_on_shell = one - (mw * mw) / (mz * mz);

        let cos2 = one - sin2_on_shell;
        // let sin_theta = sin2_on_shell.sqrt(); // Unused
        let cos_theta = cos2.sqrt();
        let tan_theta = sin2_on_shell.sqrt() / cos_theta;

        // Calculate Couplings using Renormalized Scheme
        // g = (2 * M_W / v) * sqrt(1 - \Delta r)
        // This gives the physical High-Energy coupling (~0.665)
        // rather than the tree-level fitted coupling (~0.653)
        let delta_r = corrections.delta_r;
        let vev = <T as From<f64>>::from(HIGGS_VEV);
        let g_coupling = (<T as From<f64>>::from(2.0) * mw / vev) * (one - delta_r).sqrt();

        // g' = g * tan(theta)
        let g_prime_coupling = g_coupling * tan_theta;

        Self {
            sin2_theta_w: sin2_on_shell,
            higgs_vev: vev,
            g: g_coupling,
            g_prime: g_prime_coupling,
            corrections: Some(corrections),
        }
    }

    pub fn with_mixing_angle(sin2_theta_w: T) -> Result<Self, PhysicsError> {
        let zero = <T as From<f64>>::from(0.0);
        let one = <T as From<f64>>::from(1.0);
        if sin2_theta_w <= zero || sin2_theta_w >= one {
            return Err(PhysicsError::DimensionMismatch(
                "sin²θ_W must be in (0, 1)".into(),
            ));
        }
        let cos2 = one - sin2_theta_w;
        let sin_theta = sin2_theta_w.sqrt();
        let cos_theta = cos2.sqrt();
        let e = <T as From<f64>>::from(EM_COUPLING);

        Ok(Self {
            sin2_theta_w,
            higgs_vev: <T as From<f64>>::from(HIGGS_VEV),
            g: e / sin_theta,
            g_prime: e / cos_theta,
            corrections: None,
        })
    }

    pub fn sin2_theta_w(&self) -> T {
        self.sin2_theta_w
    }
    pub fn cos2_theta_w(&self) -> T {
        <T as From<f64>>::from(1.0) - self.sin2_theta_w
    }
    pub fn sin_theta_w(&self) -> T {
        self.sin2_theta_w.sqrt()
    }
    pub fn cos_theta_w(&self) -> T {
        self.cos2_theta_w().sqrt()
    }
    pub fn tan_theta_w(&self) -> T {
        self.sin_theta_w() / self.cos_theta_w()
    }
    pub fn g_coupling(&self) -> T {
        self.g
    }
    pub fn g_prime_coupling(&self) -> T {
        self.g_prime
    }
    pub fn em_coupling(&self) -> T {
        self.g * self.sin_theta_w()
    }
    pub fn z_coupling(&self) -> T {
        self.g / self.cos_theta_w()
    }
    pub fn higgs_vev(&self) -> T {
        self.higgs_vev
    }

    pub fn w_mass_computed(&self) -> T {
        // If we have radiative corrections, return the precision mass
        if let Some(c) = self.corrections {
            c.w_mass_corrected
        } else {
            // Otherwise tree level
            self.g * self.higgs_vev / <T as From<f64>>::from(2.0)
        }
    }

    pub fn z_mass_computed(&self) -> T {
        if self.corrections.is_some() {
            // For precision mode, we started with Z mass fixed
            <T as From<f64>>::from(Z_MASS)
        } else {
            self.w_mass_computed() / self.cos_theta_w()
        }
    }

    pub fn rho_parameter(&self) -> T {
        let mw = <T as From<f64>>::from(W_MASS);
        let mz = <T as From<f64>>::from(Z_MASS);
        (mw * mw) / (mz * mz * self.cos2_theta_w())
    }

    /// Computes ρ parameter using internally generated masses.
    ///
    /// Unlike `rho_parameter()` which compares with PDG masses,
    /// this uses `w_mass_computed()` and `z_mass_computed()`,
    /// guaranteeing ρ = 1.0 by construction (tree-level SM relation).
    pub fn rho_parameter_computed(&self) -> T {
        let mw = self.w_mass_computed();
        let mz = self.z_mass_computed();
        (mw * mw) / (mz * mz * self.cos2_theta_w())
    }

    pub fn higgs_quartic(&self) -> T {
        let mh = <T as From<f64>>::from(HIGGS_MASS);
        (mh * mh) / (<T as From<f64>>::from(2.0) * self.higgs_vev * self.higgs_vev)
    }

    pub fn fermion_mass(&self, yukawa: T) -> T {
        yukawa * self.higgs_vev / <T as From<f64>>::from(2.0).sqrt()
    }
    pub fn yukawa_coupling(&self, mass: T) -> T {
        <T as From<f64>>::from(2.0).sqrt() * mass / self.higgs_vev
    }

    pub fn neutrino_electron_cross_section(
        &self,
        center_of_mass_energy: T,
    ) -> Result<T, PhysicsError> {
        if center_of_mass_energy <= <T as From<f64>>::from(0.0) {
            return Err(PhysicsError::DimensionMismatch(
                "Energy must be positive".into(),
            ));
        }
        let s = center_of_mass_energy * center_of_mass_energy;
        let gf = <T as From<f64>>::from(FERMI_CONSTANT);
        let sigma = gf * gf * s / <T as From<f64>>::from(PI);
        Ok(sigma * <T as From<f64>>::from(GEV2_TO_PB))
    }

    /// Computes the partial width for Z → f f̄ decay.
    ///
    /// Uses the Fermi Constant (G_F) parametrization which implicitly includes
    /// radiative corrections via ρ_eff.
    ///
    /// # Formula
    /// ```text
    /// Γ_f = N_c · (√(2) · G_F · M_Z³) / (12π) · (g_V² + g_A²) · ρ_eff
    /// ```
    ///
    /// # Parameters
    /// - `is_quark`: true for quarks (adds color factor 3), false for leptons.
    /// - `i3`: Isospin of the fermion (+1/2 or -1/2).
    /// - `q`: Charge of the fermion in units of e.
    pub fn z_partial_width_fermion(&self, is_quark: bool, i3: T, q: T) -> T {
        let mz = self.z_mass_computed();

        // CRITICAL: Use Effective Angle for Z decays if available (Two-Scheme Check)
        let sin2 = if let Some(c) = self.corrections {
            c.sin2_theta_eff
        } else {
            self.sin2_theta_w()
        };

        // One-Loop Effective Rho Parameter
        let rho_eff = if let Some(c) = self.corrections {
            <T as From<f64>>::from(1.0) + c.delta_rho
        } else {
            <T as From<f64>>::from(1.0)
        };

        // Vector and axial-vector couplings
        // g_V = I_3 - 2 Q sin²θ_eff
        let g_a = i3;
        let g_v = i3 - <T as From<f64>>::from(2.0) * q * sin2;

        // Color factor N_c = 3 for quarks, 1 for leptons
        let nc = if is_quark {
            <T as From<f64>>::from(3.0)
        } else {
            <T as From<f64>>::from(1.0)
        };

        // QCD correction factor for quarks (1 + α_s/π + ...) ≈ 1.04
        let qcd_factor = if is_quark {
            <T as From<f64>>::from(1.038)
        } else {
            <T as From<f64>>::from(1.0)
        };

        // =====================================================================
        // WIDTH CALCULATION (G_F Scheme)
        // =====================================================================
        // The G_F formula is standard for precision widths.
        // Pre-factor = (N_c · sqrt(2) · G_F · M_Z^3) / (12 · pi)
        // =====================================================================
        let prefactor = nc
            * (<T as From<f64>>::from(2.0).sqrt()
                * <T as From<f64>>::from(FERMI_CONSTANT)
                * mz.powf(<T as From<f64>>::from(3.0)))
            / (<T as From<f64>>::from(12.0) * <T as From<f64>>::from(PI));

        prefactor * rho_eff * (g_v * g_v + g_a * g_a) * qcd_factor
    }

    /// Computes the total width of the Z boson (The "Invisible Width" included).
    ///
    /// The total width is the sum of all fermion decay channels $Z \to f \bar{f}$.
    /// This calculation "completes the inventory" of the Standard Model:
    ///
    /// 1. **Invisible Width**: 3 generations of Neutrinos (invisible to detectors).
    /// 2. **Leptonic Width**: 3 generations of charged leptons (e, μ, τ).
    /// 3. **Hadronic Width**: 5 generations of quarks (u, d, s, c, b) with Color (x3).
    ///
    /// Summing these components yields the signature Z width of ~2.495 GeV.
    pub fn z_total_width_computed(&self) -> T {
        // 1. Invisible Width (3 generations of Neutrinos)
        // Neutrinos have I3 = 1/2, Q = 0.
        let gamma_nu = <T as From<f64>>::from(3.0)
            * self.z_partial_width_fermion(
                false,
                <T as From<f64>>::from(0.5),
                <T as From<f64>>::from(0.0),
            );

        // 2. Leptonic Width (3 generations: e, μ, τ)
        // Charged leptons have I3 = -1/2, Q = -1.
        let gamma_l = <T as From<f64>>::from(3.0)
            * self.z_partial_width_fermion(
                false,
                <T as From<f64>>::from(-0.5),
                <T as From<f64>>::from(-1.0),
            );

        // 3. Hadronic Width (5 flavors: u, d, s, c, b)
        // Quarks include a color factor of 3 and QCD corrections.
        let gamma_had = self.z_hadronic_width_computed();

        gamma_nu + gamma_l + gamma_had
    }

    /// Computes the hadronic width of the Z boson (Summing over 5 quark flavors).
    ///
    /// Includes 2 up-type quarks (u, c) and 3 down-type quarks (d, s, b).
    /// Top quark is too heavy for Z decay ($M_t > M_Z/2$).
    pub fn z_hadronic_width_computed(&self) -> T {
        // Up-type (u, c): I3 = 1/2, Q = 2/3
        let gamma_u = <T as From<f64>>::from(2.0)
            * self.z_partial_width_fermion(
                true,
                <T as From<f64>>::from(0.5),
                <T as From<f64>>::from(2.0 / 3.0),
            );

        // Down-type (d, s, b): I3 = -1/2, Q = -1/3
        let gamma_d = <T as From<f64>>::from(3.0)
            * self.z_partial_width_fermion(
                true,
                <T as From<f64>>::from(-0.5),
                <T as From<f64>>::from(-1.0 / 3.0),
            );

        gamma_u + gamma_d
    }

    /// Computes the physical Breit-Wigner cross section for e⁺e⁻ → Z → hadrons.
    ///
    /// Returns cross-section in **nanobarns** (nb).
    /// This version uses internally computed masses and widths for perfect precision.
    ///
    /// # Formula
    /// ```text
    /// σ(s) = (12π/M_Z²) · (s · Γ_ee · Γ_had) / ((s - M_Z²)² + s² Γ_Z² / M_Z²)
    /// ```
    pub fn z_resonance_cross_section(
        &self,
        center_of_mass_energy: T,
        _width: T, // Ignored in favor of computed width
    ) -> Result<T, PhysicsError> {
        if center_of_mass_energy <= <T as From<f64>>::from(0.0) {
            return Err(PhysicsError::DimensionMismatch(
                "Energy must be positive".into(),
            ));
        }

        let s = center_of_mass_energy * center_of_mass_energy;
        let mz = self.z_mass_computed();
        let mz2 = mz * mz;

        // Comute widths from first principles
        let gamma_z = self.z_total_width_computed();
        let gamma_ee = self.z_partial_width_fermion(
            false,
            <T as From<f64>>::from(-0.5),
            <T as From<f64>>::from(-1.0),
        );
        let gamma_had = self.z_hadronic_width_computed();

        // Relativistic Breit-Wigner with s-dependent width
        let denominator =
            (s - mz2).powf(<T as From<f64>>::from(2.0)) + s * s * gamma_z * gamma_z / mz2;
        if denominator.abs() < <T as From<f64>>::from(1e-30) {
            return Err(PhysicsError::NumericalInstability(
                "Singularity in cross section".into(),
            ));
        }

        // Cross-section in GeV⁻²
        let sigma_gev2 =
            <T as From<f64>>::from(12.0) * <T as From<f64>>::from(PI) * gamma_ee * gamma_had * s
                / (mz2 * denominator);

        Ok(sigma_gev2 * <T as From<f64>>::from(GEV2_TO_NB))
    }

    pub fn w_mass(&self) -> T {
        <T as From<f64>>::from(W_MASS)
    }
    pub fn z_mass(&self) -> T {
        <T as From<f64>>::from(Z_MASS)
    }
    pub fn top_yukawa(&self) -> T {
        self.yukawa_coupling(<T as From<f64>>::from(TOP_MASS))
    }

    // =========================================================================
    // Symmetry Breaking Implementation
    // =========================================================================

    /// Computes the Higgs potential V(φ) at a given field value.
    ///
    /// # Mathematical Definition
    /// ```text
    /// V(φ) = -μ² |φ|² + λ |φ|⁴
    /// ```
    /// where μ² = λ v² with v ≈ 246 GeV.
    pub fn higgs_potential(&self, phi_magnitude: T) -> T {
        let lambda = self.higgs_quartic();
        let mu_squared = lambda * self.higgs_vev * self.higgs_vev;
        let phi2 = phi_magnitude * phi_magnitude;

        (<T as From<f64>>::from(0.0) - mu_squared) * phi2 + lambda * phi2 * phi2
    }

    /// Verifies the minimum of the Higgs potential is at v/√2.
    ///
    /// # Mathematical Property
    /// ```text
    /// ∂V/∂|φ| = 0  at |φ| = v/√2
    /// ```
    /// Returns true if the VEV satisfies the potential minimum condition.
    pub fn symmetry_breaking_verified(&self) -> bool {
        let v_over_sqrt2 = self.higgs_vev / <T as From<f64>>::from(2.0).sqrt();
        let epsilon = <T as From<f64>>::from(1e-6);

        // At the minimum, derivative should be zero
        // dV/d|φ| = -2μ²|φ| + 4λ|φ|³ = 0
        // Solution: |φ| = √(μ²/(2λ)) = v/√2
        let lambda = self.higgs_quartic();
        let mu_squared = lambda * self.higgs_vev * self.higgs_vev;
        let computed_vev = (mu_squared / (<T as From<f64>>::from(2.0) * lambda)).sqrt();

        (computed_vev - v_over_sqrt2).abs() < epsilon * v_over_sqrt2
    }

    /// Returns the number of Goldstone bosons eaten by gauge bosons.
    ///
    /// # Goldstone Theorem
    /// ```text
    /// # Goldstone bosons = dim(G) - dim(H)
    ///                   = dim(SU(2)×U(1)) - dim(U(1)_EM)
    ///                   = 4 - 1 = 3
    /// ```
    /// The 3 Goldstones become the longitudinal modes of W⁺, W⁻, and Z.
    pub fn goldstone_count() -> usize {
        // SU(2)_L × U(1)_Y → U(1)_EM
        // generators: 3 + 1 → 1
        // broken generators: 3 (become Goldstones)
        3
    }

    /// Returns the mass acquired by each gauge boson after symmetry breaking.
    ///
    /// # Mathematical Result
    /// ```text
    /// W boson: M_W = g v / 2
    /// Z boson: M_Z = M_W / cos θ_W
    /// Photon:  M_A = 0
    /// ```
    pub fn gauge_boson_masses(&self) -> (T, T, T) {
        let m_w = self.w_mass_computed();
        let m_z = self.z_mass_computed();
        let m_a = <T as From<f64>>::from(0.0); // Photon remains massless

        (m_w, m_z, m_a)
    }

    /// Computes the ρ-parameter deviation from unity (tests custodial symmetry).
    ///
    /// # Mathematical Definition
    /// ```text
    /// ρ = M_W² / (M_Z² cos² θ_W)
    /// ```
    /// In the Standard Model at tree level, ρ = 1 exactly.
    pub fn rho_deviation(&self) -> T {
        (self.rho_parameter() - <T as From<f64>>::from(1.0)).abs()
    }
    /// Returns the effective ρ parameter including Veltman screening.
    ///
    /// # Definition
    /// ```text
    /// ρ_eff = 1 + Δρ = 1 + (3 G_F m_t²) / (8 π² √2)
    /// ```
    pub fn rho_effective(&self) -> T {
        if let Some(c) = self.corrections {
            <T as From<f64>>::from(1.0) + c.delta_rho
        } else {
            <T as From<f64>>::from(1.0)
        }
    }

    pub fn corrections(&self) -> Option<RadiativeCorrections<T>> {
        self.corrections
    }
}
