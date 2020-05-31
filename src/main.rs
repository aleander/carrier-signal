#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
extern crate serde;

use std::sync::Arc;

use rocket::State as RocketState;
use rocket_contrib::templates::Template;

mod simulation;
mod state;

use simulation::Simulation;
use state::WrappedState;

#[get("/")]
fn index(sim: RocketState<WrappedState>) -> Template {
    let state = sim.lock().unwrap();

    
    Template::render("stuff", state.clone())
}

fn main() {
    let mut simulation = Simulation::new();
    let state = Arc::clone(&simulation.state);

    std::thread::spawn(move || {
        simulation.run();
    });

    rocket::ignite()
        .attach(Template::fairing())
        .manage(state)
        .mount("/", routes![index])
        .launch();
}
