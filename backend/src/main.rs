use axum::{
  extract::Path,
  http::{HeaderMap, StatusCode},
  response::{Html, IntoResponse},
  routing::get,
  Router,
};
use std::{net::SocketAddr, path::PathBuf};
use tokio::fs;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
  let app = Router::new()
      // Serve static files (JS/WASM/...) from /code_editor/assets/*path
      .route("/code_editor/assets/{*path}", get(serve_asset))
      // Serve index.html for any /code_editor route (SPA fallback)
      .route("/code_editor", get(serve_index))
      .route("/code_editor/{*path}", get(serve_index))
      .layer(TraceLayer::new_for_http());

  let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
  println!("🚀 Serving: http://{}/code_editor", addr);

  axum::serve(
      tokio::net::TcpListener::bind(addr).await.unwrap(),
      app.into_make_service(),
  )
  .await
  .unwrap();
}

async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
  let base = PathBuf::from("../target/dx/code_editor/release/web/public/assets");
  let file_path = base.join(&path);

  match fs::read(&file_path).await {
      Ok(contents) => {
          let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
          let mut headers = HeaderMap::new();
          headers.insert("Content-Type", mime.to_string().parse().unwrap());
          (headers, contents).into_response()
      }
      Err(_) => (StatusCode::NOT_FOUND, "Asset Not Found").into_response(),
  }
}

async fn serve_index() -> impl IntoResponse {
  let index_path = "../target/dx/code_editor/release/web/public/index.html";

  match fs::read_to_string(index_path).await {
      Ok(contents) => Html(contents).into_response(),
      Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
  }
}
