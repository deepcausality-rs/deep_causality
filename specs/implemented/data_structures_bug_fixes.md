# Summary
- **Context**: The `storage_array_3d.rs` file implements the `Storage` trait for 3D arrays `[[[T; W]; H]; D]`, providing indexed access to grid-based data structures used throughout the deep_causality codebase.
- **Bug**: The indexing order in the `get` and `set` methods is incorrect - it uses `self[p.y][p.x][p.z]` instead of `self[p.z][p.y][p.x]`.
- **Actual vs. expected**: The implementation maps coordinates as `[height][width][depth]` when it should map as `[depth][height][width]` to match the array type `[[[T; W]; H]; D]` and the semantic meaning of PointIndex fields.
- **Impact**: The bug causes index out-of-bounds panics when using non-cubic grids with distinct dimensions, and silently returns wrong values in other cases, corrupting spatial data operations.

# Code with bug

**File: `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`**

```rust
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
where
    T: Copy,
    [[[T; W]; H]; D]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // <-- BUG 🔴 Wrong indexing order
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.y][p.x][p.z] = elem  // <-- BUG 🔴 Wrong indexing order
    }

    fn height(&self) -> Option<&usize> {
        Some(&H)
    }

    fn depth(&self) -> Option<&usize> {
        Some(&D)
    }

    fn width(&self) -> Option<&usize> {
        Some(&W)
    }
}
```

# Evidence

## Example

Consider a 3D array with distinct dimensions to illustrate the problem:
```rust
const WIDTH: usize = 3;   // X-axis
const HEIGHT: usize = 4;  // Y-axis
const DEPTH: usize = 2;   // Z-axis

let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];
```

The array type `[[[i32; 3]; 4]; 2]` has this structure:
- Outermost array: 2 elements (DEPTH)
- Middle arrays: 4 elements each (HEIGHT)
- Innermost arrays: 3 elements each (WIDTH)
- Access pattern: `storage[depth_idx][height_idx][width_idx]`

To access position (width=1, height=2, depth=1):
- Direct array access: `storage[1][2][1]`
- PointIndex creation: `PointIndex::new3d(x=1, y=2, z=1)` where x=width, y=height, z=depth

**Current buggy behavior:**
```rust
let point = PointIndex::new3d(1, 2, 1);  // x=1, y=2, z=1
// Implementation does: self[p.y][p.x][p.z] = self[2][1][1]
// But depth=2 is out of bounds (DEPTH=2, so valid indices are 0-1)
// Result: PANIC with "index out of bounds: the len is 2 but the index is 2"
```

**Expected correct behavior:**
```rust
let point = PointIndex::new3d(1, 2, 1);  // x=1, y=2, z=1
// Should do: self[p.z][p.y][p.x] = self[1][2][1]
// This correctly maps to storage[depth=1][height=2][width=1]
```

## Inconsistency with own spec / docstring

### Reference spec

From `deep_causality_data_structures/src/grid_type/mod.rs`:
```rust
/// - `W`: Width (X-axis).
/// - `H`: Height (Y-axis).
/// - `D`: Depth (Z-axis).
```

From `deep_causality_data_structures/src/grid_type/storage.rs`:
```rust
/// Returns the width (X-axis) dimension of the storage, if defined.
fn width(&self) -> Option<&usize>;

/// Returns the height (Y-axis) dimension of the storage, if defined.
fn height(&self) -> Option<&usize>;

/// Returns the depth (Z-axis) dimension of the storage, if defined.
fn depth(&self) -> Option<&usize>;
```

From `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`:
```rust
// T Type
// W Width
// H Height
// D Depth
/// Implements `Storage` for 3D arrays `[[[T; W]; H]; D]`
/// indexed along X (width), Y (height), and Z (depth) axes.
```

From `deep_causality_data_structures/src/grid_type/point.rs`:
```rust
pub struct PointIndex {
    pub x: usize,     // Width (based on semantic meaning)
    pub y: usize,     // Height
    pub z: usize,     // Depth
    pub t: usize,     // Time
    point_type: PointIndexType,
}
```

### Current code

```rust
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // Indexes as [height][width][depth]
    }
}
```

### Contradiction

