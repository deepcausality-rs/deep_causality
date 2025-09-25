/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use arrow_array::cast::__private::DataType;
use arrow_array::{Array, Float64Array, Int64Array, RecordBatchReader, StringArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;

pub(crate) fn first_stage(data_path: &str) {
    load_and_analyze_data(data_path)
}

fn load_and_analyze_data(data_path: &str) {
    let file = File::open(data_path).expect("Failed to open file");
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create ParquetRecordBatchReaderBuilder")
        .build()
        .expect("Failed to build ParquetRecordBatchReader");

    let schema = reader.schema();

    let mut total_nan_count = 0;
    let mut unique_patient_ids = std::collections::HashSet::new();
    let mut unique_patient_sepsis = std::collections::HashSet::new();
    let mut total_patient_ids = 0;
    let mut patient_42_records: Vec<Vec<String>> = Vec::new();

    for batch in reader.into_iter() {
        let batch = batch.expect("Failed to read record batch");

        if let Some(patient_id_column) = batch.column_by_name("Patient_ID") {
            let patient_id_array = patient_id_column
                .as_any()
                .downcast_ref::<Float64Array>()
                .expect("Failed to cast Patient_ID column into Float64Array");

            total_nan_count += patient_id_array.null_count();
            total_patient_ids += patient_id_array.len() - patient_id_array.null_count();

            for i in 0..patient_id_array.len() {
                if patient_id_array.is_valid(i) {
                    let patient_id = patient_id_array.value(i);
                    unique_patient_ids.insert(patient_id as u64);

                    if let Some(sepsis_label_column) = batch.column_by_name("SepsisLabel") {
                        let sepsis_label_array = sepsis_label_column
                            .as_any()
                            .downcast_ref::<Float64Array>()
                            .expect("Failed to cast SepsisLabel column into Float64Array");

                        if sepsis_label_array.is_valid(i) && sepsis_label_array.value(i) == 1.0 {
                            unique_patient_sepsis.insert(patient_id as u64);
                        }
                    }

                    if patient_id == 42.0 {
                        let mut record_row: Vec<String> = Vec::new();
                        for (col_index, column) in batch.columns().iter().enumerate() {
                            let field = schema.field(col_index);
                            let column_name = field.name();
                            if column.is_valid(i) {
                                match column.data_type() {
                                    DataType::Float64 => {
                                        let arr =
                                            column.as_any().downcast_ref::<Float64Array>().unwrap();
                                        record_row.push(format!(
                                            "{}: {}",
                                            column_name,
                                            arr.value(i)
                                        ));
                                    }
                                    DataType::Int64 => {
                                        let arr =
                                            column.as_any().downcast_ref::<Int64Array>().unwrap();
                                        record_row.push(format!(
                                            "{}: {}",
                                            column_name,
                                            arr.value(i)
                                        ));
                                    }
                                    DataType::Utf8 => {
                                        let arr =
                                            column.as_any().downcast_ref::<StringArray>().unwrap();
                                        record_row.push(format!(
                                            "{}: {}",
                                            column_name,
                                            arr.value(i)
                                        ));
                                    }
                                    _ => record_row
                                        .push(format!("{}: Unsupported Type", column_name)),
                                }
                            } else {
                                record_row.push(format!("{}: NULL", column_name));
                            }
                        }
                        patient_42_records.push(record_row);
                    }
                }
            }
        }
    }

    let fields = schema.fields();

    println!("Data Schema:");
    for field in fields {
        println!("Column: {}", field.name());
    }
    println!();

    if total_nan_count > 1 {
        println!(
            "Total number of NaN values in Patient_ID column: {}",
            total_nan_count
        );
    }

    let unique_patient_ids_count = unique_patient_ids.len();

    println!("Data Records:");
    println!("Total unique Patient_IDs: {}", unique_patient_ids_count);
    println!(
        "Total number of data records for all patients: {}",
        total_patient_ids
    );
    println!(
        "Average number of data records per patient: {}",
        total_patient_ids / unique_patient_ids_count
    );
    println!(
        "Total unique patients with SepsisLabel = 1: {}",
        unique_patient_sepsis.len()
    );
    println!(
        "Percentage of patients with SepsisLabel = 1: {:.2}%",
        (unique_patient_sepsis.len() as f64 / unique_patient_ids_count as f64) * 100.0
    );
    println!();

    // println!("Records for Patient_ID 42:");
    // if patient_42_records.is_empty() {
    //     println!("No records found for Patient_ID 42.");
    // } else {
    //     for (record_index, record) in patient_42_records.iter().enumerate() {
    //         println!("--- Record {} ---", record_index + 1);
    //         for field_string in record {
    //             println!("{}", field_string);
    //         }
    //     }
    // }
}
