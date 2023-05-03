use crate::prelude::*;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, ErrorStack)]
pub enum FSMError {
    #[error_message("Failed to add state to finite state machine.")]
    AddState,
    #[error_message("Failed to remove state from finite state machine.")]
    RemoveState,
    #[error_message("State {0} doesn't exist")]
    StateDoesntExist(String),
}

#[derive(Clone)]
pub struct FiniteStateMachine {
    fsm: FSMCore,
}

impl Behaviour for FiniteStateMachine {
    fn process(
        &mut self,
        _renderable: &mut RenderableParams,
        _time: Duration,
        _scene: &Scene,
        _abs_position: Point<isize>,
    ) {
        todo!()
    }
    fn get_pixel(
        &self,
        _current_frame: &Img,
        _uv_coords: Point<f64>,
        _time: Duration,
        _abs_position: Point<isize>,
    ) -> Rgba<u8> {
        todo!()
    }
}

impl FiniteStateMachine {
    pub fn builder() -> FSMBuilder {
        FSMBuilder::new()
    }
}

#[derive(Clone)]
pub struct FSMCore {
    states: HashMap<String, Box<dyn FSMState>>,
    state: String,
}

impl FSMCore {
    pub fn add_state(&mut self, name: String, state: Box<dyn FSMState>) -> Result<(), FSMError> {
        self.states
            .insert(name, state)
            .ok_or(Report::new(FSMError::AddState))?;
        Ok(())
    }
    pub fn remove_state(&mut self, name: String) -> Result<Box<dyn FSMState>, FSMError> {
        self.states
            .remove(&name)
            .ok_or(Report::new(FSMError::RemoveState))
    }
    pub fn get_state(&self, name: String) -> Option<&Box<dyn FSMState>> {
        self.states.get(&name)
    }
    pub fn get_state_mut(&mut self, name: String) -> Option<&mut Box<dyn FSMState>> {
        self.states.get_mut(&name)
    }
    pub fn change_state(&mut self, name: String) -> Result<(), FSMError> {
        if self.states.contains_key(&name) {
            return Err(Report::new(FSMError::StateDoesntExist(name)));
        }
        self.states
            .get_mut(&self.state)
            .ok_or(Report::new(FSMError::StateDoesntExist(self.state.clone())))?
            .exit(name.clone());

        self.state = name.clone();

        self.states
            .get_mut(&self.state)
            .ok_or(Report::new(FSMError::StateDoesntExist(self.state.clone())))?
            .entry(Some(name));

        Ok(())
    }
}

#[derive(ErrorStack, Debug)]
#[error_message("Failed to build a finite state machine")]
pub enum FSMBuilderError {
    #[error_message("There was no initial state specified.")]
    NoInitState,
    #[error_message("No states were added to the machine.")]
    NoStates,
}

#[derive(Clone)]
pub struct FSMBuilder {
    pub states: HashMap<String, Box<dyn FSMState>>,
    pub state: String,
}

impl FSMBuilder {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            state: "".to_owned(),
        }
    }
    pub fn add_state(&mut self, name: String, state: Box<dyn FSMState>) -> &mut Self {
        self.states.insert(name, state).unwrap();
        self
    }
    pub fn init_state(&mut self, name: String) -> &mut Self {
        self.state = name;
        self
    }
    pub fn build(self) -> Result<FiniteStateMachine, FSMBuilderError> {
        if self.states.is_empty() {
            return Err(Report::new(FSMBuilderError::NoStates));
        }
        if self.state.is_empty() {
            return Err(Report::new(FSMBuilderError::NoInitState));
        }
        Ok(FiniteStateMachine {
            fsm: FSMCore {
                states: self.states,
                state: self.state,
            },
        })
    }
}

pub trait FSMState: DynClone + Send + Sync {
    fn process(
        &mut self,
        fsm: &mut FSMCore,
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
    fn entry(&mut self, from: Option<String>);
    fn exit(&mut self, to: String);
}

clone_trait_object! {FSMState}

#[derive(Clone)]
pub struct FSMStateClosure<D, P, S, Entry, Exit>
where
    D: Clone + Send + Sync,
    P: Clone
        + Send
        + Sync
        + Fn(&mut D, &mut FSMCore, &mut RenderableParams, Duration, &Scene, Point<isize>),
    S: Clone + Send + Sync + Fn(&D, &Img, Point<f64>, Duration, Point<isize>) -> Rgba<u8>,
    Entry: Clone + Send + Sync + Fn(&mut D, Option<String>),
    Exit: Clone + Send + Sync + Fn(&mut D, String),
{
    pub data: D,
    pub process: P,
    pub shader: S,
    pub entry: Entry,
    pub exit: Exit,
}

impl<D, P, S, Entry, Exit> FSMState for FSMStateClosure<D, P, S, Entry, Exit>
where
    D: Clone + Send + Sync,
    P: Clone
        + Send
        + Sync
        + Fn(&mut D, &mut FSMCore, &mut RenderableParams, Duration, &Scene, Point<isize>),
    S: Clone + Send + Sync + Fn(&D, &Img, Point<f64>, Duration, Point<isize>) -> Rgba<u8>,
    Entry: Clone + Send + Sync + Fn(&mut D, Option<String>),
    Exit: Clone + Send + Sync + Fn(&mut D, String),
{
    fn entry(&mut self, from: Option<String>) {
        (self.entry)(&mut self.data, from)
    }
    fn exit(&mut self, to: String) {
        (self.exit)(&mut self.data, to)
    }
    fn get_pixel(
        &self,
        current_frame: &Img,
        uv_coords: Point<f64>,
        time: Duration,
        abs_position: Point<isize>,
    ) -> Rgba<u8> {
        (self.shader)(&self.data, current_frame, uv_coords, time, abs_position)
    }
    fn process(
        &mut self,
        fsm: &mut FSMCore,
        renderable: &mut RenderableParams,
        time: Duration,
        scene: &Scene,
        abs_position: Point<isize>,
    ) {
        (self.process)(&mut self.data, fsm, renderable, time, scene, abs_position)
    }
}
