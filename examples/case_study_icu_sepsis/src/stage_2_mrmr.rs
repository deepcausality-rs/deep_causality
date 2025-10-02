/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use arrow_array::cast::__private::DataType;
use arrow_array::{Array, Float64Array, Int64Array, RecordBatchReader, StringArray};
use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_tensor::CausalTensor;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;

const NUM_FEATURES: usize = 39; // 41 original features - 1 excluded column (Patient_ID) = 40 features. Target is 40. So 39 features to select.
const TARGET_COLUMN: usize = 41; // SepsisLabel
const EXCLUDE_COLUMN: usize = 42; // Patient ID.

pub(crate) fn second_stage(data_path: &str) {
    feature_select(data_path)
}

fn feature_select(data_path: &str) {
    // 1. Load the raw data into a tensor
    let (tensor, column_names, target_column) = load_and_convert_data(data_path);

    // 2. Run the feature selector for CDL version
    let selected_features_with_scores =
        mrmr_features_selector(&tensor, NUM_FEATURES, target_column).unwrap();

    // 3. Interpret the results
    println!("Selected features and their normalized scores (CDL):");
    for (index, score) in selected_features_with_scores {
        let (original_index, column_name) = &column_names[index];
        println!(
            "- Feature: {} (Index: {}), Importance Score: {:.4}",
            column_name, original_index, score
        );
    }
}

fn load_and_convert_data(
    data_path: &str,
) -> (CausalTensor<Option<f64>>, Vec<(usize, String)>, usize) {
    let file = File::open(data_path).expect("Failed to open file");
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create ParquetRecordBatchReaderBuilder")
        .build()
        .expect("Failed to build ParquetRecordBatchReader");

    let schema = reader.schema();
    let original_column_names: Vec<String> = schema
        .fields()
        .iter()
        .map(|f| f.name().to_string())
        .collect();
    let original_num_cols = schema.fields().len();
    let mut collected_column_data: Vec<Vec<Option<f64>>> = vec![Vec::new(); original_num_cols];
    let mut num_rows = 0;

    for batch in reader.into_iter() {
        let batch = batch.expect("Failed to read record batch");
        num_rows += batch.num_rows();

        for (col_idx, column_data) in collected_column_data.iter_mut().enumerate() {
            if col_idx == EXCLUDE_COLUMN {
                continue; // Skip the excluded column during data collection
            }
            let column = batch.column(col_idx);
            match column.data_type() {
                DataType::Float64 => {
                    let float_array = column.as_any().downcast_ref::<Float64Array>().unwrap();
                    for row_idx in 0..batch.num_rows() {
                        if float_array.is_valid(row_idx) {
                            let value = float_array.value(row_idx);
                            if value.is_nan() {
                                column_data.push(None);
                            } else {
                                column_data.push(Some(value));
                            }
                        } else {
                            column_data.push(None);
                        }
                    }
                }
                DataType::Int64 => {
                    let int_array = column.as_any().downcast_ref::<Int64Array>().unwrap();
                    for row_idx in 0..batch.num_rows() {
                        if int_array.is_valid(row_idx) {
                            column_data.push(Some(int_array.value(row_idx) as f64));
                        } else {
                            column_data.push(None);
                        }
                    }
                }
                DataType::Utf8 => {
                    let string_array = column.as_any().downcast_ref::<StringArray>().unwrap();
                    for row_idx in 0..batch.num_rows() {
                        if string_array.is_valid(row_idx) {
                            let value_str = string_array.value(row_idx);
                            match value_str.parse::<f64>() {
                                Ok(val) => column_data.push(Some(val)),
                                Err(_) => column_data.push(None), // Failed to parse
                            }
                        } else {
                            column_data.push(None);
                        }
                    }
                }
                _ => {
                    // Panic for any other unsupported data types
                    panic!(
                        "Unsupported data type encountered in column {}. Only Float64, Int64, and Utf8 (parsable to f64) are supported.",
                        schema.field(col_idx).name()
                    );
                }
            }
        }
    }

    // Filter out columns that are entirely None and the EXCLUDE_COLUMN
    let mut filtered_data: Vec<Option<f64>> = Vec::new();
    let mut new_num_cols = 0;
    let mut removed_cols_indices = Vec::new(); // Renamed to avoid confusion
    let mut filtered_column_names: Vec<(usize, String)> = Vec::new();

    for (col_idx, column_data) in collected_column_data.into_iter().enumerate() {
        if col_idx == EXCLUDE_COLUMN {
            println!(
                "Explicitly excluded column {} (index {}).",
                schema.field(col_idx).name(),
                col_idx
            );
            removed_cols_indices.push(col_idx);
            continue;
        }

        if column_data.iter().all(Option::is_none) {
            println!(
                "Removed column {} (index {}) due to all None values.",
                schema.field(col_idx).name(),
                col_idx
            );
            removed_cols_indices.push(col_idx);
        } else {
            new_num_cols += 1;
            filtered_data.extend(column_data);
            filtered_column_names.push((col_idx, original_column_names[col_idx].clone()));
        }
    }

    // Adjust TARGET_COLUMN if it was removed or if columns before it were removed
    let mut adjusted_target_column = TARGET_COLUMN;
    for &removed_col_idx in &removed_cols_indices {
        if removed_col_idx < adjusted_target_column {
            adjusted_target_column -= 1;
        }
    }
    // If the original TARGET_COLUMN was removed, this will be an issue. For now, panic.
    if removed_cols_indices.contains(&TARGET_COLUMN) {
        panic!(
            "Target column {} was removed due to all None values.",
            TARGET_COLUMN
        );
    }

    println!("Original number of columns: {}", original_num_cols);
    println!("New number of columns after filtering: {}", new_num_cols);
    println!("Target column index: {}", adjusted_target_column);

    (
        CausalTensor::new(filtered_data, vec![num_rows, new_num_cols])
            .expect("Failed to create CausalTensor after filtering"),
        filtered_column_names,
        adjusted_target_column,
    )
}
