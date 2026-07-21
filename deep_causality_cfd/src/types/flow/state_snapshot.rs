/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Snapshot packing and resume for a running CFD state.
//!
//! `deep_causality_file` owns the container (header, sections, checksum, refusal rules); this
//! module owns what goes into the sections, in two tiers:
//!
//! * **Field snapshot** ([`pack_tt_fields`] / [`unpack_tt_fields`]): named tensor-train fields
//!   with their grid shape. The area-of-interest artifact and the golden-baseline payload.
//! * **Full resume** ([`pack_resume`] / [`unpack_resume`]): the whole [`CoupledField`] and the
//!   step index, so one workflow suspends with a line and a different workflow continues days
//!   later. Serialized: every carried scalar field, the aero-force and control channels, the
//!   ambient, the navigation engine (nominal state, filter vector, full covariance, carried
//!   clock offset, elapsed time), and the provenance log's message sequence. The classified
//!   regime is deliberately not serialized: it is re-decided from the evolved state every
//!   coupled step. Log entries are re-stamped on restore; `EffectLog` value equality ignores
//!   timestamps by its own contract.
//!
//! Values are stored as raw bit patterns via the file crate's [`BitCodec`], so a resumed state
//! is bit-identical to the suspended one. The world fingerprint is caller-supplied bytes (an
//! example hashes its constants); the strict loader refuses a package whose fingerprint does
//! not match the current world.

use crate::CfdScalar;
use crate::navigation::{NAV_STATES, NavFilter, ReentryNavEngine};
use crate::types::Ambient;
use crate::types::flow::coupling::CoupledField;
use deep_causality_core::EffectLog;
use deep_causality_file::{
    BitCodec, SnapshotPackage, SnapshotSection, SnapshotTier, fingerprint64,
};
use deep_causality_haft::{IoAction, LogAddEntry};
use deep_causality_num::FromPrimitive;
use deep_causality_physics::PhysicsError;
use deep_causality_tensor::{CausalTensor, CausalTensorTrain};

/// Section layout version written by this build (bumped per section, not per container).
const SECTION_V1: u8 = 1;

/// Named tensor-train fields, as a field-tier snapshot stores them.
pub type NamedTtFields<R> = Vec<(String, CausalTensorTrain<R>)>;

// ── byte helpers ─────────────────────────────────────────────────────────────

fn write_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_str(out: &mut Vec<u8>, s: &str) {
    write_u32(out, u32::try_from(s.len()).unwrap_or(u32::MAX));
    out.extend_from_slice(s.as_bytes());
}

fn short(section: &str) -> PhysicsError {
    PhysicsError::CalculationError(format!("snapshot section '{section}' is truncated"))
}

/// Read a presence flag written as exactly `0` (absent) or `1` (present). Any other byte is a
/// corrupt section, refused loudly rather than silently treated as absent.
fn presence_flag(byte: u8, section: &str) -> Result<bool, PhysicsError> {
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(PhysicsError::CalculationError(format!(
            "snapshot section '{section}': invalid presence flag {other} (expected 0 or 1)"
        ))),
    }
}

