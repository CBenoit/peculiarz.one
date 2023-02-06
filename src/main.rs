use axum::body::{boxed, Body};
use axum::http::{Response, StatusCode};
use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use tokio::io::AsyncReadExt as _;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./static")]
    static_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // enable console logging
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/bread-world", get(bread_world))
        .route("/api/bread-world/recipes", get(get_recipes))
        .route("/app/bread-world.wasm", get(bread_world_wasm))
        .route("/app/bread-world.js", get(bread_world_js))
        .fallback_service(get(|req| async move {
            match ServeDir::new(&opt.static_dir).oneshot(req).await {
                Ok(res) => {
                    let status = res.status();
                    match status {
                        StatusCode::NOT_FOUND => {
                            let index_path = PathBuf::from(&opt.static_dir).join("index.html");
                            let index_content = match fs::read_to_string(index_path).await {
                                Err(_) => {
                                    return Response::builder()
                                        .status(StatusCode::NOT_FOUND)
                                        .body(boxed(Body::from("index file not found")))
                                        .unwrap()
                                }
                                Ok(index_content) => index_content,
                            };

                            Response::builder()
                                .status(StatusCode::OK)
                                .body(boxed(Body::from(index_content)))
                                .unwrap()
                        }
                        _ => res.map(boxed),
                    }
                }
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(boxed(Body::from(format!("error: {err}"))))
                    .expect("error response"),
            }
        }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));

    tracing::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Unable to start server");

    Ok(())
}

async fn get_recipes() -> impl IntoResponse {
    "not yet implemented"
}

async fn bread_world() -> impl IntoResponse {
    let content = fs::read_to_string("./static/bread-world.html").await.unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(boxed(Body::from(content)))
        .unwrap()
}

async fn bread_world_wasm() -> impl IntoResponse {
    let mut content = Vec::new();

    fs::File::open("./static/app/bread-world.wasm")
        .await
        .unwrap()
        .read_to_end(&mut content)
        .await
        .unwrap();

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/wasm")
        .body(boxed(Body::from(content)))
        .unwrap()
}

async fn bread_world_js() -> impl IntoResponse {
    let content = fs::read_to_string("./static/app/bread-world.js").await.unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/javascript")
        .body(boxed(Body::from(content)))
        .unwrap()
}
