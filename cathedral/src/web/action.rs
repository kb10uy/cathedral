use crate::{
    db::{function::*, schema::*},
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
use rand::prelude::*;
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

    let rows = fetch_songs_with_versions(&sd.sqlite_pool, &candidate_ids)
        .await
        .map_err(pass_sqlx_error)?;
    let result_rows = candidate_ids
        .into_iter()
        .flat_map(|cid| rows.iter().find(|(s, _)| s.id == cid))
        .map(|(s, v)| SongsSearchResponse {
            version_abbrev: v.abbrev.to_string(),
            id: s.id,
            genre: s.genre.to_string(),
            title: s.title.to_string(),
            artist: s.artist.to_string(),
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
    let diffs = fetch_diffs_by_song_ids(&sd.sqlite_pool, &[song.id])
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

    let mut song_ids = vec![];
    let mut diff_ids = vec![];
    for query in queries {
        if let Some(filters_str) = query.strip_prefix('?') {
            // diff filter query
            let (filters, count) =
                parse_extended_query(filters_str).map_err(pass_filter_query_error)?;
            let filtered_ids = query_filter_diffs(&sd.sqlite_pool, &filters)
                .await
                .map_err(pass_sqlx_error)?;

            let mut rng = thread_rng();
            let chosen_ids: Vec<_> = filtered_ids
                .choose_multiple(&mut rng, count)
                .copied()
                .collect();
            diff_ids.extend_from_slice(&chosen_ids);
        } else if let Some(compact_str) = query.strip_prefix('!') {
            // diff compact query
            let compact_queries = parse_compact_queries(compact_str);

            for (query, count) in compact_queries {
                let filtered_ids = query_filter_diffs(&sd.sqlite_pool, &query)
                    .await
                    .map_err(pass_sqlx_error)?;

                let mut rng = thread_rng();
                let chosen_ids: Vec<_> = filtered_ids
                    .choose_multiple(&mut rng, count)
                    .copied()
                    .collect();
                diff_ids.extend_from_slice(&chosen_ids);
            }
        } else {
            // song title
            let mut candidate_distasnce = isize::MAX;
            let mut candidate_id = 0;

            for (id, title) in &sd.id_song_pairs[..] {
                let distance = searcher.distance(query, title);
                if distance < candidate_distasnce {
                    candidate_distasnce = distance;
                    candidate_id = *id;
                }
            }

            song_ids.push(candidate_id);
        };
    }

    let by_diff_diffs = fetch_diffs_by_ids(&sd.sqlite_pool, &diff_ids)
        .await
        .map_err(pass_sqlx_error)?;
    let by_song_diffs = fetch_diffs_by_song_ids(&sd.sqlite_pool, &song_ids)
        .await
        .map_err(pass_sqlx_error)?;

    let merged_song_ids: Vec<_> = song_ids
        .iter()
        .copied()
        .chain(by_diff_diffs.iter().map(|d| d.song_id))
        .collect();
    let song_version_pairs = fetch_songs_with_versions(&sd.sqlite_pool, &merged_song_ids)
        .await
        .map_err(pass_sqlx_error)?;

    // by_song_diffs have all diffs which correspond to songs in song_ids,
    // so merged_song_ids don't have to care about it.

    let mut texts = vec![];
    for diff in by_diff_diffs {
        let Some((song, version)) = song_version_pairs.iter().find(|(s, _)| s.id == diff.song_id) else {
            continue;
        };
        texts.push(format!(
            "* [{} {} :level-{}:] **{}** ({})",
            diff.play_side,
            diff.difficulty.to_emoji_str(),
            diff.level,
            song.title,
            version.abbrev
        ));
    }

    let mut attachments = vec![];
    for song_id in song_ids {
        let Some((song, version)) = song_version_pairs.iter().find(|(s, _)| s.id == song_id) else {
            continue;
        };

        let song_diffs = by_song_diffs.iter().filter(|d| d.song_id == song.id);
        let sp_diffs: Vec<_> = song_diffs
            .clone()
            .filter(|d| d.play_side == PlaySide::Single)
            .map(|d| format!("{} :level-{}:", d.difficulty.to_emoji_str(), d.level))
            .collect();
        let dp_diffs: Vec<_> = song_diffs
            .clone()
            .filter(|d| d.play_side == PlaySide::Single)
            .map(|d| format!("{} :level-{}:", d.difficulty.to_emoji_str(), d.level))
            .collect();

        attachments.push(AttachmentSongInfo {
            title: format!("{} / {}", song.title, song.artist),
            footer: version.name.clone(),
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
        text: texts.join("\n"),
        attachments,
    })))
}

fn parse_extended_query(query: &str) -> Result<(Vec<FilterQuery>, usize), FilterQueryError> {
    query
        .trim()
        .split_ascii_whitespace()
        .try_fold(vec![], |mut fs, q| {
            fs.push(q.parse()?);
            Ok(fs)
        })
        .map(|q| (q, 1))
}

fn parse_compact_queries(text: &str) -> Vec<([FilterQuery; 3], usize)> {
    let mut queries = vec![];
    for query_text in text.trim().split_ascii_whitespace() {
        if query_text.len() <= 3 {
            continue;
        }

        let (play_side, difficulty) = match &query_text[..3] {
            "spb" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "spn" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "sph" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "spa" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "spl" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "dpn" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "dph" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "dpa" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            "dpl" => (
                FilterQuery::PlaySide(PlaySide::Single),
                FilterQuery::Difficulty(Difficulty::Beginner),
            ),
            _ => continue,
        };

        let mut level_count = query_text[3..].split('*');
        let Some(level) = level_count.next() else {
            continue;
        };
        let Ok(level) = level.parse::<i64>() else {
            continue;
        };
        let count = if let Some(count) = level_count.next() {
            let Ok(value) = count.parse::<usize>() else {
                continue;
            };
            value
        } else {
            1
        };

        queries.push(([play_side, difficulty, FilterQuery::Level(level)], count));
    }
    queries
}
