use crate::core::{AutoGradMode, NoGradGuard};
use crate::tensor::*;

struct OptimizerOptions {}
pub struct OptimizerParamGroup {
    params: Vec<Tensor>,
    // options: OptimizerOptions
}

impl OptimizerParamGroup {
    pub fn new(params: Vec<Tensor>) -> Self {
        Self { params }
    }

    pub fn params(&self) -> &Vec<Tensor> {
        &self.params
    }
}

pub trait Optimizer {
    fn step<F>(&mut self, closure: Option<F>) -> Tensor
    where
        F: FnMut() -> Tensor;
    fn param_groups(&self) -> &Vec<OptimizerParamGroup>;
    fn zero_grad(&self) {
        for group in self.param_groups() {
            for p in group.params() {
                if let Some(grad) = p.grad().as_mut() {
                    grad.detach_();
                    grad.zero_();
                }
            }
        }
    }
}

pub struct SGDOptionsBuilder {
    lr: f64,
    momentum: f64,
    dampening: f64,
    weight_decay: f64,
    nesterov: bool,
}
impl Default for SGDOptionsBuilder {
    fn default() -> Self {
        Self {
            lr: 0.001,
            momentum: 0.0,
            dampening: 0.0,
            weight_decay: 0.0,
            nesterov: false,
        }
    }
}
impl SGDOptionsBuilder {
    pub fn new(lr: f64) -> Self {
        Self {
            lr,
            ..Default::default()
        }
    }
    pub fn momentum(&mut self, momentum: f64) -> &mut Self {
        self.momentum = momentum;
        self
    }
    pub fn build(&self) -> SGDOptions {
        let mut options = SGDOptions::default();
        options.set_lr(self.lr);
        options.set_momentum(self.momentum);
        options.set_weight_decay(self.weight_decay);
        options.set_nesterov(self.nesterov);
        options.set_dampening(self.dampening);
        options
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SGDOptions {
    lr: f64,
    momentum: f64,
    dampening: f64,
    weight_decay: f64,
    nesterov: bool,
}

impl Default for SGDOptions {
    fn default() -> Self {
        Self {
            lr: 0.001,
            momentum: 0.0,
            dampening: 0.0,
            weight_decay: 0.0,
            nesterov: false,
        }
    }
}

impl SGDOptions {
    pub fn new(lr: f64) -> Self {
        Self {
            lr,
            momentum: 0.0,
            dampening: 0.0,
            weight_decay: 0.0,
            nesterov: false,
        }
    }

    pub fn set_lr(&mut self, lr: f64) {
        self.lr = lr
    }
    pub fn set_momentum(&mut self, momentum: f64) {
        self.momentum = momentum
    }
    pub fn set_dampening(&mut self, dampening: f64) {
        self.dampening = dampening;
    }
    pub fn set_weight_decay(&mut self, weight_decay: f64) {
        self.weight_decay = weight_decay;
    }
    pub fn set_nesterov(&mut self, nestrov: bool) {
        self.nesterov = nestrov;
    }
    pub fn lr(&self) -> f64 {
        self.lr
    }
    pub fn momentum(&self) -> f64 {
        self.momentum
    }
    pub fn dampening(&self) -> f64 {
        self.dampening
    }
    pub fn weight_decay(&self) -> f64 {
        self.weight_decay
    }
    pub fn nesterov(&self) -> bool {
        self.nesterov
    }
}

pub struct Sgd {
    param_groups: Vec<OptimizerParamGroup>,
    options: SGDOptions,
}

impl Sgd {
    pub fn new(params: Vec<Tensor>, options: SGDOptions) -> Self {
        Sgd::new_from_param_group(vec![OptimizerParamGroup::new(params)], options)
    }

    fn new_from_param_group(param_groups: Vec<OptimizerParamGroup>, options: SGDOptions) -> Self {
        Self {
            param_groups,
            options,
        }
    }
}

impl Optimizer for Sgd {
    fn step<F>(&mut self, closure: Option<F>) -> Tensor
    where
        F: FnMut() -> Tensor,
    {
        let mut loss = Tensor::default();
        let _guard = NoGradGuard::default();
        if let Some(mut fn_) = closure {
            let _mode = AutoGradMode::new(true);
            loss = fn_();
        }
        let weight_decay = self.options.weight_decay();
        let learning_rate = self.options.lr();
        let _momentum = self.options.momentum();
        let _dampening = self.options.dampening();
        let _nesterov = self.options.nesterov();

        for group in self.param_groups.iter_mut() {
            for p in group.params() {
                match p.grad().as_mut() {
                    Some(d_p) => {
                        if weight_decay != 0.0 {
                            // eprintln!("Weight Grad Before: {:?}", borrow_);
                            d_p.add_scalar(weight_decay);
                        }
                        // if momentum != 0.0 {
                        //     let buf;

                        //     if nesterov {
                        //         d_p = d_p.add(buf, momentum);
                        //     } else {
                        //         d_p = buf;
                        //     }
                        // }

                        p.add_with_alpha_(d_p, -1.0 * learning_rate);
                    }
                    None => continue,
                }
            }
        }
        loss
    }

    fn param_groups(&self) -> &Vec<OptimizerParamGroup> {
        self.param_groups.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::{Optimizer, SGDOptions, Sgd};
    use crate::{autograd::ones, core::manual_seed, nn::Module};
    use crate::{
        autograd::{backward, full},
        c10::TensorOptions,
    };
    use crate::{
        nn::{Functional, Linear},
        tensor::Tensor,
    };

    #[test]
    fn sgd_step() {
        crate::init_rovo();
        manual_seed(0);
        let linear = Linear::new(3, 2);
        let sigmoid = Functional::new(Functional::sigmoid());
        let mut sgd = Sgd::new(linear.parameters().unwrap(), SGDOptions::new(0.1));
        let x = full(&[4, 3], 1.5, TensorOptions::with_requires_grad());
        let target = ones(&[4, 2], None);

        let step = |optimizer: &mut Sgd,
                    linear: Linear,
                    sigmoid: Functional,
                    inputs: Tensor,
                    target: Tensor| {
            // Note: Can't put the following line into closure beacuse
            // zero_grad uses immutable reference and step uses mutable reference.
            optimizer.zero_grad();

            let closure = || {
                let h = linear.forward(&[&inputs]);
                let y = sigmoid.forward(&[&h]);
                let loss = crate::tensor::binary_cross_entropy(
                    &y,
                    &target,
                    None,
                    crate::tensor::loss::Reduction::Mean,
                );
                backward::backward(&vec![loss.clone()], &vec![], false);
                loss
            };
            optimizer.step(Some(closure))
        };
        let result = step(&mut sgd, linear, sigmoid, x.clone(), target);
        println!("Result: {:?}", result);
        println!("Input Grad: {:?}", x.grad());
    }
}