fn read_u32(bytes: &[u8], o: &mut usize, section: &str) -> Result<u32, PhysicsError> {
    let s = bytes.get(*o..*o + 4).ok_or_else(|| short(section))?;
    *o += 4;
    Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn read_u64(bytes: &[u8], o: &mut usize, section: &str) -> Result<u64, PhysicsError> {
    let s = bytes.get(*o..*o + 8).ok_or_else(|| short(section))?;
    *o += 8;
    Ok(u64::from_le_bytes([
        s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
    ]))
}

fn read_str(bytes: &[u8], o: &mut usize, section: &str) -> Result<String, PhysicsError> {
    let len = read_u32(bytes, o, section)? as usize;
    let s = bytes.get(*o..*o + len).ok_or_else(|| short(section))?;
    *o += len;
    String::from_utf8(s.to_vec()).map_err(|_| {
        PhysicsError::CalculationError(format!("snapshot section '{section}': non-UTF-8 name"))
    })
}

fn write_value<R: BitCodec>(out: &mut Vec<u8>, v: &R) {
    v.write_bits(out);
}

fn read_value<R: BitCodec>(bytes: &[u8], o: &mut usize, section: &str) -> Result<R, PhysicsError> {
    R::read_bits(bytes, o).ok_or_else(|| short(section))
}

fn read_values<R: BitCodec>(
    bytes: &[u8],
    o: &mut usize,
    n: usize,
    section: &str,
) -> Result<Vec<R>, PhysicsError> {
    // Guard the preallocation against a malformed count read from the file: every encoded value
    // occupies at least one byte, so a count exceeding the section's remaining bytes cannot be
    // satisfied and would otherwise demand an unbounded allocation before any byte is read.
    let remaining = bytes.len().saturating_sub(*o);
    if n > remaining {
        return Err(short(section));
    }
    let mut out = Vec::new();
    out.try_reserve(n).map_err(|_| {
        PhysicsError::CalculationError(format!(
            "snapshot section '{section}': value count {n} exceeds available memory"
        ))
    })?;
    for _ in 0..n {
        out.push(read_value::<R>(bytes, o, section)?);
    }
    Ok(out)
}

// ── the full resume tier ─────────────────────────────────────────────────────

/// Pack a coupled field and its step index into a full resume package.
///
/// # Errors
/// A body-forced ambient is refused (section v1 does not serialize body forces), and a field
/// carrying no state at all still packs (empty sections are valid).
pub fn pack_resume<R>(
    field: &CoupledField<R>,
    step: usize,
    world_fingerprint: &[u8],
) -> Result<SnapshotPackage, PhysicsError>
where
    R: CfdScalar + BitCodec,
{
    if field.ambient().body_force().is_some() {
        return Err(PhysicsError::CalculationError(
            "snapshot: a body-forced ambient is not serializable (section v1)".into(),
        ));
    }

    // "scalars": every named scalar field, in insertion order.
    let mut scalars = Vec::new();
    write_u32(
        &mut scalars,
        u32::try_from(field.scalars().len()).unwrap_or(u32::MAX),
    );
    for (name, values) in field.scalars() {
        write_str(&mut scalars, name);
        write_u64(&mut scalars, values.len() as u64);
        for v in values {
            write_value(&mut scalars, v);
        }
    }

    // "channels": the aero-force and control channels (present flags + values).
    let mut channels = Vec::new();
    match field.aero_force() {
        Some(f) => {
            channels.push(1);
            for v in &f {
                write_value(&mut channels, v);
            }
        }
        None => channels.push(0),
    }
    match field.control_action() {
        Some(a) => {
            channels.push(1);
            write_value(&mut channels, &a);
        }
        None => channels.push(0),
    }
    match field.throttle_action() {
        Some(a) => {
            channels.push(1);
            write_value(&mut channels, &a);
        }
        None => channels.push(0),
    }

    // "ambient": kinematic viscosity and freestream speed.
    let mut ambient = Vec::new();
    write_value(&mut ambient, field.ambient().nu());
    write_value(&mut ambient, field.ambient().freestream());

    // "nav": the full navigation engine, when threaded.
    let mut nav = Vec::new();
    match field.nav() {
        Some(engine) => {
            nav.push(1);
            for v in &engine.position() {
                write_value(&mut nav, v);
            }
            for v in &engine.velocity() {
                write_value(&mut nav, v);
            }
            write_value(&mut nav, &engine.gm());
            write_value(&mut nav, &engine.carried_clock_offset());
            write_value(&mut nav, &engine.elapsed_time());
            for v in &engine.filter().state().to_array() {
                write_value(&mut nav, v);
            }
            for row in engine.filter().covariance() {
                for v in row {
                    write_value(&mut nav, v);
                }
            }
        }
        None => nav.push(0),
    }

    // "log": the provenance message sequence.
    let mut log = Vec::new();
    let messages: Vec<&str> = field.log().messages().collect();
    write_u32(&mut log, u32::try_from(messages.len()).unwrap_or(u32::MAX));
    for m in messages {
        write_str(&mut log, m);
    }

    // "step": the step index the state was suspended at.
    let mut step_bytes = Vec::new();
    write_u64(&mut step_bytes, step as u64);

    Ok(SnapshotPackage::new(
        R::SCALAR_TAG,
        SnapshotTier::Resume,
        fingerprint64(world_fingerprint),
        vec![
            SnapshotSection::new("scalars", SECTION_V1, scalars),
            SnapshotSection::new("channels", SECTION_V1, channels),
            SnapshotSection::new("ambient", SECTION_V1, ambient),
            SnapshotSection::new("nav", SECTION_V1, nav),
            SnapshotSection::new("log", SECTION_V1, log),
            SnapshotSection::new("step", SECTION_V1, step_bytes),
        ],
    ))
}

fn section<'a>(package: &'a SnapshotPackage, name: &str) -> Result<&'a [u8], PhysicsError> {
    let s = package.section(name).ok_or_else(|| {
        PhysicsError::CalculationError(format!("snapshot: missing section '{name}'"))
    })?;
    if s.version() != SECTION_V1 {
        return Err(PhysicsError::CalculationError(format!(
            "snapshot: section '{name}' has unsupported layout version {}",
            s.version()
        )));
    }
    Ok(s.bytes())
}

