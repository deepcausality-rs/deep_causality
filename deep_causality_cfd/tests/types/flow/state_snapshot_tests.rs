/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The state snapshot: full-resume round trips through disk (bit-identical), the stale-world
//! refusal at the CFD seam, tier separation, and tensor-train field round trips.

use deep_causality_cfd::{
    Ambient, CoupledField, FiniteRateIonizationStage, InsErrorState, NavFilter, PhysicsStage,
    ReentryNavEngine, StepContext, load_resume_state, pack_resume, pack_tt_fields, quantize_2d,
    save_resume_state, unpack_resume, unpack_tt_fields,
};
use deep_causality_haft::LogAddEntry;
use deep_causality_tensor::{CausalTensor, Truncation};

const WORLD: &[u8] = b"snapshot-test-world-v1";

fn populated_field() -> CoupledField<f64> {
    let mut field = CoupledField::new(Ambient::new(0.01_f64, 0.0, None));
    field.set_scalar("T_tr", vec![8_044.0, 6_500.0, 300.0]);
    field.set_scalar("T_ve", vec![4_000.0, 3_200.0, 300.0]);
    field.set_scalar("alpha", vec![1.0e-4, 2.0e-5, 0.0]);
    field.set_aero_force([0.1, -0.2, 0.3]);
    field.set_control_action(0.25);
    field.set_throttle_action(0.6);
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [2_500.0; 17]).unwrap();
    field.set_nav(ReentryNavEngine::new(
        [6.45e6, 0.0, 0.0],
        [-1_300.0, 7_860.0, 0.0],
        3.986e14,
        filter,
    ));
    field.log_mut().add_entry("regime -> slip (test)");
    field.log_mut().add_entry("nav: aided (test)");
    field
}

#[test]
fn a_resume_package_round_trips_through_disk_bit_exact() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("suspended.dcsnap");
    let field = populated_field();

    save_resume_state(&path, &field, 119, WORLD).expect("saves");
    let (restored, step) = load_resume_state::<f64>(&path, WORLD).expect("loads");

    assert_eq!(step, 119);
    assert_eq!(restored.scalars().len(), field.scalars().len());
    for ((name_a, a), (name_b, b)) in field.scalars().iter().zip(restored.scalars()) {
        assert_eq!(name_a, name_b);
        for (x, y) in a.iter().zip(b) {
            assert_eq!(x.to_bits(), y.to_bits(), "scalar '{name_a}' bit-identical");
        }
    }
    assert_eq!(restored.aero_force(), field.aero_force());
    assert_eq!(restored.control_action(), field.control_action());
    assert_eq!(restored.throttle_action(), field.throttle_action());

    let nav_a = field.nav().expect("nav");
    let nav_b = restored.nav().expect("nav restored");
    assert_eq!(nav_a.position(), nav_b.position());
    assert_eq!(nav_a.velocity(), nav_b.velocity());
    assert_eq!(nav_a.gm(), nav_b.gm());
    // The KS clock fields are serialized by `pack_resume` and restored by `ReentryNavEngine::
    // restore`; assert them so a regression that drops either is caught (a future non-zero nav
    // state would then fail here rather than pass silently).
    assert_eq!(nav_a.carried_clock_offset(), nav_b.carried_clock_offset());
    assert_eq!(nav_a.elapsed_time(), nav_b.elapsed_time());
    // The nominal attitude round-trips too (serialized by `pack_resume`, restored by `restore`).
    assert_eq!(nav_a.attitude(), nav_b.attitude());
    assert_eq!(
        nav_a.filter().state().to_array(),
        nav_b.filter().state().to_array()
    );
    assert_eq!(nav_a.filter().covariance(), nav_b.filter().covariance());

    // Log value equality ignores timestamps by EffectLog's own contract; the restored log
    // continues appending after the recorded entries.
    assert_eq!(restored.log(), field.log());
}

#[test]
fn a_resumed_state_steps_bit_identically_to_the_unsuspended_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("mid_march.dcsnap");

    let mut original = populated_field();
    save_resume_state(&path, &original, 7, WORLD).expect("saves");
    let (mut resumed, _step) = load_resume_state::<f64>(&path, WORLD).expect("loads");

    // The same physics stage applied to both must produce the same bits: suspend/resume is
    // invisible to the marched numbers.
    let stage = FiniteRateIonizationStage::new(2.645e22_f64).with_sheath_renewal(8.35e-5);
    let ctx = StepContext::<2, f64>::qtt(2.0e-5, 8);
    stage.apply(&ctx, &mut original).expect("original steps");
    stage.apply(&ctx, &mut resumed).expect("resumed steps");

    let ne_a = original.scalar("n_e").expect("n_e");
    let ne_b = resumed.scalar("n_e").expect("n_e");
    for (x, y) in ne_a.iter().zip(ne_b) {
        assert_eq!(
            x.to_bits(),
            y.to_bits(),
            "post-resume step is bit-identical"
        );
    }
}

#[test]
fn a_stale_world_fingerprint_refuses_at_the_seam() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("stale.dcsnap");
    let field = populated_field();
    save_resume_state(&path, &field, 1, WORLD).expect("saves");

    let err = load_resume_state::<f64>(&path, b"edited-constants-v2").expect_err("stale world");
    assert!(err.to_string().contains("different world"), "{err}");
}

#[test]
fn the_tiers_do_not_cross() {
    let field = populated_field();
    let resume = pack_resume(&field, 3, WORLD).expect("packs");

    let trunc = Truncation::<f64>::by_tol(1e-12).expect("truncation");
    let dense = CausalTensor::new(vec![1.0_f64; 16], vec![4, 4]).expect("tensor");
    let tt = quantize_2d(&dense, &trunc).expect("quantizes");
    let field_tier = pack_tt_fields(&[("rho".to_string(), tt)], (4, 4), WORLD);

    assert!(
        unpack_resume::<f64>(&field_tier).is_err(),
        "field tier cannot resume"
    );
    assert!(
        unpack_tt_fields::<f64>(&resume).is_err(),
        "resume tier is not a field snapshot"
    );
}

#[test]
fn tt_fields_round_trip_with_bit_exact_cores() {
    let trunc = Truncation::<f64>::by_tol(1e-12).expect("truncation");
    // A smooth ramp: compresses to low rank, decompresses exactly enough to compare cores.
    let data: Vec<f64> = (0..64).map(|i| 1.0 + (i as f64) * 0.125).collect();
    let dense = CausalTensor::new(data, vec![8, 8]).expect("tensor");
    let tt = quantize_2d(&dense, &trunc).expect("quantizes");

    let package = pack_tt_fields(&[("T_tr".to_string(), tt.clone())], (8, 8), WORLD);
    let (fields, grid) = unpack_tt_fields::<f64>(&package).expect("unpacks");

    assert_eq!(grid, (8, 8));
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "T_tr");
    let restored = &fields[0].1;
    assert_eq!(restored.cores().len(), tt.cores().len());
    for (a, b) in tt.cores().iter().zip(restored.cores()) {
        assert_eq!(a.shape(), b.shape());
        for (x, y) in a.data().iter().zip(b.data()) {
            assert_eq!(x.to_bits(), y.to_bits(), "core values bit-identical");
        }
    }
}
