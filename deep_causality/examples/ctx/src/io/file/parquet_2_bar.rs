// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use chrono::{DateTime, TimeZone, Utc};
use parquet::record::{Row, RowAccessor};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::types::data_symbol::DataSymbol;
use crate::types::date_time_bar::DateTimeBar;

pub fn convert_field_to_date_time_bar(
    record: &Row,
    symbol: &str,
)
    -> DateTimeBar
{
    // parquet index. We can safely unwrap b/c all data are complete and correct.
    // 0 date_time Int
    // 1 symbol String
    // 2 open f64
    // 3 high f64
    // 4 low f64
    // 5 close f64
    // 6 volume f64
    // 7 trades u64
    let date_time = get_date_time_field(record).expect("Failed to get date_time field");
    let symbol = DataSymbol::from_str(symbol).expect("Cannot convert str symbol to DataSymbol");
    let open = Decimal::from_f64(record.get_double(2).unwrap()).unwrap();
    let high = Decimal::from_f64(record.get_double(3).unwrap()).unwrap();
    let low = Decimal::from_f64(record.get_double(4).unwrap()).unwrap();
    let close = Decimal::from_f64(record.get_double(5).unwrap()).unwrap();
    let volume = Decimal::from_f64(record.get_double(6).unwrap()).unwrap();

    DateTimeBar::new(symbol, date_time, open, high, low, close, volume)
}

fn get_date_time_field(
    row: &Row
)
    -> Result<DateTime<Utc>, Box<dyn std::error::Error>>
{
    if row.get_string(0).is_ok() {
        // supported timezone syntax for DateTime from string https://github.com/chronotope/chrono/issues/219
        let fmt = "%Y-%m-%d %H:%M:%S%.6f%z";
        let s = row.get_string(0).expect("Cannot extract datetime str");
        // supported timezone syntax for DateTime from string https://github.com/chronotope/chrono/issues/219
        let date_time: DateTime<Utc> = DateTime::parse_from_str(s, fmt)
            .expect("Cannot convert string to DateTime").with_timezone(&Utc);

        return Ok(date_time);
    }

    if row.get_long(0).is_ok() {
        let millis = row.get_long(0).expect("Cannot extract datetime millis");
        let date_time: DateTime<Utc> = Utc.timestamp_millis_opt(millis).unwrap();
        return Ok(date_time);
    }

    if row.get_timestamp_micros(0).is_ok() {
        let micros = row.get_timestamp_micros(0).expect("Cannot extract datetime millis");
        let millis = micros / 1000;

        let date_time: DateTime<Utc> = Utc.timestamp_millis_opt(millis).unwrap();
        return Ok(date_time);
    }

    panic!("get_date_time_field: Cannot extract datetime field");
}
