use std::{collections::HashMap, net::SocketAddr};

use async_trait::async_trait;
use axum::{extract::ws::Message, Error};
use matchbox_protocol::PeerId;
use matchbox_signaling::{
    common_logic::SignalingChannel, NoCallbacks, SignalingServerBuilder, SignalingState,
    SignalingTopology, WsStateMeta,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;
use tracing_subscriber::prelude::*;
use uuid::Uuid;

fn setup_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "signalserver=info,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_file(false)
                .with_target(false),
        )
        .init();
}

#[derive(Default, Debug, Clone)]
struct ServerState {
    rooms: HashMap<Uuid, Room>,
    next: Option<NextPeer>,
}
impl SignalingState for ServerState {}

#[derive(Debug, Clone)]
struct NextPeer {
    room_id: Uuid,
}

#[derive(Debug, Clone)]
struct Room {
    peers: Vec<Peer>,
}

#[derive(Debug, Clone)]
struct Peer {
    uuid: PeerId,
    sender: SignalingChannel,
}

#[tokio::main]
async fn main() {
    setup_logging();

    info!("Initialising Signal Server");

    let server_state = ServerState::default();

    let addr: SocketAddr = "0.0.0.0:3536".parse().unwrap();
    let server = SignalingServerBuilder::new(addr, ChatRoomTopology, server_state)
        .on_connection_request(move |connection| {
            info!("Connection Request {connection:?}");
            Ok(true)
        })
        .on_id_assignment(move |(origin, peer_id)| {
            info!("Client connected {origin:?}: {peer_id:?}");
        })
        .cors()
        .trace()
        .build();

        server.serve().await.expect("Unable to run signalling server");
}

struct ChatRoomTopology;

#[async_trait]
impl SignalingTopology<NoCallbacks, ServerState> for ChatRoomTopology {
    async fn state_machine(upgrade: WsStateMeta<NoCallbacks, ServerState>) {
        let WsStateMeta {
            peer_id,
            sender,
            mut receiver,
            mut state,
            ..
        } = upgrade;

        let connecting_peer = Peer {
            uuid: peer_id,
            sender,
        };

        // for peer in state.rooms
    }
}
