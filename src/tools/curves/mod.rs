// Make a linear piecewise "curve" which can be used for linear interpolation and a quadratic curve for quadratic interpolation (ie: smooth)
// also add a closure implementation of the curve method
use dyn_clone::{clone_trait_object, DynClone};
use num_traits::Float;
use std::marker::PhantomData;

pub trait Curve: DynClone + Send + Sync {
    type Value: Float;
    fn get_value(&self, t: Self::Value) -> Self::Value;
}
clone_trait_object!(Curve<Value = f32>);
clone_trait_object!(Curve<Value = f64>);

#[derive(Clone, Copy)]
pub struct ClosureCurve<T, C>
where
    T: Float + Clone + Send + Sync,
    C: Fn(T) -> T + Clone + Send + Sync,
{
    pub func: C,
    phantom: PhantomData<T>,
}

//Impliment curve for ClosureCurve
impl<T, C> Curve for ClosureCurve<T, C>
where
    T: Float + Clone + Send + Sync,
    C: Fn(T) -> T + Clone + Send + Sync,
{
    type Value = T;
    fn get_value(&self, t: T) -> T {
        (self.func)(t)
    }
}

impl<T, C> ClosureCurve<T, C>
where
    T: Float + Clone + Send + Sync,
    C: Fn(T) -> T + Clone + Send + Sync,
{
    fn new(closure: C) -> Self {
        Self {
            func: closure,
            phantom: PhantomData,
        }
    }
}
