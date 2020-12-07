use crate::autograd;
use crate::nn::{functional as F, module};
use crate::tensor::Tensor;
#[derive(Debug, Clone, Copy)]
pub struct LinearConfig {
    pub bias: bool,
    pub in_features: usize,
    pub out_features: usize,
}

impl Default for LinearConfig {
    fn default() -> Self {
        LinearConfig {
            bias: true,
            in_features: 0,
            out_features: 0,
        }
    }
}
#[derive(Debug)]
pub struct Linear {
    pub ws: Option<Tensor>,
    pub bs: Option<Tensor>,
    options: LinearConfig,
}

impl Linear {
    pub fn new(in_dim: usize, out_dim: usize) -> Self {
        let options = LinearConfig {
            in_features: in_dim,
            out_features: out_dim,
            ..LinearConfig::default()
        };
        let mut self_ = Self {
            ws: None,
            bs: None,
            options,
        };
        self_.reset();
        self_
    }

    fn reset(&mut self) {
        let ws = autograd::empty(
            &[self.options.out_features, self.options.in_features],
            None,
            None,
        );
        module::register_parameter(&ws, true);
        self.ws = Some(ws);
        if self.options.bias {
            let bs_ = autograd::empty(&[self.options.out_features], None, None);
            module::register_parameter(&bs_, true);
            self.bs = Some(bs_);
        };
        self.reset_parameters();
    }
    fn reset_parameters(&mut self) {
        crate::nn::init::kaiming_uniform_(
            self.ws.as_ref().unwrap(),
            (5.0f64).sqrt(),
            crate::nn::init::FanModeType::FanIn,
            crate::nn::init::NonlinerityType::LeakyReLU,
        );
        if let Some(bs) = self.bs.as_ref() {
            let (fan_in, _fan_out) =
                crate::nn::init::calculate_fan_in_fan_out(self.ws.as_ref().unwrap());
            let bound = 1.0 / (fan_in as f64).sqrt();
            bs.uniform(-bound, bound)
        }
    }
}

impl module::Module for Linear {
    fn forward(&self, xs: &[&Tensor]) -> Tensor {
        F::linear(xs[0], self.ws.as_ref().unwrap(), self.bs.as_ref().unwrap())
    }

    fn parameters(&self) -> Vec<Tensor> {
        if self.bs.is_some() {
            vec![
                self.ws.as_ref().unwrap().clone(),
                self.bs.as_ref().unwrap().clone(),
            ]
        } else {
            vec![self.ws.as_ref().unwrap().clone()]
        }
    }
}

#[cfg(test)]
mod test {
    use super::Linear;
    use crate::autograd;
    use crate::autograd::backward;
    use crate::c10::TensorOptions;
    use crate::core::manual_seed;
    use crate::nn::Module;

    #[test]
    fn linear_backward_test() {
        crate::init_rovo();
        manual_seed(0);
        let linear = Linear::new(4, 3);
        let x = autograd::full(&[2, 4], 3.0, TensorOptions::with_requires_grad());
        let y = linear.forward(&[&x]);
        // Expected: [[-2.0227153, 0.6529779, -0.6904765],[ -2.0227153, 0.6529779, -0.6904765]]
        println!("Result: {:?}", y);
        //Expected : -0.686738
        println!("Mean: {:?}", y.mean());

        backward::backward(&vec![y], &vec![], false);

        //Expected : [
        //           [-0.04011544, 0.08910112, -0.09542262, -0.011634579],
        //           [-0.04011544, 0.08910112, -0.09542262, -0.011634579]
        //          ]
        println!("Input Grad: {:?}", x.grad());
    }
}
