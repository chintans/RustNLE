use nle_render::RenderEngine;

#[tokio::main]
async fn main() {
    match RenderEngine::new().await {
        Ok(_) => println!("RenderEngine initialized successfully"),
        Err(e) => println!("RenderEngine initialization failed: {:?}", e),
    }
}
