/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Multi-Physics Pipeline: Complete QFT → QCD → Thermal → Detection
//!
//! Demonstrates **modular composition** via `CausalEffectPropagationProcess`
//! (the Causal Monad) for a complete high-energy physics simulation.
//!
//! ## Key Design Pattern
//!
//! Each physics stage is a **standalone function** that can be:
//! - Tested independently
//! - Replaced without affecting the pipeline
//! - Composed in different orders
//! - Reused across different simulations
//!
//! ```ignore
//! klein_gordon()
//!     .context(world)                                // The model parameters
//!     .bind_or_error(stage_field_to_partons, ...)    // Modular: Field → q-q̄
//!     .bind_or_error(stage_lund_fragmentation, ...)  // Modular: q-q̄ → hadrons
//!     .bind_or_error(stage_thermalization, ...)      // Modular: hadrons → thermal
//!     .bind_or_error(stage_quantum_detection, ...)   // Modular: thermal → detection
//! ```
//!
//! This is the power of the Causal Monad: **decoupled physics modules**
//! that compose seamlessly with automatic error propagation.
//!
//! The model's world is the pipeline's context: a `Context` of `Data` contextoids holding the
//! initial field profile, the energy scale and its clamps, the thermal share, gradient,
//! diffusivity and clamps, the Higgs mass, the QGP critical temperature and the amplitude clamps.
//! The seed reads the profile and the mass from it; each stage reads what it needs from the
//! context and hands the context on.
use deep_causality_algebra::Real;
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    CausalFlow, CausalityError, CausalityErrorEnum, PropagatingEffect, PropagatingProcess,
};
use deep_causality_multivector::{HilbertState, Metric};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift_usize, lower};
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    FourMomentum, Hadron, LundParameters, heat_diffusion, klein_gordon,
    lund_string_fragmentation_kernel,
};
use deep_causality_quantum::born_probability;
use deep_causality_tensor::CausalTensor;
mod model;

/// The model's world as a context: one `Data` contextoid per world fact. The pipeline's fields
/// live on its own 1D meshes, so the context's spatial, temporal and spacetime slots are empty.
type PipelineContext = Context<Data<Fact>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// One world fact of the model: a scalar parameter, or a profile sampled across the cells.
#[derive(Debug, Clone, PartialEq)]
enum Fact {
    Scalar(FloatType),
    Profile(Vec<FloatType>),
}

/// `Data` requires a default payload: the zero scalar.
impl Default for Fact {
    fn default() -> Self {
        Fact::Scalar(ZERO)
    }
}

/// What a stage hands on: its value, with the model parameters for the next stage.
type PipelineProcess<T> = PropagatingProcess<T, (), PipelineContext>;

/// Contextoid id: scale from the Klein-Gordon field energy to a centre-of-mass energy in GeV.
const FIELD_ENERGY_SCALE: ContextoidId = 1;
/// Contextoid id: lower end of the range the centre-of-mass energy is held to, in GeV.
const MIN_CMS_ENERGY_GEV: ContextoidId = 2;
/// Contextoid id: upper end of the range the centre-of-mass energy is held to, in GeV.
const MAX_CMS_ENERGY_GEV: ContextoidId = 3;
/// Contextoid id: fraction of the hadron energy that goes into the thermal bath.
const TEMPERATURE_SHARE: ContextoidId = 4;
/// Contextoid id: lower end of the range the initial temperature is held to, in MeV.
const MIN_TEMPERATURE_MEV: ContextoidId = 5;
/// Contextoid id: upper end of the range the initial temperature is held to, in MeV.
const MAX_TEMPERATURE_MEV: ContextoidId = 6;
/// Contextoid id: the fractional temperature drop per cell.
const TEMPERATURE_GRADIENT: ContextoidId = 7;
/// Contextoid id: thermal diffusivity.
const DIFFUSIVITY: ContextoidId = 8;
/// Contextoid id: scalar mass driving the Klein-Gordon evolution, in GeV.
const HIGGS_MASS_GEV: ContextoidId = 9;
/// Contextoid id: QGP transition temperature, in MeV.
const CRITICAL_TEMPERATURE_MEV: ContextoidId = 10;
/// Contextoid id: lower end of the range the detection amplitude is held to. With the upper
/// end it keeps either basis state from being exactly empty.
const MIN_AMPLITUDE: ContextoidId = 11;
/// Contextoid id: upper end of the range the detection amplitude is held to.
const MAX_AMPLITUDE: ContextoidId = 12;
/// Contextoid id: initial Klein-Gordon field profile across the cells of its mesh, the field the
/// pipeline starts from.
const PHI_PROFILE: ContextoidId = 13;

