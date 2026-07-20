/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The governing-model selector and the flight-phase axes it optionally bands (\[2\]/\[3\]).

use super::super::coupling::{CoupledField, PhysicsStage, StepContext};
use super::peak;
use crate::CfdScalar;
use crate::types::flow::BlackoutTrigger;
use alloc::format;
use alloc::string::String;
use deep_causality_haft::LogAddEntry;
use deep_causality_physics::{ElectronDensity, Length, PhysicsError, knudsen_number_kernel};

/// The field scalar counting logged regime transitions, incremented once per genuine regime change.
/// Cumulative across legs, because the coupled field carries it.
pub const REGIME_TRANSITIONS_FIELD: &str = "regime_transitions";

/// The governing continuum/rarefaction model selected from the Knudsen number. The classic bands:
/// continuum Navier–Stokes below `Kn ≈ 0.01`, slip-corrected continuum to `≈ 0.1`, transitional to
/// `≈ 10`, free-molecular above. (Thresholds are configurable on [`RegimeClassify`].)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoverningModel {
    /// Continuum Navier–Stokes (`Kn < slip_threshold`).
    Continuum,
    /// Slip-corrected continuum (`slip_threshold ≤ Kn < transitional_threshold`).
    Slip,
    /// Transitional regime (`transitional_threshold ≤ Kn < free_molecular_threshold`).
    Transitional,
    /// Free-molecular flow (`Kn ≥ free_molecular_threshold`).
    FreeMolecular,
}

impl GoverningModel {
    /// A short, stable label for provenance messages.
    pub fn name(self) -> &'static str {
        match self {
            GoverningModel::Continuum => "continuum",
            GoverningModel::Slip => "slip",
            GoverningModel::Transitional => "transitional",
            GoverningModel::FreeMolecular => "free-molecular",
        }
    }
}

/// The compressibility band of the flight phase (change `add-retropulsion-coupled-stages`,
/// capability `flight-regime-classifier`), read from the carrier-published `"flight_mach"`.
/// [`Unknown`](Self::Unknown) is the neutral value a world that publishes no Mach carries, so the
/// corridor's classification is unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachRegime {
    /// No `"flight_mach"` published — the axis is neutral.
    Unknown,
    /// `M ≤ subsonic_ceiling`.
    Subsonic,
    /// Between the subsonic ceiling and the supersonic floor.
    Transonic,
    /// `M ≥ supersonic_floor`.
    Supersonic,
}

impl MachRegime {
    /// A short, stable label for provenance messages.
    pub fn name(self) -> &'static str {
        match self {
            MachRegime::Unknown => "mach-unknown",
            MachRegime::Subsonic => "subsonic",
            MachRegime::Transonic => "transonic",
            MachRegime::Supersonic => "supersonic",
        }
    }
}

/// The propulsion state of the flight phase, read from the `"ignited"` flag.
/// [`Unknown`](Self::Unknown) is the neutral value for a world that carries no propulsion state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrustState {
    /// No `"ignited"` scalar published — the axis is neutral.
    Unknown,
    /// Carried but unlit.
    Coast,
    /// Lit.
    Burn,
}

impl ThrustState {
    /// A short, stable label for provenance messages.
    pub fn name(self) -> &'static str {
        match self {
            ThrustState::Unknown => "thrust-unknown",
            ThrustState::Coast => "coast",
            ThrustState::Burn => "burn",
        }
    }
}

/// The classifier's decision at a step: the selected [`GoverningModel`], the Knudsen number it was
/// selected from, the plasma/comms state (angular plasma frequency + whether GNSS is denied), and
/// the powered-descent flight phase (Mach band, thrust state, touchdown).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegimeClass<R: CfdScalar> {
    /// The selected governing model.
    pub model: GoverningModel,
    /// The Knudsen number `Kn = λ / L` the model was selected from.
    pub knudsen: R,
    /// The angular plasma frequency `ω_p` (rad/s) at the peak electron density.
    pub plasma_frequency: R,
    /// Whether GNSS / comms are denied (plasma frequency above the configured band).
    pub gnss_denied: bool,
    /// The compressibility band (neutral when no `"flight_mach"` is published).
    pub mach_regime: MachRegime,
    /// The propulsion state (neutral when no `"ignited"` flag is published).
    pub thrust_state: ThrustState,
    /// Whether the vehicle is at or below the configured touchdown altitude floor.
    pub touchdown: bool,
}

