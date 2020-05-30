#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::fmt::Write as FmtWrite;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use legion::prelude::*;
use rand::prelude::*;
use rocket::State;

#[derive(Clone, Debug, PartialEq)]
struct Name {
    name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f64,
    dy: f64,
}

struct ObjectOutput {
    pub name: String,
    pub x: f64,
    pub y: f64,
}

struct SimState {
    pub iteration: u64,
    pub objects: Vec<ObjectOutput>
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
        let universe = Universe::new();
        let mut world = universe.create_world();
        let mut rng = thread_rng();

        world.insert(
            (),
            (0..999).map(|n| {
                (
                    Name { name: format!("Entity {}", n)},
                    Position { x: 0.0, y: 0.0 },
                    Velocity {
                        dx: rng.gen_range(0.0, 1.0),
                        dy: rng.gen_range(0.0, 1.0),
                    },
                )
            }),
        );

        
        loop {            
            let update_query = <(Write<Position>, Read<Velocity>)>::query();
            for (mut pos, vel) in update_query.iter(&mut world) {
                pos.x += vel.dx;
                pos.y += vel.dy;
            }

            let mut state = s.lock().unwrap();
            (*state).iteration += 1;
            let mut result: Vec<ObjectOutput> = vec![];

            for (name, pos) in <(Read<Name>, Read<Position>)>::query().iter(&mut world) {
                result.push(ObjectOutput{ name: name.name.clone(), x: pos.x, y: pos.y });
            }
            (*state).objects = result;

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
