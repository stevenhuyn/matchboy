extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use std::{cell::RefCell, time::Duration};

use futures::{select, FutureExt};
use futures_timer::Delay;
use log::{info, warn};
use matchbox_socket::{Packet, PeerId, PeerState, RtcIceServerConfig, WebRtcSocket};
use serde::{Deserialize, Serialize};
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

thread_local! {
    pub static QUEUE: RefCell<Vec<Message>> = RefCell::new(Vec::new());
    pub static HISTORY: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

#[wasm_bindgen]
pub fn send_message(message: String) {
    info!("Sending message: {message}");
    QUEUE.with(|state| state.borrow_mut().push(Message::Message(message)));
}

#[wasm_bindgen]
pub fn get_history() -> JsValue {
    HISTORY.with(|state| {
        let history = state.borrow();
        let collected: Vec<String> = history.iter().map(|message| message.to_string()).collect();

        // Convert Vec<String> to JsValue
        serde_wasm_bindgen::to_value(&collected).unwrap()
    })
}

#[wasm_bindgen]
pub fn clear_history() {
    info!("Clearing History");
    HISTORY.with(|state| state.borrow_mut().clear());
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Message {
    Join,
    Leave,
    Message(String),
}

impl Message {
    pub fn to_chat_message(&self, peer_id: PeerId) -> String {
        let binding = &peer_id.0.to_string();
        let name = &binding.as_str()[..5];
        match self {
            Message::Join => format!("{} joined", name),
            Message::Leave => format!("{} left", name),
            Message::Message(message) => format!("{}: {}", name, message),
        }
    }
}

#[wasm_bindgen]
pub async fn connect(url: &str) {
    info!("Connecting to matchbox");

    let ice_server = RtcIceServerConfig {
        urls: vec!["stun:stun.l.google.com:19302".to_string()],
        username: None,
        credential: None,
    };

    let (mut socket, loop_fut) = WebRtcSocket::builder(url)
        .ice_server(ice_server)
        .add_reliable_channel()
        .build();

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

                    let bin_message = bincode::serialize(&Message::Join).unwrap();
                    let packet: Packet = bin_message.into_boxed_slice();
                    socket.send(packet, peer);
                }
                PeerState::Disconnected => {
                    let chat_message = Message::Leave.to_chat_message(peer);
                    HISTORY.with(|state| state.borrow_mut().push(chat_message));
                }
            }
        }

        // Accept any messages incoming
        for (peer, packet) in socket.receive() {
            if let Ok(message) = bincode::deserialize::<Message>(&packet) {
                let chat_message = message.to_chat_message(peer);
                HISTORY.with(|state| state.borrow_mut().push(chat_message));
            }
        }

        // Handle any messages to send
        // TODO: Move into the select
        while let Some(message) = QUEUE.with(|state| state.borrow_mut().pop()) {
            match message {
                Message::Message(ref chat) => {
                    info!("Consuming message action: Message({})", &chat);
                }
                _ => {
                    warn!("Not valid manual user action");
                    continue;
                }
            }

            let bin_message = bincode::serialize(&message).unwrap();
            let packet: Packet = bin_message.into_boxed_slice();
            let chat_message = message.to_chat_message(socket.id().unwrap());

            info!("Adding message: {chat_message}");
            HISTORY.with(|state| state.borrow_mut().push(chat_message));

            socket.update_peers();
            let peers: Vec<PeerId> = socket.connected_peers().collect();

            for peer in peers {
                socket.send(packet.clone(), peer);
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
