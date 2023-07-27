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
    let open = Decimal::from_f64(record.get_double(1).unwrap()).unwrap();
    let high = Decimal::from_f64(record.get_double(2).unwrap()).unwrap();
    let low = Decimal::from_f64(record.get_double(3).unwrap()).unwrap();
    let close = Decimal::from_f64(record.get_double(4).unwrap()).unwrap();
    let volume = Decimal::from_f64(record.get_double(5).unwrap()).unwrap();

    DateTimeBar::new(symbol, date_time, open, high, low, close, volume)
}

pub fn convert_field_to_sampled_date_time_bar(
    row: &Row
)
    -> DateTimeBar
{
    // parquet index. We can safely unwrap b/c all data are complete and correct.
    // 0 date_time_sampled string
    // 1 symbol String
    // 2 open f64
    // 3 high f64
    // 4 low f64
    // 5 close f64
    // 6 delta_close f64 (skipped b/c not in struct)
    // 7 volume f64
    // 8 delta_volume f64 (skipped b/c not in struct)

    let date_time: DateTime<Utc> = get_date_time_field(row).expect("Failed to get date_time field");
    let symbol: &str = row.get_string(1).expect("Cannot extract str symbol");
    let open_price: f64 = row.get_double(2).expect("Cannot extract open price");
    let high_price: f64 = row.get_double(3).expect("Cannot extract high price");
    let low_price: f64 = row.get_double(4).expect("Cannot extract low price");
    let close_price: f64 = row.get_double(5).expect("Cannot extract close price");
    let volume: f64 = get_volume_field(row);
    DateTimeBar::new(
        DataSymbol::from_str(symbol).expect("Failed to parse symbol"),
        date_time,
        Decimal::from_f64(open_price).expect("Failed to parse open price"),
        Decimal::from_f64(high_price).expect("Failed to parse high price"),
        Decimal::from_f64(low_price).expect("Failed to parse low price"),
        Decimal::from_f64(close_price).expect("Failed to parse close price"),
        Decimal::from_f64(volume).expect("Failed to parse volume"),
    )
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

fn get_volume_field(
    row: &Row
)
    -> f64
{
    return if row.get_double(6).is_ok() {
        row.get_double(6).expect("Cannot extract volume")
    } else {
        row.get_double(7).expect("Cannot extract volume")
    };
}