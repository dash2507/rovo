use crate::{
    aten::{
        native::{ArgMaxOps, MeanOps, SharedOps},
        numeric_utils::lower_bound,
        GRAIN_SIZE,
    },
    tensor::TensorIterator,
    AT_DISPATCH_ALL_TYPES_AND, AT_DISPATCH_ALL_TYPES_AND_COMPLEX_AND2,
};
use std::ptr::NonNull;

pub fn argmax_kernel_impl(iter: &TensorIterator) {
    AT_DISPATCH_ALL_TYPES_AND!(_, iter.dtype_(1), "argmax_cpu", || {
        binary_kernel_reduce(
            iter,
            ArgMaxOps::<Scalart>::new(),
            (lower_bound::<Scalart>(), 0),
        )
    });
}

pub fn mean_kernel_impl(iter: &TensorIterator) {
    AT_DISPATCH_ALL_TYPES_AND_COMPLEX_AND2!(_, _, iter.dtype(), "mean_cpu", || {
        let factor = iter.num_output_elements() as Scalart / iter.numel() as Scalart;
        binary_kernel_reduce(iter, MeanOps::<Scalart, Scalart>::new(factor), 0 as Scalart);
    });
}

fn binary_kernel_reduce<O, I, ACC_T, DATA_T>(iter: &TensorIterator, ops: O, init: I)
where
    O: SharedOps<ACC_T, ProjectArg = ACC_T, ReduceArg1 = ACC_T, ReduceArg2 = DATA_T>,
    ACC_T: Copy + std::fmt::Debug,
    DATA_T: Copy,
    I: Into<ACC_T> + Copy,
{
    let num_outputs = iter.noutputs();
    let closure = |sub_iter: &TensorIterator| {
        let reduction_body = |mut acc: ACC_T, begin: usize, end: usize| -> ACC_T {
            let ntensors = sub_iter.ntensors();
            let loop_ = |data: &[NonNull<u8>], strides: &[usize], size: usize| {
                assert_eq!(ntensors - num_outputs, 1);
                let mut in_ = data[ntensors - 1].as_ptr();
                let stride = strides[ntensors - 1];
                for i in 0..size {
                    acc = O::reduce(acc, unsafe { *(in_ as *mut DATA_T) }, begin + i);
                    in_ = unsafe { in_.add(stride) };
                }
            };
            sub_iter.serial_for_each(loop_, begin..end);
            O::translate_idx(acc, sub_iter.view_offsets()[0])
        };
        let mut total_acc: ACC_T = init.into();
        let numel = sub_iter.numel();
        if numel < GRAIN_SIZE {
            total_acc = reduction_body(total_acc, 0, numel);
        } else {
            todo!();
        }
        set_results(ops.project(total_acc), sub_iter, num_outputs);
    };
    iter.foreach_reduced_elt(closure, true);
}

fn set_results<R>(result: R, iter: &TensorIterator, num_outputs: usize) {
    assert_eq!(num_outputs, 1);
    set_result(0, result, iter, num_outputs);
}

fn set_result<R>(index: usize, result: R, iter: &TensorIterator, num_outputs: usize) {
    // static_assert(std::is_same<res_t, typename traits::arg2_t>::value, "data types must match");
    if index < num_outputs {
        let out = iter.data_ptr(index).as_ptr() as *mut R;
        unsafe { *out = result };
    }
}
