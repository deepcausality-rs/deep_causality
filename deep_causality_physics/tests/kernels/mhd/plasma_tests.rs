/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    ElectronDensity, Mass, PhysicalField, PhysicsErrorEnum, Speed, Temperature,
    debye_length_kernel, larmor_radius_kernel, plasma_frequency_kernel,
};

#[test]
fn test_plasma_frequency_matches_reference() {
    // ω_p = √(n_e e²/(ε₀ m_e)).
    //
    // Provenance: evaluated from the CODATA 2022 values of e, ε₀ and m_e at n_e = 1e18 m⁻³.
    // This kernel reads its constants internally, so pinning the full-precision value checks
    // the constants as well as the formula — a fixture that left them at 1 could not.
    //
    // Cross-checked against the NRL Plasma Formulary's practical form,
    // ω_pe = 5.64e4 · √(n_cm⁻³) rad/s, which gives 5.64e10 for n = 1e12 cm⁻³. The two agree to
    // 2.6e-4, which is the rounding of NRL's three-digit coefficient; the assertion below uses
    // the CODATA value because it is the more precise of the two.
    const OMEGA_P_1E18: f64 = 56_414_602_254.294_99;

    let n_e = ElectronDensity::<f64>::new(1.0e18).unwrap();
    let w = plasma_frequency_kernel(n_e).unwrap();
    assert!(
        (w.value() - OMEGA_P_1E18).abs() / OMEGA_P_1E18 < 1e-12,
        "omega_p = {}, expected {OMEGA_P_1E18}",
        w.value()
    );

    // ω_p ∝ √n_e: a millionfold drop in density lowers the frequency by exactly a thousand.
    // The exponent is what this pins; the absolute value above cannot distinguish √n from n.
    let sparse = plasma_frequency_kernel(ElectronDensity::<f64>::new(1.0e12).unwrap()).unwrap();
    assert!(
        (sparse.value() * 1000.0 - w.value()).abs() / w.value() < 1e-12,
        "omega_p(1e12) * 1000 = {}, expected {}",
        sparse.value() * 1000.0,
        w.value()
    );
}

