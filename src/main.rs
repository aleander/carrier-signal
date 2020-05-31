#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate serde;

use std::sync::{Arc, Mutex};

use rocket::State as RocketState;
use rocket_contrib::templates::Template;

mod simulation;
mod state;

use simulation::Simulation;
use state::{State, WrappedState};

#[get("/")]
fn index(sim: RocketState<WrappedState>) -> Template {
    let state = sim.lock().unwrap();

    
    Template::render("stuff", state.clone())
}

fn simulation(s: WrappedState) {
    std::thread::spawn(move || {
        let mut simulation = Simulation::new();

        simulation.run(s);
    });
}

fn main() {
    let state = Arc::new(Mutex::new(State {
        iteration: 0, objects: vec![]
    }));

    simulation(Arc::clone(&state));

    rocket::ignite()
        .attach(Template::fairing())
        .manage(Arc::clone(&state))
        .mount("/", routes![index])
        .launch();
}
