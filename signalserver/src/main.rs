use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    extract::ws::Message,
    http::{header::CONTENT_TYPE, Method},
};
use futures::StreamExt;
use matchbox_protocol::{JsonPeerEvent, PeerId, PeerRequest};
use matchbox_signaling::{
    common_logic::{parse_request, SignalingChannel},
    topologies::full_mesh::FullMesh,
    ClientRequestError, NoCallbacks, SignalingServer, SignalingServerBuilder, SignalingState,
    SignalingTopology, WsStateMeta,
};
use tokio::sync::mpsc::unbounded_channel;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
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
    rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
    next: Arc<Mutex<Option<NextPeer>>>,
}
impl SignalingState for ServerState {}

#[derive(Debug, Clone)]
struct NextPeer {
    pub room_id: Uuid,
}

#[derive(Debug, Clone)]
struct Room {
    peers: Vec<Peer>,
}

#[derive(Debug, Clone)]
struct Peer {
    pub uuid: PeerId,
    pub sender: SignalingChannel,
}

#[tokio::main]
async fn main() {
    setup_logging();

    info!("Initialising Signal Server");

    let state = ServerState::default();

    let uuid = Uuid::parse_str("c957a42c-ec98-41fd-be84-4cd7f4a584fd").unwrap();

    state
        .rooms
        .lock()
        .unwrap()
        .insert(uuid, Room { peers: Vec::new() });

    let railway_env = env::var("RAILWAY_PROJECT_NAME");
    tracing::debug!("railway_env: {:?}", railway_env);
    let railway_env = railway_env.is_ok();
    let origins = match railway_env {
        false => vec!["https://localhost:5173".parse().unwrap()],
        true => vec!["https://matchboy.stevenhuyn.com".parse().unwrap()],
    };

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);

    let host = match railway_env {
        false => [127, 0, 0, 1],
        true => [0, 0, 0, 0],
    };

    let port_string = env::var("PORT").unwrap_or_else(|_| String::from("3536"));
    let port = port_string.parse::<u16>().unwrap_or(3000);
    let addr = SocketAddr::from((host, port));

    let server = SignalingServer::full_mesh_builder(addr)
        .on_connection_request(|connection| {
            info!("Connecting: {connection:?}");
            Ok(true) // Allow all connections
        })
        .on_id_assignment(|(socket, id)| info!("{socket} received {id}"))
        .on_peer_connected(|id| info!("Joined: {id}"))
        .on_peer_disconnected(|id| info!("Left: {id}"))
        .mutate_router(|router| router.layer(cors.clone()))
        .trace()
        .build();

    server
        .serve()
        .await
        .expect("Unable to run signalling server");
}