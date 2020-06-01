use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Object {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct State {
    pub iteration: u64,
    pub objects: Vec<Object>,
}
