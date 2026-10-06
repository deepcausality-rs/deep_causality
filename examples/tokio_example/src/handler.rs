/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::BaseModelTokio;
use crate::utils;
use deep_causality::{
    BaseCausaloid, CausalityError, MonadicCausable, NumericalValue, PropagatingEffect,
};
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct EventHandler {
    /// The inference model instance. This is wrapped in an Arc/RwLock to allow
    /// shared ownership between multiple threads in Tokio.
    model: Arc<RwLock<BaseModelTokio>>,
}

impl EventHandler {
    pub fn new(model: BaseModelTokio) -> Self {
        Self {
            model: Arc::new(RwLock::new(model)),
        }
    }
}

impl EventHandler {
    pub async fn run_background_inference(&self) -> Result<(), Box<dyn Error + Send>> {
        // These are simple test data. However, for an API event handler you would extract
        // the data from the incoming request.
        let data = utils::get_test_data();
        // Extract the causaloid from the model.
        let causaloid = {
            let model = self.model.read().map_err(|_| -> Box<dyn Error + Send> {
                Box::new(CausalityError::Custom("Model lock is poisoned"))
            })?;
            Arc::clone(model.causaloid())
            // Release rw lock early for concurrency
        };

        // Again, for an API event handler you would pass through the data from the request.
        for d in data.into_iter() {
            self.handle_inference(d, &causaloid)?
        }

        Ok(())
    }

    fn handle_inference(
        &self,
        data: f64,
        bc: &BaseCausaloid<NumericalValue, bool>,
    ) -> Result<(), Box<dyn Error + Send>> {
        // New API: Use PropagatingEffect::pure for input creation
        let input_effect: PropagatingEffect<NumericalValue> = PropagatingEffect::pure(data);
        let res = bc.evaluate(&input_effect);

        match res.error() {
            None => {
                let value = res.value_cloned().ok_or_else(|| -> Box<dyn Error + Send> {
                    Box::new(CausalityError::ValueNotAvailable())
                })?;
                println!("EventHandler: Inference successful with res: {}", value)
            }
            Some(error) => println!("EventHandler: Inference failed with error: {}", error),
        }

        Ok(())
    }
}