/// Cells in the 1D temperature field: the resolution of the thermal grid.
const TEMPERATURE_CELLS: usize = 10;

/// Small whole numbers, at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

/// `clamp` at the working scalar. `Ord::clamp` is unavailable for a partially ordered float, so
/// this spells out the two comparisons rather than pinning the type to a primitive.
fn clamp(value: FloatType, low: FloatType, high: FloatType) -> FloatType {
    if value < low {
        low
    } else if value > high {
        high
    } else {
        value
    }
}

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision.
pub type FloatType = f64;

// =============================================================================
// MAIN: Pipeline Composition via Causal Monad
// =============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Multi-Physics Pipeline: QFT → QCD → Thermal → Detection");
    println!("  (Modular Stages Composed via Causal Monad)");
    println!("═══════════════════════════════════════════════════════════════\n");

    // The model's world: the context the seed and every stage read.
    let world = pipeline_world()?;

    // Initial field profile: a linear ramp across the cells.
    let phi_manifold = model::make_1d_manifold(read_profile(&world, PHI_PROFILE)?)?;
    let [mass] = read(&world, [HIGGS_MASS_GEV])?;

    // =========================================================================
    // The Causal Monad Pipeline: Each stage is a decoupled function
    // =========================================================================
    // The decoupled stages, driven by `CausalFlow`. Each stage has the
    // `(value, (), Option<PipelineContext>) -> PropagatingProcess` shape the
    // `bind_or_error` passthrough takes: it reads the model parameters from the
    // context and hands the context on. `From` seeds the flow, `.context`
    // attaches the parameters, and `finish` is the terminal: a stage that
    // failed is printed and becomes the error the process exits with.
    let summary = CausalFlow::from(klein_gordon(&phi_manifold, mass))
        .context(world)
        .bind_or_error(stage_field_to_partons, "Field → Partons failed")
        .bind_or_error(stage_lund_fragmentation, "Lund fragmentation failed")
        .bind_or_error(stage_thermalization, "Thermalization failed")
        .bind_or_error(stage_quantum_detection, "Detection failed")
        .finish()
        .inspect_err(print_summary_err)?;
    print_summary_ok(summary);

    Ok(())
}

// =============================================================================
// STAGE 1: Field Energy → Virtual Quark-Antiquark Creation
// =============================================================================

