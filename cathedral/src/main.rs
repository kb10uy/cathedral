mod cli;
mod db;
mod schema;
mod web;

use crate::{
    cli::Arguments,
    db::{fetch_title_pair, open_sqlite_file},
    web::action::{mattermost_enqueue, songs_search, songs_show},
};

use std::sync::Arc;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router, Server,
};
use clap::Parser;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SharedData {
    webhook_token: String,
    candidates_count: usize,
    sqlite_pool: SqlitePool,
    id_song_pairs: Arc<[(i64, String)]>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    tracing_subscriber::fmt::init();

    let sqlite_pool = open_sqlite_file(&args.sqlite_filename).await?;
    let id_song_pairs = fetch_title_pair(&sqlite_pool).await?;
    let shared_data = SharedData {
        webhook_token: args.mattermost_token,
        candidates_count: 5,
        sqlite_pool,
        id_song_pairs: id_song_pairs.into(),
    };

    let router = Router::new()
        .route("/songs/search", get(songs_search))
        .route("/songs/show", get(songs_show))
        .route("/mattermost/enqueue", post(mattermost_enqueue))
        .with_state(shared_data);

    Server::bind(&args.bind)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
