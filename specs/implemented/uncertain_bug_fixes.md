# Summary
- **Context**: The `take_samples()` method is used to draw multiple samples from uncertain values for statistical analysis, particularly for probability estimation via `estimate_probability_exceeds()` and `estimate_probability()`.
- **Bug**: `take_samples()` uses deterministic sequential indices (0, 1, 2, ..., n-1) instead of random indices, causing it to return identical samples every time it's called with the same parameters.
- **Actual vs. expected**: The method returns the same cached samples on every invocation instead of drawing fresh random samples from the probability distribution.
- **Impact**: Statistical estimations like `estimate_probability_exceeds()` and `estimate_probability()` return deterministic results instead of Monte Carlo estimates, making them unreliable for repeated calls and unable to provide uncertainty quantification.

# Code with bug

In `deep_causality_uncertain/src/types/uncertain/uncertain_sampling.rs`:

```rust
pub fn take_samples(&self, n: usize) -> Result<Vec<f64>, UncertainError> {
    (0..n).map(|i| self.sample_with_index(i as u64)).collect() // <-- BUG 🔴 Uses sequential indices instead of random
}
```

And for bool:

```rust
pub fn take_samples(&self, n: usize) -> Result<Vec<bool>, UncertainError> {
    (0..n).map(|i| self.sample_with_index(i as u64)).collect() // <-- BUG 🔴 Uses sequential indices instead of random
}
```

The bug is that both implementations use `i as u64` (sequential: 0, 1, 2, ..., n-1) as the sample index, rather than random indices like `sample()` does:

```rust
pub fn sample(&self) -> Result<f64, UncertainError> {
    let sample_index = deep_causality_rand::rng().random::<u64>(); // <-- Correct: uses random index
    self.sample_with_index(sample_index)
}
```

# Evidence

## Example

Consider an `Uncertain::uniform(0.0, 100.0)` distribution. When calling `take_samples(10)`:

**First call:**
- Indices used: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
- Results (for example): [10.58, 12.13, 53.07, 38.35, 18.08, 62.56, 36.41, 60.13, 68.89, 74.08]
- These values are cached with keys: [(id, 0), (id, 1), ..., (id, 9)]

**Second call to `take_samples(10)`:**
- Indices used: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] (same as before)
- Results: [10.58, 12.13, 53.07, 38.35, 18.08, 62.56, 36.41, 60.13, 68.89, 74.08] (identical!)
- All values retrieved from cache

**Expected behavior:** Each call should use random indices (e.g., [18446744073709551615, 7382049234, ...]) and return different samples, demonstrating proper Monte Carlo sampling with natural variation.

## Failing test

### Test script

```rust
/*
 * Test to demonstrate the bug in take_samples()
 * This test shows that take_samples() always returns the same values
 * for the same Uncertain object because it uses deterministic indices.
 */

use deep_causality_uncertain::Uncertain;

fn main() {
    // Create a uniform distribution (non-deterministic)
    let u = Uncertain::uniform(0.0, 100.0);

    // Take 10 samples twice
    let samples1 = u.take_samples(10).unwrap();
    let samples2 = u.take_samples(10).unwrap();

    println!("First call to take_samples(10):");
    for (i, val) in samples1.iter().enumerate() {
        println!("  Sample {}: {}", i, val);
    }

    println!("\nSecond call to take_samples(10):");
    for (i, val) in samples2.iter().enumerate() {
        println!("  Sample {}: {}", i, val);
    }

    // These should be different (random samples from a uniform distribution)
    // but they will be IDENTICAL because take_samples uses indices 0..10 both times
    println!("\nAre they identical? {}", samples1 == samples2);
    assert_ne!(samples1, samples2, "Bug: take_samples returns identical values on repeated calls");

    // For comparison, let's use sample() which uses random indices
    println!("\nUsing sample() (random indices):");
    let random_samples: Vec<f64> = (0..10)
        .map(|_| u.sample().unwrap())
        .collect();
    for (i, val) in random_samples.iter().enumerate() {
        println!("  Sample {}: {}", i, val);
    }
}
```

### Test output

