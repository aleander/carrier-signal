mod simulation;
mod state;

use std::{convert::Infallible, sync::Arc};

use futures::SinkExt;
use serde_json::json;
use tera::{Context, Tera};
use tokio::sync::watch;
use warp::{Filter, ws::{Message, WebSocket}};

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
    let tera = move || {
        let tr = tr.clone();
        move |with_template| render(with_template, tr.clone())
    };
    let state = move || {
        let state_channel = state_channel.clone();
        move || state_channel.clone()
    };

    let sim = tokio::spawn(async move {
        simulation.run().await;
    });

    let hello = warp::get()
        .and(warp::path::end())
        .map(state())
        .and_then(|channel| async move { get_state(channel).await })
        .map(|state| WithTemplate {
            name: "stuff.tera",
            context: Context::from_serialize(state).unwrap(),
        })
        .map(tera());

    let wsstate = state();
    let ws = warp::path("state")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| (ws, wsstate()))
        .map(|(ws, state): (warp::ws::Ws, watch::Receiver<State>)| {
            ws.on_upgrade(move |websocket| watch_state(websocket, state))
        });

    let server =
        tokio::spawn(async move { warp::serve(hello.or(ws)).run(([127, 0, 0, 1], 3030)).await });

    tokio::try_join!(sim, server).unwrap();
}

async fn watch_state(mut ws: WebSocket, mut state_channel: watch::Receiver<State>) {
    loop {
        let state = state_channel.recv().await;

        let result = match state {
            Some(state) => ws.send(Message::text(json!(state).to_string())).await,
            None => ws.send(Message::text("Oh...")).await,
        };

        if let Err(_) = result {
            break;
        }
    };
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
