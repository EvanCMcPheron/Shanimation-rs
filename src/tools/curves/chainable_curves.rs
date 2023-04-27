use super::{min_f, max_f, Curve};
use crate::prelude::*;
use num_traits::Float;
use std::fmt::Display;

pub trait ChainableCurve: Curve {}

#[derive(Clone)]
pub struct LinearPiecewiseCurve<T: Float + Clone + Send + Sync + Display> ( pub Vec<Point<T>> );

impl<T: Float + Clone + Send + Sync + Display> Curve for LinearPiecewiseCurve<T> {
    type Value = T;
    fn get_value(&self, t: Self::Value) -> Self::Value {
        if self.0.is_empty() {
            return T::nan();
        } if self.0.len() == 1 || t <= self.0[0].x {
            return self.0[0].y;
        } if t >= self.0.last().unwrap().x {
            return self.0.last().unwrap().y;
        }

        let mut bound_indexs = (0, 0);
        for (i, p) in self.0.iter().enumerate() {
            bound_indexs.1 = i;
            if p.x > t {
                break;
            }
            bound_indexs.0 = i;
            continue;
        }
        let slope = (self.0[bound_indexs.1].y - self.0[bound_indexs.0].y) / (self.0[bound_indexs.1].x - self.0[bound_indexs.0].x);
        let intercept = self.0[bound_indexs.0].y - slope * self.0[bound_indexs.0].x;
        slope * t + intercept
    }
}

impl<T: Float + Clone + Send + Sync + Display> ChainableCurve for LinearPiecewiseCurve<T> {}

#[derive(Clone)]
pub struct SmoothCurve<T: Float + Clone + Send + Sync + Display> (pub Vec<Point<T>>); // (Point, max speed used to reach this point (if it's not the first))

impl<T: Float + Clone + Send + Sync + Display> Curve for SmoothCurve<T> 
    where f64: From<T> + Into<T>,
{
    type Value = T;
    fn get_value(&self, t: Self::Value) -> Self::Value {
        if self.0.is_empty() {
            return T::nan();
        } if self.0.len() == 1 || t <= self.0[0].x {
            return self.0[0].y;
        } if t >= self.0.last().unwrap().x {
            return self.0.last().unwrap().y;
        }

        let mut bound_indexs = (0, 0);
        for (i, p) in self.0.iter().enumerate() {
            bound_indexs.1 = i;
            if p.x > t {
                break;
            }
            bound_indexs.0 = i;
            continue;
        }

        let t_to_f64 = |t: T| -> f64 { Into::<f64>::into(t) };

        let p1 = self.0[bound_indexs.0];
        let p2 = self.0[bound_indexs.1];

        let dx = t_to_f64(p2.x - p1.x);
        let dy = t_to_f64(p2.y - p1.y);

        let x = t_to_f64(t - p1.x);
        let max_vel = 2.0 * dy / dx;

        if x < dx / 2.0 {
            return T::from(max_vel * x.powi(2) / dx + t_to_f64(p1.y)).unwrap();
        }

        let f = |x: f64| -max_vel * x.powi(2) / dx + 2.0 * max_vel * x;

        return T::from(f(x) - f(dx / 2.0) + max_vel * dx / 4.0 + t_to_f64(p1.y)).unwrap();
    }
}

impl<T: Float + Clone + Send + Sync + Display> ChainableCurve for SmoothCurve<T> 
    where f64: From<T> + Into<T>,
{}
