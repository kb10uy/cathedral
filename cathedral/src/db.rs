use crate::schema::{Diff, Song, SongSummary, Version};

use std::path::Path;

use sqlx::{Result as SqlxResult, SqlitePool};

pub async fn open_sqlite_file(path: &Path) -> SqlxResult<SqlitePool> {
    let conn = SqlitePool::connect(&format!(
        "sqlite://{}?mode=ro",
        path.to_str().expect("invalid filename")
    ))
    .await?;
    Ok(conn)
}

pub async fn fetch_title_pair(pool: &SqlitePool) -> SqlxResult<Vec<(i64, String)>> {
    let rows = sqlx::query_as(
        r#"
        SELECT songs.id, songs.title
        FROM songs
        ORDER BY songs.id;
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn fetch_version(pool: &SqlitePool, id: i64) -> SqlxResult<Version> {
    let rows = sqlx::query_as(
        r#"
        SELECT
            versions.id AS id,
            versions.name AS name,
            versions.abbrev AS abbrev
        FROM versions
        WHERE versions.id = ?;
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(rows)
}

/// The rows returned are not guaranteed to be the same number or in the same order AS the IDs specified.
pub async fn fetch_song_summaries(pool: &SqlitePool, ids: &[i64]) -> SqlxResult<Vec<SongSummary>> {
    let sql = format!(
        r#"
        SELECT
            songs.id AS id,
            songs.genre AS genre,
            songs.title AS title,
            songs.artist AS artist,
            versions.abbrev AS version_abbrev
        FROM songs
        INNER JOIN versions ON songs.version_id = versions.id
        WHERE songs.id IN ({});
        "#,
        &"?,".repeat(ids.len())[..(ids.len() * 2 - 1)],
    );
    let rows: Vec<SongSummary> = ids
        .iter()
        .fold(sqlx::query_as(&sql), |q, id| q.bind(id))
        .fetch_all(pool)
        .await?;

    Ok(rows)
}

pub async fn fetch_song(pool: &SqlitePool, id: i64) -> SqlxResult<Option<Song>> {
    let row = sqlx::query_as(
        r#"
        SELECT
            songs.id AS id,
            songs.version_id AS version_id,
            songs.genre AS genre,
            songs.title AS title,
            songs.artist AS artist,
            songs.min_bpm AS min_bpm,
            songs.max_bpm AS max_bpm,
            songs.unlock_info AS unlock_info
        FROM songs
        WHERE songs.id = ?;
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_diffs(pool: &SqlitePool, song_id: i64) -> SqlxResult<Vec<Diff>> {
    let row = sqlx::query_as(
        r#"
        SELECT
            diffs.song_id AS song_id,
            diffs.play_side AS play_side,
            diffs.difficulty AS difficulty,
            diffs.level AS level,
            diffs.cn_type AS note_type,
            diffs.bss_type AS scratch_type
        FROM diffs
        WHERE diffs.song_id = ?;
        "#,
    )
    .bind(song_id)
    .fetch_all(pool)
    .await?;

    Ok(row)
}
