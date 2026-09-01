use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web::cookie::Key;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use anyhow::{anyhow, Result};
use base64::prelude::*;
use include_dir::{include_dir, Dir};
use log::info;
use redis_work_queue::{KeyPrefix, WorkQueue};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::{Arc, Mutex};

use crate::app::{ApiError, AppState};
use crate::redis::RedisQueue;
use crate::{api, auth};

//mod pings; // Undo this when developing it

// Embed the 'static' directory into the binary
static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/frontend/dist");

async fn serve_file(path: web::Path<String>) -> impl Responder {
    let file_path = path.into_inner();
    if let Some(file) = STATIC_DIR.get_file(&file_path) {
        let content = file.contents();
        let mime = mime_guess::from_path(&file_path).first_or_octet_stream();
        HttpResponse::Ok().content_type(mime.as_ref()).body(content)
    } else {
        HttpResponse::NotFound().json(ApiError::from("File not found".to_string()))
    }
}

async fn serve_index() -> impl Responder {
    if let Some(file) = STATIC_DIR.get_file("index.html") {
        let content = file.contents();
        let mime = mime_guess::from_path("index.html").first_or_octet_stream();
        HttpResponse::Ok().content_type(mime.as_ref()).body(content)
    } else {
        HttpResponse::NotFound().json(ApiError::from("File not found".to_string()))
    }
}

fn require_env_var(name: &str) -> String {
    let value = env::var(name).unwrap_or_else(|_| panic!("{name} must be set"));
    assert!(
        !value.trim().is_empty(),
        "{name} environmental variable cannot be empty/missing"
    );
    value
}

pub async fn main() -> Result<()> {
    let host = require_env_var("HOST");
    let host_inner = host.clone();
    let port: i32 = require_env_var("PORT")
        .parse()
        .expect("PORT must be a valid integer");

    let database_url = require_env_var("DATABASE_URL");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    let session_key = env::var("SESSION_KEY")
        .map_err(|e| anyhow!("Failed to get Env Var: {}", e))
        .and_then(|key64| {
            BASE64_STANDARD
                .decode(key64)
                .map_err(|e| anyhow!("Failed to decode session key: {}", e))
        })
        .map(|key| Key::from(&key))
        .unwrap_or(Key::generate());

    let redis_url = require_env_var("REDIS_URL");

    let redis_conn = redis::Client::open(redis_url.clone())
        .expect("Failed to create Redis Client")
        .get_multiplexed_async_connection()
        .await
        .expect("Failed to create Redis Connection");

    let csh_userinfo_url = require_env_var("CSH_USERINFO_URL");

    info!("Starting server at http://{host}:{port}");
    HttpServer::new(move || {
        let (google_client, csh_client) = auth::get_clients(&host_inner, port);

        App::new()
            .app_data(web::Data::new(AppState {
                db: db_pool.clone(),
                redis: Arc::new(Mutex::new(RedisQueue {
                    redis: redis_conn.clone(),
                    work_queue: WorkQueue::new(KeyPrefix::from("rideboard")),
                })),
                google_oauth: google_client,
                google_userinfo_url: "https://openidconnect.googleapis.com/v1/userinfo".to_string(),
                csh_oauth: csh_client,
                csh_userinfo_url: csh_userinfo_url.clone(),
            }))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), session_key.clone())
                    .cookie_secure(env::var("DEVELOPMENT").is_err())
                    .build(),
            )
            .wrap(Logger::default())
            .service(api::scope())
            .route("/", web::get().to(serve_index))
            .route("/history", web::get().to(serve_index))
            .route("/{id}", web::get().to(serve_index))
            .route("/history/{id}", web::get().to(serve_index))
            .route("/event/{id}", web::get().to(serve_index))
            .route("/login", web::get().to(serve_index))
            .route("/{filename:.*}", web::get().to(serve_file))
    })
    .bind(format!("{host}:{port}"))?
    .run()
    .await
    .map_err(|err| anyhow!("Failed to run server: {}", err))
}
