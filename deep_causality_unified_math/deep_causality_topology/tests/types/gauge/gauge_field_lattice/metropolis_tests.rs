/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_stats::Xoshiro256;
use deep_causality_topology::GaugeGroup;
use deep_causality_topology::LatticeComplex;
use deep_causality_topology::LatticeGaugeField;
use deep_causality_topology::LinkVariable;
use deep_causality_topology::SU3;
use std::sync::Arc;

// Define a test gauge group (U1 is simplest for testing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct U1;

impl GaugeGroup for U1 {
    const LIE_ALGEBRA_DIM: usize = 1;
    const IS_ABELIAN: bool = true;

    fn matrix_dim() -> usize {
        1
    }
    fn name() -> &'static str {
        "U1"
    }
}

#[test]
fn test_metropolis_sweep_empty_lattice() {
    let shape = [1];
    let lattice = Arc::new(LatticeComplex::new(shape, [false]));

    use std::collections::HashMap;
    let empty_links = HashMap::new();
    let mut field = LatticeGaugeField::<U1, 1, Complex<f64>, f64, ()>::from_links_unchecked(
        lattice,
        empty_links,
        1.0,
        (),
    );

    // Verify it is empty
    assert!(field.has_no_links());

    let mut rng = Xoshiro256::new();
    let acceptance = field.try_metropolis_sweep(0.1, &mut rng).unwrap();

    assert_eq!(acceptance, 0.0);
}

#[test]
fn test_metropolis_update_acceptance() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let beta = 1.0;

    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_identity(lattice.clone(), beta).unwrap();
    let mut rng = Xoshiro256::new();

    let edge = field.link_cells()[0].clone();

    let accepted = field.try_metropolis_update(&edge, 0.01, &mut rng).unwrap();
    assert!([true, false].contains(&accepted));

    let mut hot_field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_random(lattice, beta, &mut rng).unwrap();
    let edge_hot = hot_field.link_cells()[0].clone();

    let _ = hot_field
        .try_metropolis_update(&edge_hot, 0.1, &mut rng)
        .unwrap();
}

#[test]
fn test_metropolis_sweep_f64_optimization() {
    let shape = [4, 4];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let beta = 1.0;
    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_identity(lattice, beta).unwrap();
    let mut rng = Xoshiro256::new();

    let rate = field.try_metropolis_sweep(0.1, &mut rng).unwrap();
    assert!((0.0..=1.0).contains(&rate));
}

#[test]
fn test_metropolis_update_nan_handling() {
    // Basic test to ensure no panic
}

#[test]
fn test_metropolis_update_rejects_non_positive_epsilon() {
    use deep_causality_topology::TopologyErrorEnum;
    let shape = [2, 2];
    let lattice = Arc::new(LatticeComplex::new(shape, [true, true]));
    let mut field =
        LatticeGaugeField::<U1, 2, Complex<f64>, f64>::try_identity(lattice, 1.0).unwrap();
    let edge = field.link_cells()[0].clone();
    let mut rng = Xoshiro256::new();

    let err = field
        .try_metropolis_update(&edge, 0.0, &mut rng)
        .unwrap_err();
    match err.0 {
        TopologyErrorEnum::LatticeGaugeError(ref msg) => {
            assert!(msg.contains("Metropolis epsilon"));
        }
        ref other => panic!("Expected LatticeGaugeError, got {:?}", other),
    }

    let err_neg = field
        .try_metropolis_update(&edge, -0.1, &mut rng)
        .unwrap_err();
    match err_neg.0 {
        TopologyErrorEnum::LatticeGaugeError(_) => {}
        ref other => panic!("Expected LatticeGaugeError, got {:?}", other),
    }
}

#[test]
fn test_metropolis_sweep_empty_field_returns_zero_rate() {
    // Edge-case path through `if total == 0 { return Ok(0.0) }` in try_metropolis_sweep.
    use std::collections::HashMap;
    let lattice = Arc::new(LatticeComplex::<2, f64>::new([2, 2], [true, true]));
    let links: HashMap<_, LinkVariable<U1, Complex<f64>, f64>> = HashMap::new();
    let mut field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 1.0, ());
    let mut rng = Xoshiro256::new();
    let rate = field.try_metropolis_sweep(0.1, &mut rng).unwrap();
    assert_eq!(rate, 0.0);
}

