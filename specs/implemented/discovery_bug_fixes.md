# Summary
- **Context**: `DataDiscretizer` is a data preprocessing component in the CDL pipeline that discretizes continuous numerical data into bins using either equal-width or equal-frequency binning strategies.
- **Bug**: NaN values in input data are silently converted to bin 0 instead of being preserved as NaN or causing an error.
- **Actual vs. expected**: NaN values should either remain as NaN in the output tensor or trigger a `PreprocessError`, but instead they are incorrectly binned as 0.0.
- **Impact**: Data corruption occurs silently - NaN values (which represent missing or undefined data) are treated as valid data points in the lowest bin, leading to incorrect statistical analysis and potentially wrong causal discovery results.

# Code with bug

In `deep_causality_discovery/src/types/data_preprocessor/data_discretizer.rs`:

## bin_equal_width function (lines 70-97)
```rust
fn bin_equal_width(data: &[f64], num_bins: usize) -> Result<Vec<f64>, PreprocessError> {
    if num_bins < 2 {
        return Err(PreprocessError::ConfigError(
            "Number of bins must be at least 2".to_string(),
        ));
    }

    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    if (max - min).abs() < f64::EPSILON {
        return Ok(vec![0.0; data.len()]);
    }

    let bin_width = (max - min) / num_bins as f64;
    let mut binned_data = Vec::with_capacity(data.len());

    for &val in data {
        let mut bin_index = ((val - min) / bin_width) as usize;  // <-- BUG 🔴 NaN becomes 0
        if bin_index >= num_bins {
            bin_index = num_bins - 1;
        }
        binned_data.push(bin_index as f64);
    }

    Ok(binned_data)
}
```

## bin_equal_frequency function (lines 99-140)
```rust
fn bin_equal_frequency(data: &[f64], num_bins: usize) -> Result<Vec<f64>, PreprocessError> {
    // ... validation code ...

    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&a, &b| {
        data[a]
            .partial_cmp(&data[b])
            .unwrap_or(std::cmp::Ordering::Equal)  // <-- BUG 🔴 NaN treated as equal
    });

    let mut binned_data = vec![0.0; n];
    let step = n as f64 / num_bins as f64;

    for i in 0..num_bins {
        let start_k = (i as f64 * step).round() as usize;
        let end_k = ((i + 1) as f64 * step).round() as usize;

        for &original_index in &indices[start_k..end_k.min(n)] {
            binned_data[original_index] = i as f64;  // <-- BUG 🔴 NaN gets binned
        }
    }

    Ok(binned_data)
}
```

# Evidence

## Example

### Equal-width binning with NaN

Input column: `[10.0, NaN, 30.0, 40.0, 50.0]`

When computing the bin for the NaN value:
1. `min = 10.0`, `max = 50.0` (NaN is ignored by `f64::min`)
2. `bin_width = (50.0 - 10.0) / 2 = 20.0`
3. For NaN: `bin_index = ((NaN - 10.0) / 20.0) as usize`
4. `NaN - 10.0 = NaN`
5. `NaN / 20.0 = NaN`
6. `NaN as usize = 0` (Rust converts NaN to 0 when casting to integer)
7. Result: NaN is placed in bin 0

**Expected**: NaN should remain NaN or cause an error
**Actual**: NaN is silently converted to 0.0 (bin 0)

### Equal-frequency binning with NaN

Input column: `[10.0, NaN, 30.0]`

When sorting for equal-frequency binning:
1. The `partial_cmp` returns `None` for NaN comparisons
2. `unwrap_or(std::cmp::Ordering::Equal)` treats NaN as equal to everything
3. NaN's position in the sorted indices is unpredictable
4. NaN gets assigned to whatever bin contains its index position

**Expected**: NaN should remain NaN or cause an error
**Actual**: NaN gets binned based on arbitrary sort position

## Failing test

### Test script
```rust
#[cfg(test)]
mod tests {
    use deep_causality_discovery::{
        BinningStrategy, ColumnSelector, DataDiscretizer, DataPreprocessor, PreprocessConfig,
    };
    use deep_causality_tensor::CausalTensor;

    /// This test demonstrates a bug where NaN values are silently binned to bin 0
    /// instead of being preserved as NaN or causing an error.
    ///
    /// Expected behavior: NaN values should either:
    /// 1. Be preserved as NaN in the output tensor, OR
    /// 2. Cause a PreprocessError to be returned
    ///
    /// Actual behavior: NaN is silently converted to bin 0 (0.0)
    #[test]
    fn test_nan_handling_equal_width() {
        let discretizer = DataDiscretizer;

        // Create a 2x2 tensor with a NaN value
        // [[1.0, 10.0],
        //  [2.0, NaN]]
        let data = vec![1.0, 10.0, 2.0, f64::NAN];
        let tensor = CausalTensor::new(data, vec![2, 2]).unwrap();

        let config = PreprocessConfig::new(
            BinningStrategy::EqualWidth,
            2,
            ColumnSelector::All,
        );

        let result = discretizer.process(tensor, &config).unwrap();
        let result_data = result.as_slice();

        println!("Input: [1.0, 10.0, 2.0, NaN]");
        println!("Output: {:?}", result_data);
        println!("Element at index 3 (was NaN): {}", result_data[3]);
        println!("Is it NaN? {}", result_data[3].is_nan());

        // The bug: result_data[3] should be NaN but it's 0.0
        // This assertion will FAIL, demonstrating the bug
        assert!(
            result_data[3].is_nan(),
            "Expected NaN to be preserved in output, but got {}",
            result_data[3]
        );
    }

    /// Test with EqualFrequency binning strategy
    #[test]
    fn test_nan_handling_equal_frequency() {
        let discretizer = DataDiscretizer;

        let data = vec![1.0, 10.0, 2.0, f64::NAN, 3.0, 30.0];
        let tensor = CausalTensor::new(data, vec![3, 2]).unwrap();

        let config = PreprocessConfig::new(
            BinningStrategy::EqualFrequency,
            2,
            ColumnSelector::All,
        );

        let result = discretizer.process(tensor, &config).unwrap();
        let result_data = result.as_slice();

        println!("Input column 1: [10.0, NaN, 30.0]");
        println!("Output column 1: [{}, {}, {}]", result_data[1], result_data[3], result_data[5]);
        println!("Element at index 3 (was NaN): {}", result_data[3]);
        println!("Is it NaN? {}", result_data[3].is_nan());

        // The bug: result_data[3] should be NaN but it's not
        assert!(
            result_data[3].is_nan(),
            "Expected NaN to be preserved in output, but got {}",
            result_data[3]
        );
    }
}

fn main() {
    println!("Run with: cargo test --package deep_causality_discovery --test test_nan_bug_unit_test");
}
```

