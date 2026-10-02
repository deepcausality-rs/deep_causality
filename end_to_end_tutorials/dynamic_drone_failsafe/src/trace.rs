/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Records a part's flight as CSV tables when the part runs with `trace <dir>`. The tables feed
//! the animation in `video/drone_failsafe`; a run without the argument records nothing.

use crate::{Drone, Terrain, Touchdown};
use deep_causality_num::{lift_i64, lower};
use std::io::Write;
use std::path::{Path, PathBuf};

/// The world's extent the animation draws, across and along the valley, in m, at this spacing. The
/// creek runs below 12 m across, and a drifting drone crosses it below zero.
const WORLD_ACROSS_M: (i64, i64) = (-24, 140);
const WORLD_ALONG_M: (i64, i64) = (0, 520);
const WORLD_STEP_M: usize = 2;

/// The flight's columns every part's trace starts with.
pub const FLIGHT_HEADER: &str = "t,x,y,agl";

/// The drone's second, position across and along, and height above the ground: the leading cells
/// of a row under [`FLIGHT_HEADER`].
pub fn flight_cells(drone: &Drone) -> String {
    let (x, y) = drone.position();
    format!(
        "{},{:.3},{:.3},{:.3}",
        drone.time_s(),
        lower(x),
        lower(y),
        lower(drone.altitude_agl_m())
    )
}

/// The ground on a grid: elevation, slope, surface and canopy height at every point.
pub fn world_table(terrain: &Terrain) -> TraceTable {
    let mut table = TraceTable::new("world", "x,y,elevation_m,slope_deg,surface,canopy_m");
    for x in (WORLD_ACROSS_M.0..=WORLD_ACROSS_M.1).step_by(WORLD_STEP_M) {
        for y in (WORLD_ALONG_M.0..=WORLD_ALONG_M.1).step_by(WORLD_STEP_M) {
            let (xm, ym) = (lift_i64(x), lift_i64(y));
            table.push(format!(
                "{x},{y},{:.3},{:.2},{},{:.1}",
                lower(terrain.elevation_m(xm, ym)),
                lower(terrain.slope_deg(xm, ym)),
                variant_name(&terrain.surface(xm, ym)),
                lower(terrain.canopy_m(xm, ym)),
            ));
        }
    }
    table
}

/// Where each person stands.
pub fn crew_table(terrain: &Terrain) -> TraceTable {
    let mut table = TraceTable::new("crew", "x,y");
    for (x, y) in terrain.crew_m() {
        table.push(format!("{:.2},{:.2}", lower(*x), lower(*y)));
    }
    table
}

/// Where and when the drone came down, and what became of it.
pub fn touchdown_table(name: &str, time_s: usize, touchdown: &Touchdown) -> TraceTable {
    let mut table = TraceTable::new(name, "t,x,y,outcome,surface,nearest_person_m");
    let (x, y) = touchdown.position();
    table.push(format!(
        "{time_s},{:.3},{:.3},{},{},{:.2}",
        lower(x),
        lower(y),
        variant_name(&touchdown.outcome()),
        variant_name(&touchdown.surface()),
        lower(touchdown.nearest_person_m()),
    ));
    table
}

/// The directory named by a `trace <dir>` argument, or `None` when the part runs without one.
pub fn trace_dir() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, dir] if command == "trace" => Some(PathBuf::from(dir)),
        _ => None,
    }
}

/// The name of an enum value's variant, without its fields: `HoldOver { x: 1.0, y: 2.0 }` gives
/// `HoldOver`.
pub fn variant_name(value: &impl std::fmt::Debug) -> String {
    let full = format!("{value:?}");
    full.split([' ', '{', '('])
        .next()
        .unwrap_or_default()
        .to_string()
}

/// One CSV table: a header line and its rows, written to `<dir>/<name>.csv`.
#[derive(Debug, Clone)]
pub struct TraceTable {
    name: String,
    header: String,
    rows: Vec<String>,
}

impl TraceTable {
    pub fn new(name: &str, header: &str) -> Self {
        Self {
            name: name.to_string(),
            header: header.to_string(),
            rows: Vec::new(),
        }
    }

    pub fn push(&mut self, row: String) {
        self.rows.push(row);
    }

    /// Writes the table to `<dir>/<name>.csv`, creating the directory, and returns the path.
    pub fn write(&self, dir: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{}.csv", self.name));
        let mut file = std::io::BufWriter::new(std::fs::File::create(&path)?);
        writeln!(file, "{}", self.header)?;
        for row in &self.rows {
            writeln!(file, "{row}")?;
        }
        file.flush()?;
        Ok(path)
    }
}