/// Converts Klein-Gordon field evolution into virtual quark-antiquark endpoints.
///
/// # Physics
/// - Computes total field energy: E = Σ|φ|²
/// - Creates back-to-back q-q̄ pair with combined energy = E_cms
///
/// # Maintenance
/// - Modify quark masses here
/// - Change energy scaling independently
/// - Add multiple q-q̄ pairs for multi-jet events
fn stage_field_to_partons(
    evolved_tensor: CausalTensor<FloatType>,
    _: (),
    world: Option<PipelineContext>,
) -> PipelineProcess<Vec<(FourMomentum<FloatType>, FourMomentum<FloatType>)>> {
    let (world, [energy_scale, min_energy, max_energy]) = match stage_facts(
        world,
        [FIELD_ENERGY_SCALE, MIN_CMS_ENERGY_GEV, MAX_CMS_ENERGY_GEV],
    ) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("Stage 1: Klein-Gordon Scalar Field");
    println!("───────────────────────────────────");

    // Compute field energy
    let field_energy: FloatType = evolved_tensor
        .data()
        .iter()
        .map(|&v| Real::abs(v) * Real::abs(v))
        .sum::<FloatType>()
        * energy_scale;

    let cms_energy = clamp(field_energy, min_energy, max_energy);
    println!("  Field energy: E_cms = {:.2} GeV\n", cms_energy);

    // Create virtual q-q̄ pair (back-to-back in CM frame)
    let half_e = cms_energy / TWO;
    let quark = FourMomentum::<FloatType>::new(half_e, ZERO, ZERO, half_e);
    let antiquark = FourMomentum::<FloatType>::new(half_e, ZERO, ZERO, -half_e);

    println!("Stage 2: QCD String Creation");
    println!("────────────────────────────");
    println!("  q:  (E={:.1}, pz=+{:.1}) GeV", half_e, half_e);
    println!("  q̄:  (E={:.1}, pz=-{:.1}) GeV", half_e, half_e);

    emit(vec![(quark, antiquark)], world)
}

// =============================================================================
// STAGE 2: Lund String Fragmentation → Hadrons
// =============================================================================

/// Fragments QCD strings into hadrons using the Lund model.
///
/// # Physics
/// - Iterative string breaking with q-q̄ pair creation
/// - Produces π, K, ρ, ω, η, etc.
///
/// # Maintenance
/// - Tune Lund parameters (a, b, σ_pt) independently
/// - Replace with different fragmentation model
/// - Add particle filtering or cuts
fn stage_lund_fragmentation(
    endpoints: Vec<(FourMomentum<FloatType>, FourMomentum<FloatType>)>,
    _: (),
    world: Option<PipelineContext>,
) -> PipelineProcess<(usize, FloatType)> {
    let (world, []) = match stage_facts(world, []) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("\nStage 3: Lund String Fragmentation");
    println!("───────────────────────────────────");

    let params = LundParameters::default();
    let mut rng = deep_causality_rand::rng();

    match lund_string_fragmentation_kernel(&endpoints, &params, &mut rng) {
        Ok(hadrons) => {
            let valid: Vec<&Hadron<FloatType>> =
                hadrons.iter().filter(|h| h.energy() > ZERO).collect();

            println!(
                "  Produced {} hadrons ({} physical)",
                hadrons.len(),
                valid.len()
            );
            print_hadron_sample(&valid);

            let total_e: FloatType = valid.iter().map(|h| h.energy()).sum();
            emit((valid.len(), total_e), world)
        }
        Err(e) => PropagatingProcess::from_error(e.into()),
    }
}

// =============================================================================
// STAGE 3: Thermalization via Heat Diffusion
// =============================================================================

