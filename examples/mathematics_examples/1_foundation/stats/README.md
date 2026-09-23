# Foundation: `deep_causality_stats`

The statistics of a sample, and the distributions those statistics describe. One crate holds
both, because a distribution *is* its density and its moments.

`deep_causality_rand` holds the entropy: a source of bits, the raw machine word, the Boolean
draw, the Sobol sequence, and a value uniform over a range.

Every function is generic over the working scalar and returns a `Result`, so an empty slice or
a length mismatch arrives as a value the caller handles.

| Example | What it covers | Command |
|---|---|---|
| [descriptive_and_distributions.rs](descriptive_and_distributions.rs) | `mean` / `variance` / `std_dev`, `pearson` correlation, and sampling `Normal` and `StandardUniform` then recovering their parameters from the draws | `cargo run -p mathematics_examples --example descriptive_and_distributions_examples` |