### Test output
```
running 2 tests
test tests::test_nan_handling_equal_width ... FAILED
test tests::test_nan_handling_equal_frequency ... FAILED

failures:

---- tests::test_nan_handling_equal_width stdout ----
Input: [1.0, 10.0, 2.0, NaN]
Output: [0.0, 0.0, 1.0, 0.0]
Element at index 3 (was NaN): 0
Is it NaN? false

thread 'tests::test_nan_handling_equal_width' (6605) panicked at test_nan_bug_unit_test.rs:42:9:
Expected NaN to be preserved in output, but got 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::test_nan_handling_equal_frequency stdout ----
Input column 1: [10.0, NaN, 30.0]
Output column 1: [0, 0, 1]
Element at index 3 (was NaN): 0
Is it NaN? false

thread 'tests::test_nan_handling_equal_frequency' (6604) panicked at test_nan_bug_unit_test.rs:72:9:
Expected NaN to be preserved in output, but got 0


failures:
    tests::test_nan_handling_equal_frequency
    tests::test_nan_handling_equal_width

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Explanation

The root cause is Rust's behavior when casting NaN to integers:

```rust
let nan_val = f64::NAN;
let as_usize = nan_val as usize;  // Results in 0
```

This is documented Rust behavior - casting NaN to an integer type yields 0. The `bin_equal_width` function performs this cast without checking if the value is NaN first:

```rust
let mut bin_index = ((val - min) / bin_width) as usize;
```

When `val` is NaN, the entire expression `((NaN - min) / bin_width)` evaluates to NaN, which then becomes 0 when cast to `usize`.

Similarly, in `bin_equal_frequency`, the sorting operation treats NaN as equal to all values (due to `unwrap_or(std::cmp::Ordering::Equal)`), which means NaN values end up in an arbitrary position in the sorted order and get binned accordingly.

# Full context

The `DataDiscretizer` is part of the CDL (Causal Discovery Language) pipeline in the `deep_causality_discovery` crate. The pipeline workflow is:

1. **Configuration** - Set up pipeline parameters
2. **Data Loading** - Load data from CSV/Parquet into `CausalTensor<f64>`
3. **Data Cleaning (Optional)** - `OptionNoneDataCleaner` converts NaN to `Option<None>`
4. **Data Preprocessing (Optional)** - `DataDiscretizer` bins continuous values
5. **Feature Selection** - MRMR algorithm selects relevant features
6. **Causal Discovery** - SURD algorithm discovers causal relationships
7. **Analysis** - Interpret and format results

The `DataDiscretizer` is called via the `preprocess()` method on `CDL<WithData>` (in `deep_causality_discovery/src/types/cdl/cdl_with_data.rs:16-43`). It implements the `DataPreprocessor` trait which transforms a `CausalTensor<f64>` according to a `PreprocessConfig`.

According to the documentation (README.md lines 53-56), the data cleaning step is "Optional but recommended". This means:

1. Users may skip the cleaning step and pass data with NaN values directly to preprocessing
2. Even if cleaned, subsequent operations could reintroduce NaN values
3. The `DataDiscretizer` should handle NaN values gracefully rather than silently corrupting them

The discretizer is used in production for:
- Preparing continuous data for algorithms that work better with discretized data
- Reducing the dimensionality of continuous features
- Making data more robust to outliers

When NaN values are silently converted to bin 0, the resulting discretized data contains incorrect information that propagates through the entire pipeline, potentially leading to:
- Incorrect feature selection (MRMR algorithm receives corrupted data)
- Invalid causal relationships discovered (SURD algorithm operates on wrong data)
- Misleading analysis results and recommendations
- Users being unaware their data was corrupted since no error is raised

# Why has this bug gone undetected?

This bug has likely gone undetected for several reasons:

1. **Expected workflow compliance**: Most users likely follow the recommended workflow which includes the `clean_data` step before preprocessing. The `OptionNoneDataCleaner` converts all NaN values to `Option::None` before discretization, so the discretizer never encounters NaN values in typical usage.

2. **Silent failure**: The bug doesn't cause a crash or panic - it silently produces incorrect output. Without explicit validation that NaN values are preserved or rejected, the corruption goes unnoticed in the binned output.

3. **Test coverage gap**: The existing test suite for `DataDiscretizer` (in `deep_causality_discovery/tests/types/preprocessor/data_discretizer_tests.rs`) tests various scenarios but never includes test cases with NaN values in the input data. All tests use clean numerical data.

4. **Low-level arithmetic behavior**: The cast of NaN to 0 happens at the Rust language level and isn't immediately obvious. Developers might not realize that `NaN as usize` yields 0 without explicitly testing for it.

5. **Production data characteristics**: If production data sources typically have very few or no NaN values after loading, or if data validation happens elsewhere in the pipeline, this bug would rarely manifest in practice.

6. **Optional preprocessing**: Since preprocessing is optional in the pipeline, users working with data that doesn't need discretization might never trigger this code path, further reducing the chance of encountering the bug.

# Recommended fix

Add NaN detection and handling to both binning functions:

```rust
fn bin_equal_width(data: &[f64], num_bins: usize) -> Result<Vec<f64>, PreprocessError> {
    if num_bins < 2 {
        return Err(PreprocessError::ConfigError(
            "Number of bins must be at least 2".to_string(),
        ));
    }

    // Check for NaN values  // <-- FIX 🟢
    if data.iter().any(|&x| x.is_nan()) {
        return Err(PreprocessError::BinningError(
            "Cannot bin data containing NaN values. Use MissingValueImputer first.".to_string(),
        ));
    }

    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    if (max - min).abs() < f64::EPSILON {
        return Ok(vec![0.0; data.len()]);
    }

    let bin_width = (max - min) / num_bins as f64;
    let mut binned_data = Vec::with_capacity(data.len());

    for &val in data {
        let mut bin_index = ((val - min) / bin_width) as usize;
        if bin_index >= num_bins {
            bin_index = num_bins - 1;
        }
        binned_data.push(bin_index as f64);
    }

    Ok(binned_data)
}
```

Apply the same fix to `bin_equal_frequency`. This approach:
1. Fails fast with a clear error message
2. Directs users to use `MissingValueImputer` first
3. Prevents silent data corruption
4. Maintains the existing API contract (returns `Result`)



# Summary
- **Context**: The CSV data loader is part of the CDL (Causal Discovery Language) pipeline and is responsible for loading data from CSV files into tensors for causal analysis.
- **Bug**: The CSV data loader ignores the `exclude_indices` configuration parameter, loading all columns instead of excluding the specified ones.
- **Actual vs. expected**: When `exclude_indices` is provided in the configuration, the CSV loader should skip those columns, but it currently loads all columns regardless of this setting.
- **Impact**: Users cannot exclude unwanted columns when loading CSV files, leading to incorrect data being fed into the causal discovery pipeline, which can produce invalid causal relationships and analysis results.

# Code with bug

The CSV loader implementation in `deep_causality_discovery/src/types/data_loader/csv.rs`:

```rust
impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            let file = File::open(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DataLoadingError::FileNotFound(path.to_string())
                } else {
                    DataLoadingError::OsError(e.to_string())
                }
            })?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(csv_config.has_headers())
                .delimiter(csv_config.delimiter())
                .from_reader(file);

            let mut data = Vec::new();
            let mut width = 0;
            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                if width == 0 {
                    width = record.len();
                }
                for field in record.iter() {  // <-- BUG 🔴: Iterates over ALL fields, ignoring exclude_indices
                    data.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }
            }

            let height = if width == 0 { 0 } else { data.len() / width };
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataLoadingError::OsError(e.to_string()))
        } else {
            Err(DataLoadingError::OsError(
                "Invalid config type for CsvDataLoader".to_string(),
            ))
        }
    }
}
```

The bug is that the loop at line 41 iterates over all fields without checking if the field's index is in the `exclude_indices` list from the config.

# Evidence

## Example

Consider a CSV file with 4 columns:
```
1.0,2.0,3.0,4.0
5.0,6.0,7.0,8.0
```

When loading with `exclude_indices = vec![1, 2]` (to exclude columns at indices 1 and 2):

**Expected behavior:**
- Load only columns at indices 0 and 3
- Result tensor: `[1.0, 4.0, 5.0, 8.0]` with shape `[2, 2]`

**Actual behavior:**
- Loads all columns (indices 0, 1, 2, 3)
- Result tensor: `[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]` with shape `[2, 4]`

## Failing test

### Test script
```rust
/*
 * Test to demonstrate the bug in CSV data loader:
 * exclude_indices configuration is ignored
 */

