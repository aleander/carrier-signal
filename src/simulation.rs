use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


use legion::prelude::*;
use rand::prelude::*;

use crate::state::{State, WrappedState, Object};

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

pub struct Simulation {
    world: World,
    pub state: WrappedState,
}

impl Simulation {
    pub fn new() -> Self {
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
    
        let mut result = Self {world, state: Arc::new(Mutex::new(State {
            iteration: 0, objects: vec![]
        }))};

        result.render();

        result
    }

    pub fn update(&mut self) {
        let update_query = <(Write<Position>, Read<Velocity>)>::query();
        for (mut pos, vel) in update_query.iter(&mut self.world) {
            pos.x += vel.dx;
            pos.y += vel.dy;
        }
        self.state.lock().unwrap().iteration += 1;
    }

    fn render(&mut self) {
        let mut state = self.state.lock().unwrap();

        state.objects = vec![];

        for (name, pos) in <(Read<Name>, Read<Position>)>::query().iter(&mut self.world) {
            state.objects.push(Object{ name: name.name.clone(), x: pos.x, y: pos.y });
        }
    }

    pub fn run(&mut self) {
        loop {
            self.update();
            self.render();

            thread::sleep(Duration::from_secs(1))
        }
    }
}
