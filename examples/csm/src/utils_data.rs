/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::NumericalValue;

pub fn get_smoke_sensor_data() -> [NumericalValue; 12] {
    [
        10.0, 8.0, 3.4, 7.0, 12.1, 30.89, 45.3, 60.89, 78.23, 89.8, 88.7, 91.3,
    ]
}

pub fn get_fire_sensor_data() -> [NumericalValue; 12] {
    [
        20.0, 21.0, 23.4, 22.0, 22.1, 33.89, 54.3, 60.89, 78.23, 89.8, 95.7, 99.3,
    ]
}

pub fn get_explosion_sensor_data() -> [NumericalValue; 12] {
    [
        14.6, 14.6, 14.6, 222.0, 270.1, 90.89, 54.3, 29.89, 14.6, 14.6, 14.6, 14.6,
    ]
}
