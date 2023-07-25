// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum DataSymbol {
    BTCUSD,
    ETHUSD,
}

impl DataSymbol
{
    pub fn from_str(s: &str) -> Option<DataSymbol> {
        match s.to_lowercase().as_str() {
            "btcusd" => Some(DataSymbol::BTCUSD),
            "ethusd" => Some(DataSymbol::ETHUSD),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            DataSymbol::BTCUSD => "btcusd",
            DataSymbol::ETHUSD => "ethusd",
        }
    }
}

impl Default for DataSymbol
{
    fn default() -> Self
    {
        DataSymbol::BTCUSD
    }
}

impl Display for DataSymbol
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
