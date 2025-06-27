/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::utils;
use deep_causality::prelude::{BaseCausaloid, BaseModel, Causable};
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct EventHandler {
    /// The inference model instance. This is wrapped in an Arc/RwLock to allow
    /// shared ownership between multiple threads.
    model: Arc<RwLock<BaseModel>>,
}

impl EventHandler {
    pub fn new(model: Arc<RwLock<BaseModel>>) -> Self {
        Self { model }
    }
}

impl EventHandler {
    pub async fn run_inference(&self) -> Result<(), Box<dyn Error + Send>> {
        let data = utils::get_test_data();
        let model = self.model.read().unwrap();

        let bc = model.causaloid();

        for d in data.iter() {
            self.handle_inference(d, bc)?
        }

        Ok(())
    }

    fn handle_inference(
        &self,
        data: &f64,
        bc: &BaseCausaloid,
    ) -> Result<(), Box<dyn Error + Send>> {
        let res = bc.verify_single_cause(data).unwrap_or_else(|e| {
            println!("EventHandler: {e}");
            false
        });

        println!("EventHandler: {res}");

        Ok(())
    }
}
