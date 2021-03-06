use aten::typedefault;
use loss::Reduction;
use util_autograd::{collect_next_edges, compute_requires_grad, set_history};

use crate::autograd::SavedTensor;
use crate::c10::Scalar;
use crate::ops::*;
use crate::tensor::*;
use crate::util_autograd;
use crate::{
    aten::{
        self,
        native::{self, *},
    },
    c10::ScalarType,
};
use std::cell::RefCell;
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::rc::Rc;

#[inline(always)]
fn check_no_requires_grad(tensor: &Tensor, name: &str) {
    if tensor.defined() && tensor.requires_grad() {
        panic!("the derivative for {} is not implemented", name);
    }
}

impl Add<Self> for Tensor {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        &self + &rhs
    }
}

impl Add<Self> for &Tensor {
    type Output = Tensor;
    fn add(self, rhs: Self) -> Self::Output {
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[self, rhs]) {
            grad_fn = Some(Rc::new(RefCell::new(Node::new(AddBackwardTensors {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
            }))));
            grad_fn
                .as_mut()
                .unwrap()
                .borrow_mut()
                .set_next_edges(util_autograd::collect_next_edges(&[self, rhs]));
        }
        let result = add(self, rhs, 1.0);

        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl<S> Add<S> for &Tensor
where
    S: Into<Scalar>,
{
    type Output = Tensor;
    fn add(self, rhs: S) -> Self::Output {
        let rhs = rhs.into();
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[&self]) {
            let mut _grad_fn = AddBackwardScalar {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[&self]));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }

        let result = add_scalar(self, rhs, 1.0);

        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl<S> Add<S> for Tensor
where
    S: Into<Scalar>,
{
    type Output = Tensor;
    fn add(self, rhs: S) -> Self::Output {
        let result = &self + rhs;
        result
    }
}

impl Mul<Self> for &Tensor {
    type Output = Tensor;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[self, rhs]) {
            let mut _grad_fn = MulBackwardTensors {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
                _self: None,
                other: None,
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[&self, &rhs]));
            _grad_fn._self = Some(SavedTensor::new(self, false));
            _grad_fn.other = Some(SavedTensor::new(rhs, false));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }
        let result = mul(self, rhs);

        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl<S> Mul<S> for &Tensor
where
    S: Into<Scalar>,
{
    type Output = Tensor;
    fn mul(self, rhs: S) -> Self::Output {
        let rhs: Scalar = rhs.into();
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[&self]) {
            let mut _grad_fn = MulBackwardScalar {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
                _self: None,
                other: rhs,
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[&self]));
            _grad_fn._self = Some(SavedTensor::new(self, false));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }
        let result = mul_scalar(self, rhs);

        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl<S> Mul<S> for Tensor
where
    S: Into<Scalar>,
{
    type Output = Tensor;
    fn mul(self, rhs: S) -> Self::Output {
        let result = &self * rhs;
        result
    }
}

impl Mul<&Self> for Tensor {
    type Output = Tensor;
    fn mul(self, rhs: &Self) -> Self::Output {
        &self * rhs
    }
}

impl Sub<Self> for &Tensor {
    type Output = Tensor;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[&self, &rhs]) {
            let mut _grad_fn = SubBackwardTensors {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[&self, &rhs]));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }
        let result = sub(self, rhs, 1.0);

        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl Div<Self> for &Tensor {
    type Output = Tensor;
    fn div(self, rhs: Self) -> Self::Output {
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[self, rhs]) {
            let mut _grad_fn = DivBackwardTensors {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
                _self: None,
                other: None,
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self, rhs]));
            _grad_fn._self = Some(SavedTensor::new(self, false));
            _grad_fn.other = Some(SavedTensor::new(rhs, false));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }
        let result = div(self, rhs);
        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl Div<Self> for Tensor {
    type Output = Tensor;
    fn div(self, rhs: Self) -> Self::Output {
        &self / &rhs
    }
}

