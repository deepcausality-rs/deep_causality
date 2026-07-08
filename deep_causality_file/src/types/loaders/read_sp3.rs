/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! RINEX SP3 precise-orbit (`.sp3`) loader as a lazy [`IoAction`].

use crate::{DataLoadingError, OrbitData, SatId};
use chrono::{NaiveDate, NaiveDateTime};
use deep_causality_algebra::RealField;
use deep_causality_haft::IoAction;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

fn invalid_data<E: std::fmt::Display>(context: &str, err: E) -> DataLoadingError {
    DataLoadingError::parse(context, err.to_string())
}

/// A lazy IO description that, when [`run`](IoAction::run), parses one satellite's ECEF position
/// series (metres) from a RINEX SP3 file. Construct with [`read_orbit_data`].
pub struct ReadOrbitData<R> {
    path: PathBuf,
    target_sat: String,
    _marker: PhantomData<R>,
}

impl<R> IoAction for ReadOrbitData<R>
where
    R: RealField + From<f64>,
{
    type Output = Vec<OrbitData<R>>;
    type Error = DataLoadingError;

    fn run(self) -> Result<Vec<OrbitData<R>>, DataLoadingError> {
        parse_orbit_data(&self.path, &self.target_sat)
    }
}

/// Describe (but do not perform) reading one satellite's SP3 orbit series from `path`.
pub fn read_orbit_data<R>(path: impl AsRef<Path>, target_sat: &str) -> ReadOrbitData<R> {
    ReadOrbitData {
        path: path.as_ref().to_path_buf(),
        target_sat: target_sat.to_string(),
        _marker: PhantomData,
    }
}

/// The pure parse: open the SP3 file and extract `target_sat`'s ECEF position series (metres).
fn parse_orbit_data<R>(path: &Path, target_sat: &str) -> Result<Vec<OrbitData<R>>, DataLoadingError>
where
    R: RealField + From<f64>,
{
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut data = Vec::new();
    let mut current_time: Option<NaiveDateTime> = None;

    let sat_id = SatId::try_from(target_sat)
        .map_err(|e| DataLoadingError::unknown(format!("target sat_id '{target_sat}': {e}")))?;

    for line in reader.lines() {
        let line = line?;

        if line.len() < 2 {
            continue;
        }

        if line.starts_with('*') {
            // Epoch Line: "*  2016  7  1  0  0  0.00000000"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 7 {
                continue;
            }
            let year = parts[1]
                .parse::<i32>()
                .map_err(|e| invalid_data("epoch year", e))?;
            let month = parts[2]
                .parse::<u32>()
                .map_err(|e| invalid_data("epoch month", e))?;
            let day = parts[3]
                .parse::<u32>()
                .map_err(|e| invalid_data("epoch day", e))?;
            let hour = parts[4]
                .parse::<u32>()
                .map_err(|e| invalid_data("epoch hour", e))?;
            let min = parts[5]
                .parse::<u32>()
                .map_err(|e| invalid_data("epoch minute", e))?;
            let sec = parts[6]
                .parse::<f64>()
                .map_err(|e| invalid_data("epoch second", e))? as u32;

            let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| {
                invalid_data(
                    "epoch date",
                    format!("out-of-range Y-M-D {year}-{month}-{day}"),
                )
            })?;
            current_time = Some(date.and_hms_opt(hour, min, sec).ok_or_else(|| {
                invalid_data(
                    "epoch time",
                    format!("out-of-range H:M:S {hour}:{min}:{sec}"),
                )
            })?);
        } else if line.starts_with('P') {
            // Position line: handles "P E14" (standard) and "PE14" (compact).
            if let Some(time) = current_time {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let (sat_id_str, coord_start_index) = if parts[0] == "P" {
                    if parts.len() < 2 {
                        continue;
                    }
                    (parts[1], 2)
                } else {
                    (&parts[0][1..], 1)
                };

                if sat_id_str == target_sat {
                    // SP3 is in km; convert to metres.
                    if parts.len() < coord_start_index + 3 {
                        continue;
                    }
                    let x_f64 = parts[coord_start_index]
                        .parse::<f64>()
                        .map_err(|e| invalid_data("position x", e))?
                        * 1000.0;
                    let y_f64 = parts[coord_start_index + 1]
                        .parse::<f64>()
                        .map_err(|e| invalid_data("position y", e))?
                        * 1000.0;
                    let z_f64 = parts[coord_start_index + 2]
                        .parse::<f64>()
                        .map_err(|e| invalid_data("position z", e))?
                        * 1000.0;

                    data.push(OrbitData::new(
                        time,
                        sat_id,
                        R::from(x_f64),
                        R::from(y_f64),
                        R::from(z_f64),
                    ));
                }
            }
        }
    }
    Ok(data)
}
