// src/p2p.rs
use hyper::{Body, Request, Response, Server, Client, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMessage {
    pub service_id: String,
    pub version: crate::SemVerX,
    pub payload: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct P2PService {
    pub service_id: String,
    pub port: u16,
    pub peers: Arc<RwLock<HashMap<String, String>>>,
    pub state: Arc<RwLock<ServiceState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceState {
    pub healthy: bool,
    pub last_heartbeat: i64,
    pub version: crate::SemVerX,
    pub recovery_count: u32,
}

impl P2PService {
    pub async fn new(service_id: String, port: u16) -> Self {
        Self {
            service_id,
            port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(ServiceState {
                healthy: true,
                last_heartbeat: chrono::Utc::now().timestamp(),
                version: crate::SemVerX::new(1, 0, 0),
                recovery_count: 0,
            })),
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let peers = self.peers.clone();
        let state = self.state.clone();
        let service_id = self.service_id.clone();

        let make_svc = make_service_fn(move |_conn| {
            let peers = peers.clone();
            let state = state.clone();
            let service_id = service_id.clone();
            
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    handle_request(req, peers.clone(), state.clone(), service_id.clone())
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);
        println!("[{}] P2P Service listening on {}", self.service_id, addr);
        
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }
        
        Ok(())
    }

    pub async fn connect_peer(&self, peer_id: String, peer_addr: String) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id, peer_addr);
    }

    pub async fn send_to_peer(
        &self,
        peer_id: &str,
        message: ServiceMessage,
    ) -> Result<Response<Body>, Box<dyn std::error::Error>> {
        let peers = self.peers.read().await;
        if let Some(peer_addr) = peers.get(peer_id) {
            let client = Client::new();
            let uri = format!("http://{}/message", peer_addr).parse()?;
            
            let req = Request::builder()
                .method(Method::POST)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&message)?))?;
            
            Ok(client.request(req).await?)
        } else {
            Err("Peer not found".into())
        }
    }
}

async fn handle_request(
    req: Request<Body>,
    peers: Arc<RwLock<HashMap<String, String>>>,
    state: Arc<RwLock<ServiceState>>,
    service_id: String,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") => {
            let state = state.read().await;
            let response = serde_json::json!({
                "service_id": service_id,
                "healthy": state.healthy,
                "version": state.version.to_string(),
                "recovery_count": state.recovery_count,
            });
            Ok(Response::new(Body::from(response.to_string())))
        }
        (&Method::POST, "/message") => {
            // Handle incoming messages from peers
            Ok(Response::new(Body::from("Message received")))
        }
        _ => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap())
        }
    }
}
