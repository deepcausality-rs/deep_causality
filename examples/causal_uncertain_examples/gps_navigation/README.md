# GPS Navigation Example

A stateless four-stage `PropagatingEffect` chain that propagates GPS, speed,
and fuel-efficiency uncertainty through a planning pipeline.

## Pipeline

```
PropagatingEffect::pure(start)
    .bind(distance_stage)   // Stage 1: position noise → distance (mi)
    .bind(time_stage)       // Stage 2: distance + speed noise → travel time (min)
    .bind(route_stage)      // Stage 3: compare against alternative route
    .bind(fuel_stage)       // Stage 4: distance + efficiency noise → fuel (gal)
```

Each stage takes the previous stage's value out of `CausalEffect::Value`,
computes the next `Uncertain<f64>` quantity, and re-lifts it with
`PropagatingEffect::pure`. The chain short-circuits if any stage receives a
non-`Value` variant.

## What the example demonstrates

- **`Uncertain<f64>` construction:** `normal`, `uniform`, `point`.
- **Arithmetic and non-linear transforms:** `+`, `-`, `*`, `/`, unary `-`,
  `map` for `sqrt`.
- **Statistical summaries:** `expected_value`, `standard_deviation`.
- **Uncertain comparisons:** `lt_uncertain`, `gt_uncertain`, `greater_than`,
  `within_range`.
- **Conditional reasoning:** `Uncertain::conditional`,
  `implicit_conditional`, `probability_exceeds`.
- **Monadic chaining:** four `bind` calls on `PropagatingEffect`; each stage
  is a stateless `CausalEffect<T> -> PropagatingEffect<U>` function.

## How to run

```bash
cargo run -p causal_uncertain_examples --example gps_navigation
```