impl<R: CfdScalar> RegimeClass<R> {
    /// The discrete tuple that identifies a *regime* for change detection: the governing model,
    /// comms denial, and the three flight-phase axes. The continuous Knudsen / plasma-frequency /
    /// Mach values are excluded — only a band, denial, thrust, or touchdown transition is a regime
    /// change worth logging. For a world that publishes none of the flight scalars the three new
    /// components are constant, so change detection reduces to today's `(model, gnss_denied)` pair
    /// and the corridor's logged transitions are unchanged.
    fn key(&self) -> (GoverningModel, bool, MachRegime, ThrustState, bool) {
        (
            self.model,
            self.gnss_denied,
            self.mach_regime,
            self.thrust_state,
            self.touchdown,
        )
    }
}

/// The governing-model selector (\[2\]/\[3\]). Reads the peak mean free path from a `"mean_free_path"`
/// field and forms `Kn = λ / L` against the configured characteristic length, reads the peak
/// electron density from `"n_e"` and maps it through a [`BlackoutTrigger`] to the GNSS-denial flag,
/// then records the [`RegimeClass`] on the field — logging a provenance entry whenever the regime
/// (governing model or comms-denial) changes. A no-op if `"mean_free_path"` is absent.
#[derive(Debug, Clone, Copy)]
pub struct RegimeClassify<R: CfdScalar> {
    mfp_field: &'static str,
    ne_field: &'static str,
    characteristic_length: R,
    slip_threshold: R,
    transitional_threshold: R,
    free_molecular_threshold: R,
    trigger: BlackoutTrigger<R>,
    mach_field: &'static str,
    ignited_field: &'static str,
    altitude_field: &'static str,
    /// The powered-descent bands, **opt-in**. `None` — the default — leaves all three flight axes
    /// neutral *even when the carrier publishes their scalars*, so the corridor (whose carrier
    /// publishes `"flight_mach"` every step) classifies and logs exactly as before.
    flight_axes: Option<FlightAxes<R>>,
}

/// The band edges the powered-descent flight axes are read against (see
/// [`RegimeClassify::with_flight_axes`]).
#[derive(Debug, Clone, Copy)]
struct FlightAxes<R: CfdScalar> {
    subsonic_ceiling: R,
    supersonic_floor: R,
    touchdown_altitude: R,
}

impl<R: CfdScalar> RegimeClassify<R> {
    /// A classifier over the standard Knudsen bands (`0.01` / `0.1` / `10`) for a body of
    /// characteristic length `characteristic_length` (m), with `trigger` mapping the peak electron
    /// density to the GNSS-denial decision.
    pub fn new(characteristic_length: R, trigger: BlackoutTrigger<R>) -> Self {
        Self {
            mfp_field: "mean_free_path",
            ne_field: "n_e",
            characteristic_length,
            slip_threshold: R::from_f64(0.01).unwrap_or_else(R::zero),
            transitional_threshold: R::from_f64(0.1).unwrap_or_else(R::zero),
            free_molecular_threshold: R::from_f64(10.0).unwrap_or_else(R::one),
            trigger,
            mach_field: "flight_mach",
            ignited_field: "ignited",
            altitude_field: "flight_altitude",
            flight_axes: None,
        }
    }

    /// Override the Knudsen band thresholds (`slip ≤ transitional ≤ free_molecular`).
    pub fn with_thresholds(mut self, slip: R, transitional: R, free_molecular: R) -> Self {
        self.slip_threshold = slip;
        self.transitional_threshold = transitional;
        self.free_molecular_threshold = free_molecular;
        self
    }

    /// **Opt into** the powered-descent flight axes: the subsonic ceiling and supersonic floor the
    /// `"flight_mach"` scalar is banded against, and the altitude at or below which the vehicle
    /// counts as touched down.
    ///
    /// Without this call the three axes stay neutral **even though the compressible carrier
    /// publishes `"flight_mach"` every step**, so a corridor world's classification, regime key, and
    /// logged messages are exactly the pre-change ones. Only a burn-phase world opts in.
    pub fn with_flight_axes(
        mut self,
        subsonic_ceiling: R,
        supersonic_floor: R,
        touchdown_altitude: R,
    ) -> Self {
        self.flight_axes = Some(FlightAxes {
            subsonic_ceiling,
            supersonic_floor,
            touchdown_altitude,
        });
        self
    }

