use std::time::{Duration, Instant};

use legion::prelude::*;
use nalgebra::{Vector3, Point3};
use rand::prelude::*;
use tokio::{sync::watch, time};

use crate::state::{Object, State};

#[derive(Clone, Debug, PartialEq)]
struct Name (String);

#[derive(Clone, Debug, PartialEq)]
struct Id (u64);

#[derive(Clone, Debug, PartialEq)]
struct Position (Point3<f64>);

#[derive(Clone, Debug, PartialEq)]
struct Velocity (Vector3<f64>);

pub struct Simulation {
    world: World,
    iteration: u64,
    last: Instant,
    tx: watch::Sender<State>,
    rx: watch::Receiver<State>,
}

fn render(world: &mut World) -> Vec<Object> {
    let mut result = vec![];

    for (id, name, pos) in <(Read<Id>, Read<Name>, Read<Position>)>::query().iter(world) {
        result.push(Object {
            id: id.0,
            name: name.0.clone(),
            x: pos.0.x,
            y: pos.0.y,
            z: pos.0.z
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
                    Id(n),
                    Name (format!("Entity {}", n)),
                    Position(Point3::new(0.0, 0.0, 0.0)),
                    Velocity(Vector3::new(
                        rng.gen_range(-1.0, 1.0),
                        rng.gen_range(-1.0, 1.0),
                        rng.gen_range(-1.0, 1.0),
                    )),
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
            pos.0 += vel.0 * dt.as_secs_f64();
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
