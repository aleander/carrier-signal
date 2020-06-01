mod server;
mod simulation;
mod state;

use simulation::Simulation;
use server::setup_server;

#[tokio::main]
async fn main() {
    let mut simulation = Simulation::new();
    let state_channel = simulation.state();

    let sim = tokio::spawn(async move {
        simulation.run().await;
    });

    let server = tokio::spawn(async move {
        setup_server(state_channel).await;
    });

    tokio::try_join!(sim, server).unwrap();
}