The array type `[[[T; W]; H]; D]` is nested as `[D outer][H middle][W inner]`, meaning the correct access pattern is `array[depth_index][height_index][width_index]`.

According to the spec:
- `p.x` represents the X-axis (Width)
- `p.y` represents the Y-axis (Height)
- `p.z` represents the Z-axis (Depth)

Therefore, the correct indexing should be:
```rust
&self[p.z][p.y][p.x]  // [depth][height][width]
```

But the implementation uses:
```rust
&self[p.y][p.x][p.z]  // [height][width][depth]
```

This violates the documented coordinate system mapping and array structure.

## Inconsistency within the codebase

### Reference code

**File: `deep_causality_data_structures/src/grid_type/storage_array_2d.rs`**
```rust
// T Type
// W Width
// H Height
/// Implements `Storage` for 2D arrays `[[T; W]; H]`
/// indexed along X (width) and Y (height) axes.
impl<T, const W: usize, const H: usize> Storage<T> for [[T; W]; H]
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x]  // Correctly maps [H][W] to [height][width]
    }
}
```

For the 2D case:
- Array type: `[[T; W]; H]` = `[H outer][W inner]`
- Indexing: `self[p.y][p.x]` = `self[height][width]`
- This is **correct**: outer dimension H is accessed by p.y (height), inner dimension W by p.x (width)

### Current code

**File: `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`**
```rust
// T Type
// W Width
// H Height
// D Depth
/// Implements `Storage` for 3D arrays `[[[T; W]; H]; D]`
/// indexed along X (width), Y (height), and Z (depth) axes.
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // Maps [D][H][W] to [height][width][depth]
    }
}
```

For the 3D case:
- Array type: `[[[T; W]; H]; D]` = `[D outer][H middle][W inner]`
- Indexing: `self[p.y][p.x][p.z]` = `self[height][width][depth]`
- This is **incorrect**: should be `self[p.z][p.y][p.x]` to map outer→depth, middle→height, inner→width

### Contradiction

Following the 2D pattern, the 3D implementation should maintain the same principle: outer dimensions are accessed first, inner dimensions last. The 2D implementation correctly accesses `[outer][inner]` as `[p.y][p.x]` because the array is `[[T; W]; H]` = `[H][W]`.

For 3D, the array `[[[T; W]; H]; D]` means `[D][H][W]`, so following the same principle, it should be accessed as `[p.z][p.y][p.x]` to map `[depth][height][width]`.

The current implementation breaks this pattern by using `[p.y][p.x][p.z]`, which incorrectly treats the array as if it were `[[[T; D]; W]; H]`.

## Failing test

### Test script

```rust
/*
 * Unit test demonstrating the indexing bug in storage_array_3d.rs
 */

#[cfg(test)]
mod tests {
    use deep_causality_data_structures::{PointIndex, Storage};

    #[test]
    fn test_3d_array_indexing_with_distinct_dimensions() {
        // Use distinct dimensions to reveal the bug
        const WIDTH: usize = 3;   // X-axis
        const HEIGHT: usize = 4;  // Y-axis
        const DEPTH: usize = 2;   // Z-axis

        let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];

        // Raw array structure: [[[T; W]; H]; D]
        // This means: storage[depth_idx][height_idx][width_idx]

        // Set a value at a specific location using raw array indexing
        // depth=1, height=2, width=1
        storage[1][2][1] = 99;

        // Now access the same location using Storage trait
        // According to PointIndex semantics:
        // - x represents width (X-axis)
        // - y represents height (Y-axis)
        // - z represents depth (Z-axis)
        let point = PointIndex::new3d(
            1,  // x = width = 1
            2,  // y = height = 2
            1   // z = depth = 1
        );

        // This should retrieve the value we just set (99)
        let retrieved = *<[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::get(&storage, point);

        assert_eq!(
            retrieved, 99,
            "Expected to retrieve 99 from position (width=1, height=2, depth=1), but got {}",
            retrieved
        );
    }

    #[test]
    fn test_3d_array_set_and_get_consistency() {
        const WIDTH: usize = 3;
        const HEIGHT: usize = 4;
        const DEPTH: usize = 2;

        let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];

        // Test a specific non-symmetric position
        let point = PointIndex::new3d(2, 3, 1);  // x=2, y=3, z=1
        let test_value = 42;

        // Set using Storage trait
        <[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::set(&mut storage, point, test_value);

        // Get using Storage trait
        let retrieved = *<[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::get(&storage, point);

        // This should work, but let's verify what raw position was actually set
        // Expected: storage[z=1][y=3][x=2] = storage[1][3][2]
        let raw_value = storage[1][3][2];

        assert_eq!(
            retrieved, test_value,
            "Storage trait get/set should be consistent"
        );

        assert_eq!(
            raw_value, test_value,
            "Value should be at storage[depth=1][height=3][width=2], but it's not there"
        );
    }
}
```

