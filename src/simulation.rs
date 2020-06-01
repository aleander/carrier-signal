use std::time::{Duration, Instant};

use legion::prelude::*;
use rand::prelude::*;
use tokio::{sync::watch, time};

use crate::state::{Object, State};

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
    iteration: u64,
    last: Instant,
    tx: watch::Sender<State>,
    rx: watch::Receiver<State>,
}

fn render(world: &mut World) -> Vec<Object> {
    let mut result = vec![];

    for (name, pos) in <(Read<Name>, Read<Position>)>::query().iter(world) {
        result.push(Object {
            name: name.name.clone(),
            x: pos.x,
            y: pos.y,
        });
    }

    result
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
                    Name {
                        name: format!("Entity {}", n),
                    },
                    Position { x: 0.0, y: 0.0 },
                    Velocity {
                        dx: rng.gen_range(0.0, 1.0),
                        dy: rng.gen_range(0.0, 1.0),
                    },
                )
            }),
        );

        let (tx, rx) = watch::channel(State {
            iteration: 0,
            objects: render(&mut world),
        });
        
        Self { world, last: Instant::now(), iteration: 0, tx, rx }
    }

    fn update(&mut self) {
        let update_query = <(Write<Position>, Read<Velocity>)>::query();
        let now = Instant::now();
        let dt = now.duration_since(self.last);
        for (mut pos, vel) in update_query.iter(&mut self.world) {
            pos.x += vel.dx * dt.as_secs_f64();
            pos.y += vel.dy * dt.as_secs_f64();
        }
        self.last = now;
        self.iteration += 1;
    }

    fn render(&mut self) -> State {
        State {
            iteration: self.iteration,
            objects: render(&mut self.world),
        }
    }

    pub async fn run(&mut self) {
        let mut interval = time::interval(Duration::from_millis(1000 / 60));

        loop {
            self.update();
            let state = self.render();
            self.tx.broadcast(state).unwrap();

            interval.tick().await;
        }
    }

    pub fn state(&mut self) -> watch::Receiver<State> {
        self.rx.clone()
    }
}
