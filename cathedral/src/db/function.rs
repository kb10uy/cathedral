use crate::db::schema::{Diff, Song, Version};

use std::path::Path;

use sqlx::{FromRow, Result as SqlxResult, SqlitePool};

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
            versions.id AS version_id,
            versions.name AS version_name,
            versions.abbrev AS version_abbrev
        FROM versions
        WHERE versions.id = ?;
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(rows)
}

pub async fn fetch_songs_with_versions(
    pool: &SqlitePool,
    song_ids: &[i64],
) -> SqlxResult<Vec<(Song, Version)>> {
    #[derive(Debug, FromRow)]
    struct RawRow {
        #[sqlx(flatten)]
        song: Song,
        #[sqlx(flatten)]
        version: Version,
    }

    let sql = format!(
        r#"
        SELECT
            songs.id AS song_id,
            songs.genre AS song_genre,
            songs.title AS song_title,
            songs.artist AS song_artist,
            songs.min_bpm AS song_min_bpm,
            songs.max_bpm AS song_max_bpm,
            songs.unlock_info AS song_unlock_info,
            songs.version_id AS version_id,
            versions.name AS version_name,
            versions.abbrev AS version_abbrev
        FROM songs
        INNER JOIN versions ON songs.version_id = versions.id
        WHERE songs.id IN ({});
        "#,
        &"?,".repeat(song_ids.len())[..(song_ids.len() * 2 - 1)],
    );
    let rows: Vec<RawRow> = song_ids
        .iter()
        .fold(sqlx::query_as(&sql), |q, id| q.bind(id))
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| (r.song, r.version)).collect())
}

pub async fn fetch_song(pool: &SqlitePool, id: i64) -> SqlxResult<Option<Song>> {
    let row = sqlx::query_as(
        r#"
        SELECT
            songs.id AS song_id,
            songs.version_id AS version_id,
            songs.genre AS song_genre,
            songs.title AS song_title,
            songs.artist AS song_artist,
            songs.min_bpm AS song_min_bpm,
            songs.max_bpm AS song_max_bpm,
            songs.unlock_info AS song_unlock_info
        FROM songs
        WHERE songs.id = ?;
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn fetch_diffs(pool: &SqlitePool, song_ids: &[i64]) -> SqlxResult<Vec<Diff>> {
    let sql = format!(
        r#"
        SELECT
            diffs.song_id AS song_id,
            diffs.play_side AS diff_play_side,
            diffs.difficulty AS diff_difficulty,
            diffs.level AS diff_level,
            diffs.cn_type AS diff_note_type,
            diffs.bss_type AS diff_scratch_type
        FROM diffs
        WHERE diffs.song_id IN ({});
        "#,
        &"?,".repeat(song_ids.len())[..(song_ids.len() * 2 - 1)],
    );
    let rows: Vec<Diff> = song_ids
        .iter()
        .fold(sqlx::query_as(&sql), |q, id| q.bind(id))
        .fetch_all(pool)
        .await?;

    Ok(rows)
}
