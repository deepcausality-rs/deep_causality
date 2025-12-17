# Summary
- **Context**: The `random_bool` function in the `Rng` trait generates random boolean values with a given probability `p` in the range [0.0, 1.0].
- **Bug**: When the random value equals the maximum possible value (u64::MAX) and `p=1.0`, the function returns `false` instead of `true`.
- **Actual vs. expected**: The function uses a strict less-than comparison (`<`) instead of less-than-or-equal (`<=`), causing it to return `false` when the generated probability equals `p`, even though `p=1.0` should always return `true`.
- **Impact**: Users cannot reliably generate events that should always occur (probability 1.0), violating the documented contract and breaking applications that depend on deterministic behavior for `p=1.0`.

# Code with bug
```rust
fn random_bool(&mut self, p: f64) -> bool {
    if !(0.0..=1.0).contains(&p) {
        panic!("p={} is outside range [0.0, 1.0]", p);
    }
    self.next_u64() as f64 / (u64::MAX as f64) < p  // <-- BUG 🔴 should use <= instead of <
}
```
Location: `deep_causality_rand/src/traits/rng.rs:41-46`

# Evidence

## Example

Consider what happens when `next_u64()` returns `u64::MAX` (18,446,744,073,709,551,615):

1. `next_u64()` returns `u64::MAX` = 18,446,744,073,709,551,615
2. Convert to `f64`: `u64::MAX as f64` = 18,446,744,073,709,552,000.0 (note: precision loss due to f64 representation)
3. Calculate probability: `18446744073709552000.0 / 18446744073709552000.0 = 1.0`
4. Compare with `p=1.0`: `1.0 < 1.0` = `false`
5. Function returns `false` (incorrect!)

Expected behavior: When `p=1.0`, the function should **always** return `true`.

## Inconsistency with own spec

### Reference spec

The existing test suite explicitly documents the expected behavior:

From `deep_causality_rand/tests/types/rand/std_rng_tests.rs:131-134`:
```rust
#[test]
fn test_xoshiro256_random_bool_edge_cases() {
    let mut rng = StdRng::new_with_u64(12345);
    assert!(rng.random_bool(1.0), "Should always be true for p=1.0");
    assert!(!rng.random_bool(0.0), "Should always be false for p=0.0");
    // ... panic tests ...
}
```

From `deep_causality_rand/tests/types/rand/os_random_rng_tests.rs:145-148`:
```rust
#[test]
fn test_os_random_rng_random_bool_edge_cases() {
    let mut rng = OsRandomRng::new();
    assert!(rng.random_bool(1.0), "Should always be true for p=1.0");
    assert!(!rng.random_bool(0.0), "Should always be false for p=0.0");
    // ... panic tests ...
}
```

### Current code

```rust
fn random_bool(&mut self, p: f64) -> bool {
    if !(0.0..=1.0).contains(&p) {
        panic!("p={} is outside range [0.0, 1.0]", p);
    }
    self.next_u64() as f64 / (u64::MAX as f64) < p
}
```

### Contradiction

The tests assert that `random_bool(1.0)` **"should always be true for p=1.0"**, but the implementation uses a strict less-than comparison (`<`). When the generated probability value equals exactly `1.0`, the comparison `1.0 < 1.0` evaluates to `false`, violating the documented contract.

## Failing test

### Test script
```rust
/*
 * Test to demonstrate the random_bool bug with p=1.0
 */

#[cfg(test)]
mod tests {
    use deep_causality_rand::{Rng, RngCore};

    // Mock RNG that returns u64::MAX, which should produce probability 1.0
    struct MaxRng;

    impl RngCore for MaxRng {
        fn next_u64(&mut self) -> u64 {
            u64::MAX
        }
    }

    impl Rng for MaxRng {}

    #[test]
    fn test_random_bool_edge_case_max_u64() {
        let mut rng = MaxRng;
        
        // When next_u64() returns u64::MAX, random_bool(1.0) should return true
        // because the probability should be evaluated as >= p, not just < p
        let result = rng.random_bool(1.0);
        assert!(result, "random_bool(1.0) should always return true, even when next_u64() = u64::MAX");
    }

    #[test]
    fn test_random_bool_should_use_inclusive_comparison() {
        let mut rng = MaxRng;
        
        // The calculation: (u64::MAX as f64) / (u64::MAX as f64) = 1.0
        // With p=1.0, the condition should be: 1.0 <= 1.0 (true)
        // But current implementation uses: 1.0 < 1.0 (false)
        
        let random_val = u64::MAX as f64;
        let divisor = u64::MAX as f64;
        let probability = random_val / divisor;
        
        println!("probability = {}", probability);
        println!("probability < 1.0 = {}", probability < 1.0);
        println!("probability <= 1.0 = {}", probability <= 1.0);
        
        assert!(rng.random_bool(1.0), "Should return true when generated probability equals p");
    }
}

fn main() {
    println!("Run with: cargo test");
}
```

