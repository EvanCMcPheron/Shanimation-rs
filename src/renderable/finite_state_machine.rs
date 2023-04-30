use crate::prelude::*;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone)]
pub struct FiniteStateMachine {
    pub states: HashMap<String, Box<dyn FSMState>>,
    pub state: String,
}

impl Behaviour for FiniteStateMachine {
    fn process(
        &mut self,
        renderable: &mut RenderableParams,
        time: Duration,
        scene: &Scene,
        abs_position: Point<isize>,
    ) {
        todo!()
    }
    fn get_pixel(
        &self,
        current_frame: &Img,
        uv_coords: Point<f64>,
        time: Duration,
        abs_position: Point<isize>,
    ) -> Rgba<u8> {
        todo!()
    }
}

impl FiniteStateMachine {
    pub fn builder() -> FSMBuilder {
        todo!()
    }
}

#[derive(Clone)]
pub struct FSMBuilder {
    pub states: HashMap<String, Box<dyn FSMState>>,
    pub state: String,
}

impl FSMBuilder {
    pub fn new() -> Self {
        todo!()
    }
    pub fn add_state(&mut self, name: String, state: Box<dyn FSMState>) {
        todo!()
    }
    pub fn init_state(&mut self, name: String) {
        todo!()
    }
    pub fn build(self) -> FiniteStateMachine {
        todo!()
    }
}

pub trait FSMState: DynClone + Send + Sync {
    fn process(
        &mut self,
        fsm: &mut FiniteStateMachine,
        renderable: &mut RenderableParams,
        time: Duration,
        scene: &Scene,
        abs_position: Point<isize>,
    );
    fn get_pixel(
        &self,
        current_frame: &Img,
        uv_coords: Point<f64>,
        time: Duration,
        abs_position: Point<isize>,
    ) -> Rgba<u8>;
    fn entry(
        &mut self,
        fsm: &mut FiniteStateMachine,
        renderable: &mut RenderableParams,
        time: Duration,
        scene: &Scene,
        abs_position: Point<isize>,
    );
    fn exit(
        &mut self,
        fsm: &mut FiniteStateMachine,
        renderable: &mut RenderableParams,
        time: Duration,
        scene: &Scene,
        abs_position: Point<isize>,
    );
}

clone_trait_object! {FSMState}
