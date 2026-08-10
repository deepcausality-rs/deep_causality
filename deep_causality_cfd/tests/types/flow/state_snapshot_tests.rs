/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The state snapshot: full-resume round trips through disk (bit-identical), the stale-world
//! refusal at the CFD seam, tier separation, and tensor-train field round trips.

use deep_causality_cfd::{
    Ambient, CoupledField, FiniteRateIonizationStage, InsErrorState, NavFilter, PhysicsStage,
    Quaternion, ReentryNavEngine, StepContext, load_resume_state, pack_resume, pack_tt_fields,
    quantize_2d, save_resume_state, unpack_resume, unpack_tt_fields,
};
use deep_causality_file::{
    BitCodec, SnapshotPackage, SnapshotSection, SnapshotTier, fingerprint64,
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
    let mut engine = ReentryNavEngine::new(
        [6.45e6, 0.0, 0.0],
        [-1_300.0, 7_860.0, 0.0],
        3.986e14,
        filter,
    );
    // Evolve the engine to a NON-identity attitude and nonzero clock/elapsed before snapshotting, so the
    // round-trip test bites on a pack/unpack ordering regression of the four attitude floats (an identity
    // quaternion (1,0,0,0) and zero clock would mask a w/x/y/z or tau/elapsed swap).
    let omega = [0.03, -0.02, 0.05];
    for _ in 0..8 {
        engine
            .predict(0.5, [0.1, -0.2, 0.3], omega, [1.0e-6; 17])
            .expect("predict");
    }
    let p = engine.position();
    engine
        .correct_position([p[0] + 3.0, p[1] - 1.0, p[2] + 2.0], 4.0)
        .expect("fix");
    field.set_nav(engine);
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

// ── The "nav" section's layout version ────────────────────────────────────────────────────────────

/// The 17 filter-state values a hand-built section carries: distinct and non-zero, so a reader that
/// mistakes four attitude floats for the head of the state vector shifts every one of them.
fn probe_state() -> [f64; 17] {
    core::array::from_fn(|i| 1.0 + (i as f64) * 0.25)
}

/// A resume package whose `"nav"` section is written at `version` in the **pre-attitude v1 layout**:
/// `position(3), velocity(3), gm, tau, elapsed, state(17), cov(17×17)`, with no quaternion. This is
/// what a build older than the attitude work wrote.
fn nav_v1_package(version: u8) -> SnapshotPackage {
    let mut nav = vec![1u8]; // presence flag
    for v in [6.45e6_f64, 1.0e5, -2.0e5] {
        v.write_bits(&mut nav);
    }
    for v in [-1_300.0_f64, 7_860.0, 12.5] {
        v.write_bits(&mut nav);
    }
    3.986e14_f64.write_bits(&mut nav); // gm
    (-4.5e-9_f64).write_bits(&mut nav); // carried clock offset
    17.5_f64.write_bits(&mut nav); // elapsed
    for v in probe_state() {
        v.write_bits(&mut nav);
    }
    for i in 0..17 {
        for j in 0..17 {
            let entry = if i == j { 2_500.0_f64 } else { 0.0 };
            entry.write_bits(&mut nav);
        }
    }

    let mut ambient = Vec::new();
    0.01_f64.write_bits(&mut ambient);
    0.0_f64.write_bits(&mut ambient);
    let mut scalars = Vec::new();
    scalars.extend_from_slice(&0u32.to_le_bytes());
    let mut log = Vec::new();
    log.extend_from_slice(&0u32.to_le_bytes());
    let mut step = Vec::new();
    step.extend_from_slice(&23u64.to_le_bytes());

    SnapshotPackage::new(
        f64::SCALAR_TAG,
        SnapshotTier::Resume,
        fingerprint64(WORLD),
        vec![
            SnapshotSection::new("scalars", 1, scalars),
            SnapshotSection::new("channels", 1, vec![0u8, 0, 0]),
            SnapshotSection::new("ambient", 1, ambient),
            SnapshotSection::new("nav", version, nav),
            SnapshotSection::new("log", 1, log),
            SnapshotSection::new("step", 1, step),
        ],
    )
}

#[test]
fn a_v1_nav_section_resumes_with_the_identity_attitude_and_unshifted_filter_data() {
    // The attitude work added four floats to the "nav" section. Read without the version byte, a v1
    // package feeds the first four filter-state entries to the quaternion and shifts everything after
    // them — the whole covariance lands one row late. The version tells the reader which layout it is
    // holding: no attitude floats to read, so the nominal resumes at identity, which is exactly the
    // `C ≈ I` attitude the writing build modelled, and every other value lands where it belongs.
    let (field, step) = unpack_resume::<f64>(&nav_v1_package(1)).expect("a v1 nav section resumes");
    let nav = field.nav().expect("nav restored");

    assert_eq!(step, 23);
    assert_eq!(nav.attitude(), Quaternion::<f64>::identity());
    assert_eq!(nav.position(), [6.45e6, 1.0e5, -2.0e5]);
    assert_eq!(nav.velocity(), [-1_300.0, 7_860.0, 12.5]);
    assert_eq!(nav.gm(), 3.986e14);
    assert_eq!(nav.carried_clock_offset(), -4.5e-9);
    assert_eq!(nav.elapsed_time(), 17.5);
    assert_eq!(
        nav.filter().state().to_array(),
        probe_state(),
        "the filter state is read unshifted, not four values late"
    );
    assert_eq!(nav.filter().covariance()[0][0], 2_500.0);
    assert_eq!(nav.filter().covariance()[16][16], 2_500.0);
}

#[test]
fn a_nav_section_from_an_unknown_future_layout_is_refused() {
    // The version byte has to refuse forward as well as decode backward: a layout this build does not
    // know is a hard error naming the section, never a best-effort parse of whatever bytes are there.
    let err =
        unpack_resume::<f64>(&nav_v1_package(7)).expect_err("an unknown nav layout is refused");
    let text = err.to_string();
    assert!(text.contains("nav"), "{text}");
    assert!(text.contains("unsupported layout version"), "{text}");
}

#[test]
fn a_packed_nav_section_declares_the_attitude_layout_version() {
    // What this build writes is v2 — the layout that carries the four attitude floats. A build that
    // predates them rejects v2 on the version check instead of misreading the quaternion as filter
    // state, which is the half of the compatibility problem an unversioned bump cannot fix in the past.
    let package = pack_resume(&populated_field(), 5, WORLD).expect("packs");
    let nav = package.section("nav").expect("nav section");
    assert_eq!(nav.version(), 2, "the attitude-carrying nav layout is v2");
    // The sections that did not change keep their own version: the bump is per section, not global.
    for name in ["scalars", "channels", "ambient", "log", "step"] {
        assert_eq!(
            package.section(name).expect(name).version(),
            1,
            "section '{name}' is unchanged at v1"
        );
    }
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
