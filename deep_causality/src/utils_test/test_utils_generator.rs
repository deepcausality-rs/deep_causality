/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::*;
use std::hash::Hash;

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

// A test processor to act as a destination for the generative output.
pub struct TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub causaloid_dest: Option<Causaloid<D, S, T, ST, SYM, VS, VT>>,
    pub context_dest: Option<Context<D, S, T, ST, SYM, VS, VT>>,
}

impl<D, S, T, ST, SYM, VS, VT> Default for TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<D, S, T, ST, SYM, VS, VT> TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub fn new() -> Self {
        Self {
            causaloid_dest: None,
            context_dest: None,
        }
    }
}

// Implement the processor trait so it can be used to test generators.
impl<D, S, T, ST, SYM, VS, VT, G> GenerativeProcessor<D, S, T, ST, SYM, VS, VT, G>
    for TestProcessor<D, S, T, ST, SYM, VS, VT>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G>,
{
    fn get_causaloid_dest(&mut self) -> &mut Option<Causaloid<D, S, T, ST, SYM, VS, VT>> {
        &mut self.causaloid_dest
    }

    fn get_context_dest(&mut self) -> &mut Option<Context<D, S, T, ST, SYM, VS, VT>> {
        &mut self.context_dest
    }
}

// Type alias for brevity in tests
pub type TestProcessorAlias = TestProcessor<
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

// Define a dummy generator for testing standalone outputs.
pub struct DummyGenerator;
impl
    Generatable<
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > for DummyGenerator
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
            DummyGenerator,
        >,
        ModelGenerativeError,
    > {
        Ok(GenerativeOutput::NoOp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_generator_no_op() {
        let mut generator = DummyGenerator;
        let trigger = GenerativeTrigger::ManualIntervention("test".to_string());

        // Create a context of the correct type: TestContext (which is Context<MockData, ...>)
        let context: TestContext = TestContext::with_capacity(1, "Test Context", 10);

        let result = generator.generate(&trigger, &context);

        assert!(result.is_ok());
        match result.unwrap() {
            GenerativeOutput::NoOp => {
                // This is the expected outcome
            }
            _ => {
                panic!("DummyGenerator should always return GenerativeOutput::NoOp");
            }
        }
    }
}
