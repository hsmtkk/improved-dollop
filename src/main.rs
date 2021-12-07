use axum::Router;
use axum::routing::{get, post};
use axum::extract::{Extension, Path};
use axum::response::Redirect;
use axum::response::Json as RespJson;
use axum::extract::Json as ExtJson;
use log::{debug, info};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() {
    env_logger::init();

    let port = must_env_var("PORT");
    let heroku_app_name = must_env_var("HEROKU_APP_NAME");
    let db_path = must_env_var("DB_PATH");

    let bind_addr = format!("0.0.0.0:{}", port);

    let mgr = SqliteConnectionManager::file(db_path);
    let pool = Pool::new(mgr).expect("new pool");

    init_database(pool.clone());

    // build our application with a single route
    let app = Router::new()
    .layer(axum::AddExtensionLayer::new(pool.clone()))
    .route("/short", post(short))
    .route("/expand", post(expand))
    .route("/redirect/:id", get(redirect));

    // run it with hyper on localhost:3000
    axum::Server::bind(&bind_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn init_database(pool: Pool<SqliteConnectionManager>){
    debug!("init_database start");
    let conn = pool.get().expect("get DB connection");
    conn.execute("CREATE TABLE IF NOT EXISTS idurlmap (id TEXT, url TEXT)", params![]).expect("create table");
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

async fn short(Extension(pool): Extension<Pool<SqliteConnectionManager>>, ExtJson(payload): ExtJson<ShortRequest>) -> RespJson<ShortResponse> {
    info!("short {:?}", payload);
    let id = nanoid::nanoid!(8);
    let conn = pool.get().expect("get DB connection");
    let rows = conn.execute("INSERT INTO idurlmap (id, url) VALUES(?, ?)", params![id, payload.url]).expect("insert");
    info!("inserted {} rows", rows);
    RespJson(ShortResponse{id})
}

#[derive(Debug, Deserialize)]
struct ExpandRequest {
    id: String,
}

#[derive(Serialize)]
struct ExpandResponse {
    url: String,
}

async fn expand(Extension(pool): Extension<Pool<SqliteConnectionManager>>, ExtJson(payload): ExtJson<ExpandRequest>) -> RespJson<ExpandResponse>{
    info!("expand {:?}", payload);
    let conn = pool.get().expect("get DB connection");
    let url: String = conn.query_row("SELECT url FROM idurlmap WHERE id = ?", params![payload.id], |r| r.get(0)).expect("select");
    RespJson(ExpandResponse{url})
}

async fn redirect(Extension(pool): Extension<Pool<SqliteConnectionManager>>, Path(id): Path<String>) -> Redirect {
    info!("redirect {}", id);
    let conn = pool.get().expect("get DB connection");
    let url: String = conn.query_row("SELECT url FROM idurlmap WHERE id = ?", params![id], |r| r.get(0)).expect("select");
    Redirect::permanent(url.parse().unwrap())
}

fn must_env_var(key:&str) -> String {
    let val = std::env::var(key).unwrap_or_else(|_| format!("{} environment variable must be defined", key));
    val
}
