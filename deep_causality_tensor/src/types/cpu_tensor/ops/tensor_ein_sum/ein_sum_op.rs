use deep_causality_ast::ConstTree;

// The core enum for the EinSum AST. Generic over the Tensor type.
#[derive(Clone, Debug, PartialEq)]
pub enum EinSumOp<Tensor> {
    // LEAF: The source Tensor data
    TensorSource {
        tensor: Tensor,
    },

    // GENERIC OPS (for flexibility)
    Contraction {
        lhs_axes: Vec<usize>,
        rhs_axes: Vec<usize>,
    },
    Reduction {
        axes: Vec<usize>,
    },

    // EXPLICIT EIN_SUM OPS (for usability)
    MatMul,
    DotProd,
    Trace {
        axes1: usize,
        axes2: usize,
    },
    TensorProduct,
    ElementWiseProduct,
    Transpose {
        new_order: Vec<usize>,
    },
    DiagonalExtraction {
        axes1: usize,
        axes2: usize,
    },
    BatchMatMul,
}

// The EinSum AST
pub type EinSumAST<Tensor> = ConstTree<EinSumOp<Tensor>>;

impl<Tensor> EinSumOp<Tensor> {
    pub fn tensor_source(tensor: Tensor) -> EinSumAST<Tensor> {
        EinSumAST::new(EinSumOp::TensorSource { tensor })
    }

    pub fn contraction<L, R>(
        lhs: L,
        rhs: R,
        lhs_axes: Vec<usize>,
        rhs_axes: Vec<usize>,
    ) -> EinSumAST<Tensor>
    where
        L: Into<Tensor>,
        R: Into<Tensor>,
    {
        let lhs_leaf = EinSumOp::tensor_source(lhs.into());
        let rhs_leaf = EinSumOp::tensor_source(rhs.into());
        EinSumAST::with_children(
            EinSumOp::Contraction { lhs_axes, rhs_axes },
            vec![lhs_leaf, rhs_leaf],
        )
    }

    pub fn reduction<O: Into<Tensor>>(operand: O, axes: Vec<usize>) -> EinSumAST<Tensor> {
        let operand_leaf = EinSumOp::tensor_source(operand.into());
        EinSumAST::with_children(EinSumOp::Reduction { axes }, vec![operand_leaf])
    }

    pub fn mat_mul<L, R>(lhs: L, rhs: R) -> EinSumAST<Tensor>
    where
        L: Into<Tensor>,
        R: Into<Tensor>,
    {
        let lhs_leaf = EinSumOp::tensor_source(lhs.into());
        let rhs_leaf = EinSumOp::tensor_source(rhs.into());
        EinSumAST::with_children(EinSumOp::MatMul, vec![lhs_leaf, rhs_leaf])
    }

    pub fn dot_prod<L, R>(lhs: L, rhs: R) -> EinSumAST<Tensor>
    where
        L: Into<Tensor>,
        R: Into<Tensor>,
    {
        let lhs_leaf = EinSumOp::tensor_source(lhs.into());
        let rhs_leaf = EinSumOp::tensor_source(rhs.into());
        EinSumAST::with_children(EinSumOp::DotProd, vec![lhs_leaf, rhs_leaf])
    }

    pub fn trace<O: Into<Tensor>>(operand: O, axes1: usize, axes2: usize) -> EinSumAST<Tensor> {
        let operand_leaf = EinSumOp::tensor_source(operand.into());
        EinSumAST::with_children(EinSumOp::Trace { axes1, axes2 }, vec![operand_leaf])
    }

    pub fn tensor_product<L, R>(lhs: L, rhs: R) -> EinSumAST<Tensor>
    where
        L: Into<Tensor>,
        R: Into<Tensor>,
    {
        let lhs_leaf = EinSumOp::tensor_source(lhs.into());
        let rhs_leaf = EinSumOp::tensor_source(rhs.into());
        EinSumAST::with_children(EinSumOp::TensorProduct, vec![lhs_leaf, rhs_leaf])
    }

    pub fn element_wise_product<L, R>(lhs: L, rhs: R) -> EinSumAST<Tensor>
    where
        L: Into<Tensor>,
        R: Into<Tensor>,
    {
        let lhs_leaf = EinSumOp::tensor_source(lhs.into());
        let rhs_leaf = EinSumOp::tensor_source(rhs.into());
        EinSumAST::with_children(EinSumOp::ElementWiseProduct, vec![lhs_leaf, rhs_leaf])
    }

    pub fn transpose<O: Into<Tensor>>(operand: O, new_order: Vec<usize>) -> EinSumAST<Tensor> {
        let operand_leaf = EinSumOp::tensor_source(operand.into());
        EinSumAST::with_children(EinSumOp::Transpose { new_order }, vec![operand_leaf])
    }

    pub fn diagonal_extraction<O: Into<Tensor>>(
        operand: O,
        axes1: usize,
        axes2: usize,
    ) -> EinSumAST<Tensor> {
        let operand_leaf = EinSumOp::tensor_source(operand.into());
        EinSumAST::with_children(
            EinSumOp::DiagonalExtraction { axes1, axes2 },
            vec![operand_leaf],
        )
    }

    pub fn batch_mat_mul<L, R>(lhs: L, rhs: R) -> EinSumAST<Tensor>
    where
        L: Into<Tensor>,
        R: Into<Tensor>,
    {
        let lhs_leaf = EinSumOp::tensor_source(lhs.into());
        let rhs_leaf = EinSumOp::tensor_source(rhs.into());
        EinSumAST::with_children(EinSumOp::BatchMatMul, vec![lhs_leaf, rhs_leaf])
    }

    /// Maps the tensor type of the operation.
    pub fn map_tensor<U, F>(self, f: F) -> EinSumOp<U>
    where
        F: FnOnce(Tensor) -> U,
    {
        match self {
            EinSumOp::TensorSource { tensor } => EinSumOp::TensorSource { tensor: f(tensor) },
            EinSumOp::Contraction { lhs_axes, rhs_axes } => {
                EinSumOp::Contraction { lhs_axes, rhs_axes }
            }
            EinSumOp::Reduction { axes } => EinSumOp::Reduction { axes },
            EinSumOp::MatMul => EinSumOp::MatMul,
            EinSumOp::DotProd => EinSumOp::DotProd,
            EinSumOp::Trace { axes1, axes2 } => EinSumOp::Trace { axes1, axes2 },
            EinSumOp::TensorProduct => EinSumOp::TensorProduct,
            EinSumOp::ElementWiseProduct => EinSumOp::ElementWiseProduct,
            EinSumOp::Transpose { new_order } => EinSumOp::Transpose { new_order },
            EinSumOp::DiagonalExtraction { axes1, axes2 } => {
                EinSumOp::DiagonalExtraction { axes1, axes2 }
            }
            EinSumOp::BatchMatMul => EinSumOp::BatchMatMul,
        }
    }
}
