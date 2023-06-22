use crate::{
    db::{fetch_diffs, fetch_song, fetch_song_summaries},
    schema::{Diff, Song},
    SharedData,
};

use std::collections::BinaryHeap;

use axum::{
    extract::{Query, State},
    response::Result as AxumResult,
    Json,
};
use lyricism::{query_delete, query_insert, query_replace, query_substring, Lyricism};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResult {
    reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongsSearchQuery {
    q: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongsSearchResult {
    version_abbrev: String,
    id: i64,
    genre: String,
    title: String,
    artist: String,
}

/// GET /songs/search?q=...
pub async fn songs_search(
    State(sd): State<SharedData>,
    Query(query): Query<SongsSearchQuery>,
) -> AxumResult<Json<Vec<SongsSearchResult>>> {
    let searcher = Lyricism::new(query_insert, query_delete, query_replace, query_substring);
    let mut candidates = BinaryHeap::new();

    for (id, title) in &sd.id_song_pairs[..] {
        let distance = searcher.distance(&query.q, title);
        candidates.push((distance, *id));
        if candidates.len() > sd.candidates_count {
            candidates.pop();
        }
    }
    // collect as most-relevant first
    let mut candidate_pairs: Vec<_> = candidates.into_iter().collect();
    candidate_pairs.sort_by_key(|(d, _)| *d);
    let candidate_ids: Vec<_> = candidate_pairs.iter().map(|(_, id)| *id).collect();

    let rows = fetch_song_summaries(&sd.sqlite_pool, &candidate_ids)
        .await
        .map_err(pass_sqlx_error)?;
    let result_rows = candidate_ids
        .into_iter()
        .flat_map(|cid| rows.iter().find(|r| r.id == cid))
        .map(|r| SongsSearchResult {
            version_abbrev: r.version_abbrev.to_string(),
            id: r.id,
            genre: r.genre.to_string(),
            title: r.title.to_string(),
            artist: r.artist.to_string(),
        })
        .collect();

    Ok(Json(result_rows))
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongsShowQuery {
    id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongsShowResult {
    song: Song,
    diffs: Vec<Diff>,
}

/// GET /songs/show?id=...
pub async fn songs_show(
    State(sd): State<SharedData>,
    Query(query): Query<SongsShowQuery>,
) -> AxumResult<Json<SongsShowResult>> {
    let song = fetch_song(&sd.sqlite_pool, query.id)
        .await
        .map_err(pass_sqlx_error)?
        .ok_or_else(|| pass_not_found_error(&format!("song id {}", query.id)))?;
    let diffs = fetch_diffs(&sd.sqlite_pool, song.id)
        .await
        .map_err(pass_sqlx_error)?;

    Ok(Json(SongsShowResult { song, diffs }))
}

fn pass_sqlx_error(err: SqlxError) -> Json<ErrorResult> {
    Json(ErrorResult {
        reason: format!("db error: {}", err),
    })
}

fn pass_not_found_error(subreason: &str) -> Json<ErrorResult> {
    Json(ErrorResult {
        reason: format!("not found: {subreason}"),
    })
}