impl<S> Div<S> for &Tensor
where
    S: Into<Scalar>,
{
    type Output = Tensor;

    fn div(self, rhs: S) -> Self::Output {
        let rhs: Scalar = rhs.into();
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[self]) {
            let mut _grad_fn = DivBackwardScalar {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
                _self: None,
                other: rhs,
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self]));
            _grad_fn._self = Some(SavedTensor::new(self, false));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }
        let result = div_scalar(self, rhs);
        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

impl<S> Div<S> for Tensor
where
    S: Into<Scalar>,
{
    type Output = Self;

    fn div(self, rhs: S) -> Self::Output {
        &self / rhs
    }
}

impl Neg for &Tensor {
    type Output = Tensor;
    fn neg(self) -> Self::Output {
        let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
        if util_autograd::compute_requires_grad(&[&self]) {
            let mut _grad_fn = NegBackward {
                next_edges: None,
                input_metadata_: smallvec::smallvec![],
            };
            _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[&self]));
            grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
        }
        let result = neg(self);

        if grad_fn.is_some() {
            util_autograd::set_history(&result, grad_fn.unwrap());
        }
        result
    }
}

pub fn transpose(self_: &Tensor, dim0: i64, dim1: i64) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = TBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }

    let result = native::transpose(self_, dim0, dim1);
    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }
    result
}

pub fn transpose_(self_: &Tensor, dim0: i64, dim1: i64) -> &Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = TBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }

    let result = native::transpose_(self_, dim0, dim1);
    if grad_fn.is_some() {
        util_autograd::set_history(result, grad_fn.unwrap());
    }
    result
}

pub fn t(self_: &Tensor) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = TBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }

    let result = native::t(self_);
    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }
    result
}

pub fn mm<T: AsRef<Tensor>>(mat1: &Tensor, mat2: T, consume: bool) -> Tensor {
    let mat2 = mat2.as_ref();
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[mat1, mat2]) {
        let mut _grad_fn = MmBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            self_: None,
            mat2_: None,
            mat2_sizes: vec![],
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[mat1, mat2]));
        _grad_fn.self_ = Some(SavedTensor::new(mat1, false));
        // if consume {
        //     _grad_fn.mat2_ = Some(SavedTensor::new(mat2, false));
        // } else {
            _grad_fn.mat2_ = Some(SavedTensor::new(&mat2, false));
        // }
        _grad_fn.mat2_sizes = mat2.sizes().to_vec();

        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    let result = aten::mm(mat1, mat2);
    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }
    result
}

pub fn addmm(
    self_: &Tensor,
    mat1: &Tensor,
    mat2: &Tensor,
    alpha: impl Into<Scalar>,
    beta: impl Into<Scalar>,
) -> Tensor {
    let alpha: Scalar = alpha.into();
    let beta: Scalar = beta.into();
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_, mat1, mat2]) {
        let mut _grad_fn = AddmmBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            mat1_: None,
            mat2_: None,
            mat2_sizes: vec![],
            alpha,
            beta,
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_, mat1, mat2]));
        _grad_fn.mat1_ = Some(SavedTensor::new(mat1, false));
        _grad_fn.mat2_ = Some(SavedTensor::new(mat2, false));
        _grad_fn.mat2_sizes = mat2.sizes().to_vec();
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    let result = aten::addmm(self_, mat1, mat2, alpha, beta);
    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }
    result
}

pub fn mean(self_: &Tensor) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = MeanBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            self_numel: self_.numel(),
            self_sizes: self_.sizes().to_vec(),
            self_scalar_type: self_.scalar_type(),
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    let result = aten::native::mean(self_, None);
    if grad_fn.is_some() {
        util_autograd::set_history(&self_, grad_fn.unwrap());
    }
    result
}

pub fn sum(self_: &Tensor, dtype: Option<ScalarType>) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = SumBackward0 {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            self_sizes: self_.sizes().to_vec(),
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }

    let result = aten::native::sum(self_, dtype);

    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }

    result
}
pub fn sum_dim_int_list(self_: &Tensor, dim: &[usize], keep_dim: bool) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = SumBackward1 {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            self_sizes: self_.sizes().to_vec(),
            dim: dim.to_vec(),
            keep_dim,
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }

    let result = aten::native::sum_dim_int_list(self_, dim.to_vec(), keep_dim, None);

    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }

    result
}

