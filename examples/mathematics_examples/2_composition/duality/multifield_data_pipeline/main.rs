/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Multifield <-> Tensor Iso: Building Data Pipelines Across an Encapsulated Type
//!
//! `CausalMultiField<T>` keeps its fields `pub(crate)`. From outside the
//! `deep_causality_multivector` crate that means:
//!
//! - You cannot construct a `CausalMultiField` from precomputed tensor data
//!   (no public constructor takes the four-tuple — the only constructors
//!   that do exist (`zeros`, `ones`) build the tensor from scratch).
//! - You cannot extract the underlying tensor to hand it to a generic
//!   tensor-aware function.
//! - You cannot write a generic "map this field's underlying tensor"
//!   helper outside the crate.
//!
//! Without the iso, the only escape hatch would be to add ad-hoc
//! constructors and accessors to the multivector crate every time a new
//! consumer shows up — which is exactly the encapsulation-break the
//! `pub(crate)` was designed to prevent.
//!
//! With the iso, the door opens by *exactly one* typed bridge:
//!
//! ```ignore
//! let field: CausalMultiField<F> = (tensor, metric, dx, shape).into();
//! let (tensor, metric, dx, shape): MultiFieldCarrier<F> = field.into();
//! ```
//!
//! Anything you can build on these two `From` impls (a data-loading
//! pipeline, an external transform, a serialization layer) becomes
//! expressible *without* modifying the multivector crate.
//!
//! ## What this example does
//!
//! Three stages of a realistic external pipeline that round-trip data
//! through `CausalMultiField` without breaking encapsulation:
//!
//! 1. **Load**: simulate "data arrives from disk" as a `Vec<f32>`. Build
//!    a `CausalMultiField` from a precomputed `CausalTensor` plus the
//!    grid metadata — using the reverse iso.
//! 2. **Transform**: apply a generic tensor-level operation to the
//!    underlying carrier without writing a multifield-specific wrapper.
//!    This uses both directions of the iso (unpack, transform, repack).
//! 3. **Export**: extract the final tensor for downstream consumers
//!    (e.g. visualization, archival) — using the forward iso.
//!
//! Every stage prints what it did so the reader can see the data flow.

use deep_causality_algebra::Real;
use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, MultiFieldCarrier};
use deep_causality_num::{Lift, lift, lower};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Pipeline
// =============================================================================

/// can be f32, f64, or f106;
/// The working scalar. Every element of the underlying tensor carries it.
pub type FloatType = f32;

