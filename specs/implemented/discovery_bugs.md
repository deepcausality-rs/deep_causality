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