pub fn sigmoid(tensor: &Tensor) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    // SigmoidBackWard requires same computation as forward pass,
    // hence result is directly reused.
    let result = aten::native::sigmoid(tensor);

    if util_autograd::compute_requires_grad(&[tensor]) {
        let mut _grad_fn = SigmoidBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            result_: Some(SavedTensor::new(&result, false)),
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[tensor]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    if let Some(fn_) = grad_fn {
        util_autograd::set_history(&result, fn_);
    }
    result
}

pub fn squeeze(tensor: &Tensor) -> Tensor {
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    // SigmoidBackWard requires same computation as forward pass,
    // hence result is directly reused.
    let result = aten::native::sigmoid(tensor);

    if util_autograd::compute_requires_grad(&[tensor]) {
        let mut _grad_fn = SigmoidBackward {
            next_edges: None,
            input_metadata_: smallvec::smallvec![],
            result_: Some(SavedTensor::new(&result, false)),
        };
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[tensor]));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    if let Some(fn_) = grad_fn {
        util_autograd::set_history(&result, fn_);
    }
    result
}

pub fn binary_cross_entropy(
    input: &Tensor,
    target: &Tensor,
    weight: Option<&Tensor>,
    reduction: super::loss::Reduction,
) -> Tensor {
    check_no_requires_grad(target, "target");
    if let Some(w) = weight {
        check_no_requires_grad(w, "weight");
    }
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[input]) {
        let mut _grad_fn = BinaryCrossEntropyBackward::default();
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[input]));
        _grad_fn.self_ = Some(SavedTensor::new(input, false));
        _grad_fn.target_ = Some(SavedTensor::new(target, false));
        _grad_fn.weight_ = Some(SavedTensor::new_with_optional(weight, false));
        _grad_fn.reduction = reduction;
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    let result = loss::binary_cross_entropy(input, target, weight, reduction);
    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }
    result
}

pub fn log_softmax(self_: &Tensor, dim: i64, dtype: Option<ScalarType>) -> Tensor {
    let result = typedefault::log_softmax_int(self_, dim, dtype);
    result
}

pub fn _log_softmax(self_: &Tensor, dim: i64, half_to_float: bool) -> Tensor {
    let result = log_softmax_cpu(self_, dim, half_to_float);
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if util_autograd::compute_requires_grad(&[self_]) {
        let mut _grad_fn = LogSoftmaxBackward::default();
        _grad_fn.set_next_edges(util_autograd::collect_next_edges(&[self_]));
        _grad_fn.self_ = Some(SavedTensor::new(self_, false));
        _grad_fn.result = Some(SavedTensor::new(&result, true));
        _grad_fn.dim = dim;
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }

    if grad_fn.is_some() {
        util_autograd::set_history(&result, grad_fn.unwrap());
    }
    result
}
pub fn nll_loss_forward(
    self_: &Tensor,
    target: &Tensor,
    weight: Option<&Tensor>,
    reduction: Reduction,
    ignore_index: i64,
) -> (Tensor, Tensor) {
    let (output, total_weight) =
        native::nll_loss_forward_cpu(self_, target, weight, reduction, ignore_index);
    let any_requires_grad = compute_requires_grad(&[self_]);
    if let Some(w) = weight {
        check_no_requires_grad(w, "weight");
    }
    let mut grad_fn: Option<Rc<RefCell<Node>>> = None;
    if any_requires_grad {
        let mut _grad_fn = NllLossBackward::default();
        _grad_fn.set_next_edges(collect_next_edges(&[self_]));
        _grad_fn.self_ = Some(SavedTensor::new(self_, false));
        _grad_fn.target = Some(SavedTensor::new(target, false));
        _grad_fn.weight = Some(SavedTensor::new_with_optional(weight, false));
        _grad_fn.total_weight = Some(SavedTensor::new(&total_weight, true));
        grad_fn = Some(Rc::new(RefCell::new(Node::new(_grad_fn))));
    }
    if grad_fn.is_some() {
        set_history(&output, grad_fn.unwrap());
    }
    (output, total_weight)
}
