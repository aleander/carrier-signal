#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate serde;

mod simulation;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rocket::State;
use rocket_contrib::templates::Template;
use serde::Serialize;

use simulation::{Object, Simulation};

#[derive(Clone, Serialize)]
struct SimState {
    pub iteration: u64,
    pub objects: Vec<Object>
}

type WrappedState = Arc<Mutex<SimState>>;

#[get("/")]
fn index(sim: State<WrappedState>) -> Template {
    let state = sim.lock().unwrap();

    
    Template::render("stuff", state.clone())
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
        .attach(Template::fairing())
        .manage(Arc::clone(&state))
        .mount("/", routes![index])
        .launch();
}