```
First call to take_samples(10):
  Sample 0: 10.582431565369554
  Sample 1: 12.133595750760895
  Sample 2: 53.065501481943286
  Sample 3: 38.352123022021665
  Sample 4: 18.07559799559133
  Sample 5: 62.56215089912261
  Sample 6: 36.41416669468358
  Sample 7: 60.12669749578308
  Sample 8: 68.88759187507623
  Sample 9: 74.07987793840039

Second call to take_samples(10):
  Sample 0: 10.582431565369554
  Sample 1: 12.133595750760895
  Sample 2: 53.065501481943286
  Sample 3: 38.352123022021665
  Sample 4: 18.07559799559133
  Sample 5: 62.56215089912261
  Sample 6: 36.41416669468358
  Sample 7: 60.12669749578308
  Sample 8: 68.88759187507623
  Sample 9: 74.07987793840039

Are they identical? true

Using sample() (random indices):
  Sample 0: 56.78532398889811
  Sample 1: 91.57552721966582
  Sample 2: 9.081261306746491
  Sample 3: 11.400212107144924
  Sample 4: 27.65057015850889
  Sample 5: 57.76500594850032
  Sample 6: 98.06631168389994
  Sample 7: 81.76033125554028
  Sample 8: 61.955167461891094
  Sample 9: 43.299094372924564
```

The test clearly shows that `take_samples(10)` returns identical values on repeated calls, while using `sample()` directly (which uses random indices) produces the expected variation.

## Failing test for estimate_probability_exceeds

### Test script

```rust
/*
 * Test to demonstrate the bug in estimate_probability_exceeds()
 * This test shows that estimate_probability_exceeds() always returns
 * the same value when called multiple times because take_samples()
 * uses deterministic indices.
 */

use deep_causality_uncertain::Uncertain;

fn main() {
    // Create a normal distribution
    let u = Uncertain::normal(0.0, 1.0);

    // Call estimate_probability_exceeds multiple times with the same parameters
    println!("Calling estimate_probability_exceeds(0.0, 100) multiple times:");
    let mut estimates = Vec::new();
    for i in 1..=10 {
        let prob = u.estimate_probability_exceeds(0.0, 100).unwrap();
        println!("  Iteration {}: {}", i, prob);
        estimates.push(prob);
    }

    // Check if all estimates are identical (bug behavior)
    let all_identical = estimates.windows(2).all(|w| w[0] == w[1]);
    println!("\nAll estimates identical? {}", all_identical);
    assert!(!all_identical, "Bug: estimate_probability_exceeds returns identical values on repeated calls");

    // For comparison, let's manually sample using random indices
    println!("\nFor comparison, manual sampling with random indices:");
    for i in 1..=10 {
        let samples: Vec<f64> = (0..100)
            .map(|_| u.sample().unwrap())
            .collect();
        let count = samples.iter().filter(|&&s| s > 0.0).count();
        let prob = count as f64 / 100.0;
        println!("  Iteration {}: {}", i, prob);
    }
    println!("\nNotice the variation in the manual sampling - this is expected Monte Carlo behavior");
}
```

### Test output

```
Calling estimate_probability_exceeds(0.0, 100) multiple times:
  Iteration 1: 0.54
  Iteration 2: 0.54
  Iteration 3: 0.54
  Iteration 4: 0.54
  Iteration 5: 0.54
  Iteration 6: 0.54
  Iteration 7: 0.54
  Iteration 8: 0.54
  Iteration 9: 0.54
  Iteration 10: 0.54

All estimates identical? true

For comparison, manual sampling with random indices:
  Iteration 1: 0.51
  Iteration 2: 0.4
  Iteration 3: 0.56
  Iteration 4: 0.52
  Iteration 5: 0.47
  Iteration 6: 0.49
  Iteration 7: 0.5
  Iteration 8: 0.53
  Iteration 9: 0.53
  Iteration 10: 0.49

Notice the variation in the manual sampling - this is expected Monte Carlo behavior
```

This demonstrates that `estimate_probability_exceeds()` returns exactly `0.54` on every single call, which is incorrect for Monte Carlo estimation. The expected behavior is shown in the manual sampling section, where different calls produce slightly different estimates (0.40 to 0.56), which is the natural variation in Monte Carlo methods.

# Full context

The `take_samples()` method is a core component of the uncertain value system used for statistical analysis. It's called by several key methods:

1. **`estimate_probability_exceeds()`** in `deep_causality_uncertain/src/types/uncertain/uncertain_f64.rs`: Uses `take_samples()` to estimate the probability that a value exceeds a threshold through Monte Carlo sampling:
   ```rust
   pub fn estimate_probability_exceeds(
       &self,
       threshold: f64,
       num_samples: usize,
   ) -> Result<f64, UncertainError> {
       let samples = self.take_samples(num_samples)?;
       let count = samples.iter().filter(|&&s| s > threshold).count();
       Ok(count as f64 / num_samples as f64)
   }
   ```