fn main() {
    print_header();

    // Grid parameters: 2x2x2 spatial grid, Cl(2,0) metric (matrix_dim = 4),
    // so the underlying tensor shape is [2, 2, 2, 4, 4] = 128 elements.
    let grid_shape: [usize; 3] = [2, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let d = lift::<FloatType>(0.1);
    let dx: [FloatType; 3] = [d, d, d];

    // We can't pre-compute `matrix_dim` from outside the crate (the helper
    // is pub(crate)). So we materialize a known-good multifield via the
    // existing `zeros` constructor, then USE THE FORWARD ISO to learn the
    // shape — and then immediately use the REVERSE ISO to reassemble a
    // new multifield from a precomputed tensor of that shape. This is the
    // shape any external loader would have to discover before reading a
    // file.
    let probe = CausalMultiField::<FloatType>::zeros(grid_shape, metric, dx);
    let (probe_tensor, _, _, _): MultiFieldCarrier<FloatType> = probe.into();
    let underlying_shape: Vec<usize> = probe_tensor.shape().to_vec();
    print_shape(&underlying_shape, grid_shape);

    // ---------------------------------------------------------------------
    // Stage 1: LOAD — simulate data arriving from an external source.
    // The pipeline owns the Vec<f32> and the metadata. It does NOT have
    // multivector-internal access.
    // ---------------------------------------------------------------------
    let element_count: usize = underlying_shape.iter().product();
    // Mock "loaded" data: a ramp from 0.0 to 1.0 across the tensor's flat
    // index space. Stands in for whatever a real loader would return.
    let raw: Vec<FloatType> = (0..element_count)
        .map(|i| i.lift::<FloatType>() / element_count.lift::<FloatType>())
        .collect();
    let tensor_in = CausalTensor::new(raw, underlying_shape.clone())
        .expect("the element count matches the discovered shape");
    print_load_sample(element_count, &tensor_in.data()[0..4]);

    let field = load_multifield(tensor_in, metric, dx, grid_shape);
    print_assembled(field.metric(), field.dx(), field.shape());

    // ---------------------------------------------------------------------
    // Stage 2: TRANSFORM — apply a generic tensor-level operation through
    // the iso, preserving the multifield's metadata.
    // ---------------------------------------------------------------------
    // Element-wise scaling. Any tensor function with this signature plugs
    // straight in — that's the value of the generic helper.
    let scaled = map_underlying_tensor(field, |t| lift::<FloatType>(2.0) * &t);
    print_transform_scaled(scaled.metric(), scaled.dx(), scaled.shape());

    // Chain a second transform. Same generic helper.
    let shifted = map_underlying_tensor(scaled, |t| lift::<FloatType>(0.5) + &t);
    print_transform_shifted(shifted.metric(), shifted.dx(), shifted.shape());

    // ---------------------------------------------------------------------
    // Stage 3: EXPORT — extract the final tensor for a downstream
    // consumer (visualization, archival, serialization, etc).
    // ---------------------------------------------------------------------
    let (final_tensor, out_metric, out_dx, out_shape) = export_multifield(shifted);
    print_export(
        final_tensor.shape(),
        &final_tensor.data()[0..4],
        out_metric,
        &out_dx,
        &out_shape,
    );

    // ---------------------------------------------------------------------
    // Sanity checks: data went through the pipeline correctly.
    // ---------------------------------------------------------------------
    print_checks_header();

    // First element: input was 0.0, scaled by 2.0 → 0.0, plus 0.5 → 0.5.
    let first = final_tensor.data()[0];
    let expected_first: FloatType =
        lift::<FloatType>(0.0) * lift::<FloatType>(2.0) + lift::<FloatType>(0.5);
    // The tolerance is a multiple of the working type's epsilon, so it moves with the alias. A
    // literal threshold is correct at one precision and arbitrary at the other three.
    let tol: FloatType = lift::<FloatType>(8.0) * <FloatType as Real>::epsilon();
    assert!(
        Real::abs(first - expected_first) < tol,
        "first element drifted: {} vs {}",
        lower(first),
        lower(expected_first)
    );
    print_check("first element OK: ", first, expected_first);

    // Last element: input was (n-1)/n ≈ 0.992..., scaled by 2.0 ≈ 1.984...,
    // plus 0.5 ≈ 2.484...
    let last = final_tensor.data()[element_count - 1];
    let n = element_count.lift::<FloatType>();
    let expected_last: FloatType =
        ((n - lift::<FloatType>(1.0)) / n) * lift::<FloatType>(2.0) + lift::<FloatType>(0.5);
    // Wider than the first check: this value went through a division as well as the scale
    // and the shift.
    let tol_last: FloatType = lift::<FloatType>(96.0) * <FloatType as Real>::epsilon();
    assert!(
        Real::abs(last - expected_last) < tol_last,
        "last element drifted: {} vs {}",
        lower(last),
        lower(expected_last)
    );
    print_check("last element OK:  ", last, expected_last);

    // Metadata preserved through the pipeline.
    assert_eq!(out_metric, metric, "metric was not preserved");
    assert_eq!(out_dx, dx, "grid spacing was not preserved");
    assert_eq!(out_shape, grid_shape, "grid shape was not preserved");
    print_metadata_ok();

    print_why_it_matters();
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Multifield Data-Pipeline Iso Showcase ===\n");
}

fn print_shape(underlying: &[usize], grid: [usize; 3]) {
    println!("Discovered underlying tensor shape: {underlying:?}");
    println!(
        "(grid {:?} + metric.dim()={} → matrix slot {}×{})",
        grid, 2, underlying[3], underlying[4]
    );
}

fn print_load_sample(element_count: usize, sample: &[FloatType]) {
    println!("\n--- Stage 1: LOAD (build CausalMultiField from external tensor) ---");
    println!(
        "  loaded {element_count} elements as CausalTensor (sample: tensor.data()[0..4] = {sample:?})"
    );
}

fn print_assembled(metric: Metric, dx: &[FloatType; 3], shape: &[usize; 3]) {
    println!("  assembled CausalMultiField: metric={metric:?}, dx={dx:?}, shape={shape:?}");
}

fn print_transform_scaled(metric: Metric, dx: &[FloatType; 3], shape: &[usize; 3]) {
    println!("\n--- Stage 2: TRANSFORM (apply tensor-level op via iso) ---");
    println!(
        "  scaled the underlying tensor by 2.0; multifield metadata preserved: \
         metric={metric:?}, dx={dx:?}, shape={shape:?}"
    );
}

fn print_transform_shifted(metric: Metric, dx: &[FloatType; 3], shape: &[usize; 3]) {
    println!(
        "  added 0.5 element-wise; multifield metadata still preserved: \
         metric={metric:?}, dx={dx:?}, shape={shape:?}"
    );
}

fn print_export(
    shape: &[usize],
    sample: &[FloatType],
    metric: Metric,
    dx: &[FloatType; 3],
    grid: &[usize; 3],
) {
    println!("\n--- Stage 3: EXPORT (extract tensor for downstream pipeline) ---");
    println!("  extracted CausalTensor: shape={shape:?}, first 4 values: {sample:?}");
    println!("  carrier metadata: metric={metric:?}, dx={dx:?}, shape={grid:?}");
}

fn print_checks_header() {
    println!("\n--- Sanity checks ---");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_check(label: &str, actual: FloatType, expected: FloatType) {
    println!("  {label}{} ≈ {}", lower(actual), lower(expected));
}

fn print_metadata_ok() {
    println!("  metadata preserved end-to-end.\n");
}

fn print_why_it_matters() {
    println!("--- What the iso unlocked ---");
    println!("- `load_multifield(...)`     could not exist outside the multivector crate");
    println!("  without the iso: no public constructor takes (tensor, metric, dx, shape).");
    println!("- `map_underlying_tensor(...)` could not exist outside either: no public");
    println!("  accessor returns the OWNED tensor.");
    println!("- `export_multifield(...)`    could not exist outside either: same reason.");
    println!();
    println!("With the iso, all three helpers live in EXTERNAL code (this example) and");
    println!("compose with any tensor-aware library. The multivector crate's encapsulation");
    println!("stays intact; new consumers do not require new public methods on");
    println!("CausalMultiField itself.");
}

// =============================================================================
// External pipeline helpers — these live OUTSIDE the multivector crate.
// Without the iso they could not exist.
// =============================================================================

/// Construct a multifield from a precomputed `(tensor, metric, dx, shape)`.
///
/// Without the iso this function could not exist in external code: there
/// is no public constructor on `CausalMultiField` that takes the four
/// components directly.
fn load_multifield(
    tensor: CausalTensor<FloatType>,
    metric: Metric,
    dx: [FloatType; 3],
    shape: [usize; 3],
) -> CausalMultiField<FloatType> {
    (tensor, metric, dx, shape).into()
}

/// Apply a generic tensor-level transformation to a multifield's
/// underlying carrier, preserving metric, grid spacing, and grid shape.
///
/// Without the iso this function could not exist in external code: there
/// is no public way to reach `field.data` and `field.metric` without it.
fn map_underlying_tensor<TransformF>(
    field: CausalMultiField<FloatType>,
    transform: TransformF,
) -> CausalMultiField<FloatType>
where
    TransformF: FnOnce(CausalTensor<FloatType>) -> CausalTensor<FloatType>,
{
    let (tensor, metric, dx, shape): MultiFieldCarrier<FloatType> = field.into();
    (transform(tensor), metric, dx, shape).into()
}

/// Extract the underlying tensor (and the grid metadata, in case the
/// consumer needs it too).
///
/// Without the iso there is no public accessor that hands out an owned
/// `CausalTensor` from a multifield.
fn export_multifield(field: CausalMultiField<FloatType>) -> MultiFieldCarrier<FloatType> {
    field.into()
}
