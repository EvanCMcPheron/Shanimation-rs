use crate::prelude::*;
use dyn_clone::{clone_trait_object, DynClone};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

#[derive(Debug, ErrorStack)]
pub enum FSMError {
    #[error_message("Failed to add a state to the states hashmap")]
    AddState,
    #[error_message("State {0} doesn't exist")]
    StateDoesntExist(String),
    #[error_message("RwLock returned an error, ownership issue.")]
    OwnershipIssue,
}

#[derive(Clone)]
pub struct FiniteStateMachine {
    fsm: FSMCore,
}

impl Behaviour for FiniteStateMachine {
    fn process(
        &mut self,
        renderable: &mut RenderableParams,
        time: Duration,
        scene: &Scene,
        abs_position: Point<isize>,
    ) {
        let state = self.fsm
            .get_state_mut(&self.fsm.state)
            .unwrap()
            .process(fsm, renderable, time, scene, abs_position);
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
        FSMBuilder::new()
    }
}

#[derive(Clone)]
pub struct FSMCore {
    states: HashMap<String, Arc<RwLock<Box<dyn FSMState>>>>,
    state: String,
}

impl FSMCore {
    pub fn add_state(&mut self, name: &str, state: Box<dyn FSMState>) -> Result<(), FSMError> {
        self.states
            .insert(name.to_owned(), Arc::new(RwLock::new(state)))
            .ok_or(Report::new(FSMError::AddState))?;
        Ok(())
    }
    pub fn remove_state(&mut self, name: &str) -> Result<(), FSMError> {
        self.states
            .remove(name)
            .ok_or(Report::new(FSMError::StateDoesntExist(name.to_owned())))
            .attach_printable_lazy(|| "Failed to find and remove state in states hashmap.")?;
        Ok(())
    }
    pub fn get_state(&self, name: &str) -> Result<RwLockReadGuard<Box<dyn FSMState>>, FSMError> {
        Ok(self
            .states
            .get(name)
            .ok_or(Report::new(FSMError::StateDoesntExist(name.to_owned())))?
            .read()
            .map_err(|e| {
                Report::new(FSMError::OwnershipIssue).attach_printable(format!(
                    "Failed to get read lock rwlock containing state: \n{}",
                    e.to_string()
                ))
            })?)
    }
    pub fn get_state_mut(
        &mut self,
        name: &str,
    ) -> Result<RwLockWriteGuard<Box<dyn FSMState>>, FSMError> {
        Ok(self
            .states
            .get(name)
            .ok_or(Report::new(FSMError::StateDoesntExist(name.to_owned())))?
            .write()
            .map_err(|e| {
                Report::new(FSMError::OwnershipIssue).attach_printable(format!(
                    "Failed to get write lock rwlock containing state: \n{}",
                    e.to_string()
                ))
            })?)
    }
    fn get_state_mut_interior(
        &self,
        name: &str,
    ) -> Result<RwLockWriteGuard<Box<dyn FSMState>>, FSMError> {
        Ok(self
            .states
            .get(name)
            .ok_or(Report::new(FSMError::StateDoesntExist(name.to_owned())))?
            .write()
            .map_err(|e| {
                Report::new(FSMError::OwnershipIssue).attach_printable(format!(
                    "Failed to get write lock rwlock containing state: \n{}",
                    e.to_string()
                ))
            })?)
    }
    pub fn change_state(&mut self, name: &str) -> Result<(), FSMError> {
        if self.states.contains_key(name) {
            return Err(Report::new(FSMError::StateDoesntExist(name.to_owned())));
        }

        let mut current_state = self
            .get_state_mut_interior(&self.state)
            .attach_printable("Failed to get mutable access to current state")?;

        let next_state = self
            .get_state_mut_interior(name)
            .attach_printable("Failed to get mutable access to next state")?;

        current_state.exit(next_state); //Passes next state to state exit fn, where the reference drops, allowing for it to be grabbed again right after

        let mut next_state = self
            .get_state_mut_interior(name)
            .attach_printable("Failed to get mutable access to next state")?;

        next_state.entry(Some(current_state));

        Ok(())
    }
    pub fn init_state(&mut self) -> Result<(), FSMError> {
        if self.states.contains_key(&self.state) {
            return Err(Report::new(FSMError::StateDoesntExist(self.state.clone())));
        }

        Ok(self.get_state_mut_interior(&self.state)?.entry(None))
    }
    pub fn run_process(&mut self) -> Result<(), FSMError> {
        let state = self.get_state_mut_interior(&self.state).unwrap();
        
        
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
    #[error_message("Failed to init state.")]
    FailedToInit,
}

#[derive(Clone)]
pub struct FSMBuilder {
    pub states: HashMap<String, Arc<RwLock<Box<dyn FSMState>>>>,
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
        self.states
            .insert(name, Arc::new(RwLock::new(state)))
            .unwrap();
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

        let mut fsm = FiniteStateMachine {
            fsm: FSMCore {
                states: self.states,
                state: self.state,
            },
        };

        fsm.fsm
            .init_state()
            .change_context(FSMBuilderError::FailedToInit)?;

        Ok(fsm)
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
    fn entry(&mut self, from: Option<RwLockWriteGuard<Box<dyn FSMState>>>);
    fn exit(&mut self, to: RwLockWriteGuard<Box<dyn FSMState>>);
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
    Entry: Clone + Send + Sync + Fn(&mut D, Option<RwLockWriteGuard<Box<dyn FSMState>>>),
    Exit: Clone + Send + Sync + Fn(&mut D, RwLockWriteGuard<Box<dyn FSMState>>),
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
    Entry: Clone + Send + Sync + Fn(&mut D, Option<RwLockWriteGuard<Box<dyn FSMState>>>),
    Exit: Clone + Send + Sync + Fn(&mut D, RwLockWriteGuard<Box<dyn FSMState>>),
{
    fn entry(&mut self, from: Option<RwLockWriteGuard<Box<dyn FSMState>>>) {
        (self.entry)(&mut self.data, from)
    }
    fn exit(&mut self, to: RwLockWriteGuard<Box<dyn FSMState>>) {
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