/// Unpack a full resume package into the coupled field and the suspended step index.
///
/// # Errors
/// A field-tier package is refused (it has no state passengers to resume from); truncated or
/// missing sections surface as calculation errors naming the section.
pub fn unpack_resume<R>(package: &SnapshotPackage) -> Result<(CoupledField<R>, usize), PhysicsError>
where
    R: CfdScalar + BitCodec + FromPrimitive,
{
    if package.tier() != SnapshotTier::Resume {
        return Err(PhysicsError::CalculationError(
            "snapshot: a field-tier package cannot resume a run; save a full resume package".into(),
        ));
    }

    // "ambient" first: the field is constructed around it.
    let bytes = section(package, "ambient")?;
    let mut o = 0;
    let nu: R = read_value(bytes, &mut o, "ambient")?;
    let freestream: R = read_value(bytes, &mut o, "ambient")?;
    let mut field = CoupledField::new(Ambient::new(nu, freestream, None));

    // "scalars".
    let bytes = section(package, "scalars")?;
    let mut o = 0;
    let count = read_u32(bytes, &mut o, "scalars")?;
    for _ in 0..count {
        let name = read_str(bytes, &mut o, "scalars")?;
        let len = read_u64(bytes, &mut o, "scalars")? as usize;
        let values = read_values::<R>(bytes, &mut o, len, "scalars")?;
        field.set_scalar(name, values);
    }

    // "channels".
    let bytes = section(package, "channels")?;
    let mut o = 0;
    let aero_present = presence_flag(*bytes.first().ok_or_else(|| short("channels"))?, "channels")?;
    o += 1;
    if aero_present {
        let f: Vec<R> = read_values(bytes, &mut o, 3, "channels")?;
        field.set_aero_force([f[0], f[1], f[2]]);
    }
    let control_present =
        presence_flag(*bytes.get(o).ok_or_else(|| short("channels"))?, "channels")?;
    o += 1;
    if control_present {
        let a: R = read_value(bytes, &mut o, "channels")?;
        field.set_control_action(a);
    }
    // Throttle channel — the second command axis, packed after the bank channel.
    let throttle_present =
        presence_flag(*bytes.get(o).ok_or_else(|| short("channels"))?, "channels")?;
    o += 1;
    if throttle_present {
        let a: R = read_value(bytes, &mut o, "channels")?;
        field.set_throttle_action(a);
    }

    // "nav".
    let bytes = section(package, "nav")?;
    let mut o = 0;
    let nav_present = presence_flag(*bytes.first().ok_or_else(|| short("nav"))?, "nav")?;
    o += 1;
    if nav_present {
        let p: Vec<R> = read_values(bytes, &mut o, 3, "nav")?;
        let v: Vec<R> = read_values(bytes, &mut o, 3, "nav")?;
        let gm: R = read_value(bytes, &mut o, "nav")?;
        let tau: R = read_value(bytes, &mut o, "nav")?;
        let elapsed: R = read_value(bytes, &mut o, "nav")?;
        let state_vec: Vec<R> = read_values(bytes, &mut o, NAV_STATES, "nav")?;
        let mut state = [R::zero(); NAV_STATES];
        state.copy_from_slice(&state_vec);
        let mut cov = [[R::zero(); NAV_STATES]; NAV_STATES];
        for row in cov.iter_mut() {
            let r: Vec<R> = read_values(bytes, &mut o, NAV_STATES, "nav")?;
            row.copy_from_slice(&r);
        }
        let filter = NavFilter::restore(crate::navigation::InsErrorState::from_array(state), cov);
        field.set_nav(ReentryNavEngine::restore(
            [p[0], p[1], p[2]],
            [v[0], v[1], v[2]],
            gm,
            filter,
            tau,
            elapsed,
        ));
    }

    // "log": re-log the message sequence (entries re-stamp; value equality ignores timestamps).
    let bytes = section(package, "log")?;
    let mut o = 0;
    let count = read_u32(bytes, &mut o, "log")?;
    let mut log = EffectLog::new();
    for _ in 0..count {
        let message = read_str(bytes, &mut o, "log")?;
        log.add_entry(&message);
    }
    *field.log_mut() = log;

    // "step".
    let bytes = section(package, "step")?;
    let mut o = 0;
    let step_u64 = read_u64(bytes, &mut o, "step")?;
    // Checked conversion: a `u64` step index that does not fit this platform's `usize` (e.g. on
    // a 32-bit target) is refused rather than silently truncated to a wrong resume point.
    let step = usize::try_from(step_u64).map_err(|_| {
        PhysicsError::CalculationError(format!(
            "snapshot section 'step': step index {step_u64} exceeds this platform's usize"
        ))
    })?;

    Ok((field, step))
}

