use semverx::p2p::P2PService;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = P2PService::new("rust-service".to_string(), 3001).await;
    
    // Connect to Go service
    service.connect_peer("go-service".to_string(), "127.0.0.1:3002".to_string()).await;
    
    // Connect to Python service  
    service.connect_peer("python-service".to_string(), "127.0.0.1:3003".to_string()).await;
    
    service.start().await?;
    Ok(())
}
