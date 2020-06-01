mod simulation;
mod state;

use simulation::Simulation;

#[tokio::main]
async fn main() {
    let mut simulation = Simulation::new();
    let mut state_channel = simulation.state();
    
    tokio::spawn(async move {
        simulation.run().await;
    });

    loop {
        let state = state_channel.recv().await.unwrap();
        println!("Iteration {}", state.iteration)
    }
}