// ── the field-snapshot tier ──────────────────────────────────────────────────

/// Pack named tensor-train fields and their grid shape into a field-tier package.
pub fn pack_tt_fields<R>(
    fields: &[(String, CausalTensorTrain<R>)],
    grid: (usize, usize),
    world_fingerprint: &[u8],
) -> SnapshotPackage
where
    R: CfdScalar + BitCodec,
{
    let mut tt = Vec::new();
    write_u64(&mut tt, grid.0 as u64);
    write_u64(&mut tt, grid.1 as u64);
    write_u32(&mut tt, u32::try_from(fields.len()).unwrap_or(u32::MAX));
    for (name, train) in fields {
        write_str(&mut tt, name);
        write_u32(
            &mut tt,
            u32::try_from(train.cores().len()).unwrap_or(u32::MAX),
        );
        for core in train.cores() {
            let shape = core.shape();
            write_u32(&mut tt, u32::try_from(shape.len()).unwrap_or(u32::MAX));
            for d in shape {
                write_u64(&mut tt, *d as u64);
            }
            write_u64(&mut tt, core.data().len() as u64);
            for v in core.data() {
                write_value(&mut tt, v);
            }
        }
    }

    SnapshotPackage::new(
        R::SCALAR_TAG,
        SnapshotTier::Field,
        fingerprint64(world_fingerprint),
        vec![SnapshotSection::new("tt_fields", SECTION_V1, tt)],
    )
}

