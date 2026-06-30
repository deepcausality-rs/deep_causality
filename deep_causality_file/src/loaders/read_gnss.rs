/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Composed GNSS loader (clock + orbit) over the haft IO monad, plus an ergonomic [`DataManager`]
//! facade that performs the read for consumers that just want the data.

use crate::loaders::read_clk::read_clock_data;
use crate::loaders::read_sp3::read_orbit_data;
use crate::{ClockData, DataLoadingError, GnssDataResult, OrbitData};
use deep_causality_haft::IoAction;
use deep_causality_num::RealField;
use std::path::Path;

/// Describe (but do not perform) reading one satellite's clock **and** orbit series. The two file
/// reads are composed with the IO monad (`and_then` + `map`): nothing touches the filesystem until
/// `.run()` at the program edge, and the first error short-circuits.
pub fn read_gnss_single_satellite<R>(
    clk_path: impl AsRef<Path>,
    sp3_path: impl AsRef<Path>,
    target_sat: &str,
) -> impl IoAction<Output = (Vec<ClockData<R>>, Vec<OrbitData<R>>), Error = DataLoadingError>
where
    R: RealField + From<f64> + 'static,
{
    let sp3 = sp3_path.as_ref().to_path_buf();
    let sat = target_sat.to_string();
    read_clock_data::<R>(clk_path, target_sat)
        .and_then(move |clocks| read_orbit_data::<R>(sp3, &sat).map(move |orbits| (clocks, orbits)))
}

/// An ergonomic facade that performs the GNSS reads (runs the [`IoAction`]s) and hands back the data.
/// Consumers that want to compose the reads lazily should use [`read_gnss_single_satellite`] and the
/// `read_clock_data` / `read_orbit_data` actions directly.
#[derive(Debug, Default, Clone, Copy)]
pub struct DataManager;

impl DataManager {
    pub fn new() -> Self {
        Self
    }

    /// Load one satellite's clock + orbit series (performs the IO).
    pub fn load_gnss_single_satellite<R>(
        &self,
        clk_path: impl AsRef<Path>,
        sp3_path: impl AsRef<Path>,
        target_sat: &str,
    ) -> GnssDataResult<R>
    where
        R: RealField + From<f64> + 'static,
    {
        read_gnss_single_satellite::<R>(clk_path, sp3_path, target_sat).run()
    }

    /// Load one satellite's clock-bias series (performs the IO).
    pub fn load_gnss_clock_data<R>(
        &self,
        clk_path: impl AsRef<Path>,
        target_sat: &str,
    ) -> Result<Vec<ClockData<R>>, DataLoadingError>
    where
        R: RealField + From<f64>,
    {
        read_clock_data::<R>(clk_path, target_sat).run()
    }

    /// Load one satellite's SP3 orbit series (performs the IO).
    pub fn load_gnss_orbit_data<R>(
        &self,
        sp3_path: impl AsRef<Path>,
        target_sat: &str,
    ) -> Result<Vec<OrbitData<R>>, DataLoadingError>
    where
        R: RealField + From<f64>,
    {
        read_orbit_data::<R>(sp3_path, target_sat).run()
    }
}
