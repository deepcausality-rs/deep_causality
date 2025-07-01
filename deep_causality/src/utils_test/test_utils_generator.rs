/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared setup for `with_generator` tests.
//! This is not a test module, but a utility module for other tests.
#![allow(dead_code)] // Allow unused code, as not all tests will use all helpers

use crate::prelude::*;

// A mock data structure used across multiple tests.
#[derive(Debug, Clone, Eq, Hash, Copy, PartialEq, Default)]
pub struct MockData {
    pub id: u64,
    pub data: u8,
}

impl Identifiable for MockData {
    fn id(&self) -> u64 {
        self.id
    }
}

impl Datable for MockData {
    type Data = u8;
    fn get_data(&self) -> Self::Data {
        self.data
    }
    fn set_data(&mut self, value: Self::Data) {
        self.data = value
    }
}

// A custom generative action enum, can be empty for most tests.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum MockCustomAction {}

impl
    Generatable<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        MockCustomAction,
    > for MockCustomAction
{
    fn generate(
        &mut self,
        _trigger: &GenerativeTrigger<MockData>,
        _context: &Context<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
        >,
    ) -> Result<
        GenerativeOutput<
            MockData,
            EuclideanSpace,
            EuclideanTime,
            EuclideanSpacetime,
            BaseSymbol,
            FloatType,
            FloatType,
            MockCustomAction,
        >,
        ModelGenerativeError,
    > {
        unimplemented!()
    }
}

// Type aliases for brevity in tests
pub type TestCausaloid = Causaloid<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub type TestContext = Context<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub type TestContextoid = Contextoid<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

pub type TestModel = Model<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;
