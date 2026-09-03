/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::conversion_error::ConversionError;
use std::fmt::Display;

/// Galileo Satellite Identifiers
///
/// This enum covers all Galileo satellites that appear in the GNSS CLK/SP3 data.
/// The constellation has grown over time; this list includes all satellites
/// from the 2016-2018 dataset plus additional launches through 2023.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SatId {
    E01,
    E02,
    E03,
    E04,
    E05,
    E07,
    E08,
    E09,
    E10,
    E11,
    E12,
    E13,
    E14,
    E15,
    E18,
    E19,
    E21,
    E22,
    E24,
    E25,
    E26,
    E27,
    E30,
    E31,
    E33,
    E34,
    E36,
}

impl TryFrom<&str> for SatId {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "PE01" | "E01" => Ok(SatId::E01),
            "PE02" | "E02" => Ok(SatId::E02),
            "PE03" | "E03" => Ok(SatId::E03),
            "PE04" | "E04" => Ok(SatId::E04),
            "PE05" | "E05" => Ok(SatId::E05),
            "PE07" | "E07" => Ok(SatId::E07),
            "PE08" | "E08" => Ok(SatId::E08),
            "PE09" | "E09" => Ok(SatId::E09),
            "PE10" | "E10" => Ok(SatId::E10),
            "PE11" | "E11" => Ok(SatId::E11),
            "PE12" | "E12" => Ok(SatId::E12),
            "PE13" | "E13" => Ok(SatId::E13),
            "PE14" | "E14" => Ok(SatId::E14),
            "PE15" | "E15" => Ok(SatId::E15),
            "PE18" | "E18" => Ok(SatId::E18),
            "PE19" | "E19" => Ok(SatId::E19),
            "PE21" | "E21" => Ok(SatId::E21),
            "PE22" | "E22" => Ok(SatId::E22),
            "PE24" | "E24" => Ok(SatId::E24),
            "PE25" | "E25" => Ok(SatId::E25),
            "PE26" | "E26" => Ok(SatId::E26),
            "PE27" | "E27" => Ok(SatId::E27),
            "PE30" | "E30" => Ok(SatId::E30),
            "PE31" | "E31" => Ok(SatId::E31),
            "PE33" | "E33" => Ok(SatId::E33),
            "PE34" | "E34" => Ok(SatId::E34),
            "PE36" | "E36" => Ok(SatId::E36),
            _ => Err(ConversionError(format!("Unknown Satellite: {}", value))),
        }
    }
}

impl Display for SatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Galileo/{}", self.as_str())
    }
}

impl SatId {
    pub fn as_str(&self) -> &'static str {
        match self {
            SatId::E01 => "E01",
            SatId::E02 => "E02",
            SatId::E03 => "E03",
            SatId::E04 => "E04",
            SatId::E05 => "E05",
            SatId::E07 => "E07",
            SatId::E08 => "E08",
            SatId::E09 => "E09",
            SatId::E10 => "E10",
            SatId::E11 => "E11",
            SatId::E12 => "E12",
            SatId::E13 => "E13",
            SatId::E14 => "E14",
            SatId::E15 => "E15",
            SatId::E18 => "E18",
            SatId::E19 => "E19",
            SatId::E21 => "E21",
            SatId::E22 => "E22",
            SatId::E24 => "E24",
            SatId::E25 => "E25",
            SatId::E26 => "E26",
            SatId::E27 => "E27",
            SatId::E30 => "E30",
            SatId::E31 => "E31",
            SatId::E33 => "E33",
            SatId::E34 => "E34",
            SatId::E36 => "E36",
        }
    }

    /// Returns a numeric identifier for the satellite (the number portion).
    pub fn as_num(&self) -> u32 {
        match self {
            SatId::E01 => 1,
            SatId::E02 => 2,
            SatId::E03 => 3,
            SatId::E04 => 4,
            SatId::E05 => 5,
            SatId::E07 => 7,
            SatId::E08 => 8,
            SatId::E09 => 9,
            SatId::E10 => 10,
            SatId::E11 => 11,
            SatId::E12 => 12,
            SatId::E13 => 13,
            SatId::E14 => 14,
            SatId::E15 => 15,
            SatId::E18 => 18,
            SatId::E19 => 19,
            SatId::E21 => 21,
            SatId::E22 => 22,
            SatId::E24 => 24,
            SatId::E25 => 25,
            SatId::E26 => 26,
            SatId::E27 => 27,
            SatId::E30 => 30,
            SatId::E31 => 31,
            SatId::E33 => 33,
            SatId::E34 => 34,
            SatId::E36 => 36,
        }
    }
}