/// Unpack a field-tier package into its named tensor trains and grid shape.
///
/// # Errors
/// A resume-tier package is refused (use [`unpack_resume`]); malformed cores surface as
/// calculation errors.
pub fn unpack_tt_fields<R>(
    package: &SnapshotPackage,
) -> Result<(NamedTtFields<R>, (usize, usize)), PhysicsError>
where
    R: CfdScalar + BitCodec,
{
    if package.tier() != SnapshotTier::Field {
        return Err(PhysicsError::CalculationError(
            "snapshot: a resume-tier package holds a full state; use unpack_resume".into(),
        ));
    }
    let bytes = section(package, "tt_fields")?;
    let mut o = 0;
    let nx = read_u64(bytes, &mut o, "tt_fields")? as usize;
    let ny = read_u64(bytes, &mut o, "tt_fields")? as usize;
    let count = read_u32(bytes, &mut o, "tt_fields")?;
    let mut fields = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name = read_str(bytes, &mut o, "tt_fields")?;
        let core_count = read_u32(bytes, &mut o, "tt_fields")?;
        let mut cores = Vec::with_capacity(core_count as usize);
        for _ in 0..core_count {
            let ndims = read_u32(bytes, &mut o, "tt_fields")?;
            let mut shape = Vec::with_capacity(ndims as usize);
            for _ in 0..ndims {
                shape.push(read_u64(bytes, &mut o, "tt_fields")? as usize);
            }
            let len = read_u64(bytes, &mut o, "tt_fields")? as usize;
            let data = read_values::<R>(bytes, &mut o, len, "tt_fields")?;
            let core = CausalTensor::new(data, shape).map_err(|e| {
                PhysicsError::CalculationError(format!("snapshot: invalid core: {e}"))
            })?;
            cores.push(core);
        }
        let train = CausalTensorTrain::from_cores(cores).map_err(|e| {
            PhysicsError::CalculationError(format!("snapshot: invalid tensor train: {e}"))
        })?;
        fields.push((name, train));
    }
    Ok((fields, (nx, ny)))
}

// ── the one-line seams ───────────────────────────────────────────────────────

/// Save a full resume package to `path` in one call: pack the field and step, checksum, write.
///
/// # Errors
/// Packing failures surface as physics errors; file failures are wrapped with the path named.
pub fn save_resume_state<R>(
    path: impl AsRef<std::path::Path>,
    field: &CoupledField<R>,
    step: usize,
    world_fingerprint: &[u8],
) -> Result<(), PhysicsError>
where
    R: CfdScalar + BitCodec,
{
    let package = pack_resume(field, step, world_fingerprint)?;
    deep_causality_file::save_snapshot(path.as_ref(), package)
        .run()
        .map_err(|e| PhysicsError::CalculationError(format!("snapshot save failed: {e}")))
}

/// Load and strictly verify a resume package from `path` in one call: checksum, scalar, and
/// world fingerprint are all validated before the state is rebuilt. This is the entry point a
/// *different* workflow uses days later; the returned field goes wherever an initial field
/// would.
///
/// # Errors
/// Corrupt files, scalar mismatches, and stale world fingerprints refuse with the file named
/// (the file crate's `force_load_snapshot` exists for salvage, at the caller's explicit risk).
pub fn load_resume_state<R>(
    path: impl AsRef<std::path::Path>,
    world_fingerprint: &[u8],
) -> Result<(CoupledField<R>, usize), PhysicsError>
where
    R: CfdScalar + BitCodec + FromPrimitive,
{
    let package = deep_causality_file::load_snapshot(
        path.as_ref(),
        R::SCALAR_TAG,
        Some(fingerprint64(world_fingerprint)),
    )
    .run()
    .map_err(|e| PhysicsError::CalculationError(format!("snapshot load failed: {e}")))?;
    unpack_resume(&package)
}
