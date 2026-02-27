mod protocols;
use protocols::https_server::run_http_server;
use crate::protocols::dns::run_dns_server;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Rust protocols server...");

    tokio::select! {
        res = run_http_server() => {
            eprintln!("HTTP server exited: {:?}", res);
            res?;
        }
        res = run_dns_server() => {
            eprintln!("DNS server exited: {:?}", res);
            res?;
        }
    }

    Ok(())
}