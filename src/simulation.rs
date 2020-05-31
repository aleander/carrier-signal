use std::thread;
use std::time::Duration;

use legion::prelude::*;
use rand::prelude::*;

use crate::state::{WrappedState, Object};

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
    
        Self {world}
    }

    pub fn update(&mut self) {
        let update_query = <(Write<Position>, Read<Velocity>)>::query();
        for (mut pos, vel) in update_query.iter(&mut self.world) {
            pos.x += vel.dx;
            pos.y += vel.dy;
        }
    }

    fn render(&mut self) -> Vec<Object> {
        let mut result: Vec<Object> = vec![];

        for (name, pos) in <(Read<Name>, Read<Position>)>::query().iter(&mut self.world) {
            result.push(Object{ name: name.name.clone(), x: pos.x, y: pos.y });
        }

        result
    }

    pub fn run(&mut self, wrapped_state: WrappedState) {
        loop {
            self.update();
            {
                let mut state = wrapped_state.lock().unwrap();
                (*state).iteration += 1;
                (*state).objects = self.render();
            }
            thread::sleep(Duration::from_secs(1))
        }
    }
}
