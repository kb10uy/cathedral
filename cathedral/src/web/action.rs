use crate::{
    db::{fetch_diffs, fetch_song, fetch_song_summaries, fetch_version},
    schema::PlaySide,
    web::{error::*, schema::*},
    SharedData,
};

use std::collections::BinaryHeap;

use axum::{
    extract::{Query, State},
    response::Result as AxumResult,
    Form, Json,
};
use lyricism::{query_delete, query_insert, query_replace, query_substring, Lyricism};
use tracing::warn;

/// GET /songs/search?q=...
pub async fn songs_search(
    State(sd): State<SharedData>,
    Query(query): Query<SongsSearchQuery>,
) -> AxumResult<Json<Vec<SongsSearchResponse>>> {
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
        .map(|r| SongsSearchResponse {
            version_abbrev: r.version_abbrev.to_string(),
            id: r.id,
            genre: r.genre.to_string(),
            title: r.title.to_string(),
            artist: r.artist.to_string(),
        })
        .collect();

    Ok(Json(result_rows))
}

/// GET /songs/show?id=...
pub async fn songs_show(
    State(sd): State<SharedData>,
    Query(query): Query<SongsShowQuery>,
) -> AxumResult<Json<SongsShowResponse>> {
    let song = fetch_song(&sd.sqlite_pool, query.id)
        .await
        .map_err(pass_sqlx_error)?
        .ok_or_else(|| pass_not_found_error(&format!("song id {}", query.id)))?;
    let diffs = fetch_diffs(&sd.sqlite_pool, song.id)
        .await
        .map_err(pass_sqlx_error)?;

    Ok(Json(SongsShowResponse { song, diffs }))
}

/// GET /mattermost/enqueue
pub async fn mattermost_enqueue(
    State(sd): State<SharedData>,
    Form(form): Form<MattermostEnqueueForm>,
) -> AxumResult<Json<Option<MattermostEnqueueResult>>> {
    if sd.webhook_token != form.token {
        warn!("Invalid token arrived");
        return Err(pass_token_error());
    }
    if form.text.starts_with("//") {
        return Ok(Json(None));
    }

    let searcher = Lyricism::new(query_insert, query_delete, query_replace, query_substring);
    let queries = form
        .text
        .split('\n')
        .map(|q| q.trim())
        .filter(|q| !q.is_empty());
    let mut attachments = vec![];
    for query in queries {
        let mut candidate_distasnce = isize::MAX;
        let mut candidate_id = 0;

        for (id, title) in &sd.id_song_pairs[..] {
            let distance = searcher.distance(query, title);
            if distance < candidate_distasnce {
                candidate_distasnce = distance;
                candidate_id = *id;
            }
        }

        let song = fetch_song(&sd.sqlite_pool, candidate_id)
            .await
            .map_err(pass_sqlx_error)?
            .ok_or_else(|| pass_not_found_error(&format!("song id {}", candidate_id)))?;
        let diffs = fetch_diffs(&sd.sqlite_pool, song.id)
            .await
            .map_err(pass_sqlx_error)?;
        let sp_diffs: Vec<_> = diffs
            .iter()
            .filter(|d| d.play_side == PlaySide::Single)
            .map(|d| format!("{} :level-{}:", d.difficulty.to_emoji_str(), d.level))
            .collect();
        let dp_diffs: Vec<_> = diffs
            .iter()
            .filter(|d| d.play_side == PlaySide::Double)
            .map(|d| format!("{} :level-{}:", d.difficulty.to_emoji_str(), d.level))
            .collect();
        let version = fetch_version(&sd.sqlite_pool, song.version_id)
            .await
            .map_err(pass_sqlx_error)?;

        attachments.push(AttachmentSongInfo {
            title: format!("{} / {}", song.title, song.artist),
            footer: version.name,
            fields: vec![
                AttachmentSongField {
                    short: true,
                    title: "SP Levels".into(),
                    value: sp_diffs.join(" / "),
                },
                AttachmentSongField {
                    short: true,
                    title: "DP Levels".into(),
                    value: dp_diffs.join(" / "),
                },
                AttachmentSongField {
                    short: true,
                    title: "BPM".into(),
                    value: if let Some(min_bpm) = song.min_bpm {
                        format!("{min_bpm} - {}", song.max_bpm)
                    } else {
                        song.max_bpm.to_string()
                    },
                },
            ],
        });
    }

    Ok(Json(Some(MattermostEnqueueResult {
        username: "Cathedral".into(),
        attachments,
    })))
}
