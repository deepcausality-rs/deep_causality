/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::BaseModelTokio;
use crate::utils;
use deep_causality::{BaseCausaloid, MonadicCausable, NumericalValue, PropagatingEffect};
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
            let model = self.model.read().unwrap();
            Arc::clone(model.causaloid())
            // Release rw lock early for concurrency
        };

        // Again, for an for an API event handler you would pass through the data from the request.
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
        // Note, the base evaluate signature is the same for a Singleton, Collection or graph or caualoids.
        // For more advanced inference options, a collection evaluation can be customized when calling
        // evaluate_collection(...) and a graph has multiple evaluate_graph() methods to handle different pathways.
        // These specialized methods are only available on the respective type whereas the Singleton
        // used here only has the base evaluate() method because its just one causaloid.
        let res = bc.evaluate(&PropagatingEffect::from_numerical(data));
        if res.is_ok() {
            println!("EventHandler: Inference successful with res: {}", res.value)
        } else {
            println!(
                "EventHandler: Inference failed with error: {}",
                res.error.unwrap()
            )
        }

        Ok(())
    }
}
