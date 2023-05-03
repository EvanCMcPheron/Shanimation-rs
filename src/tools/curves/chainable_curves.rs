use super::{Curve};

use num_traits::Float;
use std::fmt::Display;

pub trait ChainableCurve: Curve {
    type Value: Float;
    fn new(
        data: Vec<(
            <Self as ChainableCurve>::Value,
            <Self as ChainableCurve>::Value,
        )>,
    ) -> Self;
}

#[derive(Clone)]
pub struct LinearPiecewiseCurve<T: Float + Clone + Send + Sync + Display>(pub Vec<(T, T)>);

impl<T: Float + Clone + Send + Sync + Display> Curve for LinearPiecewiseCurve<T> {
    type Value = T;
    fn get_value(&self, t: Self::Value) -> Self::Value {
        if self.0.is_empty() {
            return T::nan();
        }
        if self.0.len() == 1 || t <= self.0[0].0 {
            return self.0[0].1;
        }
        if t >= self.0.last().unwrap().0 {
            return self.0.last().unwrap().1;
        }

        let mut bound_indexs = (0, 0);
        for (i, p) in self.0.iter().enumerate() {
            bound_indexs.1 = i;
            if p.0 > t {
                break;
            }
            bound_indexs.0 = i;
            continue;
        }
        let slope = (self.0[bound_indexs.1].1 - self.0[bound_indexs.0].1)
            / (self.0[bound_indexs.1].0 - self.0[bound_indexs.0].0);
        let intercept = self.0[bound_indexs.0].1 - slope * self.0[bound_indexs.0].0;
        slope * t + intercept
    }
}

impl<T: Float + Clone + Send + Sync + Display> ChainableCurve for LinearPiecewiseCurve<T> {
    type Value = T;
    fn new(
        data: Vec<(
            <Self as ChainableCurve>::Value,
            <Self as ChainableCurve>::Value,
        )>,
    ) -> Self {
        LinearPiecewiseCurve(data)
    }
}

#[derive(Clone)]
pub struct SmoothCurve<T: Float + Clone + Send + Sync + Display>(pub Vec<(T, T)>); // (Point, max speed used to reach this point (if it's not the first))

impl<T: Float + Clone + Send + Sync + Display> Curve for SmoothCurve<T>
where
    f64: From<T> + Into<T>,
{
    type Value = T;
    fn get_value(&self, t: Self::Value) -> Self::Value {
        if self.0.is_empty() {
            return T::nan();
        }
        if self.0.len() == 1 || t <= self.0[0].0 {
            return self.0[0].1;
        }
        if t >= self.0.last().unwrap().0 {
            return self.0.last().unwrap().1;
        }

        let mut bound_indexs = (0, 0);
        for (i, p) in self.0.iter().enumerate() {
            bound_indexs.1 = i;
            if p.0 > t {
                break;
            }
            bound_indexs.0 = i;
            continue;
        }

        let t_to_f64 = |t: T| -> f64 { Into::<f64>::into(t) };

        let p1 = self.0[bound_indexs.0];
        let p2 = self.0[bound_indexs.1];

        let dx = t_to_f64(p2.0 - p1.0);
        let dy = t_to_f64(p2.1 - p1.1);

        let x = t_to_f64(t - p1.0);
        let max_vel = 2.0 * dy / dx;

        if x < dx / 2.0 {
            return T::from(max_vel * x.powi(2) / dx + t_to_f64(p1.1)).unwrap();
        }

        let f = |x: f64| -max_vel * x.powi(2) / dx + 2.0 * max_vel * x;

        return T::from(f(x) - f(dx / 2.0) + max_vel * dx / 4.0 + t_to_f64(p1.1)).unwrap();
    }
}

impl<T: Float + Clone + Send + Sync + Display> ChainableCurve for SmoothCurve<T>
where
    f64: From<T> + Into<T>,
{
    type Value = T;
    fn new(
        data: Vec<(
            <Self as ChainableCurve>::Value,
            <Self as ChainableCurve>::Value,
        )>,
    ) -> Self {
        SmoothCurve(data)
    }
}
