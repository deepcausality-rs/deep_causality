/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The forcing-region seam measured at M1 and the plume re-imprint channel M3 rides on it: the
//! unforced bit-identity guarantee, per-branch forcing, grid validation, and the refresh cap.

use super::{GAMMA_EFF, reference};
use deep_causality_cfd::{
    Ambient, BlackoutTrigger, CfdFlow, CompressibleMarchConfig, CompressibleMarchConfigBuilder,
    CoupledField, MarchStop, QttObserve,
};
use deep_causality_tensor::Truncation;

// ── The de-risk forcing seam (change plasma-retropulsion-de-risk) ───────────────────────────────

/// An unscheduled world (no descent, no inflow strip), optionally imprinting a forcing region.
fn plain_world(
    name: &str,
    steps: usize,
    forcing: Option<deep_causality_cfd::ForcingRegion<f64>>,
) -> CompressibleMarchConfig<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let mut builder = CompressibleMarchConfigBuilder::<f64>::new()
        .name(name)
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(steps))
        .observe(QttObserve::default())
        .reference(reference());
    if let Some(region) = forcing {
        builder = builder.forcing_region(region);
    }
    builder.build().unwrap()
}

/// A small forcing region on the 8×8 grid of `plain_world`.
fn small_region(target: [f64; 4], eta: f64) -> deep_causality_cfd::ForcingRegion<f64> {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let dx = 0.125;
    let mask =
        deep_causality_cfd::plume_mask_2d::<f64>(3, 3, dx, dx, 0.5, 0.5, 0.2, 0.15, dx, &trunc)
            .unwrap();
    deep_causality_cfd::ForcingRegion::new(mask, target, eta).unwrap()
}

#[test]
fn unforced_carrier_matches_the_bare_marcher_bit_for_bit() {
    // The no-forcing bit-identity guarantee: an unscheduled coupled run's final field must equal
    // the bare CompressibleMarcher2d's, component for component, bit for bit — the carrier adds
    // nothing to the march path when no forcing region is configured.
    use deep_causality_cfd::{
        CartesianIdentity, CompressibleMarcher2d, dequantize_2d, quantize_2d,
    };
    use deep_causality_tensor::CausalTensor;

    let steps = 3usize;
    let cfg = plain_world("bare", steps, None);
    let report = CfdFlow::march(&cfg)
        .run_coupled(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
        )
        .unwrap();
    let coupled_t_tr = report.final_field().unwrap().to_vec();

    // The same march, hand-driven on the bare marcher.
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let metric = CartesianIdentity::new(3, 3, 0.125, 0.125, trunc).unwrap();
    let marcher = CompressibleMarcher2d::new(metric, GAMMA_EFF, 0.002, 3.0, trunc).unwrap();
    let n = 8usize;
    // seed_fn(|_,_| (1, 1, 0, 1)) → conserved [ρ, ρu, ρv, E] with p = 1.
    let e0 = 1.0 / (GAMMA_EFF - 1.0) + 0.5;
    let enc = |v: f64| {
        quantize_2d(
            &CausalTensor::new(vec![v; n * n], vec![n, n]).unwrap(),
            &trunc,
        )
        .unwrap()
    };
    let mut state = [enc(1.0), enc(1.0), enc(0.0), enc(e0)];
    for _ in 0..steps {
        state = marcher.step(&state).unwrap();
    }
    // Replicate the carrier's final-field projection: T_tr = (p̂/ρ̂)·t_ref.
    let rho = dequantize_2d(&state[0], 3, 3).unwrap();
    let mx = dequantize_2d(&state[1], 3, 3).unwrap();
    let my = dequantize_2d(&state[2], 3, 3).unwrap();
    let e = dequantize_2d(&state[3], 3, 3).unwrap();
    let bare_t_tr: Vec<f64> = rho
        .as_slice()
        .iter()
        .zip(mx.as_slice())
        .zip(my.as_slice())
        .zip(e.as_slice())
        .map(|(((&r, &a), &b), &en)| {
            let u2 = (a * a + b * b) / (r * r);
            let p_hat = (GAMMA_EFF - 1.0) * (en - 0.5 * r * u2);
            let p_hat = if p_hat > 1.0e-12 { p_hat } else { 1.0e-12 };
            (p_hat / r) * reference().t_ref
        })
        .collect();

    assert_eq!(coupled_t_tr, bare_t_tr, "bit-identical unforced march");
}

