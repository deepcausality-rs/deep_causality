# Summary
- **Context**: `UniformU32` is a random number sampler that generates uniformly distributed `u32` values from a specified range, part of the `deep_causality_rand` crate's distribution system.
- **Bug**: The `new_inclusive` method panics when the upper bound equals `u32::MAX` due to integer overflow when computing `high_val + 1`.
- **Actual vs. expected**: The method panics in debug mode (and wraps to 0 in release mode causing incorrect sampling), but should handle the full range `[0, u32::MAX]` correctly.
- **Impact**: Any code attempting to sample from ranges including `u32::MAX` will panic in debug mode or produce incorrect samples in release mode, making it impossible to uniformly sample from the complete `u32` value space.

# Code with bug
```rust
fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
where
    B1: SampleBorrow<Self::X> + Sized,
    B2: SampleBorrow<Self::X> + Sized,
{
    let low_val = *low.borrow();
    let high_val = *high.borrow();
    if low_val >= high_val {
        return Err(UniformDistributionError::InvalidRange);
    }
    Ok(UniformU32 {
        low: low_val,
        high: high_val + 1,  // <-- BUG 🔴 Overflows when high_val == u32::MAX
    }) // Inclusive range
}
```

# Evidence

## Example

Consider creating an inclusive range from 100 to `u32::MAX`:

1. User calls: `Uniform::<u32>::new_inclusive(100, u32::MAX)`
2. This reaches `UniformU32::new_inclusive(100, u32::MAX)`
3. The code sets `high_val = 4294967295` (which is `u32::MAX`)
4. It attempts to compute `high_val + 1`
5. Since `u32::MAX + 1` overflows:
    - **Debug mode**: Panics with "attempt to add with overflow"
    - **Release mode**: Wraps to `0`, creating internal state `{ low: 100, high: 0 }`
6. In release mode, sampling would compute `high - low = 0 - 100`, causing further underflow/wraparound issues

This makes it impossible to sample from ranges like `[0, u32::MAX]`, `[u32::MAX - 10, u32::MAX]`, or even the single-value range `[u32::MAX, u32::MAX]`.

## Failing test

### Test script
```rust
/*
 * Test to demonstrate the overflow bug in UniformU32::new_inclusive
 */

use deep_causality_rand::*;

struct MockIntRng {
    val: u32,
}

impl RngCore for MockIntRng {
    fn next_u32(&mut self) -> u32 {
        self.val
    }
    fn next_u64(&mut self) -> u64 {
        self.val as u64
    }
    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!()
    }
}

impl Rng for MockIntRng {}

fn main() {
    println!("Demonstrating overflow bug in UniformU32::new_inclusive\n");

    // This will panic in debug mode due to overflow at line 47
    println!("Attempting: Uniform::<u32>::new_inclusive(100, u32::MAX)");
    println!("Expected: Should create a sampler for range [100, {}]", u32::MAX);
    println!("Actual: Will panic due to overflow when computing high_val + 1\n");

    match std::panic::catch_unwind(|| {
        Uniform::<u32>::new_inclusive(100, u32::MAX)
    }) {
        Ok(result) => {
            match result {
                Ok(_) => println!("✓ Unexpectedly succeeded (release mode?)"),
                Err(e) => println!("✗ Failed with error: {:?}", e),
            }
        }
        Err(_) => {
            println!("✗ PANICKED as expected in debug mode!");
            println!("\nThis panic occurs at uniform_u32.rs:47");
            println!("The code attempts: high: high_val + 1");
            println!("When high_val = u32::MAX, this overflows!");
        }
    }
}
```

### Test output
```
Demonstrating overflow bug in UniformU32::new_inclusive

Attempting: Uniform::<u32>::new_inclusive(100, u32::MAX)
Expected: Should create a sampler for range [100, 4294967295]
Actual: Will panic due to overflow when computing high_val + 1


thread 'main' (6587) panicked at /home/user/deep_causality/deep_causality_rand/src/types/distr/uniform/uniform_u32.rs:47:19:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
✗ PANICKED as expected in debug mode!

This panic occurs at uniform_u32.rs:47
The code attempts: high: high_val + 1
When high_val = u32::MAX, this overflows!
```

## Inconsistency within the codebase

### Reference code
`deep_causality_rand/src/types/distr/uniform/mod.rs` (lines 98-119) - `UniformFloat::new_inclusive`:
```rust
fn new_inclusive<B1, B2>(low_b: B1, high_b: B2) -> Result<Self, UniformDistributionError>
where
    B1: SampleBorrow<Self::X> + Sized,
    B2: SampleBorrow<Self::X> + Sized,
{
    let low = *low_b.borrow();
    let high = *high_b.borrow();
    if !(low.is_finite() && high.is_finite()) {
        return Err(UniformDistributionError::NonFinite);
    }
    if low > high {  // <-- Correctly uses > (not >=)
        return Err(UniformDistributionError::EmptyRange);
    }

    let max_rand = F::one() - F::epsilon();
    let scale = (high - low) / max_rand;
    if !scale.is_finite() {
        return Err(UniformDistributionError::NonFinite);
    }

    Ok(UniformFloat { low, scale })  // <-- No +1 operation that could overflow
}
```

### Current code
`deep_causality_rand/src/types/distr/uniform/uniform_u32.rs` (lines 35-49):
```rust
fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
where
    B1: SampleBorrow<Self::X> + Sized,
    B2: SampleBorrow<Self::X> + Sized,
{
    let low_val = *low.borrow();
    let high_val = *high.borrow();
    if low_val >= high_val {  // <-- Uses >=, rejects equal bounds
        return Err(UniformDistributionError::InvalidRange);
    }
    Ok(UniformU32 {
        low: low_val,
        high: high_val + 1,  // <-- Unchecked addition that can overflow
    })
}
```

