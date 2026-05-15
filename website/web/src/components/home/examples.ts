// Landing-page example cards. The snippets are illustrative, not necessarily
// end-to-end compilable; full runnable code lives on each example detail page.

export interface Example {
  slug: string;
  domain: string;
  headline: string;
  snippet: string;
}

export const examples: Example[] = [
  {
    slug: 'quant-finance',
    domain: 'Quant trading',
    headline: 'Stage-gated rules over a live price stream.',
    snippet: `use deep_causality::*;

let cross = Causaloid::new(|p: &Tick| {
    Ok(p.fast_ma > p.slow_ma)
});
let confirm = Causaloid::new(|p: &Tick| {
    Ok(p.volume > p.median_volume * 1.4)
});

let signal = (cross & confirm).reason_with(&tick_stream)?;
if signal.is_true() { engine.long(ticker, qty)?; }`,
  },
  {
    slug: 'robotics-control',
    domain: 'Robotics',
    headline: 'Context-aware effect propagation under changing spacetime.',
    snippet: `use deep_causality::prelude::*;

let ctx = Context::new("arm")
    .with_node(Spacetime::frame("base"))
    .with_node(Spacetime::frame("tcp"))
    .with_edge("base", "tcp", Twist::default());

let plan = causal_plan(&ctx, &goal)?;
for step in plan.propagate(&world)? {
    arm.execute(step)?;
}`,
  },
  {
    slug: 'observability-sre',
    domain: 'Observability',
    headline: 'Trace a cascading failure back to the originating event.',
    snippet: `use deep_causality::*;

let chain = Causaloid::compose(&[
    db_lag, queue_backlog, retry_storm, frontend_5xx,
]);

let root = chain.reason_back(&incident_window)?
    .first_cause()
    .ok_or(NoCauseFound)?;

alert!("root cause: {}", root.label());`,
  },
  {
    slug: 'bioinformatics-signal',
    domain: 'Bioinformatics',
    headline: 'MRMR feature selection on a noisy expression matrix.',
    snippet: `use deep_causality_algorithms::mrmr;

let selected = mrmr::select(
    &expression_matrix,
    &response,
    /* k = */ 12,
    mrmr::Criterion::Difference,
)?;

for (idx, score) in selected.iter() {
    println!("{:<24} {:.4}", gene_names[*idx], score);
}`,
  },
  {
    slug: 'physics-simulation',
    domain: 'Physics',
    headline: 'Propagate effects under a constraint that mutates each tick.',
    snippet: `use deep_causality::*;
use deep_causality_physics::*;

let mut ctx = Context::from(spacetime);
for step in sim.steps() {
    ctx.update_constraint(gravity_at(step.t));
    let effect = field.propagate(&ctx, &particles)?;
    integrator.advance(&mut particles, &effect, dt);
}`,
  },
  {
    slug: 'policy-compliance',
    domain: 'Policy & ethos',
    headline: 'Verify operational rules at runtime with the Effect Ethos.',
    snippet: `use deep_causality_ethos::*;

let ethos = Ethos::new()
    .forbid(rule::pii_leaves_region)
    .require(rule::audit_log_present)
    .require(rule::approval_when_over(10_000));

ethos.verify(&action)
    .map_err(|v| reject_with(v.violations()))?;`,
  },
];