### Test output

```
running 2 tests
test tests::test_3d_array_set_and_get_consistency ... FAILED
test tests::test_3d_array_indexing_with_distinct_dimensions ... FAILED

failures:

---- tests::test_3d_array_set_and_get_consistency stdout ----

thread 'tests::test_3d_array_set_and_get_consistency' (7650) panicked at /home/user/deep_causality/deep_causality_data_structures/src/grid_type/storage_array_3d.rs:24:9:
index out of bounds: the len is 2 but the index is 3
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_3d_array_indexing_with_distinct_dimensions stdout ----

thread 'tests::test_3d_array_indexing_with_distinct_dimensions' (7649) panicked at /home/user/deep_causality/deep_causality_data_structures/src/grid_type/storage_array_3d.rs:20:10:
index out of bounds: the len is 2 but the index is 2


failures:
    tests::test_3d_array_indexing_with_distinct_dimensions
    tests::test_3d_array_set_and_get_consistency

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Both tests panic with "index out of bounds" errors because the incorrect indexing tries to use `p.y` (height=2 or 3) as the first index into an array of length DEPTH=2.

# Full context

The `Storage` trait provides a unified interface for accessing multi-dimensional arrays in the deep_causality data structures crate. It's implemented for 1D, 2D, 3D, and 4D arrays to support grid-based spatial computations throughout the deep_causality ecosystem.

The `storage_array_3d.rs` file implements this trait for 3D arrays of the form `[[[T; W]; H]; D]`, where the dimensions represent width, height, and depth respectively. This implementation is used by:

1. **ArrayGrid**: A high-performance tensor-like structure that uses const generics for compile-time optimization. ArrayGrid delegates to the Storage trait implementations for all data access.

2. **Grid**: A wrapper around storage backends that provides a uniform API regardless of dimensionality. The Grid type is used extensively in the deep_causality codebase for spatial reasoning operations.

The bug affects any code that uses 3D grids with non-equal dimensions. The existing tests all use equal dimensions (e.g., `WIDTH=HEIGHT=DEPTH=5`) and symmetric access patterns (e.g., `PointIndex::new3d(1, 1, 1)`), which masks the indexing bug.

When users create grids with distinct dimensions (e.g., `WIDTH=3, HEIGHT=4, DEPTH=2`) and access non-symmetric positions, the bug manifests as either:
- Index out-of-bounds panics when coordinates exceed the wrongly-mapped dimension
- Silent data corruption when the wrong array element is accessed/modified

The same indexing bug exists in `storage_array_4d.rs`, which uses `self[p.y][p.x][p.z][p.t]` instead of the correct `self[p.t][p.z][p.y][p.x]`.

# Why has this bug gone undetected?

The bug has remained undetected because:

1. **All existing tests use symmetric dimensions**: The test suite uses equal values for WIDTH, HEIGHT, and DEPTH (typically all set to 5), making it impossible to detect dimension-specific indexing errors.

2. **Tests use symmetric coordinates**: Test cases like `PointIndex::new3d(1, 1, 1)` or `PointIndex::new3d(1, 2, 3)` work correctly or appear to work because:
    - With equal dimensions, out-of-bounds conditions are less likely
    - The examples in documentation and tests happen to avoid problematic edge cases

3. **Limited real-world usage with non-cubic grids**: The deep_causality codebase may primarily use cubic grids (equal dimensions) for spatial reasoning, where the bug is less likely to cause immediate failures.

4. **The 2D implementation is correct**: Since the 2D version works correctly, developers may have assumed the 3D and 4D implementations were also correct without thorough testing of non-symmetric cases.

5. **Rust's panic behavior**: When the bug does trigger (out-of-bounds access), it causes a panic rather than silent memory corruption. In development, these panics might be attributed to user error (incorrect PointIndex bounds) rather than implementation bugs.

6. **Performance focus**: The benchmarks and examples emphasize the performance benefits of const generics and compile-time optimization, but don't rigorously test correctness with varied dimensional configurations.

# Recommended fix

The fix is straightforward - reverse the indexing order to match the array structure:

**In `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`:**

```rust
fn get(&self, p: PointIndex) -> &T {
    &self[p.z][p.y][p.x]  // <-- FIX 🟢 Correct: [depth][height][width]
}

