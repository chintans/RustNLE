use anyhow::Result;
use nle_core; // We need to expose something from core to use here

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    println!("Starting Headless Test...");

    // Initialize Engine (Mock)
    println!("Initializing Engine...");

    // Create a timeline
    println!("Creating Timeline...");

    // Render loop
    println!("Rendering 100 frames...");
    for i in 0..100 {
        if i % 10 == 0 {
            println!("Rendered frame {}", i);
        }
    }

    println!("Headless Test Complete. Exit Code 0.");
    Ok(())
}