use deep_causality_discovery::{
    CsvConfig, CsvDataLoader, DataLoader, DataLoaderConfig,
};
use deep_causality_tensor::CausalTensor;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_csv(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn test_csv_exclude_indices_bug() {
    // CSV with 4 columns
    let csv_content = "1.0,2.0,3.0,4.0\n5.0,6.0,7.0,8.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    // Configure to exclude columns at indices 1 and 2
    let csv_config = CsvConfig::new(
        false,          // no headers
        b',',           // comma delimiter
        0,              // skip no rows
        None,           // all columns
        None,           // no file path
        None,           // no target index
        vec![1, 2],     // exclude indices 1 and 2
    );
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();

    // Expected: Only columns 0 and 3 should be loaded
    // Row 1: 1.0, 4.0
    // Row 2: 5.0, 8.0
    let expected = CausalTensor::new(vec![1.0, 4.0, 5.0, 8.0], vec![2, 2]).unwrap();

    println!("Result shape: {:?}", result.shape());
    println!("Result data: {:?}", result.as_slice());
    println!("Expected shape: {:?}", expected.shape());
    println!("Expected data: {:?}", expected.as_slice());

    // This assertion will FAIL because CSV loader ignores exclude_indices
    assert_eq!(result.shape(), expected.shape(), "Shape mismatch");
    assert_eq!(result.as_slice(), expected.as_slice(), "Data mismatch");
}
```

### Test output
```
running 1 test
test test_csv_exclude_indices_bug ... FAILED

failures:

---- test_csv_exclude_indices_bug stdout ----
Result shape: [2, 4]
Result data: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
Expected shape: [2, 2]
Expected data: [1.0, 4.0, 5.0, 8.0]

thread 'test_csv_exclude_indices_bug' (6434) panicked at deep_causality_discovery/tests/test_csv_exclude_bug.rs:52:5:
assertion `left == right` failed: Shape mismatch
  left: [2, 4]
 right: [2, 2]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_csv_exclude_indices_bug

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Inconsistency within the codebase

### Reference code: Parquet loader implementation

File: `deep_causality_discovery/src/types/data_loader/parquet.rs`

```rust
impl DataLoader for ParquetDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Parquet(parquet_config) = config {
            // ... setup code ...

            let exclude_indices = parquet_config.exclude_indices();

            for record_result in iter {
                let record = record_result?;
                let mut row_values = Vec::new();

                for (i, (name, field)) in record.get_column_iter().enumerate() {
                    if exclude_indices.contains(&i) {  // ✓ Correctly checks exclude_indices
                        continue;
                    }

                    let val = match field {
                        // ... field conversion code ...
                    };
                    row_values.push(val);
                }
                // ... rest of implementation ...
            }
        }
    }
}
```

### Current code: CSV loader implementation

File: `deep_causality_discovery/src/types/data_loader/csv.rs`

```rust
impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            // ... setup code ...

            // ✗ NEVER retrieves exclude_indices from config

            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                if width == 0 {
                    width = record.len();
                }
                for field in record.iter() {  // ✗ NO check for excluded indices
                    data.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }
            }
        }
    }
}
```

### Contradiction

Both data loaders implement the same `DataLoader` trait and are designed to work interchangeably in the CDL pipeline. Both accept configuration types (`CsvConfig` and `ParquetConfig`) that include an `exclude_indices` field. However:

1. The Parquet loader correctly retrieves `exclude_indices` from the config and uses it to filter columns
2. The CSV loader completely ignores the `exclude_indices` field and loads all columns

This inconsistency violates the principle of uniform behavior across data loaders and breaks the user's expectation that both loaders support the same feature set.

## Inconsistency with own spec / docstring

### Reference: CsvConfig documentation

File: `deep_causality_discovery/src/types/config/data_csv_config.rs`

```rust
/// Configuration for loading data from a CSV file.
#[derive(Debug, Clone)]
pub struct CsvConfig {
    has_headers: bool,
    delimiter: u8,
    skip_rows: usize,
    columns: Option<Vec<String>>,
    file_path: Option<String>,
    target_index: Option<usize>,
    exclude_indices: Vec<usize>,  // <-- Field is part of the config
}

impl CsvConfig {
    // ... other getters ...

    /// Indices of columns to exclude.
    pub fn exclude_indices(&self) -> &Vec<usize> {  // <-- Getter is provided
        &self.exclude_indices
    }
}
```

### Current code

The CSV loader implementation never calls `csv_config.exclude_indices()` and does not implement the documented behavior.

### Contradiction

The `CsvConfig` struct explicitly includes an `exclude_indices` field with a public getter method, indicating that this is a supported feature. The configuration even provides documentation: "Indices of columns to exclude." However, the CSV loader implementation never reads this field, making the feature non-functional.

# Full context

The CSV data loader is a core component of the deep_causality_discovery library's CDL (Causal Discovery Language) pipeline. The CDL pipeline follows a builder pattern with multiple stages:

1. **NoData state**: User calls `CDL::new().load_data(path, target_index, exclude_indices)`
2. **Data loading**: The loader selects either CSV or Parquet loader based on file extension
3. **Data processing**: The loaded tensor flows through cleaning, preprocessing, feature selection, and causal discovery stages

The `load_data` method in `deep_causality_discovery/src/types/cdl/cdl_with_no_data.rs` accepts `exclude_indices` as a parameter and passes it to both CSV and Parquet configurations. This parameter is meant to exclude certain columns (e.g., ID columns, timestamp columns, or other non-causal features) from the analysis.

When users call `load_data` with a CSV file and specify columns to exclude:
- The `exclude_indices` parameter is correctly passed to `CsvConfig::new()`
- The config is stored and passed to `CsvDataLoader.load()`
- However, the CSV loader ignores the config and loads all columns
- The resulting tensor has more columns than expected
- All downstream pipeline stages (cleaning, preprocessing, feature selection, causal discovery) operate on incorrect data
- The final causal graph may include spurious relationships from columns that should have been excluded

The Parquet loader correctly implements column exclusion, creating an inconsistency where the same pipeline code produces different results depending on file format.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Default usage doesn't trigger it**: The default value for `exclude_indices` is an empty vector (`vec![]`), and most tests use this default. When `exclude_indices` is empty, the bug has no observable effect since no columns should be excluded anyway.

2. **No test coverage**: There are no tests in the test suite that verify the `exclude_indices` functionality for the CSV loader. The existing test at `deep_causality_discovery/tests/types/data_loader/csv_data_loader_tests.rs` includes tests for headers, delimiters, and skip_rows, but none for column exclusion.

3. **Example code uses default**: The example in `deep_causality_discovery/examples/main.rs` calls `load_data(&file_path, target_index, vec![])`, passing an empty exclusion list. No production or example code actually uses the feature.

4. **Parquet dominates in practice**: Users working with large-scale causal discovery likely use Parquet files (which correctly implement the feature) rather than CSV files, so CSV-specific bugs get less exposure.

5. **Silent failure**: The loader doesn't fail or produce an error when `exclude_indices` is specified - it simply ignores it and loads all columns. Users might not notice that extra columns are being processed unless they carefully inspect the tensor dimensions.

6. **Feature may be newly added**: The `exclude_indices` feature appears to have been designed as part of the CDL pipeline API but may not have been fully implemented across all loaders. The Parquet loader got the full implementation while the CSV loader got only the configuration plumbing.

# Recommended fix

The CSV loader should implement the same column exclusion logic as the Parquet loader. Here's the recommended approach:

```rust
impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            // ... existing setup code ...

            let exclude_indices = csv_config.exclude_indices();  // <-- FIX 🟢: Get exclude_indices

            let mut data = Vec::new();
            let mut width = 0;
            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                let mut row_values = Vec::new();

                for (i, field) in record.iter().enumerate() {
                    if exclude_indices.contains(&i) {  // <-- FIX 🟢: Skip excluded columns
                        continue;
                    }
                    row_values.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }

                if width == 0 {
                    width = row_values.len();
                }
                data.extend(row_values);
            }

            let height = if width == 0 { 0 } else { data.len() / width };
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataLoadingError::OsError(e.to_string()))
        } else {
            Err(DataLoadingError::OsError(
                "Invalid config type for CsvDataLoader".to_string(),
            ))
        }
    }
}
```

Additionally, add test coverage to verify the fix works correctly and to prevent regression.

# Summary
- **Context**: The CSV data loader is part of the CDL (Causal Discovery Language) pipeline and is responsible for loading data from CSV files into tensors for causal analysis.
- **Bug**: The CSV data loader ignores the `exclude_indices` configuration parameter, loading all columns instead of excluding the specified ones.
- **Actual vs. expected**: When `exclude_indices` is provided in the configuration, the CSV loader should skip those columns, but it currently loads all columns regardless of this setting.
- **Impact**: Users cannot exclude unwanted columns when loading CSV files, leading to incorrect data being fed into the causal discovery pipeline, which can produce invalid causal relationships and analysis results.

# Code with bug

The CSV loader implementation in `deep_causality_discovery/src/types/data_loader/csv.rs`:

```rust
impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            let file = File::open(path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    DataLoadingError::FileNotFound(path.to_string())
                } else {
                    DataLoadingError::OsError(e.to_string())
                }
            })?;
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(csv_config.has_headers())
                .delimiter(csv_config.delimiter())
                .from_reader(file);

            let mut data = Vec::new();
            let mut width = 0;
            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                if width == 0 {
                    width = record.len();
                }
                for field in record.iter() {  // <-- BUG 🔴: Iterates over ALL fields, ignoring exclude_indices
                    data.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }
            }

            let height = if width == 0 { 0 } else { data.len() / width };
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataLoadingError::OsError(e.to_string()))
        } else {
            Err(DataLoadingError::OsError(
                "Invalid config type for CsvDataLoader".to_string(),
            ))
        }
    }
}
```

The bug is that the loop at line 41 iterates over all fields without checking if the field's index is in the `exclude_indices` list from the config.

# Evidence

## Example

Consider a CSV file with 4 columns:
```
1.0,2.0,3.0,4.0
5.0,6.0,7.0,8.0
```

When loading with `exclude_indices = vec![1, 2]` (to exclude columns at indices 1 and 2):

**Expected behavior:**
- Load only columns at indices 0 and 3
- Result tensor: `[1.0, 4.0, 5.0, 8.0]` with shape `[2, 2]`

**Actual behavior:**
- Loads all columns (indices 0, 1, 2, 3)
- Result tensor: `[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]` with shape `[2, 4]`

## Failing test

### Test script
```rust
/*
 * Test to demonstrate the bug in CSV data loader:
 * exclude_indices configuration is ignored
 */

use deep_causality_discovery::{
    CsvConfig, CsvDataLoader, DataLoader, DataLoaderConfig,
};
use deep_causality_tensor::CausalTensor;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_csv(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn test_csv_exclude_indices_bug() {
    // CSV with 4 columns
    let csv_content = "1.0,2.0,3.0,4.0\n5.0,6.0,7.0,8.0";
    let temp_file = create_temp_csv(csv_content);
    let path = temp_file.path().to_str().unwrap();

    let loader = CsvDataLoader;
    // Configure to exclude columns at indices 1 and 2
    let csv_config = CsvConfig::new(
        false,          // no headers
        b',',           // comma delimiter
        0,              // skip no rows
        None,           // all columns
        None,           // no file path
        None,           // no target index
        vec![1, 2],     // exclude indices 1 and 2
    );
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(path, &config).unwrap();

    // Expected: Only columns 0 and 3 should be loaded
    // Row 1: 1.0, 4.0
    // Row 2: 5.0, 8.0
    let expected = CausalTensor::new(vec![1.0, 4.0, 5.0, 8.0], vec![2, 2]).unwrap();

    println!("Result shape: {:?}", result.shape());
    println!("Result data: {:?}", result.as_slice());
    println!("Expected shape: {:?}", expected.shape());
    println!("Expected data: {:?}", expected.as_slice());

    // This assertion will FAIL because CSV loader ignores exclude_indices
    assert_eq!(result.shape(), expected.shape(), "Shape mismatch");
    assert_eq!(result.as_slice(), expected.as_slice(), "Data mismatch");
}
```

### Test output
```
running 1 test
test test_csv_exclude_indices_bug ... FAILED

failures:

---- test_csv_exclude_indices_bug stdout ----
Result shape: [2, 4]
Result data: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]
Expected shape: [2, 2]
Expected data: [1.0, 4.0, 5.0, 8.0]

thread 'test_csv_exclude_indices_bug' (6434) panicked at deep_causality_discovery/tests/test_csv_exclude_bug.rs:52:5:
assertion `left == right` failed: Shape mismatch
  left: [2, 4]
 right: [2, 2]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_csv_exclude_indices_bug

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Inconsistency within the codebase

### Reference code: Parquet loader implementation

File: `deep_causality_discovery/src/types/data_loader/parquet.rs`

```rust
impl DataLoader for ParquetDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Parquet(parquet_config) = config {
            // ... setup code ...

            let exclude_indices = parquet_config.exclude_indices();

            for record_result in iter {
                let record = record_result?;
                let mut row_values = Vec::new();

                for (i, (name, field)) in record.get_column_iter().enumerate() {
                    if exclude_indices.contains(&i) {  // ✓ Correctly checks exclude_indices
                        continue;
                    }

                    let val = match field {
                        // ... field conversion code ...
                    };
                    row_values.push(val);
                }
                // ... rest of implementation ...
            }
        }
    }
}
```

### Current code: CSV loader implementation

File: `deep_causality_discovery/src/types/data_loader/csv.rs`

```rust
impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            // ... setup code ...

            // ✗ NEVER retrieves exclude_indices from config

            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                if width == 0 {
                    width = record.len();
                }
                for field in record.iter() {  // ✗ NO check for excluded indices
                    data.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }
            }
        }
    }
}
```

### Contradiction

Both data loaders implement the same `DataLoader` trait and are designed to work interchangeably in the CDL pipeline. Both accept configuration types (`CsvConfig` and `ParquetConfig`) that include an `exclude_indices` field. However:

1. The Parquet loader correctly retrieves `exclude_indices` from the config and uses it to filter columns
2. The CSV loader completely ignores the `exclude_indices` field and loads all columns

This inconsistency violates the principle of uniform behavior across data loaders and breaks the user's expectation that both loaders support the same feature set.

## Inconsistency with own spec / docstring

### Reference: CsvConfig documentation

File: `deep_causality_discovery/src/types/config/data_csv_config.rs`

```rust
/// Configuration for loading data from a CSV file.
#[derive(Debug, Clone)]
pub struct CsvConfig {
    has_headers: bool,
    delimiter: u8,
    skip_rows: usize,
    columns: Option<Vec<String>>,
    file_path: Option<String>,
    target_index: Option<usize>,
    exclude_indices: Vec<usize>,  // <-- Field is part of the config
}

