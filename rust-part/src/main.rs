mod protocols;
use protocols::{dns::run_dns_server, dhcp::run_dhcp_server, https_server::run_http_server};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable debug logging to see if packets are actually hitting Hickory
    std::env::set_var("RUST_LOG", "hickory_server=debug,hickory_proto=debug");
    env_logger::init();

    println!("Starting Rust protocols server...");

    // Spawn servers as independent background tasks
    tokio::spawn(async {
        if let Err(e) = run_http_server().await {
            eprintln!("HTTP server error: {:?}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = run_dhcp_server().await {
            eprintln!("DHCP server error: {:?}", e);
        }
    });

    // Run the DNS server in the main thread so the program stays alive
    println!("DNS Server is starting...");
    if let Err(e) = run_dns_server().await {
        eprintln!("DNS server exited: {:?}", e);
    }

    Ok(())
}