#[test]
fn a_forcing_region_changes_the_marched_state() {
    // The same world with and without an imprint diverges — the forcing is live, not decorative.
    let unforced = plain_world("unforced", 3, None);
    let forced = plain_world(
        "forced",
        3,
        Some(small_region([0.5, -0.4, 0.0, 3.0], 0.002)),
    );
    let run = |cfg: &CompressibleMarchConfig<f64>| {
        CfdFlow::march(cfg)
            .run_coupled(
                (),
                CoupledField::new(Ambient::new(0.01, 0.0, None)),
                BlackoutTrigger::new(1.0e9),
                0.0,
            )
            .unwrap()
            .final_field()
            .unwrap()
            .to_vec()
    };
    let a = run(&unforced);
    let b = run(&forced);
    assert_ne!(a, b, "the imprint must change the marched state");
    assert!(b.iter().all(|x| x.is_finite()), "forced state stays finite");
}

#[test]
fn a_branch_world_carries_its_own_forcing_region() {
    // The fork-economics mechanism: continue the same pause into two branch worlds, one with an
    // imprint and one without — the branch rebuild picks up each world's own forcing region, so
    // a per-branch throttle intervention feeds back into that branch's own flow.
    let nominal = plain_world("nominal", 6, None);
    let coast = plain_world("coast_branch", 6, None);
    let burn = plain_world(
        "burn_branch",
        6,
        Some(small_region([0.5, -0.4, 0.0, 3.0], 0.002)),
    );

    let pause = CfdFlow::march(&nominal)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, s| s >= 2,
        )
        .unwrap();
    let fork = pause.fork();
    assert!(fork.shares_fluid_with(&pause), "O(1) fork before branching");

    let reports = pause.continue_branches(&[&coast, &burn], 3).unwrap();
    let coast_field = reports[0].final_field().unwrap().to_vec();
    let burn_field = reports[1].final_field().unwrap().to_vec();
    assert_ne!(
        coast_field, burn_field,
        "the branch's own imprint must diverge its flow from the coast branch"
    );
}

#[test]
fn a_forcing_mask_on_the_wrong_grid_is_rejected_at_build() {
    // A mask quantized for a different grid cannot silently ride into the march: the carrier
    // build rejects it before the first step.
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let dx = 1.0 / 16.0;
    // L = 4 mask (8 cores) against the L = 3 grid (6 cores).
    let wrong =
        deep_causality_cfd::plume_mask_2d::<f64>(4, 4, dx, dx, 0.5, 0.5, 0.2, 0.15, dx, &trunc)
            .unwrap();
    let region =
        deep_causality_cfd::ForcingRegion::new(wrong, [0.5, -0.4, 0.0, 3.0], 0.002).unwrap();
    let cfg = plain_world("wrong_grid", 2, Some(region));
    let err = CfdFlow::march(&cfg)
        .run_coupled(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
        )
        .expect_err("mismatched mask must be rejected");
    assert!(
        format!("{err:?}").contains("cores"),
        "names the core-count mismatch: {err:?}"
    );
}

#[test]
fn commanded_throttle_publishes_like_commanded_bank() {
    // The pinned counterfactual seam name for the retropulsion family: a branch world's throttle
    // intervention lands on the field each step through the same publish_constant mechanism.
    let trunc = Truncation::<f64>::by_bond(16).unwrap();
    let cfg = CompressibleMarchConfigBuilder::<f64>::new()
        .name("throttled")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 3.0, GAMMA_EFF, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 1.0, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(2))
        .observe(QttObserve::default())
        .reference(reference())
        .publish_constant("commanded_throttle", 0.6)
        .build()
        .unwrap();
    let pause = CfdFlow::march(&cfg)
        .run_until(
            (),
            CoupledField::new(Ambient::new(0.01, 0.0, None)),
            BlackoutTrigger::new(1.0e9),
            0.0,
            |_, _| false,
        )
        .unwrap();
    assert_eq!(pause.field().scalar("commanded_throttle"), Some(&[0.6][..]));
}
