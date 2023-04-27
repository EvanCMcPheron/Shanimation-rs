use crate::prelude::*;
use dyn_clone::{DynClone, clone_trait_object};
use std::marker::PhantomData;
use num_traits::Float;

pub mod chainable_curves;
pub mod single_curves;

pub trait Curve: DynClone + Send + Sync {
    type Value: Float;
    fn get_value(&self, t: Self::Value) -> Self::Value;
}
clone_trait_object!(Curve<Value = f32>);
clone_trait_object!(Curve<Value = f64>);


pub(crate) fn min_f<T: Float + PartialOrd>(a: T, b: T) -> T {
    if a < b {
        return a;
    }
    b
}

pub(crate) fn max_f<T: Float + PartialOrd>(a: T, b: T) -> T {
    if a > b {
        return a;
    }
    b
}
