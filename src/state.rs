use std::sync::{Arc, Mutex};

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct Object {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Serialize)]
pub struct State {
    pub iteration: u64,
    pub objects: Vec<Object>
}

pub type WrappedState = Arc<Mutex<State>>;
