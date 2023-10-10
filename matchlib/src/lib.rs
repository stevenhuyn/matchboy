extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use std::{cell::RefCell, env, time::Duration};

use futures::{future::ErrInto, select, FutureExt};
use futures_timer::Delay;
use log::{error, info};
use matchbox_socket::{PeerId, PeerState, WebRtcSocket};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello,{}!", name));
}

#[wasm_bindgen(start)]
pub fn init() {
    // Setup logging
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap();
}

pub enum Action {
    Message(String),
}

thread_local! {
    pub static QUEUE: RefCell<Vec<Action>> = RefCell::new(Vec::new());
    pub static HISTORY: RefCell<Vec<(String, String)>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn send_message(message: String) {
    info!("Sending message: {message}");
    QUEUE.with(|state| state.borrow_mut().push(Action::Message(message)));
}

#[wasm_bindgen]
pub fn get_history() -> JsValue {
    HISTORY.with(|state| {
        let history = state.borrow();
        let collected: Vec<(String, String)> = history
            .iter()
            .map(|(peer, message)| (peer.to_string(), message.to_string()))
            .collect();

        serde_wasm_bindgen::to_value(&collected).unwrap()
    })
}

#[wasm_bindgen]
pub async fn connect(url: &str) {
    info!("Connecting to matchbox");

    let (mut socket, loop_fut) = WebRtcSocket::new_reliable(url);

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(Duration::from_millis(100));
    futures::pin_mut!(timeout);

    loop {
        // Handle any new peers
        for (peer, state) in socket.update_peers() {
            match state {
                PeerState::Connected => {
                    info!("Peer joined: {peer}");
                    let packet = "hello friend!".as_bytes().to_vec().into_boxed_slice();
                    socket.send(packet, peer);
                }
                PeerState::Disconnected => {
                    info!("Peer left: {peer}");
                }
            }
        }

        // Accept any messages incoming
        for (peer, packet) in socket.receive() {
            let message = String::from_utf8_lossy(&packet);
            HISTORY.with(|state| state.borrow_mut().push((peer.0.to_string(), message.to_string())));
            info!("Message from {peer}: {message:?}");
        }

        // Handle any messages to send
        // TODO: Move into the select
        while let Some(action) = QUEUE.with(|state| state.borrow_mut().pop()) {
            match action {
                Action::Message(message) => {
                    info!("Consuming message action: {message}");

                    let packet = message.as_bytes().to_vec().into_boxed_slice();
                    let peers: Vec<PeerId> = socket.connected_peers().collect();

                    let (name, message) = (socket.id().unwrap().to_string(), message.clone());
                    HISTORY.with(|state| state.borrow_mut().push((name, message)));

                    for peer in peers {
                        socket.send(packet.clone(), peer);
                    }
                }
            }
        }

        select! {
            // Restart this loop every 100ms
            _ = (&mut timeout).fuse() => {
                timeout.reset(Duration::from_millis(100));
            }

            // Or break if the message loop ends (disconnected, closed, etc.)
            _ = &mut loop_fut => {
                break;
            }
        }
    }
}
