use crate::prelude::{
    chainable_curves::ChainableCurve,
    chainable_curves::{LinearPiecewiseCurve, SmoothCurve},
    Curve, Point,
};
use dyn_clone::{clone_trait_object, DynClone};
use num_traits::Float;
use std::{fmt::Display, time::Duration};

/// Trait for defining keyframes structs. Is defined for chainable and non-chainable curves, but is only really useful for non-chainable and only serves as a bloated abstraction when used with chainable.
pub trait KeyFrames: DynClone {
    type Value: Clone + Display + Send + Sync;
    fn get_value(&self, time: Duration) -> Self::Value;
}

clone_trait_object!(KeyFrames<Value = f32>);
clone_trait_object!(KeyFrames<Value = f64>);
clone_trait_object!(KeyFrames<Value = Point<f32>>);
clone_trait_object!(KeyFrames<Value = Point<f64>>);

#[derive(Clone)]
pub struct ScalarKeyFrames<
    T: Clone + Display + Float + Send + Sync + From<f64>,
    C: Curve<Value = T>,
> {
    pub curve: C,
}

impl<
        T: Clone + Display + Float + Send + Sync + From<f64>,
        C: ChainableCurve<Value = T> + Curve<Value = T> + Clone,
    > KeyFrames for ScalarKeyFrames<T, C>
{
    type Value = T;
    fn get_value(&self, time: Duration) -> Self::Value
    where
        <C as Curve>::Value: From<f64>,
    {
        From::<T>::from(self.curve.get_value(From::<f64>::from(time.as_secs_f64())))
    }
}

impl<
        T: Clone + Display + Float + Send + Sync + From<f64>,
        C: ChainableCurve<Value = T> + Curve<Value = T> + Clone,
    > ScalarKeyFrames<T, C>
{
    pub fn new(data: Vec<(f64, T)>) -> Self {
        Self {
            curve: C::new(
                data.iter()
                    .map(|(t, v)| (From::<f64>::from(*t), *v))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

pub type SmoothKeyframes<T> = ScalarKeyFrames<T, SmoothCurve<T>>;
pub type LinearKeyframes<T> = ScalarKeyFrames<T, LinearPiecewiseCurve<T>>;