#[test]
fn test_generate_small_su_n_update() {
    let shape = [2];
    let lattice = Arc::new(LatticeComplex::new(shape, [true]));
    let mut field =
        LatticeGaugeField::<U1, 1, Complex<f64>, f64>::try_identity(lattice, 1.0).unwrap();
    let mut rng = Xoshiro256::new();

    let edge = field.link_cells()[0].clone();

    for _ in 0..100 {
        field.try_metropolis_update(&edge, 0.2, &mut rng).unwrap();
    }
}

// ---------------------------------------------------------------------------------------------
// Regression: the proposal has to leave the identity.
//
// These run over SU(3) because the proposal is built from a traceless Hermitian generator, and
// the only traceless Hermitian 1x1 matrix is zero. Over U(1) the proposal is the identity for
// arithmetic reasons, so the tests above pass whatever the generator does.
// ---------------------------------------------------------------------------------------------

/// A cold start sits at plaquette exactly 1. One sweep has to move it off that value.
///
/// A proposal that lands back on the identity gives `U' = U` and `ΔS = 0` for every link, so the
/// sweep reports full acceptance and leaves the configuration exactly where it started.
#[test]
fn test_metropolis_sweep_moves_a_cold_start_off_the_identity() {
    let lattice = Arc::new(LatticeComplex::new([2, 2, 2, 2], [true; 4]));
    let mut field =
        LatticeGaugeField::<SU3, 4, Complex<f64>, f64>::try_identity(lattice, 6.0).unwrap();
    let mut rng = Xoshiro256::new();

    let before = field.try_average_plaquette().unwrap();
    assert!(
        (before - 1.0).abs() < 1e-12,
        "a cold start should measure plaquette 1, measured {before}"
    );

    field.try_metropolis_sweep(0.3, &mut rng).unwrap();

    let after = field.try_average_plaquette().unwrap();
    assert!(
        after < 1.0 - 1e-9,
        "one sweep left the plaquette at {after}, so every proposal returned the identity"
    );
}

/// A hot start thermalizes: sweeps carry the average plaquette away from its initial value, and
/// the acceptance rate sits below 1 because some proposals raise the action.
#[test]
fn test_metropolis_sweep_thermalizes_a_hot_start() {
    let lattice = Arc::new(LatticeComplex::new([2, 2, 2, 2], [true; 4]));
    let mut rng = Xoshiro256::new();
    let mut field =
        LatticeGaugeField::<SU3, 4, Complex<f64>, f64>::try_random(lattice, 6.0, &mut rng).unwrap();

    let before = field.try_average_plaquette().unwrap();

    let mut rates = Vec::new();
    for _ in 0..5 {
        rates.push(field.try_metropolis_sweep(0.2, &mut rng).unwrap());
    }

    let after = field.try_average_plaquette().unwrap();
    assert!(
        (after - before).abs() > 1e-3,
        "five sweeps held the plaquette at {before}, so the field never moved"
    );
    assert!(
        rates.iter().any(|&r| r < 1.0),
        "every link was accepted in every sweep, which is what a no-op proposal does: {rates:?}"
    );
}

/// A U(1) Metropolis sweep must actually move the field.
///
/// `generate_small_su_n_update` builds a traceless Hermitian generator, which is the right Lie
/// algebra for SU(N) with N >= 2. U(1) is not SU(1): its generator is a phase, so the real Lie
/// algebra is one-dimensional while the *traceless* Hermitian 1x1 matrices are `{0}`. Imposing
/// tracelessness at `n = 1` therefore made every proposal the identity, so no update could ever
/// be rejected and the field never left its starting configuration.
///
/// This pins both halves of that failure: the acceptance rate must not be exactly 1, and the
/// configuration must actually change.
#[test]
fn test_u1_metropolis_sweep_moves_the_field() {
    let lattice = Arc::new(LatticeComplex::new([4, 4], [true, true]));
    let mut rng = Xoshiro256::from_seed(42);
    let mut field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
        LatticeGaugeField::identity(lattice, 2.0);

    let before = field
        .try_average_plaquette()
        .expect("an identity field has a well-defined plaquette");

    let mut ever_rejected = false;
    for _ in 0..20 {
        let acceptance = field
            .try_metropolis_sweep(0.6, &mut rng)
            .expect("a sweep over a populated lattice succeeds");
        if acceptance < 1.0 {
            ever_rejected = true;
        }
    }

    let after = field
        .try_average_plaquette()
        .expect("the swept field has a well-defined plaquette");

    assert!(
        ever_rejected,
        "every U(1) proposal was accepted: the generator is identically zero, so the \
         proposal equals the current link and no update can be rejected"
    );
    assert!(
        (after - before).abs() > 1e-9,
        "the U(1) field did not move: <P> stayed at {before} after 20 sweeps"
    );
}

