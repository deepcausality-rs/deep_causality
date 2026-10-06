/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The placards the gates check, with their justification. The constants are exact `f64`
//! specification literals; the gates lift each one losslessly into the working `FloatType`. The
//! physical inputs of the per-point computation (gas model, heating, nose radius, atmosphere) are
//! the placard world in `model_config`.

// ── Placards (the acceptance envelope)

/// Dynamic-pressure placard, kPa. A chosen demonstration placard for a transport-category-like
/// supersonic envelope, not certification data: the recorded matrix peaks near 24 kPa at
/// M 1.20 / 11 km, so 60 kPa leaves the whole corridor inside the envelope with margin, while
/// low-altitude supersonic flight (the exceeds matrix's M 1.5 / 5 km point, about 85 kPa)
/// lands outside it.
pub const Q_MAX_PLACARD_KPA: f64 = 60.0;

/// Post-shock stagnation-temperature placard, K. A chosen demonstration ceiling for an
/// uncooled hot-structure leading edge (nickel-alloy class), not certification data: the
/// recorded matrix peaks near 1502 K at M 5.0 / 40 km, so 1700 K bounds the corridor with
/// about 13 percent margin.
pub const T0_MAX_PLACARD_K: f64 = 1700.0;
