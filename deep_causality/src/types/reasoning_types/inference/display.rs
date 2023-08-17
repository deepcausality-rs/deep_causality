// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::fmt::{Display, Formatter};

use crate::prelude::Inference;

impl Display for Inference
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "Inference: id: {}, question: {}, observation: {}, threshold: {}, effect: {}",
               self.id, self.question, self.observation, self.threshold, self.effect
        )
    }
}