2. **`estimate_probability()`** in `deep_causality_uncertain/src/types/uncertain/uncertain_bool.rs`: Uses `take_samples()` to estimate the probability a boolean condition is true:
   ```rust
   pub fn estimate_probability(&self, num_samples: usize) -> Result<f64, UncertainError> {
       let samples = self.take_samples(num_samples)?;
       if samples.is_empty() {
           Ok(0.0)
       } else {
           let true_count = samples.iter().filter(|&&x| x).count();
           Ok(true_count as f64 / samples.len() as f64)
       }
   }
   ```

3. **Direct usage**: Users can call `take_samples()` directly to analyze uncertain distributions.

The caching mechanism (`GlobalSampleCache`) is designed to optimize performance by storing previously computed samples indexed by `(uncertain_id, sample_index)`. The `sample()` method correctly uses random indices to ensure different samples are drawn each time. However, `take_samples()` was implemented with sequential indices (0..n), which means:

- All samples drawn by `take_samples(n)` are permanently cached under indices 0 through n-1
- Subsequent calls to `take_samples(n)` retrieve these same cached values
- Statistical estimation methods that depend on `take_samples()` return deterministic results instead of Monte Carlo estimates

The sampling chain is:
```
take_samples(n) -> sample_with_index(i) -> with_global_cache() -> SequentialSampler::sample()
```

Where `SequentialSampler` draws random samples from the underlying probability distributions. The problem is that `take_samples()` uses deterministic indices, so it always asks the cache for the same pre-computed samples.

# Why has this bug gone undetected?

The bug has gone undetected for several reasons:

1. **Tests use deterministic distributions**: All existing tests for `take_samples()` use `Uncertain::point()` distributions, which always return the same value regardless of the sample index:
   ```rust
   fn test_f64_take_samples() {
       let u = Uncertain::<f64>::point(88.0);  // Deterministic distribution
       let samples = u.take_samples(10).unwrap();
       assert_eq!(samples.len(), 10);
       assert!(samples.iter().all(|&s| s == 88.0));  // All values are 88.0
   }
   ```
   This test passes whether indices are random or sequential because point distributions are constant.

2. **Wide tolerance in probability tests**: The test `test_estimate_probability_exceeds_normal()` uses 10,000 samples and a tolerance of 0.05 (5%), which is loose enough that the deterministic sampling still produces estimates within the acceptable range:
   ```rust
   let prob = u.estimate_probability_exceeds(threshold, num_samples).unwrap();
   assert!((prob - 0.5).abs() < 0.05, "Expected probability near 0.5, got {}", prob);
   ```
   With 10,000 samples, the law of large numbers ensures convergence to approximately the correct probability even with deterministic indices, so the test passes despite the underlying bug.

3. **Single invocations in tests**: Tests never call `estimate_probability_exceeds()` or `take_samples()` multiple times on the same object to verify that different invocations produce different estimates. Monte Carlo methods should show natural variation between runs, but this behavior was never tested.

4. **Caching masks the issue**: The caching system works correctly and efficiently. The bug is subtle - it's not that caching is broken, but that `take_samples()` is using a poor indexing strategy that defeats the purpose of Monte Carlo sampling. The cache returns the correct values for the requested indices; the problem is that the indices themselves are inappropriate.

5. **Documentation doesn't specify the behavior**: Neither the code comments nor the API documentation explicitly state whether `take_samples()` should return the same or different samples on repeated calls, leaving this behavior ambiguous.

# Recommended fix

Change `take_samples()` to use random indices instead of sequential ones:

```rust
pub fn take_samples(&self, n: usize) -> Result<Vec<f64>, UncertainError> {
    (0..n)
        .map(|_| {
            let sample_index = deep_causality_rand::rng().random::<u64>(); // <-- FIX 🟢 Use random index
            self.sample_with_index(sample_index)
        })
        .collect()
}
```

Apply the same fix to the `Uncertain<bool>` implementation.

This change makes `take_samples()` behave consistently with `sample()`, using random indices to draw fresh samples from the distribution on each call, enabling proper Monte Carlo estimation with natural statistical variation.

## Alternative consideration

If deterministic, reproducible sampling is desired for testing or debugging purposes, consider adding a separate method like `take_samples_deterministic(n, seed)` that explicitly uses sequential or seeded indices, while keeping `take_samples()` as the random-sampling method for production statistical analysis.
