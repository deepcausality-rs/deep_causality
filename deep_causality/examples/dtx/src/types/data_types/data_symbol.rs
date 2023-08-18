// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum DataSymbol
{
    #[default]
    NoSymbol,
    BtcUsd,
}

impl DataSymbol
{
    pub fn from_str(s: &str) -> Option<DataSymbol> {
        match s.to_lowercase().as_str() {
            "btcusd" => Some(DataSymbol::BtcUsd),
            _ => None,
        }
    }
}


impl Display for DataSymbol
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
