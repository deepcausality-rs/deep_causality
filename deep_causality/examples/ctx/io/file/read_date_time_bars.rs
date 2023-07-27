// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use std::fs::File;
use std::path::Path;

use parquet::file::reader::{FileReader, SerializedFileReader};

use crate::io::file::parquet_2_bar;
use crate::types::date_time_bar::DateTimeBar;
use crate::types::parquet_config::ParquetConfig;

pub fn read_date_time_bars_from_parquet(
    config: &ParquetConfig,
)
    -> Result<Vec<DateTimeBar>, Box<dyn Error>>
{
    let mut content: Vec<DateTimeBar> = Vec::with_capacity(500); // fixed pre-allocation

    let path = config.path();
    let file = File::open(&Path::new(path)).expect("Could not open file");

    let reader = SerializedFileReader::new(file)
        .expect("Could not create parquet reader");

    let iter = reader
        .get_row_iter(None)
        .expect("Could not create parquet row iterator");

    for row in iter {
        content.push(parquet_2_bar::convert_field_to_sampled_date_time_bar(&row.unwrap()));
    }

    Ok(content)
}