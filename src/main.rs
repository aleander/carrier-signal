#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod simulation;

use std::fmt::Write as FmtWrite;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rocket::State;

use simulation::{Object, Simulation};


struct SimState {
    pub iteration: u64,
    pub objects: Vec<Object>
}

type WrappedState = Arc<Mutex<SimState>>;

#[get("/")]
fn index(sim: State<WrappedState>) -> String {
    let state = sim.lock().unwrap();

    let mut result = String::from("Hello, world!\n");
    
    write!(&mut result, "Iteration {}\n", (*state).iteration).unwrap();

    for object in &(*state).objects {
        write!(&mut result, "Object {} at {}, {}\n", object.name, object.x, object.y).unwrap();
    }

    result
}

fn simulation(s: WrappedState) {
    std::thread::spawn(move || {
        let mut simulation = Simulation::new();

        loop {
            simulation.update();
            {
                let mut state = s.lock().unwrap();
                (*state).iteration += 1;
                (*state).objects = simulation.render();
            }
            thread::sleep(Duration::from_secs(1))
        }
    });
}

fn main() {
    let state = Arc::new(Mutex::new(SimState {
        iteration: 0, objects: vec![]
    }));

    simulation(Arc::clone(&state));

    rocket::ignite()
        .manage(Arc::clone(&state))
        .mount("/", routes![index])
        .launch();
}
