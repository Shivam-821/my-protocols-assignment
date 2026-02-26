mod protocols;
use protocols::http_server::run_http_server;

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>>{
    println!("Starting Rust protocols server...");
    run_http_server().await?;
    Ok(())
}