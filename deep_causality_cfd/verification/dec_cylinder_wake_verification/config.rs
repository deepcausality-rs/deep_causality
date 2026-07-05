/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer for the cut-cell cylinder wake: the case + sensor parameters, the immersed
//! cut-cell geometry, the sensor stream, and the `CfdFlow::march` config container.
//!
//! The deterministic CFD scalar is the working precision [`FloatType`]; exact `f64` specifications
//! enter through [`ft`]. The geometry (cut-cell disk in a periodic-x channel) and the
//! `UncertainMarchConfig` are built here; `main.rs` lends the geometry to `CfdFlow::march`
//! and streams the wake probe, and `print_utils` renders the diagnostics. Presence-gate probabilities
//! and the wake-probe analysis stay in `f64` (they are not working-precision quantities).

use crate::FloatType;
use deep_causality_cfd::{
    CfdConfigBuilder, DropoutVerbosity, PhysicsError, UncertainInflowZone, UncertainMarchConfig,
};
use deep_causality_num::{FromPrimitive, Zero};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, CutCellRegistry, LatticeComplex, Manifold, Primitive,
};
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

// -- Case parameters --------------------------------------------------------------------

/// Cells across the channel height (wall-normal, y).
pub const NY: usize = 32;
/// Channel aspect ratio (streamwise length / height); the domain is `AR·H × H`.
pub const AR: f64 = 3.0;
/// Cylinder diameter as a fraction of the channel height (blockage ratio).
pub const BLOCKAGE: f64 = 0.25;
/// Reynolds number based on the cylinder diameter and the target bulk velocity.
pub const RE_D: f64 = 100.0;
/// Target bulk streamwise velocity — the sensor's nominal reading and the moving-wall lid speed.
pub const U_BULK: f64 = 1.0;
/// Cell-merging floor (B1/B2): free cut cells/edges below this wetted fraction borrow volume to
/// reach it, tightening the masked-CG projection conditioning. (Explicit stability is inherent
/// here — design D4 — so this is not a CFL guard.)
pub const MERGE_FRACTION: f64 = 0.25;
/// Number of march steps.
pub const STEPS: usize = 2000;

// -- Sensor-stream parameters (Group C) -------------------------------------------------

/// Relative 1σ noise on a *present* inflow reading.
pub const SENSOR_SIGMA: f64 = 0.03;
/// A sensor dropout (absent reading) every this many steps — exercises the BC-fallback
/// intervention and its `EffectLog` record.
pub const DROPOUT_EVERY: usize = 50;
/// Sample budgets: the Monte-Carlo presence gate and the Quasi-Monte-Carlo mean collapse.
pub const PRESENCE_SAMPLES: usize = 256;
pub const COLLAPSE_SAMPLES: usize = 256;
/// Seed for the Monte-Carlo presence gate (SPRT `to_bool`). Seeding the thread-local sampler RNG
/// fixes the gate's realization so the run is reproducible byte-for-byte.
pub const SAMPLER_SEED: u64 = 0x5EED_C0DE;
/// Base seed for the **Quasi-Monte-Carlo** collapse of the present sensor reading (Sobol + inverse
/// CDF). The per-step Sobol shift is `base ⊕ sample.id()`, so the collapse is reproducible and
/// independent across steps. (The presence gate stays Monte-Carlo — QMC is invalid for the SPRT.)
pub const QMC_COLLAPSE_SEED: u64 = 0x0B0_5E11;

/// Lift an exact `f64` specification into the working precision [`FloatType`].
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// The immersed cut-cell geometry plus the derived quantities `main` and `print_utils` need. The
/// working-precision quantities (`h`, `dt`, `nu`, `diameter`, `fluid_area`) are [`FloatType`] and are
/// used in native arithmetic; only counts and indices (`nx`, `probe_edge`, the cell tallies) are
/// `usize`.
pub struct CaseGeometry {
    /// The cut-cell manifold (periodic-x channel with the immersed disk), at the working precision.
    pub manifold: Manifold<LatticeComplex<2, FloatType>, FloatType>,
    /// Cells along the streamwise (x) axis.
    pub nx: usize,
    /// Vertex spacing `h = 1/(NY−1)`.
    pub h: FloatType,
    /// Diffusive-limit time step.
    pub dt: FloatType,
    /// Kinematic viscosity `ν = U·D/Re`.
    pub nu: FloatType,
    /// Cylinder diameter.
    pub diameter: FloatType,
    /// Edge index of the transverse-velocity wake probe (one diameter downstream).
    pub probe_edge: usize,
    /// Number of fully-solid cells.
    pub n_solid: usize,
    /// Number of partially-cut cells.
    pub n_cut: usize,
    /// Fluid area of the domain.
    pub fluid_area: FloatType,
}

