mod simulation;
mod state;

use std::convert::Infallible;

use tokio::sync::watch;
use warp::Filter;

use simulation::Simulation;
use state::State;

#[tokio::main]
async fn main() {
    let mut simulation = Simulation::new();
    let state_channel = simulation.state();

    let sim = tokio::spawn(async move {
        simulation.run().await;
    });

    let hello = warp::path!("hello" / String)
        .map(move |name| { (name, state_channel.clone()) })
        .and_then(|(name, channel)| async move { hello(name, channel).await });
       
    let server = tokio::spawn(async move { warp::serve(hello).run(([127, 0, 0, 1], 3030)).await });

    tokio::try_join!(sim, server).unwrap();
}

async fn hello(
    name: String,
    mut state_channel: watch::Receiver<State>,
) -> Result<impl warp::Reply, Infallible> {
    Ok(format!(
        "Hello, {}! We're at iteration {}",
        name,
        state_channel.recv().await.unwrap().iteration
    ))
}
