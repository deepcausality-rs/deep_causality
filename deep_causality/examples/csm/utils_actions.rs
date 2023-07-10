// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{ActionError, CausalAction};

pub fn get_smoke_alert_action() -> CausalAction
{
    let func = raise_smoke_alert;
    let descr = "Action that triggers the smoke alert";
    let version = 1;

    fn raise_smoke_alert() -> Result<(), ActionError> {
        println!("Sensor detected smoke and raised smoke alert");
        Ok(())
    }

    CausalAction::new(func, descr, version)
}

pub fn get_fire_alert_action() -> CausalAction
{
    let func = raise_fire_alert;
    let descr = "Action that triggers the fire alert";
    let version = 1;

    fn raise_fire_alert() -> Result<(), ActionError> {
        println!("Sensor detected fire and raised the fire alert");
        Ok(())
    }

    CausalAction::new(func, descr, version)
}

pub fn get_explosion_alert_action() -> CausalAction
{
    let func = raise_explosion_alert;
    let descr = "Action that triggers the explosion alert";
    let version = 1;

    fn raise_explosion_alert() -> Result<(), ActionError> {
        println!("Sensor detected an explosion and raised explosion alert");
        Ok(())
    }

    CausalAction::new(func, descr, version)
}