/// Build the immersed cut-cell cylinder geometry: the analytic disk clipped into a periodic-x
/// channel, with cell-merging small-cell stabilization and deterministic cut-cell ordering.
///
/// Counts and lattice indices are derived in `f64`/`usize` (they are not working-precision
/// quantities); the working-precision specifications lift into [`FloatType`] once through [`ft`] and
/// the geometry, viscosity, step, and area arithmetic run natively at that precision.
pub fn build_geometry() -> CaseGeometry {
    // f64 specifications and integer counts.
    let h_spec = 1.0 / (NY - 1) as f64; // channel height H = 1.
    let nx = (AR / h_spec).round() as usize;
    let diameter_spec = BLOCKAGE; // H = 1, so D = BLOCKAGE·H.
    let radius_spec = 0.5 * diameter_spec;
    let center_spec = [AR * 0.25, 0.5]; // a quarter-length in, mid-channel.
    let nu_spec = U_BULK * diameter_spec / RE_D;
    // Conservative dt: the diffusive limit, with an advective margin for the lid-driven bulk.
    let dt_spec = 0.2 * h_spec * h_spec / (4.0 * nu_spec);

    let lattice = LatticeComplex::<2, FloatType>::new([nx, NY], [true, false]); // periodic-x, wall-y.
    let base_metric = CubicalReggeGeometry::<2, FloatType>::uniform(ft(h_spec));
    let disk =
        Primitive::<2, FloatType>::ball([ft(center_spec[0]), ft(center_spec[1])], ft(radius_spec));
    // `with_deterministic_order` pins cut-cell iteration to ascending cell id, so the cut-face
    // constraint rows — hence the constrained projection's summation order — are reproducible.
    let registry = CutCellRegistry::from_primitive(&lattice, &base_metric, &disk)
        .expect("disk intersection")
        .with_deterministic_order();
    let n_solid = registry
        .iter()
        .filter(|(_, c)| c.class().is_solid())
        .count();
    let n_cut = registry.iter().filter(|(_, c)| c.class().is_cut()).count();
    // Native-precision fluid area: domain area minus the solid/cut dry volume.
    let fluid_area = ft(AR) - solid_area(&registry, ft(h_spec * h_spec));

    let registry = registry.with_cell_merging(MERGE_FRACTION);
    let metric = base_metric.with_cut_cells(registry);

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![ft(0.0); total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    // A wake probe: the y-velocity (transverse) one diameter downstream of the cylinder.
    let probe_x = ((center_spec[0] + 1.5 * diameter_spec) / h_spec).round() as usize;
    let probe_y = (0.5 / h_spec).round() as usize;
    let probe_edge = manifold
        .complex()
        .iter_cells(1)
        .position(|c| {
            c.orientation().trailing_zeros() as usize == 1
                && c.position()[0] == probe_x.min(nx - 1)
                && c.position()[1] == probe_y.min(NY - 2)
        })
        .expect("probe edge exists");

    CaseGeometry {
        manifold,
        nx,
        h: ft(h_spec),
        dt: ft(dt_spec),
        nu: ft(nu_spec),
        diameter: ft(diameter_spec),
        probe_edge,
        n_solid,
        n_cut,
        fluid_area,
    }
}

/// The sensor-fed uncertain-inflow march config: the DEC solver at `ν`/`dt`, the presence-gated +
/// QMC-collapsed inflow zone, the sensor stream, and the step horizon — built through
/// `CfdConfigBuilder`, run by `CfdFlow::march`.
///
/// # Errors
/// Any solver-config or container validation failure.
pub fn build_uncertain_config(
    nu: FloatType,
    dt: FloatType,
) -> Result<UncertainMarchConfig<FloatType>, PhysicsError> {
    let zone = UncertainInflowZone::new(1, true, 0, ft(U_BULK))
        .with_presence_gate(0.5, 0.95, 0.05, PRESENCE_SAMPLES)
        .with_collapse_samples(COLLAPSE_SAMPLES)
        .with_qmc_collapse(QMC_COLLAPSE_SEED)
        .with_verbosity(DropoutVerbosity::EachDropout);

    CfdConfigBuilder::uncertain_march::<FloatType>("cylinder-wake")
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("solver config"),
        )
        .inflow_zone(zone)
        .sensor_stream(sensor_stream(STEPS))
        .march_for(STEPS)
        .build()
}

/// The per-step sensor stream: a noisy present reading at `U_BULK`, with a periodic dropout.
pub fn sensor_stream(steps: usize) -> Vec<MaybeUncertain<FloatType>> {
    (0..steps)
        .map(|s| {
            if (s + 1) % DROPOUT_EVERY == 0 {
                MaybeUncertain::<FloatType>::always_none()
            } else {
                MaybeUncertain::<FloatType>::from_uncertain(Uncertain::normal(
                    ft(U_BULK),
                    ft(SENSOR_SIGMA),
                ))
            }
        })
        .collect()
}

/// Total solid area recorded in the registry (solid cells full; cut cells their dry part), summed in
/// native [`FloatType`].
fn solid_area(registry: &CutCellRegistry<2, FloatType>, full_area: FloatType) -> FloatType {
    registry.iter().fold(FloatType::zero(), |acc, (_, cut)| {
        acc + if cut.class().is_solid() {
            full_area
        } else {
            cut.full_volume() - cut.fluid_volume()
        }
    })
}