impl CsvConfig {
    // ... other getters ...

    /// Indices of columns to exclude.
    pub fn exclude_indices(&self) -> &Vec<usize> {  // <-- Getter is provided
        &self.exclude_indices
    }
}
```

### Current code

The CSV loader implementation never calls `csv_config.exclude_indices()` and does not implement the documented behavior.

### Contradiction

The `CsvConfig` struct explicitly includes an `exclude_indices` field with a public getter method, indicating that this is a supported feature. The configuration even provides documentation: "Indices of columns to exclude." However, the CSV loader implementation never reads this field, making the feature non-functional.

# Full context

The CSV data loader is a core component of the deep_causality_discovery library's CDL (Causal Discovery Language) pipeline. The CDL pipeline follows a builder pattern with multiple stages:

1. **NoData state**: User calls `CDL::new().load_data(path, target_index, exclude_indices)`
2. **Data loading**: The loader selects either CSV or Parquet loader based on file extension
3. **Data processing**: The loaded tensor flows through cleaning, preprocessing, feature selection, and causal discovery stages

The `load_data` method in `deep_causality_discovery/src/types/cdl/cdl_with_no_data.rs` accepts `exclude_indices` as a parameter and passes it to both CSV and Parquet configurations. This parameter is meant to exclude certain columns (e.g., ID columns, timestamp columns, or other non-causal features) from the analysis.

When users call `load_data` with a CSV file and specify columns to exclude:
- The `exclude_indices` parameter is correctly passed to `CsvConfig::new()`
- The config is stored and passed to `CsvDataLoader.load()`
- However, the CSV loader ignores the config and loads all columns
- The resulting tensor has more columns than expected
- All downstream pipeline stages (cleaning, preprocessing, feature selection, causal discovery) operate on incorrect data
- The final causal graph may include spurious relationships from columns that should have been excluded

The Parquet loader correctly implements column exclusion, creating an inconsistency where the same pipeline code produces different results depending on file format.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **Default usage doesn't trigger it**: The default value for `exclude_indices` is an empty vector (`vec![]`), and most tests use this default. When `exclude_indices` is empty, the bug has no observable effect since no columns should be excluded anyway.

2. **No test coverage**: There are no tests in the test suite that verify the `exclude_indices` functionality for the CSV loader. The existing test at `deep_causality_discovery/tests/types/data_loader/csv_data_loader_tests.rs` includes tests for headers, delimiters, and skip_rows, but none for column exclusion.

3. **Example code uses default**: The example in `deep_causality_discovery/examples/main.rs` calls `load_data(&file_path, target_index, vec![])`, passing an empty exclusion list. No production or example code actually uses the feature.

4. **Parquet dominates in practice**: Users working with large-scale causal discovery likely use Parquet files (which correctly implement the feature) rather than CSV files, so CSV-specific bugs get less exposure.

5. **Silent failure**: The loader doesn't fail or produce an error when `exclude_indices` is specified - it simply ignores it and loads all columns. Users might not notice that extra columns are being processed unless they carefully inspect the tensor dimensions.

6. **Feature may be newly added**: The `exclude_indices` feature appears to have been designed as part of the CDL pipeline API but may not have been fully implemented across all loaders. The Parquet loader got the full implementation while the CSV loader got only the configuration plumbing.

# Recommended fix

The CSV loader should implement the same column exclusion logic as the Parquet loader. Here's the recommended approach:

```rust
impl DataLoader for CsvDataLoader {
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError> {
        if let DataLoaderConfig::Csv(csv_config) = config {
            // ... existing setup code ...

            let exclude_indices = csv_config.exclude_indices();  // <-- FIX 🟢: Get exclude_indices

            let mut data = Vec::new();
            let mut width = 0;
            for result in rdr.records().skip(csv_config.skip_rows()) {
                let record = result?;
                let mut row_values = Vec::new();

                for (i, field) in record.iter().enumerate() {
                    if exclude_indices.contains(&i) {  // <-- FIX 🟢: Skip excluded columns
                        continue;
                    }
                    row_values.push(
                        field
                            .parse::<f64>()
                            .map_err(|e| DataLoadingError::OsError(e.to_string()))?,
                    );
                }

                if width == 0 {
                    width = row_values.len();
                }
                data.extend(row_values);
            }

            let height = if width == 0 { 0 } else { data.len() / width };
            CausalTensor::new(data, vec![height, width])
                .map_err(|e| DataLoadingError::OsError(e.to_string()))
        } else {
            Err(DataLoadingError::OsError(
                "Invalid config type for CsvDataLoader".to_string(),
            ))
        }
    }
}
```

Additionally, add test coverage to verify the fix works correctly and to prevent regression.


# Summary
- **Context**: `SurdCausalDiscovery` implements the `CausalDiscovery` trait and serves as a bridge between the CDL pipeline and the SURD causal discovery algorithm from `deep_causality_algorithms`.
- **Bug**: The `target_col` parameter in `SurdConfig` is completely ignored by `SurdCausalDiscovery::discover_res()`.
- **Actual vs. expected**: The method only passes `max_order` to the underlying `surd_states_cdl` algorithm, ignoring `target_col`, whereas it should rearrange the tensor so that the target column becomes axis 0 (as required by the algorithm) before calling `surd_states_cdl`.
- **Impact**: Users specifying different target columns via `SurdConfig` will always get identical results, making causal discovery incorrect when the target is not already at axis 0 of the tensor.

# Code with bug
```rust
impl SurdCausalDiscovery {
    pub fn discover_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::SurdConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        Ok(surd_states_cdl(tensor, config.max_order())?)  // <-- BUG 🔴 config.target_col() is never used
    }
}
```

The `target_col()` method exists in `SurdConfig` but is never called:
```rust
impl SurdConfig {
    pub fn target_col(&self) -> usize {  // <-- This method is never called by SurdCausalDiscovery
        self.target_col
    }
}
```

# Evidence

## Failing test

### Test script
```rust
/*
 * Test to demonstrate that target_col parameter is ignored in SurdCausalDiscovery
 */

