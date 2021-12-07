use axum::extract::Json as ExtJson;
use axum::extract::{Extension, Path};
use axum::http::status::StatusCode;
use axum::response::Json as RespJson;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::Router;
use log::{debug, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = must_env_var("PORT");
    let db_path = must_env_var("DB_PATH");

    info!("init database");
    let mgr = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(mgr).expect("new pool");
    init_database(pool.clone());

    let app = Router::new()
        .route("/short", post(short))
        .route("/expand", post(expand))
        .route("/redirect/:id", get(redirect))
        .layer(axum::AddExtensionLayer::new(pool.clone()));

    let bind_addr = format!("0.0.0.0:{}", port);
    info!("start server at {}", bind_addr);
    axum::Server::bind(&bind_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_database(pool: Pool<SqliteConnectionManager>) {
    debug!("init_database start");
    let conn = pool.get().expect("get DB connection");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS idurlmap (id TEXT, url TEXT)",
        params![],
    )
    .expect("create table");
    debug!("init_database finish");
}

#[derive(Debug, Deserialize)]
struct ShortRequest {
    url: String,
}

#[derive(Serialize)]
struct ShortResponse {
    id: String,
}

async fn short(
    Extension(pool): Extension<Pool<SqliteConnectionManager>>,
    ExtJson(payload): ExtJson<ShortRequest>,
) -> RespJson<ShortResponse> {
    info!("short {:?}", payload);
    let id = nanoid::nanoid!(8);
    let conn = pool.get().expect("get DB connection");
    let rows = conn
        .execute(
            "INSERT INTO idurlmap (id, url) VALUES(?, ?)",
            params![id, payload.url],
        )
        .expect("insert");
    info!("inserted {} rows", rows);
    RespJson(ShortResponse { id })
}

#[derive(Debug, Deserialize)]
struct ExpandRequest {
    id: String,
}

#[derive(Serialize)]
struct ExpandResponse {
    url: String,
}

async fn expand(
    Extension(pool): Extension<Pool<SqliteConnectionManager>>,
    ExtJson(payload): ExtJson<ExpandRequest>,
) -> Result<RespJson<ExpandResponse>, StatusCode> {
    info!("expand {:?}", payload);
    let conn = pool.get().expect("get DB connection");
    if let Ok(url) = conn.query_row(
        "SELECT url FROM idurlmap WHERE id = ?",
        params![payload.id],
        |r| r.get::<usize, String>(0),
    ) {
        Ok(RespJson(ExpandResponse { url }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn redirect(
    Extension(pool): Extension<Pool<SqliteConnectionManager>>,
    Path(id): Path<String>,
) -> Result<Redirect, StatusCode> {
    info!("redirect {}", id);
    let conn = pool.get().expect("get DB connection");
    if let Ok(url) = conn.query_row("SELECT url FROM idurlmap WHERE id = ?", params![id], |r| {
        r.get::<usize, String>(0)
    }) {
        Ok(Redirect::permanent(url.parse().unwrap()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn must_env_var(key: &str) -> String {
    let msg = format!("{} env var must be defined", key);
    let val = std::env::var(key).expect(&msg);
    val
}