    /// Band the published flight Mach; neutral when the world publishes none.
    fn mach_regime_of(&self, field: &CoupledField<R>) -> MachRegime {
        let Some(axes) = self.flight_axes else {
            return MachRegime::Unknown;
        };
        let Some(mach) = field
            .scalar(self.mach_field)
            .and_then(|s| s.first().copied())
        else {
            return MachRegime::Unknown;
        };
        if mach <= axes.subsonic_ceiling {
            MachRegime::Subsonic
        } else if mach >= axes.supersonic_floor {
            MachRegime::Supersonic
        } else {
            MachRegime::Transonic
        }
    }

    /// Read the propulsion state from the `"ignited"` flag; neutral when the world carries none.
    fn thrust_state_of(&self, field: &CoupledField<R>) -> ThrustState {
        if self.flight_axes.is_none() {
            return ThrustState::Unknown;
        }
        match field
            .scalar(self.ignited_field)
            .and_then(|s| s.first().copied())
        {
            None => ThrustState::Unknown,
            Some(f) if f > R::zero() => ThrustState::Burn,
            Some(_) => ThrustState::Coast,
        }
    }

    /// Whether the published altitude is at or below the touchdown floor (false when unpublished).
    fn touchdown_of(&self, field: &CoupledField<R>) -> bool {
        let Some(axes) = self.flight_axes else {
            return false;
        };
        field
            .scalar(self.altitude_field)
            .and_then(|s| s.first().copied())
            .is_some_and(|alt| alt <= axes.touchdown_altitude)
    }

    /// Select the governing model for a Knudsen number against the configured bands.
    fn model_for(&self, kn: R) -> GoverningModel {
        if kn < self.slip_threshold {
            GoverningModel::Continuum
        } else if kn < self.transitional_threshold {
            GoverningModel::Slip
        } else if kn < self.free_molecular_threshold {
            GoverningModel::Transitional
        } else {
            GoverningModel::FreeMolecular
        }
    }
}

impl<const D: usize, R: CfdScalar> PhysicsStage<D, R> for RegimeClassify<R> {
    fn apply(
        &self,
        _ctx: &StepContext<'_, D, R>,
        field: &mut CoupledField<R>,
    ) -> Result<(), PhysicsError> {
        let Some(mfp) = field.scalar(self.mfp_field) else {
            return Ok(());
        };
        // Rarefaction is worst where the mean free path is largest, so classify off its peak.
        let mfp_peak = peak(mfp);
        let length = Length::new(self.characteristic_length)?;
        let kn = knudsen_number_kernel(mfp_peak, &length)?;
        let model = self.model_for(kn);

        // The comms side: peak electron density → plasma frequency → GNSS-denial decision.
        let ne_peak = field
            .scalar(self.ne_field)
            .map(peak)
            .unwrap_or_else(R::zero);
        let blackout = self.trigger.evaluate(ElectronDensity::new(ne_peak)?)?;

        let class = RegimeClass {
            model,
            knudsen: kn,
            plasma_frequency: blackout.plasma_frequency,
            gnss_denied: blackout.denied,
            mach_regime: self.mach_regime_of(field),
            thrust_state: self.thrust_state_of(field),
            touchdown: self.touchdown_of(field),
        };

        // Log only genuine regime transitions (model band, comms-denial, or a flight-phase change).
        //
        // The decision is also published as a monotone counter. It was previously computed here and
        // discarded after logging, which left a consumer counting `"regime ->"` substrings in a
        // rendered log for a number this stage already knows.
        let changed = field.regime().map(|prev| prev.key()) != Some(class.key());
        if changed {
            let transitions = field
                .scalar(REGIME_TRANSITIONS_FIELD)
                .and_then(|s| s.first().copied())
                .unwrap_or_else(R::zero)
                + R::one();
            field.set_scalar(
                REGIME_TRANSITIONS_FIELD,
                alloc::vec::Vec::from([transitions]),
            );
            let denial = if class.gnss_denied {
                "GNSS-denied"
            } else {
                "GNSS-available"
            };
            // The flight-phase suffix appears only when a powered-descent axis is live, so a world
            // publishing none of the flight scalars logs exactly the pre-change message.
            let phase = if class.mach_regime == MachRegime::Unknown
                && class.thrust_state == ThrustState::Unknown
                && !class.touchdown
            {
                String::new()
            } else {
                format!(
                    ", {} / {}{}",
                    class.mach_regime.name(),
                    class.thrust_state.name(),
                    if class.touchdown { ", touchdown" } else { "" },
                )
            };
            field.log_mut().add_entry(&format!(
                "regime -> {} ({}), Kn={}{}",
                model.name(),
                denial,
                kn,
                phase,
            ));
        }
        field.set_regime(class);
        Ok(())
    }
}
