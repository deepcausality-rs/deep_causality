// Landing-page example cards. Every snippet is excerpted from a real crate
// under `examples/` in the monorepo. The detail page links to the source.

export interface Example {
  slug: string;
  domain: string;
  headline: string;
  snippet: string;
  /** Absolute repo path to the source file the snippet was extracted from. */
  source: string;
}

export const examples: Example[] = [
  {
    slug: 'service-root-cause',
    domain: 'Service reliability',
    headline: 'Rank the likely causes of a microservice incident from telemetry data.',
    source: 'examples/causal_discovery_examples/cdl/brcd_discovery/main.rs',
    snippet: `use deep_causality_discovery::*;

// Normal and incident telemetry, plus a supplied causal graph.
let config = CdlConfigBuilder::build_brcd_config()
    .with_normal_path(&normal_path)
    .with_anomalous_path(&anomalous_path)
    .with_brcd_config(BrcdConfig::<FloatType>::continuous(0))
    .with_cpdag_path(&cpdag_path)
    .build()
    .expect("Sock Shop carts_cpu_1 data files exist");

// Build and run causal discovery
CdlBuilder::build_brcd(&config)
    .brcd_load_input()
    .brcd_discover()
    .brcd_analyze()
    .finalize()
    .print_results();`,
  },
  {
    slug: 'counterfactual-weather',
    domain: 'Navigation',
    headline: 'Measure how weather changes navigation error.',
    source: 'examples/avionics_examples/cfd/plasma_blackout/weather/main.rs',
    snippet: `use deep_causality_cfd::*;
    
//Six atmospheres × eight receiver-noise draws: 48 simulated descents.
let table = CfdFlow::study("weather-dispersion table")
    .save_log(audit_dir.join("weather.audit"))
    .cases(model::weather_cases())
    .baseline(model::standard_day)
    // Generate alteranate worlds 
    .alternate(model::weather_world)
    .ensemble(constants::MC_DRAWS)
    .couple(|case, draw| world::corridor_coupling(model::bias_departure(case.d_temp), draw))
    .march_for(constants::STEPS, world::initial_field)
    // Collect counterfactual results from alteranate worlds 
    .reduce_ensemble(model::world_row)
    .inspect(utils_print::print_rows)
    .record(&table_path)
    .gates(model::weather_gates())
    .verdict()?;`,
  },
  {
    slug: 'aerospace-flight-envelope',
    domain: 'Aerospace',
    headline: 'Assess stall, overspeed, and terrain risk from aircraft sensor data.',
    source: 'examples/avionics_examples/control/flight_envelope_monitor/main.rs',
    snippet: `use deep_causality::*;
// Seed the chain with a typed propagating process: value, state,
// context, error channel, and an append-only audit log all live together.
let initial: FlightProcess<SensorReading> = PropagatingProcess::new(
    Ok(CausalEffect::value(reading)),
    FlightState::default(),
    Some(config),
    EffectLog::new(),
);

// CausalFlow threads state and context through all five stages;
CausalFlow::from(initial)
    .bind(|v, s, c| run_sensor_collection(v, s, c, failing_airspeed))
    .bind(|v, s, c| health_fold(v, s, c, seed_estimate.clone()))
    .bind(|v, s, c| kalman_step(v, s, c))
    .bind(|v, s, c| estimate_step(v, s, c))
    .bind(|v, s, c| run_envelope_graph(v, s, c))
    .into_process()`,
  },
  {
    slug: 'biomedical-tumor-treatment',
    domain: 'Medicine',
    headline: 'Optimize the Tumor Treating Field.',
    source: 'examples/medicine_examples/tumor_treatment/main.rs',
    snippet: `use deep_causality_core::{CausalFlow, PropagatingEffect};

// Start just off the degenerate θ = 0 pole, then ascend the exact
// autodiff gradient. A non-finite gradient short-circuits the error channel.
let start = [ft(0.1), ft(0.1)];

let pipeline = CausalFlow::value(start)
    .bind(move |p, _, _| match p.into_value() {
        Some(s) => ascend(&efficacy, s, learning_rate, ASCENT_STEPS),
        None => fail("no starting orientation"),
    })
    .into_effect();

// Read the optimised orientation off the pipeline's value channel.
if let Some(r) = pipeline.value.into_value() {
    println!("θ={:.3}, φ={:.3} after {} steps", r.theta, r.phi, r.steps);
}`,
  },
  {
    slug: 'physics-maxwell',
    domain: 'Physics',
    headline: 'Calculate an electromagnetic wave’s fields and energy flow.',
    source: 'examples/physics_examples/maxwell/main.rs',
    snippet: `use deep_causality::PropagatingEffect;
use deep_causality_core::CausalFlow;
use model::{MaxwellState, PlaneWaveConfig};

// The initial state encodes the plane-wave configuration.
let initial_state = MaxwellState::from_config(&config);

// One CausalFlow pipeline: potential → field → gauge check → Poynting flux.
let result: PropagatingEffect<MaxwellState> = CausalFlow::value(initial_state)
    .bind(|s, _, _| model::compute_potential(s.into_value().unwrap_or_default()))
    .bind(|s, _, _| model::compute_em_field(s.into_value().unwrap_or_default()))
    .bind(|s, _, _| model::check_lorenz_gauge(s.into_value().unwrap_or_default()))
    .bind(|s, _, _| model::compute_poynting_flux(s.into_value().unwrap_or_default()))
    .into_effect();

// Extract the final state from the propagating effect.
let final_state = result.value.into_value().unwrap_or_default();`,
  },
  {
    slug: 'async-event-inference',
    domain: 'Model Serving',
    headline: 'Background causal inference on a Tokio task.',
    source: 'examples/tokio_example/src/main.rs',
    snippet: `use crate::handler::EventHandler;
use crate::model::build_causal_model;

// All causal inference runs on a background Tokio task,
// so the main thread stays free to handle other work.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_handler = EventHandler::new(build_causal_model());

    tokio::spawn(async move {
        if let Err(e) = event_handler.run_background_inference().await {
            eprintln!("inference error: {e}");
        }
    })
    .await
    .expect("Failed to spawn async background task");
    Ok(())
}`,
  },
];
