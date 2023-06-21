use crate::{db::fetch_songs_summary, SharedData};

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
pub struct SearchQuery {
    q: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    version_abbrev: String,
    genre: String,
    title: String,
    artist: String,
}

pub async fn search_songs(
    State(sd): State<SharedData>,
    Query(query): Query<SearchQuery>,
) -> AxumResult<Json<Vec<SearchResult>>> {
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
    println!("{candidate_pairs:?}");
    let candidate_ids: Vec<_> = candidate_pairs.iter().map(|(_, id)| *id).collect();

    let rows = fetch_songs_summary(&sd.sqlite_pool, &candidate_ids)
        .await
        .map_err(pass_sqlx_error)?;
    let result_rows = candidate_ids
        .into_iter()
        .flat_map(|cid| rows.iter().find(|r| r.id == cid))
        .map(|r| SearchResult {
            version_abbrev: r.version_abbrev.to_string(),
            genre: r.genre.to_string(),
            title: r.title.to_string(),
            artist: r.artist.to_string(),
        })
        .collect();

    Ok(Json(result_rows))
}

fn pass_sqlx_error(err: SqlxError) -> Json<ErrorResult> {
    Json(ErrorResult {
        reason: err.to_string(),
    })
}
