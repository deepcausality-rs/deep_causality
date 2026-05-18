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
    slug: 'pearl-counterfactual',
    domain: 'Counterfactuals',
    headline: 'Pearl Rung-3: surgically intervene on a value mid-chain.',
    source: 'examples/starter_example/src/main.rs',
    snippet: `use deep_causality_core::{Intervenable, PropagatingEffect};

// Natural chain: nicotine → tar → cancer.
let before = PropagatingEffect::pure(0.8_f64)
    .bind(|nic, _, _| PropagatingEffect::pure(nicotine_to_tar(
        nic.into_value().unwrap_or_default())))
    .bind(|tar, _, _| PropagatingEffect::pure(tar_to_cancer(
        tar.into_value().unwrap_or_default())));

// Counterfactual: same start, but intervene on tar mid-chain.
let after = PropagatingEffect::pure(0.8_f64)
    .bind(|nic, _, _| PropagatingEffect::pure(nicotine_to_tar(
        nic.into_value().unwrap_or_default())))
    .intervene(0.1)
    .bind(|tar, _, _| PropagatingEffect::pure(tar_to_cancer(
        tar.into_value().unwrap_or_default())));

// Compare before.value vs after.value: that is the Pearl Rung-3 effect.`,
  },
  {
    slug: 'sensor-monitoring-csm',
    domain: 'Sensor monitoring',
    headline: 'A Causal State Machine wired to three real-time sensors.',
    source: 'examples/csm_examples/csm_basic/main.rs',
    snippet: `use deep_causality::{CSM, CausalState, PropagatingEffect};

// 1) Each sensor has a default reading that bootstraps the CSM.
let default_data: PropagatingEffect<f64> = PropagatingEffect::pure(0.0);

// 2) A CausalState pairs a sensor ID with its activation Causaloid.
let smoke_cs = CausalState::new(SMOKE_SENSOR, 1, default_data.clone(),
    get_smoke_sensor_causaloid(), None);
let fire_cs  = CausalState::new(FIRE_SENSOR, 1, default_data.clone(),
    get_fire_sensor_causaloid(), None);

// 3) The CSM wires each state to the action it should fire on activation.
let csm = CSM::new(&[(&smoke_cs, &get_smoke_alert_action()),
                     (&fire_cs,  &get_fire_alert_action())]);

// 4) Feed live evidence in; the CSM dispatches the matching action.
let evidence: PropagatingEffect<f64> = PropagatingEffect::pure(smoke_data[i]);
csm.eval_single_state(SMOKE_SENSOR, &evidence)?;`,
  },
  {
    slug: 'aerospace-flight-envelope',
    domain: 'Aerospace',
    headline: 'Five-stage PropagatingProcess for a flight-envelope monitor.',
    source: 'examples/avionics_examples/flight_envelope_monitor/main.rs',
    snippet: `// Seed the chain with a typed propagating process: value, state,
// context, error channel, and an append-only audit log all live together.
let initial: FlightProcess<SensorReading> = PropagatingProcess {
    value: EffectValue::Value(reading),
    state: FlightState::default(),
    context: Some(config),
    error: None,
    logs: EffectLog::new(),
};

// Five bind steps thread the state and context through the entire monitor.
initial
    .bind(|v, s, c| run_sensor_collection(v, s, c, failing_airspeed))
    .bind(|v, s, c| health_fold(v, s, c, seed_estimate.clone()))
    .bind(|v, s, c| kalman_step(v, s, c))
    .bind(|v, s, c| estimate_step(v, s, c))
    .bind(|v, s, c| run_envelope_graph(v, s, c))`,
  },
  {
    slug: 'biomedical-tumor-treatment',
    domain: 'Medicine',
    headline: 'Optimize the Tumor Treating Field.',
    source: 'examples/medicine_examples/tumor_treatment/main.rs',
    snippet: `use deep_causality_core::{CausalityError, EffectValue, PropagatingEffect};

let effect = PropagatingEffect::pure(new_params).bind(|params, _, _| {
    let p = match params {
        EffectValue::Value(v) => v,
        _ => (0.0, 0.0),
    };
    let score = match model::evaluate_efficacy(&tumor, p) {
        Ok(s) => s,
        Err(e) => return PropagatingEffect::from_error(
            CausalityError::new(/* ... */)
        ),
    };
    PropagatingEffect::pure(score)
});

if let EffectValue::Value(new_score) = effect.value() {
    if *new_score > current_score { /* accept */ }
}`,
  },
  {
    slug: 'physics-maxwell',
    domain: 'Physics',
    headline: 'Maxwell field derivation as a four-step monadic chain.',
    source: 'examples/physics_examples/maxwell/main.rs',
    snippet: `use deep_causality::PropagatingEffect;
use model::{MaxwellState, PlaneWaveConfig};

// The initial state encodes the plane-wave configuration.
let initial_state = MaxwellState::from_config(&config);

// Four-step bind chain: potential → field → gauge check → Poynting flux.
let result: PropagatingEffect<MaxwellState> =
    PropagatingEffect::pure(initial_state).bind(|state, _, _| {
        model::compute_potential(state.into_value().unwrap_or_default())
            .bind(|s, _, _| model::compute_em_field(s.into_value().unwrap_or_default()))
            .bind(|s, _, _| model::check_lorenz_gauge(s.into_value().unwrap_or_default()))
            .bind(|s, _, _| model::compute_poynting_flux(s.into_value().unwrap_or_default()))
    });

// Extract the final state from the propagating effect.
let final_state = result.value.into_value().unwrap_or_default();`,
  },
  {
    slug: 'async-event-inference',
    domain: 'Async / Tokio',
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