/// At strong coupling the U(1) field must order: `<P> -> I_1(beta)/I_0(beta)`, which is above
/// 0.9 for `beta = 10`. A frozen field started hot would sit near zero instead.
#[test]
fn test_u1_metropolis_thermalizes_toward_the_exact_solution() {
    // Two dimensional U(1) has a closed-form average plaquette, <P> = I_1(beta) / I_0(beta),
    // and it is checked here at two widely separated couplings rather than one. A single point
    // is a weak oracle: a 2% band around beta = 10 also contains beta = 8 and beta = 12, so
    // matching there says little. Between beta = 2 and beta = 10 the exact value moves by 0.25,
    // and an implementation with the wrong action or the wrong coupling does not track that.
    //
    // Reference values from I_1/I_0 (scipy.special):
    //   beta =  2 -> 0.697775
    //   beta = 10 -> 0.948600
    //
    // The estimate is the mean over the measurement sweeps, not the plaquette of the final
    // configuration. One configuration is a single sample of a distribution whose spread here is
    // about 0.02, far too wide to test a 0.5% agreement against.
    for (beta, epsilon, exact) in [(2.0_f64, 1.0_f64, 0.697_775_f64), (10.0, 0.5, 0.948_600)] {
        let lattice = Arc::new(LatticeComplex::new([4, 4], [true, true]));
        let mut rng = Xoshiro256::from_seed(7);
        let mut field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
            LatticeGaugeField::random(lattice, beta, &mut rng);

        for _ in 0..200 {
            field
                .try_metropolis_sweep(epsilon, &mut rng)
                .expect("a sweep over a populated lattice succeeds");
        }

        let measurements = 300;
        let mut acc = 0.0;
        for _ in 0..measurements {
            field
                .try_metropolis_sweep(epsilon, &mut rng)
                .expect("a sweep over a populated lattice succeeds");
            acc += field
                .try_average_plaquette()
                .expect("the thermalized field has a well-defined plaquette");
        }
        let mean = acc / f64::from(measurements);

        // Measured: 1.58% at beta = 2 and 0.47% at beta = 10, the residual being finite-volume
        // and the O(epsilon^2) projection of the proposal. Three percent leaves room for a
        // platform whose rounding sends the chain down a different trajectory, while still
        // separating these two couplings, whose exact values differ by 36%.
        let deviation = (mean - exact).abs() / exact;
        assert!(
            deviation < 0.03,
            "beta = {beta}: <P> = {mean}, exact I_1/I_0 = {exact}, off by {:.2}%",
            deviation * 100.0
        );
    }
}

#[test]
fn test_metropolis_sweep_is_reproducible_from_its_seed() {
    // Every update draws from the rng, so the sweep order decides how the stream is consumed.
    // Two fields built from the same seed must therefore agree exactly, down to the bit.
    //
    // A sweep order taken from a `HashMap` would not satisfy this: `RandomState` advances a
    // thread-local counter per map, so the two fields below would receive different hashers
    // inside this one process and walk their links in different orders.
    let run = || {
        let lattice = Arc::new(LatticeComplex::new([4, 4], [true, true]));
        let mut rng = Xoshiro256::from_seed(7);
        let mut field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
            LatticeGaugeField::random(lattice, 10.0, &mut rng);
        let mut rates = Vec::new();
        for _ in 0..40 {
            rates.push(
                field
                    .try_metropolis_sweep(0.5, &mut rng)
                    .expect("a sweep over a populated lattice succeeds"),
            );
        }
        (
            rates,
            field
                .try_average_plaquette()
                .expect("the field has a well-defined plaquette"),
        )
    };

    let (rates_a, plaquette_a) = run();
    let (rates_b, plaquette_b) = run();

    assert_eq!(
        rates_a, rates_b,
        "the per-sweep acceptance rates must not depend on the hasher"
    );
    assert_eq!(
        plaquette_a, plaquette_b,
        "the same seed must give the same field, bit for bit"
    );
}
