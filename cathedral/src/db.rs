use std::path::Path;

use sqlx::{FromRow, Result as SqlxResult, SqlitePool};

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct SongSummary {
    pub id: i64,
    pub version_abbrev: String,
    pub genre: String,
    pub title: String,
    pub artist: String,
}

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

/// The rows returned are not guaranteed to be the same number or in the same order as the IDs specified.
pub async fn fetch_songs_summary(pool: &SqlitePool, ids: &[i64]) -> SqlxResult<Vec<SongSummary>> {
    let sql = format!(
        r#"
        SELECT
            songs.id as id,
            songs.genre as genre,
            songs.title as title,
            songs.artist as artist,
            versions.abbrev as version_abbrev
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