/// Thermalizes hadron gas using heat diffusion.
///
/// # Physics
/// - Creates temperature field from hadron energies
/// - Evolves via diffusion equation: ∂T/∂t = κ∇²T
///
/// # Maintenance
/// - Adjust diffusivity independently
/// - Replace with hydrodynamic evolution
/// - Add viscosity corrections
fn stage_thermalization(
    (hadron_count, total_energy): (usize, FloatType),
    _: (),
    world: Option<PipelineContext>,
) -> PipelineProcess<(usize, FloatType)> {
    let (world, [share, min_temp, max_temp, gradient, diffusivity]) = match stage_facts(
        world,
        [
            TEMPERATURE_SHARE,
            MIN_TEMPERATURE_MEV,
            MAX_TEMPERATURE_MEV,
            TEMPERATURE_GRADIENT,
            DIFFUSIVITY,
        ],
    ) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("\nStage 4: Thermalization");
    println!("───────────────────────");

    // Scale to MeV (typical QGP temperature ~ 150-400 MeV)
    let temp_scale = clamp(total_energy * share, min_temp, max_temp);
    let initial_temp: Vec<FloatType> = (0..TEMPERATURE_CELLS)
        .map(|i| temp_scale * (ONE - lift_usize::<FloatType>(i) * gradient))
        .collect();
    let temp_manifold = match model::make_1d_manifold(initial_temp.clone()) {
        Ok(m) => m,
        Err(e) => {
            return PropagatingProcess::from_error(custom(format!(
                "mesh construction failed: {e:?}"
            )));
        }
    };

    let heat_result = heat_diffusion(&temp_manifold, diffusivity);

    // Use the diffused result if valid; a failed diffusion is the stage's error.
    let avg_temp = match heat_result.value() {
        Some(final_temp) => {
            // Ten cells, so neither mean can refuse; dispatched rather than divided by a
            // literal, which silently decouples from the cell count if the grid changes.
            let avg = match deep_causality_stats::mean(final_temp.data().as_slice()) {
                Ok(avg) => avg,
                Err(e) => return PropagatingProcess::from_error(custom(format!("{e:?}"))),
            };
            if Real::abs(avg) > ONE {
                Real::abs(avg)
            } else {
                // Fallback: use initial temperature average
                match deep_causality_stats::mean(&initial_temp) {
                    Ok(avg) => avg,
                    Err(e) => return PropagatingProcess::from_error(custom(format!("{e:?}"))),
                }
            }
        }
        None => {
            let error = heat_result
                .error()
                .cloned()
                .unwrap_or_else(|| custom("heat diffusion produced no temperature field"));
            return PropagatingProcess::from_error(error);
        }
    };

    println!("  Initial temp: {:.1} MeV", temp_scale);
    println!("  Equilibrium:  {:.1} MeV", avg_temp);

    emit((hadron_count, avg_temp), world)
}

// =============================================================================
// STAGE 4: Quantum Detection via Born Rule
// =============================================================================

/// Computes detection probability using the Born rule.
///
/// # Physics
/// - Creates detector wavefunction from thermal signal
/// - P(detection) = |⟨basis|ψ⟩|²
///
/// # Maintenance
/// - Change basis states independently
/// - Add multiple detector channels
/// - Implement more complex observables
fn stage_quantum_detection(
    (hadron_count, avg_temp): (usize, FloatType),
    _: (),
    world: Option<PipelineContext>,
) -> PipelineProcess<(usize, FloatType, FloatType)> {
    let (world, [critical_temp, min_amplitude, max_amplitude]) = match stage_facts(
        world,
        [CRITICAL_TEMPERATURE_MEV, MIN_AMPLITUDE, MAX_AMPLITUDE],
    ) {
        Ok(stage) => stage,
        Err(e) => return PropagatingProcess::from_error(e),
    };
    println!("\nStage 5: Quantum Detection");
    println!("──────────────────────────");

    // Detection probability scales with temperature
    // At T_c ~ 170 MeV (QGP transition), detection is 50%
    // Higher temp → higher detection probability
    let psi_val = clamp(
        avg_temp / (avg_temp + critical_temp),
        min_amplitude,
        max_amplitude,
    );
    let psi = Complex::new(Real::sqrt(psi_val), ZERO);
    let psi_orth = Complex::new(Real::sqrt(ONE - psi_val), ZERO);

    let metric = Metric::Euclidean(1);
    let state = match HilbertState::<FloatType>::new(vec![psi, psi_orth], metric) {
        Ok(s) => s,
        Err(e) => {
            return PropagatingProcess::from_error(custom(format!(
                "state construction failed: {e:?}"
            )));
        }
    };
    let basis = match HilbertState::<FloatType>::new(
        vec![Complex::new(ONE, ZERO), Complex::new(ZERO, ZERO)],
        metric,
    ) {
        Ok(b) => b,
        Err(e) => {
            return PropagatingProcess::from_error(custom(format!(
                "basis construction failed: {e:?}"
            )));
        }
    };

    let detection = born_probability(&state, &basis);
    let prob = match detection.value() {
        Some(p) => *p,
        None => {
            let error = detection
                .error()
                .cloned()
                .unwrap_or_else(|| custom("the Born rule produced no probability"));
            return PropagatingProcess::from_error(error);
        }
    };

    println!("  Critical temp: T_c = {:.0} MeV", lower(critical_temp));
    println!(
        "  |ψ⟩ = {:.3}|QGP⟩ + {:.3}|hadron⟩",
        lower(psi.re()),
        lower(psi_orth.re())
    );
    println!("  P(QGP detection) = {:.4}", lower(prob));

    emit((hadron_count, avg_temp, prob), world)
}

