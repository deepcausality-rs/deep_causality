/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! RINEX clock (`.clk`) loader as a lazy [`IoAction`].

use crate::{ClockData, DataLoadingError, SatId};
use chrono::NaiveDate;
use deep_causality_algebra::RealField;
use deep_causality_haft::IoAction;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

/// A lazy IO description that, when [`run`](IoAction::run), parses one satellite's clock-bias series
/// (`AS` records) from a RINEX `.clk` file. Construct with [`read_clock_data`]; compose with
/// `map` / `and_then`; perform the read only at the program edge with `.run()`.
pub struct ReadClockData<R> {
    path: PathBuf,
    target_sat: String,
    _marker: PhantomData<R>,
}

impl<R> IoAction for ReadClockData<R>
where
    R: RealField + From<f64>,
{
    type Output = Vec<ClockData<R>>;
    type Error = DataLoadingError;

    fn run(self) -> Result<Vec<ClockData<R>>, DataLoadingError> {
        parse_clock_data(&self.path, &self.target_sat)
    }
}

/// Describe (but do not perform) reading one satellite's clock series from `path`.
pub fn read_clock_data<R>(path: impl AsRef<Path>, target_sat: &str) -> ReadClockData<R> {
    ReadClockData {
        path: path.as_ref().to_path_buf(),
        target_sat: target_sat.to_string(),
        _marker: PhantomData,
    }
}

/// The pure parse: open the `.clk` file and extract `target_sat`'s bias series (seconds). Lenient on
/// malformed records (skipped); fails only on the file open/read (surfaced as a [`DataLoadingError`]).
fn parse_clock_data<R>(path: &Path, target_sat: &str) -> Result<Vec<ClockData<R>>, DataLoadingError>
where
    R: RealField + From<f64>,
{
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut data = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("AS") {
            // Line: "AS E18  2016 07 01 00 00 00.000000  2  0.123456789012"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }
            let sat_id = parts[1];

            if sat_id == target_sat {
                let year = parts[2].parse::<i32>().unwrap_or(0);
                let month = parts[3].parse::<u32>().unwrap_or(0);
                let day = parts[4].parse::<u32>().unwrap_or(0);
                let hour = parts[5].parse::<u32>().unwrap_or(0);
                let min = parts[6].parse::<u32>().unwrap_or(0);
                let sec = parts[7].parse::<f64>().unwrap_or(0.0) as u32;

                let time = match NaiveDate::from_ymd_opt(year, month, day)
                    .and_then(|d| d.and_hms_opt(hour, min, sec))
                {
                    Some(t) => t,
                    None => continue,
                };

                // RINEX CLK: Code, SatID, Time(YMDHMS), NumberOfValues, Bias, Sigma. Column 9 = bias.
                let bias_f64 = parts[9].parse::<f64>().unwrap_or(999_999.999_999);

                // IGS uses 999999.999999 to denote invalid/missing/broken clock data; filter it out.
                if bias_f64 > 900_000.0 {
                    continue;
                }

                let sat_id = match SatId::try_from(sat_id) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                data.push(ClockData::new(time, sat_id, R::from(bias_f64)));
            }
        }
    }
    Ok(data)
}
