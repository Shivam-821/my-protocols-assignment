use axum::{routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::fs::File;
use std::io::{self, BufReader};

pub async fn run_http_server() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8443));

    // Load cert & key (mkcert files)
    let cert_file = File::open("certs/cert.pem")?;
    let key_file = File::open("certs/key.pem")?;

    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_file))
        .collect::<Result<Vec<_>, _>>()?;
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(key_file))
        .collect::<Result<Vec<_>, _>>()?;

    let key = keys.pop().ok_or(io::Error::new(io::ErrorKind::NotFound, "No private key"))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, rustls::pki_types::PrivateKeyDer::Pkcs8(key.into()))?;

    let tls_config = RustlsConfig::from_config(std::sync::Arc::new(config));

    println!("HTTPS server listening on https://localhost:{}", addr.port());

    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler() -> &'static str {
    "Hello from Rust HTTPS server! (trusted by mkcert)"
}