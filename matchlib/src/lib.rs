extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use std::{cell::RefCell, env, time::Duration};

use futures::{select, FutureExt};
use futures_timer::Delay;
use log::info;
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
    console_log::init_with_level(log::Level::Debug).unwrap();
}

pub enum Action {
    Message(String),
}

thread_local! {
    pub static STATE: RefCell<Vec<Action>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn send_message(message: String) {
    STATE.with(|state| state.borrow_mut().push(Action::Message(message)));
}

#[wasm_bindgen]
pub async fn connect() {
    info!("Connecting to matchbox");
    let url = env::var("SIGNAL_SERVER_URL").unwrap_or("ws://localhost:3536/".to_string());

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
            info!("Message from {peer}: {message:?}");
        }

        // Handle any messages to send
        // TODO: Move into the select
        while let Some(action) = STATE.with(|state| state.borrow_mut().pop()) {
            match action {
                Action::Message(message) => {
                    let packet = message.as_bytes().to_vec().into_boxed_slice();
                    let peers: Vec<PeerId> = socket.connected_peers().collect();
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