#[test]
fn test_plasma_frequency_zero_density_errors() {
    // PlasmaFrequency requires a strictly positive value, so n_e = 0 (no plasma)
    // surfaces as an error; the blackout trigger treats that as link-available.
    let n_e = ElectronDensity::<f64>::new(0.0).unwrap();
    assert!(
        matches!(
            plasma_frequency_kernel(n_e).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_plasma_frequency_monotonic() {
    let lo = plasma_frequency_kernel(ElectronDensity::<f64>::new(1.0e17).unwrap()).unwrap();
    let hi = plasma_frequency_kernel(ElectronDensity::<f64>::new(1.0e19).unwrap()).unwrap();
    assert!(hi.value() > lo.value());
}

#[test]
fn test_debye_length() {
    // This test asserted only `is_ok()` and `value() > 0.0`, which any formula returning a
    // positive number satisfies — the Debye length had no numeric oracle anywhere in the suite.
    //
    // Provenance: λ_D = √(ε₀ k_B T_e / (n_e e²)) evaluated at the fixture's own constants,
    // ε₀ = 8.854e-12 F/m and e = 1.602e-19 C, with T = 100 K and n = 1e18 m⁻³.
    const LAMBDA_D_100K: f64 = 6.901_585_751_398_446e-7;

    let t = Temperature::new(100.0).unwrap();
    let n = 1e18;
    let eps0 = 8.854e-12;
    let e = 1.602e-19;

    let lambda = debye_length_kernel(t, n, eps0, e).unwrap();
    assert!(
        (lambda.value() - LAMBDA_D_100K).abs() / LAMBDA_D_100K < 1e-12,
        "lambda_D = {}, expected {LAMBDA_D_100K}",
        lambda.value()
    );
}

#[test]
fn test_debye_length_matches_codata_reference_at_one_electronvolt() {
    // The standard plasma-physics reference point: a 1 eV electron temperature.
    //
    // Provenance: λ_D = √(ε₀ k_B T_e / (n_e e²)) evaluated from CODATA 2022 ε₀, e and k_B at
    // T_e = 1 eV (= e/k_B = 11604.518121550082 K) and n_e = 1e18 m⁻³.
    //
    // Cross-checked against the NRL Plasma Formulary's practical form,
    // λ_D = 7.43e2 · √(T_eV / n_cm⁻³) cm, which gives 7.43e-6 m for these inputs. The two agree
    // to 5.3e-4, the rounding of NRL's three-digit coefficient.
    const T_ONE_EV_KELVIN: f64 = 11_604.518_121_550_082;
    const LAMBDA_D_1EV: f64 = 7.433_941_997_219_252e-6;

    let t = Temperature::new(T_ONE_EV_KELVIN).unwrap();
    let lambda = debye_length_kernel(t, 1.0e18, 8.854_187_818_8e-12, 1.602_176_634e-19).unwrap();
    assert!(
        (lambda.value() - LAMBDA_D_1EV).abs() / LAMBDA_D_1EV < 1e-12,
        "lambda_D = {}, expected {LAMBDA_D_1EV}",
        lambda.value()
    );
}

#[test]
fn test_debye_length_scales_as_sqrt_temperature_over_density() {
    // λ_D ∝ √(T/n). Quadrupling T doubles it; quadrupling n halves it. The pinned values above
    // fix one point; these fix the two exponents, which a single point cannot.
    let eps0 = 8.854_187_818_8e-12_f64;
    let e = 1.602_176_634e-19_f64;
    let base = debye_length_kernel(Temperature::<f64>::new(1000.0).unwrap(), 1.0e18, eps0, e)
        .unwrap()
        .value();

    let hot = debye_length_kernel(Temperature::<f64>::new(4000.0).unwrap(), 1.0e18, eps0, e)
        .unwrap()
        .value();
    assert!(
        (hot - 2.0 * base).abs() / base < 1e-12,
        "4x temperature gave {hot}, expected {}",
        2.0 * base
    );

    let dense = debye_length_kernel(Temperature::<f64>::new(1000.0).unwrap(), 4.0e18, eps0, e)
        .unwrap()
        .value();
    assert!(
        (dense - 0.5 * base).abs() / base < 1e-12,
        "4x density gave {dense}, expected {}",
        0.5 * base
    );
}

#[test]
fn test_debye_length_zero_density_error() {
    // density_n <= 0 -> Singularity (plasma.rs:31-33).
    let t = Temperature::new(100.0).unwrap();
    assert!(
        matches!(
            debye_length_kernel(t, 0.0, 8.854e-12, 1.602e-19)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
    let t2 = Temperature::new(100.0).unwrap();
    assert!(
        matches!(
            debye_length_kernel(t2, -1.0, 8.854e-12, 1.602e-19)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
}

#[test]
fn test_debye_length_non_positive_permittivity_error() {
    // epsilon_0 <= 0 -> PhysicalInvariantBroken (plasma.rs:34-38).
    let t = Temperature::new(100.0).unwrap();
    assert!(
        matches!(
            debye_length_kernel(t, 1e18, 0.0, 1.602e-19).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    let t2 = Temperature::new(100.0).unwrap();
    assert!(
        matches!(
            debye_length_kernel(t2, 1e18, -1.0, 1.602e-19)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

// NOTE on plasma.rs:41-42 — the `ok_or_else` closure body for
// `R::from_f64(BOLTZMANN_CONSTANT)`. `from_f64` is infallible for every
// concrete `RealField` used by this crate (f32/f64 always return `Some`), so
// the closure can never run. It is a defensive guard with no reachable input
// and is therefore left uncovered by design.

#[test]
fn test_larmor_radius() {
    let m = Mass::new(1.0).unwrap();
    let v = Speed::new(10.0).unwrap();
    let q = 1.0;

    let b_vec = CausalMultiVector::new(vec![0.0, 5.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b = PhysicalField::<f64>::new(b_vec);

    let res = larmor_radius_kernel(m, v, q, &b);
    assert!(res.is_ok());
    // r = mv/qB = 1*10 / 1*5 = 2
    assert!((res.unwrap().value() - 2.0).abs() < 1e-10);
}

#[test]
fn test_larmor_radius_matches_codata_reference_for_electron_and_proton() {
    // The fixture above sets m = q = 1, where the particle's identity cannot influence the
    // answer. These use the real masses and charge, so a formula that dropped or misplaced
    // either is visible.
    //
    // Provenance: r_L = m v_⊥ / (|q| B) evaluated from CODATA 2022 m_e, m_p and e at
    // v_⊥ = 1e6 m/s and B = 1 T.
    const R_L_ELECTRON: f64 = 5.685_630_111_305_193_5e-6;
    const R_L_PROTON: f64 = 1.043_968_492_895_896_3e-2;

    let e = 1.602_176_634e-19;
    let b_vec = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b = PhysicalField::<f64>::new(b_vec);
    let v = Speed::new(1.0e6).unwrap();

    let electron = larmor_radius_kernel(
        Mass::new(9.109_383_713_9e-31).unwrap(),
        v,
        -e, // the kernel takes |q|, so the electron's negative charge must not flip the sign
        &b,
    )
    .unwrap();
    assert!(
        (electron.value() - R_L_ELECTRON).abs() / R_L_ELECTRON < 1e-12,
        "electron r_L = {}, expected {R_L_ELECTRON}",
        electron.value()
    );

    let proton = larmor_radius_kernel(Mass::new(1.672_621_925_95e-27).unwrap(), v, e, &b).unwrap();
    assert!(
        (proton.value() - R_L_PROTON).abs() / R_L_PROTON < 1e-12,
        "proton r_L = {}, expected {R_L_PROTON}",
        proton.value()
    );

    // The proton is heavier by exactly the CODATA mass ratio, and r_L is linear in m, so the
    // radii must stand in that same ratio. This pins the linearity independently of either value.
    const PROTON_ELECTRON_MASS_RATIO: f64 = 1_836.152_673_426;
    assert!(
        (proton.value() / electron.value() - PROTON_ELECTRON_MASS_RATIO).abs()
            / PROTON_ELECTRON_MASS_RATIO
            < 1e-9,
        "mass ratio from radii = {}, expected {PROTON_ELECTRON_MASS_RATIO}",
        proton.value() / electron.value()
    );
}
