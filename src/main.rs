mod simulation;
mod state;

use std::convert::Infallible;
use std::sync::Arc;

use tera::{Context, Tera};
use tokio::sync::watch;
use warp::Filter;

use simulation::Simulation;
use state::State;

struct WithTemplate {
    name: &'static str,
    context: Context,
}

#[tokio::main]
async fn main() {
    let mut simulation = Simulation::new();
    let state_channel = simulation.state();

    let tr = match Tera::new("templates/**/*.tera") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let tr = Arc::new(tr);
    let tera = move |with_template| render(with_template, tr.clone());

    let sim = tokio::spawn(async move {
        simulation.run().await;
    });

    let hello = warp::get().and(warp::path::end())
        .map(move || (state_channel.clone()))
        .and_then(|channel| async move { get_state(channel).await })
        .map(|state| WithTemplate {
            name: "stuff.tera",
            context: Context::from_serialize(state).unwrap()
        })
        .map(tera);

    let server = tokio::spawn(async move { warp::serve(hello).run(([127, 0, 0, 1], 3030)).await });

    tokio::try_join!(sim, server).unwrap();
}

async fn get_state(mut channel: watch::Receiver<State>) -> Result<State, Infallible> {
    Ok(channel.recv().await.unwrap())
}

fn render(template: WithTemplate, tera: Arc<Tera>) -> impl warp::Reply {
    let render = tera
        .render(template.name, &template.context)
        .unwrap_or_else(|err| err.to_string());

    warp::reply::html(render)
}
