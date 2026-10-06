# GPS Navigation Example

A four-stage `CausalFlow` chain with no state and the route as its context
propagates GPS, speed, and fuel-efficiency uncertainty through a
route-planning pipeline.

## Pipeline

```
CausalFlow::value(start)
    .context(route_context()?)
    .try_step_with(distance_stage)   // Stage 1: position noise → distance (mi)
    .try_step_with(time_stage)       // Stage 2: distance + speed noise → travel time (min)
    .try_step_with(route_stage)      // Stage 3: compare against alternative route
    .try_step_with(fuel_stage)       // Stage 4: distance + efficiency noise → fuel (gal)
```

The start position is the chain's value. The route context holds the
destination, the parameters of the speed, traffic, fuel-efficiency and
fuel-on-hand distributions, the route and trip distances, the lateness
threshold, the safe fuel range and the fuel probability the trip requires, one
`Data<f64>` contextoid per quantity. Stage 1 receives the start position,
every later stage the previous stage's `Uncertain<f64>`; each stage also
receives the route context, builds its distributions from it, and returns the
next `Uncertain<f64>`. A missing context, a missing
contextoid or a failed sampling call short-circuits the chain into its error
channel.

## What the example demonstrates

- **`Uncertain<f64>` construction:** `normal`, `uniform`, `point`.
- **Arithmetic and non-linear transforms:** `+`, `-`, `*`, `/`, unary `-`,
  `map` for `sqrt`.
- **Statistical summaries:** `expected_value`, `standard_deviation`.
- **Uncertain comparisons:** `lt_uncertain`, `gt_uncertain`, `greater_than`,
  `within_range`.
- **Conditional reasoning:** `Uncertain::conditional`,
  `implicit_conditional`, `probability_exceeds`.
- **Monadic chaining:** four `try_step_with` calls on `CausalFlow`; each stage
  takes the carried value, the unit state and `Option<&RouteContext>`, and
  returns `Result<Uncertain<f64>, CausalityError>`.

## How to run

```bash
cargo run -p causal_uncertain_examples --example gps_navigation
```