fn set(&mut self, p: PointIndex, elem: T) {
    self[p.z][p.y][p.x] = elem  // <-- FIX 🟢 Correct: [depth][height][width]
}
```

**In `deep_causality_data_structures/src/grid_type/storage_array_4d.rs`:**

```rust
fn get(&self, p: PointIndex) -> &T {
    &self[p.t][p.z][p.y][p.x]  // Correct: [time][depth][height][width]
}

fn set(&mut self, p: PointIndex, elem: T) {
    self[p.t][p.z][p.y][p.x] = elem  // Correct: [time][depth][height][width]
}
```

Additionally, the test suite should be enhanced with tests using distinct dimensions to prevent regression.

# Related bugs

The same indexing bug exists in `deep_causality_data_structures/src/grid_type/storage_array_4d.rs`. The 4D implementation uses `self[p.y][p.x][p.z][p.t]` when it should use `self[p.t][p.z][p.y][p.x]` to correctly map the array structure `[[[[T; W]; H]; D]; C]`.


--

# Summary
- **Context**: The `storage_array_3d.rs` file implements the `Storage` trait for 3D arrays `[[[T; W]; H]; D]`, providing indexed access to grid-based data structures used throughout the deep_causality codebase.
- **Bug**: The indexing order in the `get` and `set` methods is incorrect - it uses `self[p.y][p.x][p.z]` instead of `self[p.z][p.y][p.x]`.
- **Actual vs. expected**: The implementation maps coordinates as `[height][width][depth]` when it should map as `[depth][height][width]` to match the array type `[[[T; W]; H]; D]` and the semantic meaning of PointIndex fields.
- **Impact**: The bug causes index out-of-bounds panics when using non-cubic grids with distinct dimensions, and silently returns wrong values in other cases, corrupting spatial data operations.

# Code with bug

**File: `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`**

```rust
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
where
    T: Copy,
    [[[T; W]; H]; D]: Sized,
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // <-- BUG 🔴 Wrong indexing order
    }

    fn set(&mut self, p: PointIndex, elem: T) {
        self[p.y][p.x][p.z] = elem  // <-- BUG 🔴 Wrong indexing order
    }

    fn height(&self) -> Option<&usize> {
        Some(&H)
    }

    fn depth(&self) -> Option<&usize> {
        Some(&D)
    }

    fn width(&self) -> Option<&usize> {
        Some(&W)
    }
}
```

# Evidence

## Example

Consider a 3D array with distinct dimensions to illustrate the problem:
```rust
const WIDTH: usize = 3;   // X-axis
const HEIGHT: usize = 4;  // Y-axis
const DEPTH: usize = 2;   // Z-axis

