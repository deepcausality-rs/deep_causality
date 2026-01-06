/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    DataLoader, DataLoaderConfig, DataLoadingError, ParquetConfig, ParquetDataLoader,
};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

use deep_causality_tensor::CausalTensor;
use parquet::basic::Type as PhysicalType;
use parquet::column::writer::ColumnWriter;
use parquet::data_type::ByteArray;
use parquet::file::properties::WriterProperties;
use parquet::file::writer::SerializedFileWriter;
use parquet::record::Field;
use parquet::schema::types::Type as SchemaType;
use std::sync::Arc;

// Helper function to create a test Parquet file
fn create_test_parquet_file(
    file_path: &Path,
    schema: Arc<SchemaType>,
    data: Vec<Vec<Field>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let props = Arc::new(WriterProperties::builder().build());
    let file = fs::File::create(file_path)?;
    let mut writer = SerializedFileWriter::new(file, schema.clone(), props)?;

    if data.is_empty() {
        writer.close()?;
        return Ok(());
    }

    let mut row_group_writer = writer.next_row_group()?;

    for (col_idx, column_schema) in schema.get_fields().iter().enumerate() {
        let mut column_writer = row_group_writer.next_column()?.unwrap();
        match column_writer.untyped() {
            ColumnWriter::BoolColumnWriter(typed_writer) => {
                let mut bool_data = Vec::new();
                for row in &data {
                    if let Field::Bool(b) = row[col_idx] {
                        bool_data.push(b);
                    }
                }
                let def_levels = vec![1i16; bool_data.len()];
                typed_writer.write_batch(&bool_data, Some(&def_levels), None)?;
            }
            ColumnWriter::Int32ColumnWriter(typed_writer) => {
                let mut int_data = Vec::new();
                for row in &data {
                    match row[col_idx] {
                        Field::Int(i) => int_data.push(i),
                        Field::Byte(b) => int_data.push(b as i32),
                        Field::Short(s) => int_data.push(s as i32),
                        Field::UByte(ub) => int_data.push(ub as i32),
                        Field::UShort(us) => int_data.push(us as i32),
                        Field::UInt(ui) => int_data.push(ui as i32),
                        _ => {
                            return Err(format!(
                                "Unexpected Field type for INT32 column: {:?}",
                                row[col_idx]
                            )
                            .into());
                        }
                    }
                }
                let def_levels = vec![1i16; int_data.len()];
                typed_writer.write_batch(&int_data, Some(&def_levels), None)?;
            }
            ColumnWriter::Int64ColumnWriter(typed_writer) => {
                let mut long_data = Vec::new();
                for row in &data {
                    if let Field::Long(l) = row[col_idx] {
                        long_data.push(l);
                    }
                }
                let def_levels = vec![1i16; long_data.len()];
                typed_writer.write_batch(&long_data, Some(&def_levels), None)?;
            }
            ColumnWriter::FloatColumnWriter(typed_writer) => {
                let mut float_data = Vec::new();
                for row in &data {
                    if let Field::Float(f) = row[col_idx] {
                        float_data.push(f);
                    }
                }
                let def_levels = vec![1i16; float_data.len()];
                typed_writer.write_batch(&float_data, Some(&def_levels), None)?;
            }
            ColumnWriter::DoubleColumnWriter(typed_writer) => {
                let mut double_data = Vec::new();
                for row in &data {
                    if let Field::Double(d) = row[col_idx] {
                        double_data.push(d);
                    }
                }
                let def_levels = vec![1i16; double_data.len()];
                typed_writer.write_batch(&double_data, Some(&def_levels), None)?;
            }
            ColumnWriter::ByteArrayColumnWriter(typed_writer) => {
                let mut byte_array_data = Vec::new();
                for row in &data {
                    if let Field::Bytes(b) = &row[col_idx] {
                        byte_array_data.push(b.clone());
                    }
                }
                let def_levels = vec![1i16; byte_array_data.len()];
                typed_writer.write_batch(&byte_array_data, Some(&def_levels), None)?;
            }
            _ => {
                return Err(format!(
                    "Unsupported column writer type for test: {:?}",
                    column_schema.get_physical_type()
                )
                .into());
            }
        }
        column_writer.close()?;
    }

    row_group_writer.close()?;
    writer.close()?;
    Ok(())
}

#[test]
fn test_parquet_data_loader_load_error_file_not_found() {
    let loader = ParquetDataLoader;
    let parquet_config = ParquetConfig::new(None, 1024, None, None, vec![]);
    let config = DataLoaderConfig::Parquet(parquet_config);

    let result = loader.load("non_existent_file.parquet", &config);
    assert!(result.is_err());
    if let Err(DataLoadingError::OsError(e)) = result {
        assert!(e.contains("No such file or directory"));
    } else {
        panic!("Expected OsError, got {:?}", result);
    }
}

#[test]
fn test_parquet_data_loader_load_error_invalid_config_type() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();
    fs::write(file_path, "dummy content").unwrap();

    let loader = ParquetDataLoader;
    let csv_config =
        deep_causality_discovery::CsvConfig::new(false, b',', 0, None, None, None, vec![]);
    let config = DataLoaderConfig::Csv(csv_config);

    let result = loader.load(file_path, &config);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        DataLoadingError::OsError("Invalid config type for ParquetDataLoader".to_string())
    );
}

