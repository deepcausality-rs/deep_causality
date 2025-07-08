/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::utils;
use deep_causality::{BaseCausaloid, BaseModel, Causable, Evidence};
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
    pub async fn run_background_inference(&self) -> Result<(), Box<dyn Error + Send>> {
        let data = utils::get_test_data();

        let bc = {
            let model = self.model.read().unwrap();
            Arc::clone(model.causaloid())
            // Release rw lock early for concurrency
        };

        for d in data.iter() {
            self.handle_inference(d, &bc)?
        }

        Ok(())
    }

    fn handle_inference(
        &self,
        data: &f64,
        bc: &BaseCausaloid,
    ) -> Result<(), Box<dyn Error + Send>> {
        // Wrap the raw numerical data into the unified Evidence type.
        let evidence = Evidence::Numerical(*data);

        // Call the new standard `evaluate` method and handle the Result.
        match bc.evaluate(&evidence) {
            Ok(effect) => {
                println!(
                    "EventHandler: Inference successful with effect: {:?}",
                    effect
                )
            }
            Err(e) => {
                println!("EventHandler: Inference failed with error: {e}")
            }
        }

        Ok(())
    }
}
