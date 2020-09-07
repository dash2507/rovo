use crate::core::NoGradGuard;
use crate::tensor::*;

struct OptimizerOptions {}
struct OptimizerParamGroup {
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

trait Optimizer {
    fn step(&mut self) -> Option<Tensor>;
    fn param_groups(&self) -> &Vec<OptimizerParamGroup>;
    fn zero_grad(&self) {
        for group in self.param_groups() {
            for p in group.params() {
                if let Some(grad) = p.grad() {
                    let mut grad_borrow = grad.borrow_mut();
                    grad_borrow.detach_();
                    grad_borrow.zero_();
                }
            }
        }
    }
}
pub struct SGD {
    param_groups: Vec<OptimizerParamGroup>,
}

impl SGD {
    pub fn new(params: Vec<Tensor>) -> Self {
        SGD::new_from_param_group(vec![OptimizerParamGroup::new(params)])
    }

    fn new_from_param_group(param_groups: Vec<OptimizerParamGroup>) -> Self {
        Self { param_groups }
    }
}

impl Optimizer for SGD {
    fn step(&mut self) -> Option<Tensor> {
        let _guard = NoGradGuard::default();
        let weight_decay = 0.0;
        let learning_rate = 0.001;
        for group in &mut self.param_groups {
            for p in group.params() {
                match p.grad() {
                    Some(d_p_) => {
                        if weight_decay != 0.0 {
                            let borrow_ = d_p_.borrow_mut();
                            println!("Weight Grad Before: {:?}", borrow_);
                            let d_p = &borrow_.clone() + weight_decay;
                            borrow_.move_tensor(d_p);
                        }
                        let d_p = d_p_.borrow().clone();
                        p.add_(&d_p, -1.0 * learning_rate);
                    }
                    None => continue,
                }
            }
        }
        None
    }

    fn param_groups(&self) -> &Vec<OptimizerParamGroup> {
        self.param_groups.as_ref()
    }
}

#[cfg(test)]
mod test {
    use super::{Optimizer, SGD};
    use crate::autograd::backward;
    use crate::nn::Module;
    use crate::nn::{Functional, Linear, LinearConfig};
    use crate::tensor::Tensor;

    #[test]
    fn test_sgd_step() {
        let config = LinearConfig::default();
        let linear = Linear::new(2, 1, config);
        let sigmoid = Functional::new(Functional::sigmoid());
        let mut sgd = SGD::new(linear.parameters());
        let x = Tensor::from_scalar(&[2, 2], 2.0, true);

        sgd.zero_grad();
        let y = linear.forward(&[&x]);
        let result = sigmoid.forward(&[&y]);
        println!("Result: {:?}", result);

        backward::backward(&vec![result], &vec![], false);
        {
            println!("Weights Before step: {:?}", linear.parameters());
        }
        sgd.step();
        println!("Weights after step: {:?}", linear.parameters());
    }
}