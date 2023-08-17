// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use crate::prelude::{DescriptionValue, Inferable, Inference, NumericalValue};

impl Inferable for Inference
{
    fn question(&self) -> DescriptionValue {
        self.question.to_string()
    }

    fn observation(&self) -> NumericalValue {
        self.observation
    }

    fn threshold(&self) -> NumericalValue {
        self.threshold
    }

    fn effect(&self) -> NumericalValue {
        self.effect
    }

    fn target(&self) -> NumericalValue {
        self.target
    }
}
