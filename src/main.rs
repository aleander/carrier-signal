#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::thread;
use std::time::Duration;
use std::sync::{Arc,Mutex};

use rocket::State;

struct SimState {
    pub monotonic: u64,
    pub counter: u64,
}

type WrappedState = Arc<Mutex<SimState>>;

#[get("/")]
fn index(sim: State<WrappedState>) -> String {
    let state = sim.lock().unwrap();

    format!("Hello, world! It's round {} but also {}", (*state).monotonic, (*state).counter)
}

fn update_state(s: &Arc<Mutex<SimState>>) {
    let mut state = s.lock().unwrap();

    (*state).monotonic += 1;
    (*state).counter = (*state).monotonic % 10;
}

fn main() {
    let state = Arc::new(Mutex::new(SimState {
        monotonic: 0,
        counter: 0
    }));

    let inner_state = Arc::clone(&state);
    
    thread::spawn(move || {
        loop {
            update_state(&inner_state);
            thread::sleep(Duration::from_secs(1))
        }
    });

    rocket::ignite().manage(Arc::clone(&state)).mount("/", routes![index]).launch();
}
