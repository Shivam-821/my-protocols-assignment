use std::net::SocketAddr;
use axum::{Router};
use axum::routing::get;

pub async fn run_http_server()-> Result<(), Box<dyn std::error::Error>>{
    let app = Router::new()
        .route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(&addr).await.unwrap(),
        app.into_make_service(),
    )
        .await?;
    Ok(())
}
async fn handler() -> &'static str{
    "Hello from Front http server endpoint"
}