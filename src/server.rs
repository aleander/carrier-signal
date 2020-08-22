use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{sync::watch, time::throttle};
use warp::{
    reject::Reject,
    ws::{Message, WebSocket},
    Filter,
};

use crate::state::State;

pub async fn setup_server(state_channel: watch::Receiver<State>) {
    let state = move || {
        let state_channel = state_channel.clone();
        move || state_channel.clone()
    };

    let wsstate = state();
    let ws = warp::path("state")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| (ws, wsstate()))
        .map(|(ws, state): (warp::ws::Ws, watch::Receiver<State>)| {
            ws.on_upgrade(move |websocket| watch_state(websocket, state))
        });

    warp::serve(ws).run(([127, 0, 0, 1], 3030)).await;
}

async fn watch_state(ws: WebSocket, mut state_channel: watch::Receiver<State>) {
    let (mut tx, rx) = ws.split();
    let mut rx = throttle(Duration::from_millis(1000 / 30), rx);

    while let Some(_) = rx.next().await {
        let state = state_channel.recv().await;

        let result = match state {
            Some(state) => tx.send(Message::text(json!(state).to_string())).await,
            None => break,
        };

        if let Err(_) = result {
            break;
        }
    }
}

#[derive(Debug)]
struct SimulationUnavailable;

impl Reject for SimulationUnavailable {}
