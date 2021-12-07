use axum::Router;
use axum::routing::{get, post};
use axum::extract::Extension;
use axum::response::Json as RespJson;
use axum::extract::Json as ExtJson;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = must_env_var("PORT");
    let heroku_app_name = must_env_var("HEROKU_APP_NAME");
    let db_path = must_env_var("DB_PATH");

    let bind_addr = format!("0.0.0.0:{}", port);

    let mgr = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(mgr).expect("new pool");
    let shared_pool = Arc::new(pool);

    // build our application with a single route
    let app = Router::new()
    .layer(axum::AddExtensionLayer::new(shared_pool))
    .route("/short", post(short))
    .route("/expand", post(expand))
    .route("/redirect/:id", get(redirect));

    // run it with hyper on localhost:3000
    axum::Server::bind(&bind_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct ShortRequest {
    url: String,
}

#[derive(Serialize)]
struct ShortResponse {
    id: String,
}

async fn short(Extension(shared_pool): Extension<Arc<Pool<SqliteConnectionManager>>>, ExtJson(payload): ExtJson<ShortRequest>) -> RespJson<ShortResponse> {
    unimplemented!()
}

#[derive(Deserialize)]
struct ExpandRequest {
    id: String,
}

#[derive(Serialize)]
struct ExpandResponse {
    url: String,
}

async fn expand(Extension(shared_pool): Extension<Arc<Pool<SqliteConnectionManager>>>, ExtJson(payload): ExtJson<ExpandRequest>) -> RespJson<ExpandResponse>{
    unimplemented!()
}

async fn redirect(Extension(shared_pool): Extension<Arc<Pool<SqliteConnectionManager>>>) {
    unimplemented!()
}

fn must_env_var(key:&str) -> String {
    let val = std::env::var(key).unwrap_or_else(|_| format!("{} environment variable must be defined", key));
    val
}