### Contradiction
The integer implementation has two issues that the float implementation avoids:

1. **Overflow vulnerability**: The integer version uses unchecked addition (`high_val + 1`) which panics when `high_val == u32::MAX`. The float version uses a scaling approach that doesn't require incrementing the upper bound.

2. **Incorrect validation** (secondary bug): The integer version incorrectly rejects equal bounds with `low_val >= high_val`, while the float version correctly uses `low > high`. For an inclusive range like `[5, 5]`, it should be valid to sample the single value 5, but the current code rejects it.

# Full context

The `UniformU32` struct is part of the random number generation system in `deep_causality_rand`. It implements the `UniformSampler` trait to provide uniform distribution sampling for `u32` values.

The sampler is used through the `Uniform<u32>` distribution type, which is the public API exposed to users. When users call `Uniform::<u32>::new_inclusive(low, high)`, it delegates to `UniformU32::new_inclusive`.

The `new_inclusive` method is specifically designed to create samplers for closed ranges `[low, high]` where both bounds are inclusive. This is documented in the parent module's docstring, which states: "Create a new `Uniform` instance, which samples uniformly from the closed range `[low, high]` (inclusive). Fails if `low > high`."

After construction, the `sample` method generates random values by computing `self.low + (rng.next_u32() % (self.high - self.low))`. The implementation assumes that `self.high` represents one past the maximum value to sample (hence the `+ 1` in `new_inclusive`), so the range calculation works correctly.

The bug affects any code that needs to:
- Sample from the full `u32` range `[0, u32::MAX]`
- Sample from ranges near the upper bound like `[u32::MAX - 1000, u32::MAX]`
- Sample a single maximum value `[u32::MAX, u32::MAX]`

These are legitimate use cases in applications dealing with:
- Hash values or checksums (often u32)
- Network protocols using full 32-bit address spaces
- Cryptographic applications requiring full-range sampling
- Testing and simulation code that needs boundary value coverage

## External documentation

The trait documentation in `deep_causality_rand/src/types/distr/uniform/mod.rs` specifies:

```rust
/// Create a new `Uniform` instance, which samples uniformly from the closed
/// range `[low, high]` (inclusive).
///
/// Fails if `low > high`, or if `low`, `high` or the range `high - low` is
/// non-finite. In release mode, only the range is checked.
```

Note that it states "Fails if `low > high`" (strict inequality), not "Fails if `low >= high`". This confirms that equal bounds should be accepted.

The Rust standard library's documentation for inclusive ranges `RangeInclusive` also supports empty ranges where `start == end` contains exactly one element, reinforcing that `[5, 5]` is a valid inclusive range.

# Why has this bug gone undetected?

This bug has remained undetected for several reasons:

1. **Infrequent edge case usage**: Most code using uniform distributions samples from moderate ranges (e.g., `[0, 100]`, `[1, 1000]`) and rarely needs to sample near or at `u32::MAX`. The maximum value edge case is uncommon in typical application code.

2. **Testing gaps**: The existing test suite in `deep_causality_rand/tests/types/dist/uniform/uniform_tests.rs` uses small values in its test cases (e.g., ranges like `[10, 20]`). There are no tests attempting to create ranges that include `u32::MAX`.

3. **Debug mode not always used**: Many users may build in release mode where the overflow doesn't panic but silently wraps. The resulting incorrect behavior might be attributed to randomness rather than recognized as a deterministic bug.

4. **Validation prevents one symptom**: The incorrect `>=` validation (secondary bug) actually prevents the most obvious test case `[u32::MAX, u32::MAX]` from reaching the overflow, masking the overflow bug in that specific scenario. However, `[u32::MAX - 1, u32::MAX]` still triggers the overflow.

5. **Similar types not affected**: Types like `u64` and `usize` have the same bug, but their maximum values are so large that they're even less likely to be used in practice. The `u64::MAX` case would require sampling from an enormously large space.

6. **Recent code**: The implementation was added relatively recently (commit 9a5d6d77) as part of migrating random number generation to the `deep_causality_rand` crate, and hasn't had extensive production usage yet.

# Recommended fix

Use checked arithmetic to detect overflow and handle it with a special case. The fix should:

1. Use `checked_add` to safely attempt `high_val + 1`
2. When overflow occurs (returns `None`), use a special representation
3. Update validation to use `>` instead of `>=` to allow equal bounds
4. Handle the special case in the `sample` method

Example approach (pseudocode):
```rust
fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
where
    B1: SampleBorrow<Self::X> + Sized,
    B2: SampleBorrow<Self::X> + Sized,
{
    let low_val = *low.borrow();
    let high_val = *high.borrow();
    if low_val > high_val {  // <-- FIX 🟢 Changed from >= to >
        return Err(UniformDistributionError::InvalidRange);
    }

    // Use checked_add to avoid overflow
    let range_end = high_val.checked_add(1).unwrap_or(0);  // <-- FIX 🟢 0 as special case

    Ok(UniformU32 {
        low: low_val,
        high: range_end,
    })
}

fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
    if self.high == 0 && self.low > 0 {
        // Special case: range wraps around, meaning we want [low, u32::MAX]
        // Handle this specially
    }
    self.low + (rng.next_u32() % (self.high - self.low))
}
```

Alternatively, follow the Rust `rand` crate's approach of using `range = 0` as a special sentinel value to represent the full range, as documented in their implementation.

# Related bugs

The same bug exists in:
- `deep_causality_rand/src/types/distr/uniform/uniform_u64.rs` - Same overflow issue with `u64::MAX`
- `deep_causality_rand/src/types/distr/uniform/uniform_usize.rs` - Same overflow issue with `usize::MAX`

All three files have identical implementations and should be fixed together to maintain consistency.

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