// =============================================================================
// CONTEXT: the model's parameters
// =============================================================================

/// Builds the model's world: each fact as a `Data` contextoid keyed by its contextoid id.
fn pipeline_world() -> Result<PipelineContext, ContextIndexError> {
    let facts = [
        (
            FIELD_ENERGY_SCALE,
            Fact::Scalar(const_scalar_from_float!(FloatType, 0.01)),
        ),
        (
            MIN_CMS_ENERGY_GEV,
            Fact::Scalar(const_scalar_from_int!(FloatType, 10)),
        ),
        (
            MAX_CMS_ENERGY_GEV,
            Fact::Scalar(const_scalar_from_int!(FloatType, 500)),
        ),
        (
            TEMPERATURE_SHARE,
            Fact::Scalar(const_scalar_from_float!(FloatType, 0.5)),
        ),
        (
            MIN_TEMPERATURE_MEV,
            Fact::Scalar(const_scalar_from_int!(FloatType, 100)),
        ),
        (
            MAX_TEMPERATURE_MEV,
            Fact::Scalar(const_scalar_from_int!(FloatType, 500)),
        ),
        (
            TEMPERATURE_GRADIENT,
            Fact::Scalar(const_scalar_from_float!(FloatType, 0.02)),
        ),
        (
            DIFFUSIVITY,
            Fact::Scalar(const_scalar_from_float!(FloatType, 0.1)),
        ),
        (
            HIGGS_MASS_GEV,
            Fact::Scalar(const_scalar_from_int!(FloatType, 125)),
        ),
        (
            CRITICAL_TEMPERATURE_MEV,
            Fact::Scalar(const_scalar_from_int!(FloatType, 170)),
        ),
        (
            MIN_AMPLITUDE,
            Fact::Scalar(const_scalar_from_float!(FloatType, 0.01)),
        ),
        (
            MAX_AMPLITUDE,
            Fact::Scalar(const_scalar_from_float!(FloatType, 0.99)),
        ),
        (
            PHI_PROFILE,
            Fact::Profile(vec![
                const_scalar_from_int!(FloatType, 1),
                const_scalar_from_float!(FloatType, 0.9),
                const_scalar_from_float!(FloatType, 0.8),
                const_scalar_from_float!(FloatType, 0.7),
                const_scalar_from_float!(FloatType, 0.6),
                const_scalar_from_float!(FloatType, 0.5),
                const_scalar_from_float!(FloatType, 0.4),
                const_scalar_from_float!(FloatType, 0.3),
                const_scalar_from_float!(FloatType, 0.2),
                const_scalar_from_float!(FloatType, 0.1),
            ]),
        ),
    ];

    let mut world = Context::with_capacity(1, "multi-physics model", facts.len());
    for (id, fact) in facts {
        world.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, fact)),
        ))?;
    }
    Ok(world)
}

/// Reads one world fact out of the model's context, or the error naming the contextoid id it
/// lacks.
fn fact(world: &PipelineContext, id: ContextoidId) -> Result<Fact, CausalityError> {
    world
        .get_data_by_id(id)
        .ok_or_else(|| custom(format!("the model holds no Datoid with contextoid id {id}")))
}

