use client::APIExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpaceDevs API Executor");
    println!("======================");

    // Create executor from TOML configuration
    let executor = APIExecutor::from_config_file("simple.toml")?;

    // Execute all enabled endpoints
    executor.execute_all().await?;

    println!("API execution completed successfully!");

    Ok(())
}