let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];
```

The array type `[[[i32; 3]; 4]; 2]` has this structure:
- Outermost array: 2 elements (DEPTH)
- Middle arrays: 4 elements each (HEIGHT)
- Innermost arrays: 3 elements each (WIDTH)
- Access pattern: `storage[depth_idx][height_idx][width_idx]`

To access position (width=1, height=2, depth=1):
- Direct array access: `storage[1][2][1]`
- PointIndex creation: `PointIndex::new3d(x=1, y=2, z=1)` where x=width, y=height, z=depth

**Current buggy behavior:**
```rust
let point = PointIndex::new3d(1, 2, 1);  // x=1, y=2, z=1
// Implementation does: self[p.y][p.x][p.z] = self[2][1][1]
// But depth=2 is out of bounds (DEPTH=2, so valid indices are 0-1)
// Result: PANIC with "index out of bounds: the len is 2 but the index is 2"
```

**Expected correct behavior:**
```rust
let point = PointIndex::new3d(1, 2, 1);  // x=1, y=2, z=1
// Should do: self[p.z][p.y][p.x] = self[1][2][1]
// This correctly maps to storage[depth=1][height=2][width=1]
```

## Inconsistency with own spec / docstring

### Reference spec

From `deep_causality_data_structures/src/grid_type/mod.rs`:
```rust
/// - `W`: Width (X-axis).
/// - `H`: Height (Y-axis).
/// - `D`: Depth (Z-axis).
```

From `deep_causality_data_structures/src/grid_type/storage.rs`:
```rust
/// Returns the width (X-axis) dimension of the storage, if defined.
fn width(&self) -> Option<&usize>;

/// Returns the height (Y-axis) dimension of the storage, if defined.
fn height(&self) -> Option<&usize>;

/// Returns the depth (Z-axis) dimension of the storage, if defined.
fn depth(&self) -> Option<&usize>;
```

From `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`:
```rust
// T Type
// W Width
// H Height
// D Depth
/// Implements `Storage` for 3D arrays `[[[T; W]; H]; D]`
/// indexed along X (width), Y (height), and Z (depth) axes.
```

From `deep_causality_data_structures/src/grid_type/point.rs`:
```rust
pub struct PointIndex {
    pub x: usize,     // Width (based on semantic meaning)
    pub y: usize,     // Height
    pub z: usize,     // Depth
    pub t: usize,     // Time
    point_type: PointIndexType,
}
```

### Current code

```rust
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // Indexes as [height][width][depth]
    }
}
```

### Contradiction

The array type `[[[T; W]; H]; D]` is nested as `[D outer][H middle][W inner]`, meaning the correct access pattern is `array[depth_index][height_index][width_index]`.

According to the spec:
- `p.x` represents the X-axis (Width)
- `p.y` represents the Y-axis (Height)
- `p.z` represents the Z-axis (Depth)

Therefore, the correct indexing should be:
```rust
&self[p.z][p.y][p.x]  // [depth][height][width]
```

But the implementation uses:
```rust
&self[p.y][p.x][p.z]  // [height][width][depth]
```

This violates the documented coordinate system mapping and array structure.

## Inconsistency within the codebase

### Reference code

**File: `deep_causality_data_structures/src/grid_type/storage_array_2d.rs`**
```rust
// T Type
// W Width
// H Height
/// Implements `Storage` for 2D arrays `[[T; W]; H]`
/// indexed along X (width) and Y (height) axes.
impl<T, const W: usize, const H: usize> Storage<T> for [[T; W]; H]
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x]  // Correctly maps [H][W] to [height][width]
    }
}
```

For the 2D case:
- Array type: `[[T; W]; H]` = `[H outer][W inner]`
- Indexing: `self[p.y][p.x]` = `self[height][width]`
- This is **correct**: outer dimension H is accessed by p.y (height), inner dimension W by p.x (width)

### Current code

**File: `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`**
```rust
// T Type
// W Width
// H Height
// D Depth
/// Implements `Storage` for 3D arrays `[[[T; W]; H]; D]`
/// indexed along X (width), Y (height), and Z (depth) axes.
impl<T, const W: usize, const H: usize, const D: usize> Storage<T> for [[[T; W]; H]; D]
{
    fn get(&self, p: PointIndex) -> &T {
        &self[p.y][p.x][p.z]  // Maps [D][H][W] to [height][width][depth]
    }
}
```

For the 3D case:
- Array type: `[[[T; W]; H]; D]` = `[D outer][H middle][W inner]`
- Indexing: `self[p.y][p.x][p.z]` = `self[height][width][depth]`
- This is **incorrect**: should be `self[p.z][p.y][p.x]` to map outer→depth, middle→height, inner→width

### Contradiction

Following the 2D pattern, the 3D implementation should maintain the same principle: outer dimensions are accessed first, inner dimensions last. The 2D implementation correctly accesses `[outer][inner]` as `[p.y][p.x]` because the array is `[[T; W]; H]` = `[H][W]`.

For 3D, the array `[[[T; W]; H]; D]` means `[D][H][W]`, so following the same principle, it should be accessed as `[p.z][p.y][p.x]` to map `[depth][height][width]`.

The current implementation breaks this pattern by using `[p.y][p.x][p.z]`, which incorrectly treats the array as if it were `[[[T; D]; W]; H]`.

## Failing test

### Test script

```rust
/*
 * Unit test demonstrating the indexing bug in storage_array_3d.rs
 */