/// Reads scalar world facts, in the order `ids` names them.
fn read<const N: usize>(
    world: &PipelineContext,
    ids: [ContextoidId; N],
) -> Result<[FloatType; N], CausalityError> {
    let mut values = [ZERO; N];
    for (value, id) in values.iter_mut().zip(ids) {
        *value = match fact(world, id)? {
            Fact::Scalar(scalar) => scalar,
            Fact::Profile(_) => {
                return Err(custom(format!(
                    "contextoid id {id} holds a profile, not a scalar"
                )));
            }
        };
    }
    Ok(values)
}

/// Reads a profile world fact.
fn read_profile(
    world: &PipelineContext,
    id: ContextoidId,
) -> Result<Vec<FloatType>, CausalityError> {
    match fact(world, id)? {
        Fact::Profile(profile) => Ok(profile),
        Fact::Scalar(_) => Err(custom(format!(
            "contextoid id {id} holds a scalar, not a profile"
        ))),
    }
}

/// The model's parameters the flow carries, with the scalar facts `ids` names in that order, or
/// the error a stage short-circuits with when the flow carries no model parameters.
fn stage_facts<const N: usize>(
    world: Option<PipelineContext>,
    ids: [ContextoidId; N],
) -> Result<(PipelineContext, [FloatType; N]), CausalityError> {
    let world = world.ok_or_else(|| custom("the flow carries no model parameters"))?;
    let facts = read(&world, ids)?;
    Ok((world, facts))
}

/// Hands `value` and the model's parameters on to the next stage.
fn emit<T>(value: T, world: PipelineContext) -> PipelineProcess<T>
where
    T: Default + Clone + core::fmt::Debug,
{
    PropagatingProcess::with_state(PropagatingEffect::pure(value), (), Some(world))
}

/// A stage failure, as the pipeline's error.
fn custom(reason: impl Into<String>) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(reason.into()))
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Prints a sample of produced hadrons.
fn print_hadron_sample(hadrons: &[&Hadron<FloatType>]) {
    println!("\n  Sample hadrons:");
    for (i, h) in hadrons.iter().take(5).enumerate() {
        println!(
            "    [{:2}] {} (PDG {}): E = {:.2} GeV",
            i + 1,
            pdg_name(h.pdg_id()),
            h.pdg_id(),
            h.energy()
        );
    }
    if hadrons.len() > 5 {
        println!("    ... and {} more", hadrons.len() - 5);
    }
}

/// Prints the summary banner shared by the success and failure paths.
fn print_summary_header() {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Pipeline Summary");
    println!("═══════════════════════════════════════════════════════════════");
}

/// Prints the final pipeline summary on success.
fn print_summary_ok((hadron_count, avg_temp, prob): (usize, FloatType, FloatType)) {
    print_summary_header();
    println!("  Hadron multiplicity:    {} particles", hadron_count);
    println!("  Thermal equilibrium:    {:.2} MeV", lower(avg_temp));
    println!("  Detection probability:  {:.4}", lower(prob));
    println!("\n[SUCCESS] Modular Pipeline Completed.\n");
}

/// Prints the final pipeline summary when a stage short-circuits the chain.
fn print_summary_err(err: &CausalityError) {
    print_summary_header();
    println!("  Pipeline failed: {err:?}");
    println!("\n[WARN] Check individual stage outputs.\n");
}

/// Gets particle name from PDG ID.
fn pdg_name(pdg_id: i32) -> &'static str {
    match pdg_id.abs() {
        111 => "π⁰",
        211 => {
            if pdg_id > 0 {
                "π⁺"
            } else {
                "π⁻"
            }
        }
        221 => "η",
        311 => "K⁰",
        321 => {
            if pdg_id > 0 {
                "K⁺"
            } else {
                "K⁻"
            }
        }
        113 => "ρ⁰",
        213 => {
            if pdg_id > 0 {
                "ρ⁺"
            } else {
                "ρ⁻"
            }
        }
        223 => "ω",
        331 => "η'",
        333 => "φ",
        _ => "hadron",
    }
}