use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::{
    CausalDiscovery, CausalDiscoveryConfig, SurdCausalDiscovery, SurdConfig,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_target_col_parameter_is_ignored() {
    // Create a 3D joint probability distribution tensor
    // Shape: [2, 2, 2] representing [dim0_states, dim1_states, dim2_states]
    let data = vec![
        Some(0.1), Some(0.2), // P(D0=0, D1=0, D2=0), P(D0=0, D1=0, D2=1)
        Some(0.05), Some(0.15), // P(D0=0, D1=1, D2=0), P(D0=0, D1=1, D2=1)
        Some(0.2), Some(0.1), // P(D0=1, D1=0, D2=0), P(D0=1, D1=0, D2=1)
        Some(0.1), Some(0.1), // P(D0=1, D1=1, D2=0), P(D0=1, D1=1, D2=1)
    ];
    let tensor = CausalTensor::new(data, vec![2, 2, 2]).unwrap();

    // Test 1: Use target_col = 0
    let config1 = CausalDiscoveryConfig::Surd(SurdConfig::new(MaxOrder::Max, 0));
    let discoverer = SurdCausalDiscovery;
    let result1 = discoverer.discover(tensor.clone(), &config1).unwrap();

    // Test 2: Use target_col = 1 (should give different results if parameter is used)
    let config2 = CausalDiscoveryConfig::Surd(SurdConfig::new(MaxOrder::Max, 1));
    let result2 = discoverer.discover(tensor.clone(), &config2).unwrap();

    // Test 3: Use target_col = 2 (should give different results if parameter is used)
    let config3 = CausalDiscoveryConfig::Surd(SurdConfig::new(MaxOrder::Max, 2));
    let result3 = discoverer.discover(tensor.clone(), &config3).unwrap();

    // Print the results
    println!("\n=== Testing target_col Parameter ===");
    println!("Result with target_col=0: info_leak={}", result1.info_leak());
    println!("Result with target_col=1: info_leak={}", result2.info_leak());
    println!("Result with target_col=2: info_leak={}", result3.info_leak());

    let info_leak_1 = result1.info_leak();
    let info_leak_2 = result2.info_leak();
    let info_leak_3 = result3.info_leak();

    // If target_col is being ignored, all results will be identical
    let all_identical = (info_leak_1 - info_leak_2).abs() < 1e-10
        && (info_leak_1 - info_leak_3).abs() < 1e-10;

    if all_identical {
        println!("\n❌ BUG CONFIRMED: target_col parameter is IGNORED!");
        println!("   All three different target_col values produce identical results.");
        // This assertion will fail, proving the bug exists
        panic!("BUG: target_col parameter in SurdConfig is ignored by SurdCausalDiscovery::discover_res");
    } else {
        println!("\n✓ target_col parameter is being used correctly");
    }
}
```

### Test output
```
running 1 test
test test_target_col_parameter_is_ignored ... FAILED

failures:

---- test_target_col_parameter_is_ignored stdout ----

=== Testing target_col Parameter ===
Result with target_col=0: info_leak=0.9314595241545356
Result with target_col=1: info_leak=0.9314595241545356
Result with target_col=2: info_leak=0.9314595241545356

❌ BUG CONFIRMED: target_col parameter is IGNORED!
   All three different target_col values produce identical results.

thread 'test_target_col_parameter_is_ignored' (4359) panicked at deep_causality_discovery/tests/bug_target_col_ignored.rs:54:9:
BUG: target_col parameter in SurdConfig is ignored by SurdCausalDiscovery::discover_res
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    test_target_col_parameter_is_ignored

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Inconsistency with own spec / docstring

### Reference spec / comment

From `deep_causality_discovery/src/types/config/surd_config.rs`:
```rust
pub struct SurdConfig {
    max_order: MaxOrder,
    target_col: usize,  // <-- Field exists in config
}

impl SurdConfig {
    /// The index of the target column for the causal analysis.
    pub fn target_col(&self) -> usize {
        self.target_col
    }
}
```

The docstring explicitly states this field represents "The index of the target column for the causal analysis", indicating it should be used during analysis.

### Current code

From `deep_causality_discovery/src/types/causal_discovery/surd.rs`:
```rust
impl SurdCausalDiscovery {
    pub fn discover_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::SurdConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        Ok(surd_states_cdl(tensor, config.max_order())?)
        // Note: config.target_col() is never called
    }
}
```

### Contradiction

The `SurdConfig` struct includes a `target_col` field with a public accessor method and documentation describing its purpose. However, `SurdCausalDiscovery::discover_res()` only uses `config.max_order()` and never calls `config.target_col()`, making this configuration parameter completely non-functional.

## Inconsistency with API documentation

### Reference API documentation

From `deep_causality_algorithms/src/causal_discovery/surd/surd_algo_cdl.rs` (lines 93-96):
```rust
/// # Arguments
/// * `p_raw` - A `CausalTensor<Option<f64>>` representing the joint probability distribution.
///   **Crucially, this must be a joint probability distribution of discrete, binned data.**
///   The first dimension (axis 0) must correspond to the target variable.
/// * `max_order` - An enum specifying the maximum order of interactions to compute.
```

The documentation explicitly states: **"The first dimension (axis 0) must correspond to the target variable."**

### Current API usage

In `deep_causality_discovery/src/types/causal_discovery/surd.rs`:
```rust
pub fn discover_res(
    tensor: &CausalTensor<Option<f64>>,
    config: &crate::SurdConfig,
) -> Result<SurdResult<f64>, CausalDiscoveryError> {
    Ok(surd_states_cdl(tensor, config.max_order())?)
    // Passes tensor directly without ensuring target is at axis 0
}
```

### Contradiction

The `surd_states_cdl` function requires the target variable to be at axis 0 of the input tensor. However, `SurdCausalDiscovery::discover_res()` passes the tensor directly to `surd_states_cdl` without any transformation, ignoring the `target_col` parameter that specifies which axis should be treated as the target. This violates the API contract of `surd_states_cdl`.

## Inconsistency within the codebase

### Reference code

`deep_causality_discovery/src/types/feature_selector/mrmr.rs`:
```rust
impl MrmrFeatureSelector {
    pub fn select_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::MrmrConfig,
    ) -> Result<MrmrResult, MrmrError> {
        mrmr_features_selector(tensor, config.num_features(), config.target_col())
        //                                                    ^^^^^^^^^^^^^^^^^^
        //                                                    target_col is used
    }
}
```

`deep_causality_discovery/src/types/config/mrmr_config.rs`:
```rust
pub struct MrmrConfig {
    num_features: usize,
    target_col: usize,  // <-- Same field pattern
}

impl MrmrConfig {
    pub fn target_col(&self) -> usize {
        self.target_col
    }
}
```

### Current code

`deep_causality_discovery/src/types/causal_discovery/surd.rs`:
```rust
impl SurdCausalDiscovery {
    pub fn discover_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::SurdConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        Ok(surd_states_cdl(tensor, config.max_order())?)
        //                         ^^^^^^^^^^^^^^^^^^^
        //                         target_col is NOT used
    }
}
```

`deep_causality_discovery/src/types/config/surd_config.rs`:
```rust
pub struct SurdConfig {
    max_order: MaxOrder,
    target_col: usize,  // <-- Same field pattern
}

impl SurdConfig {
    pub fn target_col(&self) -> usize {
        self.target_col
    }
}
```

### Comparison

Both `MrmrConfig` and `SurdConfig` have the same field pattern with a `target_col` field and accessor method. `MrmrFeatureSelector::select_res()` correctly passes `config.target_col()` to the underlying algorithm. However, `SurdCausalDiscovery::discover_res()` ignores `config.target_col()` entirely, creating an inconsistency in how similar configuration parameters are handled across the codebase.

# Full context

The `SurdCausalDiscovery` struct is used within the Causal Discovery Language (CDL) pipeline, which is a monadic, type-state-based pipeline for causal discovery. The pipeline flow is:

1. **Data Loading** (`WithData` state) - Loads raw data from CSV/Parquet
2. **Data Cleaning** (`WithCleanedData` state) - Cleans data using implementations like `OptionNoneDataCleaner`
3. **Feature Selection** (`WithFeatures` state) - Selects relevant features using `MrmrFeatureSelector` or similar
4. **Causal Discovery** (`WithCausalResults` state) - Performs causal discovery using `SurdCausalDiscovery` **← Bug is here**
5. **Analysis** (`WithAnalysis` state) - Analyzes the causal discovery results
6. **Finalization** (`Complete` state) - Formats and presents results

The `SurdCausalDiscovery` implementation is invoked at step 4 when users call `.causal_discovery()` with a `CausalDiscoveryConfig::Surd` configuration. The configuration is typically created with both `max_order` and `target_col` parameters:

```rust
let surd_config = SurdConfig::new(MaxOrder::Max, target_col);
let config = CausalDiscoveryConfig::Surd(surd_config);
let discoverer = SurdCausalDiscovery;
let result = discoverer.discover(tensor, &config)?;
```

The underlying SURD algorithm (`surd_states_cdl` from `deep_causality_algorithms`) performs information-theoretic decomposition of causal influences into Synergistic, Unique, and Redundant (SURD) components. The algorithm computes mutual information, conditional entropy, and other information-theoretic measures between the target variable (at axis 0) and source variables (remaining axes).

For the algorithm to work correctly, the tensor must be arranged with the target variable at axis 0. The `target_col` parameter is meant to specify which column should be treated as the target. When users have a multi-dimensional probability distribution where the target is not already at axis 0, the `discover_res` method should permute the tensor axes to move the target column to axis 0 before calling `surd_states_cdl`.

## External documentation

- [CausalTensor permute_axes documentation](from the codebase at `deep_causality_tensor/src/types/causal_tensor/ops/tensor_shape/mod.rs`)
```rust
pub(in crate::types::causal_tensor) fn permute_axes_impl(
    &self,
    axes: &[usize],
) -> Result<Self, CausalTensorError> {
    if axes.len() != self.num_dim() {
        return Err(CausalTensorError::DimensionMismatch);
    }

    // Validate axes uniqueness and bounds
    let mut seen_axes = vec![false; self.num_dim()];
    for &axis in axes {
        if axis >= self.num_dim() || seen_axes[axis] {
            return Err(CausalTensorError::InvalidParameter(
                "Invalid axes permutation".to_string(),
            ));
        }
        seen_axes[axis] = true;
    }

    let mut new_shape = Vec::with_capacity(self.num_dim());
    let mut new_strides = Vec::with_capacity(self.num_dim());

    for &axis in axes {
        new_shape.push(self.shape[axis]);
        // This creates a correct strided view of the original data.
        new_strides.push(self.strides[axis]);
    }

    Ok(Self {
        data: self.data.clone(),
        shape: new_shape,
        strides: new_strides,
    })
}
```

This function is available via the `Tensor` trait's `permute_axes` method and could be used to rearrange the tensor so that `target_col` becomes axis 0.

# Why has this bug gone undetected?

This bug has gone undetected for several reasons:

1. **No validation in existing tests**: All existing tests in `deep_causality_discovery/tests/types/causal_discovery/surd_tests.rs` use `target_col=0`, which happens to be the default axis that `surd_states_cdl` expects. Since the target is already at axis 0, the bug has no visible effect in these tests.

2. **Direct API usage bypasses the bug**: The main example in `deep_causality_discovery/examples/main.rs` calls `surd_states_cdl` directly as a closure, bypassing the `SurdCausalDiscovery` wrapper entirely:
   ```rust
   cdl.causal_discovery(|tensor| {
       surd_states_cdl(tensor, MaxOrder::Max).map_err(Into::into)
   })
   ```
   This means the config-based interface with `SurdConfig` is not exercised in the example.

3. **Limited real-world usage**: The CDL pipeline and SURD algorithm appear to be relatively new (initial implementation in commit f470998a). There may not yet be production users who have tried to specify a target column other than 0.

4. **Confusing API semantics**: The relationship between `target_col` (which implies a column index in tabular data) and tensor axes (which are multi-dimensional) is not immediately obvious. Users might assume the data is already arranged correctly and always use `target_col=0`, or they might not understand when/how to use this parameter.

5. **Algorithm documentation gap**: While the `surd_states_cdl` documentation clearly states that axis 0 must be the target, there's no documentation in `SurdConfig` or `SurdCausalDiscovery` explaining that the implementation will handle the axis permutation. This creates an expectation gap where the parameter exists but its intended behavior is unclear.

# Recommended fix

The fix should make `SurdCausalDiscovery::discover_res()` use the `target_col` parameter to permute the tensor axes before calling `surd_states_cdl`:

```rust
impl SurdCausalDiscovery {
    pub fn discover_res(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::SurdConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError> {
        let target_col = config.target_col();

        // If target is not already at axis 0, permute axes to move it there
        let arranged_tensor = if target_col != 0 {
            let num_dims = tensor.num_dim();

            // Create permutation: [target_col, 0, 1, ..., target_col-1, target_col+1, ..., num_dims-1]
            let mut axes: Vec<usize> = Vec::with_capacity(num_dims);
            axes.push(target_col); // Target goes to position 0
            for i in 0..num_dims {
                if i != target_col {
                    axes.push(i);
                }
            }

            tensor.permute_axes(&axes)?  // <-- FIX 🟢 Use permute_axes to rearrange tensor
        } else {
            tensor.clone()
        };

        Ok(surd_states_cdl(&arranged_tensor, config.max_order())?)  // <-- FIX 🟢 Use arranged tensor
    }
}
```

Additionally, validation should be added to ensure `target_col` is within bounds:
```rust
if target_col >= tensor.num_dim() {
    return Err(CausalDiscoveryError::TensorError(
        CausalTensorError::InvalidParameter(
            format!("target_col {} is out of bounds for tensor with {} dimensions",
                    target_col, tensor.num_dim())
        )
    ));
}
```

# Related bugs

There are no other instances of this specific bug in the codebase. The `MrmrFeatureSelector` correctly uses its `target_col` parameter. However, it would be prudent to audit all config-based implementations in the `deep_causality_discovery` crate to ensure all configuration parameters are being used correctly.