#[test]
fn test_parquet_data_loader_load_success_various_types() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_path_buf();

    let schema = Arc::new(
        SchemaType::group_type_builder("schema")
            .with_fields(vec![
                Arc::new(
                    SchemaType::primitive_type_builder("col_bool", PhysicalType::BOOLEAN)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_int", PhysicalType::INT32)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_long", PhysicalType::INT64)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_byte", PhysicalType::INT32)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_short", PhysicalType::INT32)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_ubyte", PhysicalType::INT32)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_ushort", PhysicalType::INT32)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_uint", PhysicalType::INT32)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_float", PhysicalType::FLOAT)
                        .build()
                        .unwrap(),
                ),
                Arc::new(
                    SchemaType::primitive_type_builder("col_double", PhysicalType::DOUBLE)
                        .build()
                        .unwrap(),
                ),
            ])
            .build()
            .unwrap(),
    );

    let data = vec![
        vec![
            Field::Bool(true),
            Field::Int(1),
            Field::Long(100),
            Field::Byte(10),
            Field::Short(20),
            Field::UByte(30),
            Field::UShort(40),
            Field::UInt(50),
            Field::Float(1.1),
            Field::Double(10.1),
        ],
        vec![
            Field::Bool(false),
            Field::Int(2),
            Field::Long(200),
            Field::Byte(11),
            Field::Short(21),
            Field::UByte(31),
            Field::UShort(41),
            Field::UInt(51),
            Field::Float(2.2),
            Field::Double(20.2),
        ],
        vec![
            Field::Bool(true),
            Field::Int(3),
            Field::Long(300),
            Field::Byte(12),
            Field::Short(22),
            Field::UByte(32),
            Field::UShort(42),
            Field::UInt(52),
            Field::Float(3.3),
            Field::Double(30.3),
        ],
    ];

    create_test_parquet_file(&file_path, schema, data.clone()).unwrap();

    let loader = ParquetDataLoader;
    let parquet_config = ParquetConfig::new(None, 1024, None, None, vec![]);
    let config = DataLoaderConfig::Parquet(parquet_config);

    let result = loader.load(file_path.to_str().unwrap(), &config).unwrap();

    let expected_data = vec![
        1.0,
        1.0,
        100.0,
        10.0,
        20.0,
        30.0,
        40.0,
        50.0,
        1.1f32 as f64,
        10.1,
        0.0,
        2.0,
        200.0,
        11.0,
        21.0,
        31.0,
        41.0,
        51.0,
        2.2f32 as f64,
        20.2,
        1.0,
        3.0,
        300.0,
        12.0,
        22.0,
        32.0,
        42.0,
        52.0,
        3.3f32 as f64,
        30.3,
    ];
    let expected_shape = vec![3, 10];
    let expected_tensor = CausalTensor::new(expected_data.clone(), expected_shape).unwrap();

    let epsilon = 1e-9; // Define a small tolerance for floating-point comparisons

    // Compare data values with a tolerance for floating-point numbers
    for (i, (&expected_val, &actual_val)) in expected_data
        .iter()
        .zip(result.as_slice().iter())
        .enumerate()
    {
        if expected_val.is_nan() {
            assert!(actual_val.is_nan(), "NaN mismatch at index {}", i);
        } else {
            assert!(
                (actual_val - expected_val).abs() < epsilon,
                "Value mismatch at index {}: expected {}, got {}",
                i,
                expected_val,
                actual_val
            );
        }
    }
    // Compare shapes directly as they are integer-based
    assert_eq!(result.shape(), expected_tensor.shape());
}

#[test]
fn test_parquet_data_loader_load_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_path_buf();

    let schema = Arc::new(
        SchemaType::group_type_builder("schema")
            .with_fields(vec![Arc::new(
                SchemaType::primitive_type_builder("col_int", PhysicalType::INT32)
                    .build()
                    .unwrap(),
            )])
            .build()
            .unwrap(),
    );

    // Create an empty Parquet file (schema only, no rows)
    create_test_parquet_file(&file_path, schema, vec![]).unwrap();

    let loader = ParquetDataLoader;
    let parquet_config = ParquetConfig::new(None, 1024, None, None, vec![]);
    let config = DataLoaderConfig::Parquet(parquet_config);

    let result = loader.load(file_path.to_str().unwrap(), &config).unwrap();
    let expected = CausalTensor::new(Vec::<f64>::new(), vec![0, 0]).unwrap();

    assert_eq!(result.as_slice(), expected.as_slice());
    assert_eq!(result.shape(), expected.shape());
}

#[test]
fn test_parquet_data_loader_load_error_unsupported_type() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_path_buf();

    let schema = Arc::new(
        SchemaType::group_type_builder("schema")
            .with_fields(vec![Arc::new(
                SchemaType::primitive_type_builder("col_byte_array", PhysicalType::BYTE_ARRAY)
                    .build()
                    .unwrap(),
            )])
            .build()
            .unwrap(),
    );

    let data = vec![vec![Field::Bytes(ByteArray::from(vec![1, 2, 3]))]];

    create_test_parquet_file(&file_path, schema, data).unwrap();

    let loader = ParquetDataLoader;
    let parquet_config = ParquetConfig::new(None, 1024, None, None, vec![]);
    let config = DataLoaderConfig::Parquet(parquet_config);

    let result = loader.load(file_path.to_str().unwrap(), &config);
    assert!(result.is_err());
    if let Err(DataLoadingError::OsError(e)) = result {
        assert!(e.contains("Unsupported data type"));
    } else {
        panic!(
            "Expected DataError::OsError for unsupported type, got {:?}",
            result
        );
    }
}