### Test output
```
running 2 tests
test tests::test_random_bool_edge_case_max_u64 ... FAILED
test tests::test_random_bool_should_use_inclusive_comparison ... FAILED

failures:

---- tests::test_random_bool_edge_case_max_u64 stdout ----

thread 'tests::test_random_bool_edge_case_max_u64' panicked at test_random_bool_bug_proper.rs:27:9:
random_bool(1.0) should always return true, even when next_u64() = u64::MAX

---- tests::test_random_bool_should_use_inclusive_comparison stdout ----
probability = 1
probability < 1.0 = false
probability <= 1.0 = true

thread 'tests::test_random_bool_should_use_inclusive_comparison' panicked at test_random_bool_bug_proper.rs:46:9:
Should return true when generated probability equals p
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::test_random_bool_edge_case_max_u64
    tests::test_random_bool_should_use_inclusive_comparison

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

# Full context

The `random_bool` function is a core method in the `Rng` trait, which is the primary interface for random number generation in the `deep_causality_rand` crate. This trait is implemented by all RNG types in the crate, including:

- `StdRng` (Xoshiro256 PRNG) - used for testing and development
- `OsRandomRng` - hardware-backed RNG using OS randomness
- Custom user implementations that override `RngCore`

The `Rng` trait is defined in `deep_causality_rand/src/traits/rng.rs` and provides several convenience methods built on top of `RngCore::next_u64()`. The `random_bool` method generates boolean values with a specified probability, which is useful for:

- Probabilistic algorithms and simulations
- Monte Carlo methods
- Stochastic processes
- Statistical sampling

The function accepts a probability parameter `p` in the range [0.0, 1.0], where:
- `p=0.0` should always return `false`
- `p=1.0` should always return `true`
- `p=0.5` should return `true` approximately 50% of the time

The implementation generates a random `u64`, converts it to a floating-point value in [0.0, 1.0], and compares it against the probability threshold. However, the strict less-than comparison causes the edge case failure when the generated value equals the threshold.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Extremely rare occurrence**: The bug only manifests when `next_u64()` returns exactly `u64::MAX`, which has a probability of 1 in 18,446,744,073,709,551,616. For a typical PRNG, this is astronomically unlikely to occur during normal testing.

2. **Existing tests use real RNGs**: The edge case tests at `std_rng_tests.rs:131-134` and `os_random_rng_tests.rs:145-148` use actual RNG implementations (`StdRng` and `OsRandomRng`) rather than deterministic mocks. These RNGs are statistically unlikely to generate `u64::MAX` during the brief test execution, so the tests pass consistently despite the bug.

3. **No deterministic edge case testing**: The test suite doesn't include tests with mock RNGs that deterministically return boundary values like `u64::MAX` to verify edge case behavior.

4. **Floating-point precision masking**: Due to f64's limited precision, many large u64 values map to the same f64 representation. The specific value `u64::MAX` is one of the few that maps to exactly `1.0` after division, making this edge case even rarer.

5. **Most use cases work correctly**: For all probability values except exactly `p=1.0` with a random value that produces exactly `1.0`, the function works as expected. The vast majority of practical uses (like `p=0.5` for coin flips) are unaffected.

# Recommended fix

Change the comparison operator from `<` to `<=`:

```rust
fn random_bool(&mut self, p: f64) -> bool {
    if !(0.0..=1.0).contains(&p) {
        panic!("p={} is outside range [0.0, 1.0]", p);
    }
    self.next_u64() as f64 / (u64::MAX as f64) <= p  // <-- FIX 🟢 use <= instead of <
}
```

This ensures that:
- When `p=1.0` and the generated probability is `1.0`, the comparison `1.0 <= 1.0` returns `true` ✓
- When `p=0.0`, the function still correctly returns `false` since only a generated value of exactly `0.0` would return `true`, and `next_u64()` cannot return a value that produces exactly `0.0` after division
- All intermediate probability values continue to work correctly
