use super::Curve;
use num_traits::Float;
use std::marker::PhantomData;

pub trait SingleCurve {}

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

impl<T, C> SingleCurve for ClosureCurve<T, C>
where
    T: Float + Clone + Send + Sync,
    C: Fn(T) -> T + Clone + Send + Sync,
{
}
