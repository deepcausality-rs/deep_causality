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
pub type TestCausaloid<I, O> = Causaloid<
    I,
    O,
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

pub type TestModel<I, O> = Model<
    I,
    O,
    MockData,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
>;

// A test processor to act as a destination for the generative output.
#[allow(clippy::type_complexity)]
pub struct TestProcessor<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    pub causaloid_dest: Option<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>,
    pub context_dest: Option<Context<D, S, T, ST, SYM, VS, VT>>,
}

impl<I, O, D, S, T, ST, SYM, VS, VT> Default for TestProcessor<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
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

impl<I, O, D, S, T, ST, SYM, VS, VT> TestProcessor<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
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
impl<I, O, D, S, T, ST, SYM, VS, VT, G> GenerativeProcessor<I, O, D, S, T, ST, SYM, VS, VT, G>
    for TestProcessor<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<I, O, D, S, T, ST, SYM, VS, VT, G>,
{
    fn get_causaloid_dest(&mut self) -> &mut Option<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>> {
        &mut self.causaloid_dest
    }

    fn get_context_dest(&mut self) -> &mut Option<Context<D, S, T, ST, SYM, VS, VT>> {
        &mut self.context_dest
    }
}

// Type alias for brevity in tests
pub type TestProcessorAlias<I, O> = TestProcessor<
    I,
    O,
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
impl<I, O>
    Generatable<
        I,
        O,
        MockData,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
        DummyGenerator,
    > for DummyGenerator
where
    I: IntoEffectValue,
    O: IntoEffectValue,
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
            I,
            O,
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
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_dummy_generator_no_op() {
//         let mut generator = DummyGenerator;
//         let trigger = GenerativeTrigger::ManualIntervention("test".to_string());
//
//         // Create a context of the correct type: TestContext (which is Context<MockData, ...>)
//         let context: TestContext = TestContext::with_capacity(1, "Test Context", 10);
//
//         // DummyGenerator is now generic over I and O, so we need to specify them.
//         // For this test, we can use bool for both I and O, as it's a simple case.
//         let result = generator.generate(&trigger, &context);
//         assert!(result.is_ok());
//     }
//
//     #[test]
//     fn test_processor_default() {
//         let proc = TestProcessorAlias::default();
//         assert!(proc.causaloid_dest.is_none());
//     }
// }
