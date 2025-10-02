/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use arrow_array::{Array, Float64Array, Int64Array, StringArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::record::RecordWriter;
use parquet_derive::ParquetRecordWriter;
use std::fs::File;
use std::sync::Arc;
use arrow_array::cast::__private::DataType;

const OUT_SEPS_TRUE_PATH: &str = "examples/case_study_icu_sepsis/data/seperated/seps_true.parquet";
const OUT_SEPS_FALSE_PATH: &str =
    "examples/case_study_icu_sepsis/data/seperated/seps_false.parquet";

// Util to separate the full data set into two disjoint data sets of which one contains
// only non-sepsis records and the other one contains only sepsis records.

#[derive(Debug, Clone, ParquetRecordWriter)]
pub struct SepsisRecord {
    pub Hour: f64,
    pub HR: Option<f64>,
    pub O2Sat: Option<f64>,
    pub Temp: Option<f64>,
    pub SBP: Option<f64>,
    pub MAP: Option<f64>,
    pub DBP: Option<f64>,
    pub Resp: Option<f64>,
    pub EtCO2: Option<f64>,
    pub BaseExcess: Option<f64>,
    pub HCO3: Option<f64>,
    pub FiO2: Option<f64>,
    pub pH: Option<f64>,
    pub PaCO2: Option<f64>,
    pub SaO2: Option<f64>,
    pub AST: Option<f64>,
    pub BUN: Option<f64>,
    pub Alkalinephos: Option<f64>,
    pub Calcium: Option<f64>,
    pub Chloride: Option<f64>,
    pub Creatinine: Option<f64>,
    pub Bilirubin_direct: Option<f64>,
    pub Glucose: Option<f64>,
    pub Lactate: Option<f64>,
    pub Magnesium: Option<f64>,
    pub Phosphate: Option<f64>,
    pub Potassium: Option<f64>,
    pub Bilirubin_total: Option<f64>,
    pub TroponinI: Option<f64>,
    pub Hct: Option<f64>,
    pub Hgb: Option<f64>,
    pub PTT: Option<f64>,
    pub WBC: Option<f64>,
    pub Fibrinogen: Option<f64>,
    pub Platelets: Option<f64>,
    pub Age: f64,
    pub Gender: f64,
    pub Unit1: f64,
    pub Unit2: f64,
    pub HospAdmTime: f64,
    pub ICULOS: f64,
    pub SepsisLabel: f64,
    pub Patient_ID: i64,
}

pub(crate) fn zero_stage(data_path: &str) {
    let (sepsis_true_records, sepsis_false_records) = load_and_partition_data(data_path);

    write_records(OUT_SEPS_TRUE_PATH, sepsis_true_records);
    write_records(OUT_SEPS_FALSE_PATH, sepsis_false_records);
}

fn load_and_partition_data(data_path: &str) -> (Vec<SepsisRecord>, Vec<SepsisRecord>) {
    let file = File::open(data_path).expect("Failed to open file");
    let reader = ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("Failed to create ParquetRecordBatchReaderBuilder")
        .build()
        .expect("Failed to build ParquetRecordBatchReader");

    let mut sepsis_true_records = Vec::new();
    let mut sepsis_false_records = Vec::new();

    for batch in reader {
        let batch = batch.expect("Failed to read record batch");
        let num_rows = batch.num_rows();

        let mut records = Vec::with_capacity(num_rows);

        for row_idx in 0..num_rows {
            let record = SepsisRecord {
                Hour: get_f64(batch.column(0), row_idx),
                HR: get_optional_f64(batch.column(1), row_idx),
                O2Sat: get_optional_f64(batch.column(2), row_idx),
                Temp: get_optional_f64(batch.column(3), row_idx),
                SBP: get_optional_f64(batch.column(4), row_idx),
                MAP: get_optional_f64(batch.column(5), row_idx),
                DBP: get_optional_f64(batch.column(6), row_idx),
                Resp: get_optional_f64(batch.column(7), row_idx),
                EtCO2: get_optional_f64_from_string(batch.column(8), row_idx),
                BaseExcess: get_optional_f64(batch.column(9), row_idx),
                HCO3: get_optional_f64(batch.column(10), row_idx),
                FiO2: get_optional_f64(batch.column(11), row_idx),
                pH: get_optional_f64(batch.column(12), row_idx),
                PaCO2: get_optional_f64(batch.column(13), row_idx),
                SaO2: get_optional_f64(batch.column(14), row_idx),
                AST: get_optional_f64(batch.column(15), row_idx),
                BUN: get_optional_f64(batch.column(16), row_idx),
                Alkalinephos: get_optional_f64(batch.column(17), row_idx),
                Calcium: get_optional_f64(batch.column(18), row_idx),
                Chloride: get_optional_f64(batch.column(19), row_idx),
                Creatinine: get_optional_f64(batch.column(20), row_idx),
                Bilirubin_direct: get_optional_f64(batch.column(21), row_idx),
                Glucose: get_optional_f64(batch.column(22), row_idx),
                Lactate: get_optional_f64(batch.column(23), row_idx),
                Magnesium: get_optional_f64(batch.column(24), row_idx),
                Phosphate: get_optional_f64(batch.column(25), row_idx),
                Potassium: get_optional_f64(batch.column(26), row_idx),
                Bilirubin_total: get_optional_f64(batch.column(27), row_idx),
                TroponinI: get_optional_f64(batch.column(28), row_idx),
                Hct: get_optional_f64(batch.column(29), row_idx),
                Hgb: get_optional_f64(batch.column(30), row_idx),
                PTT: get_optional_f64(batch.column(31), row_idx),
                WBC: get_optional_f64(batch.column(32), row_idx),
                Fibrinogen: get_optional_f64(batch.column(33), row_idx),
                Platelets: get_optional_f64(batch.column(34), row_idx),
                Age: get_f64(batch.column(35), row_idx),
                Gender: get_f64(batch.column(36), row_idx),
                Unit1: get_f64(batch.column(37), row_idx),
                Unit2: get_f64(batch.column(38), row_idx),
                HospAdmTime: get_f64(batch.column(39), row_idx),
                ICULOS: get_f64(batch.column(40), row_idx),
                SepsisLabel: get_f64(batch.column(41), row_idx),
                Patient_ID: get_i64(batch.column(42), row_idx),
            };
            records.push(record);
        }

        for record in records {
            if record.SepsisLabel == 1.0 {
                sepsis_true_records.push(record);
            } else {
                sepsis_false_records.push(record);
            }
        }
    }

    (sepsis_true_records, sepsis_false_records)
}

fn get_f64(column: &dyn Array, row_idx: usize) -> f64 {
    match column.data_type() {
        DataType::Float64 => {
            let float_array = column.as_any().downcast_ref::<Float64Array>().unwrap();
            float_array.value(row_idx)
        }
        DataType::Int64 => {
            let int_array = column.as_any().downcast_ref::<Int64Array>().unwrap();
            int_array.value(row_idx) as f64
        }
        _ => panic!("Unsupported data type"),
    }
}

fn get_i64(column: &dyn Array, row_idx: usize) -> i64 {
    match column.data_type() {
        DataType::Float64 => {
            let float_array = column.as_any().downcast_ref::<Float64Array>().unwrap();
            float_array.value(row_idx) as i64
        }
        DataType::Int64 => {
            let int_array = column.as_any().downcast_ref::<Int64Array>().unwrap();
            int_array.value(row_idx)
        }
        _ => panic!("Unsupported data type for Patient_ID"),
    }
}

fn get_optional_f64(column: &dyn Array, row_idx: usize) -> Option<f64> {
    if column.is_valid(row_idx) {
        match column.data_type() {
            DataType::Float64 => {
                let float_array = column.as_any().downcast_ref::<Float64Array>().unwrap();
                Some(float_array.value(row_idx))
            }
            DataType::Int64 => {
                let int_array = column.as_any().downcast_ref::<Int64Array>().unwrap();
                Some(int_array.value(row_idx) as f64)
            }
            _ => None,
        }
    } else {
        None
    }
}

fn get_optional_f64_from_string(column: &dyn Array, row_idx: usize) -> Option<f64> {
    if column.is_valid(row_idx) {
        let string_array = column.as_any().downcast_ref::<StringArray>().unwrap();
        string_array.value(row_idx).parse::<f64>().ok()
    } else {
        None
    }
}

fn write_records(path: &str, records: Vec<SepsisRecord>) {
    if records.is_empty() {
        return;
    }

    let schema = records.as_slice().schema().unwrap();
    let props = Arc::new(WriterProperties::builder().build());
    let file = File::create(path).unwrap();
    let mut writer = SerializedFileWriter::new(file, schema, props).unwrap();

    let mut row_group = writer.next_row_group().unwrap();
    records
        .as_slice()
        .write_to_row_group(&mut row_group)
        .unwrap();
    row_group.close().unwrap();
    writer.close().unwrap();
}