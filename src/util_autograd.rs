use crate::autograd::*;
use crate::core::GradMode;
use crate::ops::*;
use crate::tensor::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

pub fn compute_requires_grad(tensors: &[&Tensor]) -> bool {
    let mut out = false;
    if !GradMode::is_enabled() {
        out
    } else {
        for t in tensors {
            if t.requires_grad() {
                out = true;
            }
        }
        out
    }
}

pub struct TensorHook;
impl TensorHook {
    pub fn grad_fn(tensor: &Tensor) -> Option<&Rc<RefCell<Node>>> {
        if let Some(meta) = Self::get_autograd_meta(tensor) {
            meta.grad_fn_.as_ref()
        } else {
            None
        }
    }

    pub fn get_autograd_meta(tensor: &Tensor) -> Option<&mut AutogradMeta> {
        tensor.get_tensor_impl().and_then(|i| i.get_autogradmeta())
    }

    pub fn materialize_autograd_meta(tensor: &Tensor) -> &mut AutogradMeta {
        assert!(
            tensor.defined(),
            "cannot call materialize_autograd_meta() on an undefined Tensor"
        );
        let p = tensor.get_unsafe_tensor_impl();
        if p.autogradmeta.as_ref().is_none() {
            p.set_autograd_meta(Some(AutogradMetaFactory::make()))
        }
        TensorHook::get_autograd_meta(tensor).unwrap()
    }

    pub fn tensor_data(tensor: &Tensor) -> Tensor {
        let tensor_impl_copy = tensor
            .get_unsafe_tensor_impl()
            .shallow_copy_and_detach(tensor.get_unsafe_tensor_impl().version_counter());
        Tensor::from_impl(tensor_impl_copy)
    }

    pub fn version_counter(tensor: &Tensor) -> &TensorVersion {
        assert!(
            tensor.defined(),
            "cannot call version_counter() on undefined tensor",
        );
        tensor.get_unsafe_tensor_impl().version_counter()
    }

    pub fn set_grad_accumulator(tensor: &Tensor, grad_accumulator: Option<Weak<RefCell<Node>>>) {
        let t = Self::materialize_autograd_meta(tensor);
        t.grad_accumulator_ = grad_accumulator;
    }

    pub fn set_version_counter(tensor: &Tensor, version_counter: TensorVersion) {
        assert!(
            tensor.defined(),
            "cannot call set_version_counter() on undefined tensor"
        );
        let impl_ = tensor.get_unsafe_tensor_impl();
        impl_.set_version_counter(version_counter);
    }
}

pub fn grad_accumulator(tensor: &Tensor) -> Option<Rc<RefCell<Node>>> {
    if let Some(meta) = TensorHook::get_autograd_meta(tensor) {
        if meta.grad_fn_.is_some() {
            panic!("grad_accumulator() should be only called on leaf Variables");
        }
        if !meta.requires_grad {
            return None;
        }
        let accumulator = meta.grad_accumulator_.as_ref();

        if let Some(acc_) = accumulator.map_or_else(|| None, |a| a.upgrade()) {
            Some(acc_)
        } else {
            let result = Rc::new(RefCell::new(Node::new(AccumulateGrad::new(Tensor::new(
                tensor,
            )))));
            meta.grad_accumulator_ = Some(Rc::downgrade(&result));
            Some(result)
        }
    } else {
        None
    }
}

pub fn gradient_edge(tensor: &Tensor) -> Edge {
    if let Some(grad_fn) = tensor.grad_fn() {
        Edge::new(Some(grad_fn), tensor.output_nr())
    } else {
        Edge::new(grad_accumulator(tensor), 0)
    }
}

pub fn collect_next_edges(tensors: &[&Tensor]) -> Vec<Edge> {
    /*
        let mut next_edges: Vec<Edge> = vec![];
        next_edges.reserve(tensors.len());
    */
    let mut next_edges: Vec<Edge> = Vec::with_capacity(tensors.len());
    for t in tensors {
        next_edges.push(gradient_edge(*t));
    }
    next_edges
}

pub fn set_gradient_edge(tensor: &Tensor, args: (Rc<RefCell<Node>>, usize)) {
    let edge = Edge::new(Some(args.0), args.1);
    // Todo: read todo on materialize_autograd_meta
    let meta = TensorHook::materialize_autograd_meta(tensor);
    meta.set_grad_fn(edge.function);
    meta.set_output_nr(edge.input_nr);
}

pub fn set_history(tensor: &Tensor, grad_fn: Rc<RefCell<Node>>) {
    let output_nr = grad_fn.borrow_mut().add_input_metadata(tensor);
    set_gradient_edge(tensor, (grad_fn, output_nr))
}
