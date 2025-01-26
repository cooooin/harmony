mod api;
mod common;
mod consts;
mod model;
mod time;

use std::env;
use std::net::SocketAddr;

use axum_server::tls_rustls::RustlsConfig;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let cert_path = env::var("CERT_PATH");
    let key_path = env::var("KEY_PATH");

    let router = api::router();

    if cert_path.is_ok() && key_path.is_ok() {
        let address = env::var("ADDRESS").unwrap_or("[::]:443".into());
        let addr: SocketAddr = address.parse().unwrap();
        let config = RustlsConfig::from_pem_file(cert_path.unwrap(), key_path.unwrap())
            .await
            .unwrap();

        axum_server::bind_rustls(addr, config)
            .serve(router.into_make_service())
            .await
            .unwrap();
    } else {
        let address = env::var("ADDRESS").unwrap_or("[::]:80".into());
        let addr: SocketAddr = address.parse().unwrap();

        axum_server::bind(addr)
            .serve(router.into_make_service())
            .await
            .unwrap();
    }
}