#[cfg(test)]
mod tests {
    use deep_causality_data_structures::{PointIndex, Storage};

    #[test]
    fn test_3d_array_indexing_with_distinct_dimensions() {
        // Use distinct dimensions to reveal the bug
        const WIDTH: usize = 3;   // X-axis
        const HEIGHT: usize = 4;  // Y-axis
        const DEPTH: usize = 2;   // Z-axis

        let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];

        // Raw array structure: [[[T; W]; H]; D]
        // This means: storage[depth_idx][height_idx][width_idx]

        // Set a value at a specific location using raw array indexing
        // depth=1, height=2, width=1
        storage[1][2][1] = 99;

        // Now access the same location using Storage trait
        // According to PointIndex semantics:
        // - x represents width (X-axis)
        // - y represents height (Y-axis)
        // - z represents depth (Z-axis)
        let point = PointIndex::new3d(
            1,  // x = width = 1
            2,  // y = height = 2
            1   // z = depth = 1
        );

        // This should retrieve the value we just set (99)
        let retrieved = *<[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::get(&storage, point);

        assert_eq!(
            retrieved, 99,
            "Expected to retrieve 99 from position (width=1, height=2, depth=1), but got {}",
            retrieved
        );
    }

    #[test]
    fn test_3d_array_set_and_get_consistency() {
        const WIDTH: usize = 3;
        const HEIGHT: usize = 4;
        const DEPTH: usize = 2;

        let mut storage: [[[i32; WIDTH]; HEIGHT]; DEPTH] = [[[0; WIDTH]; HEIGHT]; DEPTH];

        // Test a specific non-symmetric position
        let point = PointIndex::new3d(2, 3, 1);  // x=2, y=3, z=1
        let test_value = 42;

        // Set using Storage trait
        <[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::set(&mut storage, point, test_value);

        // Get using Storage trait
        let retrieved = *<[[[i32; WIDTH]; HEIGHT]; DEPTH] as Storage<i32>>::get(&storage, point);

        // This should work, but let's verify what raw position was actually set
        // Expected: storage[z=1][y=3][x=2] = storage[1][3][2]
        let raw_value = storage[1][3][2];

        assert_eq!(
            retrieved, test_value,
            "Storage trait get/set should be consistent"
        );

        assert_eq!(
            raw_value, test_value,
            "Value should be at storage[depth=1][height=3][width=2], but it's not there"
        );
    }
}
```

### Test output

```
running 2 tests
test tests::test_3d_array_set_and_get_consistency ... FAILED
test tests::test_3d_array_indexing_with_distinct_dimensions ... FAILED

failures:

---- tests::test_3d_array_set_and_get_consistency stdout ----

thread 'tests::test_3d_array_set_and_get_consistency' (7650) panicked at /home/user/deep_causality/deep_causality_data_structures/src/grid_type/storage_array_3d.rs:24:9:
index out of bounds: the len is 2 but the index is 3
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_3d_array_indexing_with_distinct_dimensions stdout ----

thread 'tests::test_3d_array_indexing_with_distinct_dimensions' (7649) panicked at /home/user/deep_causality/deep_causality_data_structures/src/grid_type/storage_array_3d.rs:20:10:
index out of bounds: the len is 2 but the index is 2


failures:
    tests::test_3d_array_indexing_with_distinct_dimensions
    tests::test_3d_array_set_and_get_consistency

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Both tests panic with "index out of bounds" errors because the incorrect indexing tries to use `p.y` (height=2 or 3) as the first index into an array of length DEPTH=2.

# Full context

The `Storage` trait provides a unified interface for accessing multi-dimensional arrays in the deep_causality data structures crate. It's implemented for 1D, 2D, 3D, and 4D arrays to support grid-based spatial computations throughout the deep_causality ecosystem.

The `storage_array_3d.rs` file implements this trait for 3D arrays of the form `[[[T; W]; H]; D]`, where the dimensions represent width, height, and depth respectively. This implementation is used by:

1. **ArrayGrid**: A high-performance tensor-like structure that uses const generics for compile-time optimization. ArrayGrid delegates to the Storage trait implementations for all data access.

2. **Grid**: A wrapper around storage backends that provides a uniform API regardless of dimensionality. The Grid type is used extensively in the deep_causality codebase for spatial reasoning operations.

The bug affects any code that uses 3D grids with non-equal dimensions. The existing tests all use equal dimensions (e.g., `WIDTH=HEIGHT=DEPTH=5`) and symmetric access patterns (e.g., `PointIndex::new3d(1, 1, 1)`), which masks the indexing bug.

When users create grids with distinct dimensions (e.g., `WIDTH=3, HEIGHT=4, DEPTH=2`) and access non-symmetric positions, the bug manifests as either:
- Index out-of-bounds panics when coordinates exceed the wrongly-mapped dimension
- Silent data corruption when the wrong array element is accessed/modified

The same indexing bug exists in `storage_array_4d.rs`, which uses `self[p.y][p.x][p.z][p.t]` instead of the correct `self[p.t][p.z][p.y][p.x]`.

# Why has this bug gone undetected?

The bug has remained undetected because:

1. **All existing tests use symmetric dimensions**: The test suite uses equal values for WIDTH, HEIGHT, and DEPTH (typically all set to 5), making it impossible to detect dimension-specific indexing errors.

2. **Tests use symmetric coordinates**: Test cases like `PointIndex::new3d(1, 1, 1)` or `PointIndex::new3d(1, 2, 3)` work correctly or appear to work because:
    - With equal dimensions, out-of-bounds conditions are less likely
    - The examples in documentation and tests happen to avoid problematic edge cases

3. **Limited real-world usage with non-cubic grids**: The deep_causality codebase may primarily use cubic grids (equal dimensions) for spatial reasoning, where the bug is less likely to cause immediate failures.

4. **The 2D implementation is correct**: Since the 2D version works correctly, developers may have assumed the 3D and 4D implementations were also correct without thorough testing of non-symmetric cases.

5. **Rust's panic behavior**: When the bug does trigger (out-of-bounds access), it causes a panic rather than silent memory corruption. In development, these panics might be attributed to user error (incorrect PointIndex bounds) rather than implementation bugs.

6. **Performance focus**: The benchmarks and examples emphasize the performance benefits of const generics and compile-time optimization, but don't rigorously test correctness with varied dimensional configurations.

# Recommended fix

The fix is straightforward - reverse the indexing order to match the array structure:

**In `deep_causality_data_structures/src/grid_type/storage_array_3d.rs`:**

```rust
fn get(&self, p: PointIndex) -> &T {
    &self[p.z][p.y][p.x]  // <-- FIX 🟢 Correct: [depth][height][width]
}

fn set(&mut self, p: PointIndex, elem: T) {
    self[p.z][p.y][p.x] = elem  // <-- FIX 🟢 Correct: [depth][height][width]
}
```

**In `deep_causality_data_structures/src/grid_type/storage_array_4d.rs`:**

```rust
fn get(&self, p: PointIndex) -> &T {
    &self[p.t][p.z][p.y][p.x]  // Correct: [time][depth][height][width]
}

fn set(&mut self, p: PointIndex, elem: T) {
    self[p.t][p.z][p.y][p.x] = elem  // Correct: [time][depth][height][width]
}
```

Additionally, the test suite should be enhanced with tests using distinct dimensions to prevent regression.

# Related bugs

The same indexing bug exists in `deep_causality_data_structures/src/grid_type/storage_array_4d.rs`. The 4D implementation uses `self[p.y][p.x][p.z][p.t]` when it should use `self[p.t][p.z][p.y][p.x]` to correctly map the array structure `[[[[T; W]; H]; D]; C]`.
